//! Nyar语言的虚拟机实现
//!
//! 实现了基于栈机模型的虚拟机核心，包括执行上下文和指令执行逻辑。

use crate::{
    Result,
    instruction::NyarInstruction,
    value::{Function, NyarValue},
};
use gc_arena::{ Collect, Gc, Mutation, };
use std::{collections::HashMap, thread::JoinHandle};

/// 虚拟机
#[derive(Collect)]
#[collect(no_drop)]
pub struct VirtualMachine<'vm> {
    /// 执行上下文
    pub context: ExecutionContext<'vm>,
    /// 代码段
    pub code: Vec<NyarInstruction>,
    /// 常量池
    pub constants: Vec<NyarValue<'vm>>,
    /// 运行时
    pub runtime: tokio::runtime::Runtime,
    /// 效果处理器
    pub effect_handlers: HashMap<String, crate::control_flow::NyarHandler<'vm>>,
}

/// 执行上下文
#[derive(Collect)]
#[collect(no_drop)]
pub struct ExecutionContext<'vm> {
    /// 操作数栈
    pub stack: Vec<NyarValue<'vm>>,
    /// 局部变量
    pub locals: Vec<NyarValue<'vm>>,
    /// 全局变量
    pub globals: HashMap<String, NyarValue<'vm>>,
    /// 调用栈
    pub call_stack: Vec<NyarFrame>,
    /// 当前指令指针
    pub ip: usize,
    /// 当前函数
    pub current_function: Option<Gc<'vm, Function<'vm>>>,
    /// 异步任务队列
    pub async_tasks: Vec<JoinHandle<Result<NyarValue<'vm>>>>,
}

/// 调用帧
#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct NyarFrame {
    /// 返回地址
    pub return_address: usize,
    /// 局部变量基址
    pub base_pointer: usize,
}

impl<'gc> ExecutionContext<'gc> {
    /// 创建一个新的执行上下文
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            locals: Vec::new(),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            ip: 0,
            current_function: None,
            async_tasks: Vec::new(),
        }
    }

    /// 压入值到操作数栈
    pub fn push(&mut self, value: NyarValue<'gc>) {
        self.stack.push(value);
    }

    /// 从操作数栈弹出值
    pub fn pop(&mut self) -> Result<NyarValue<'gc>> {
        self.stack.pop().ok_or_else(|| nyar_error::NyarError::custom("Stack underflow"))
    }

    /// 获取栈顶值但不弹出
    pub fn peek(&self) -> Result<&NyarValue<'gc>> {
        self.stack.last().ok_or_else(|| nyar_error::NyarError::custom("Stack is empty"))
    }
}

impl<'vm> VirtualMachine<'vm> {
    /// 创建一个新的虚拟机
    pub fn new(_mc: Mutation<'vm>) -> Self {
        Self {
            context: ExecutionContext::new(),
            code: Vec::new(),
            constants: Vec::new(),
            runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap(),
            effect_handlers: HashMap::new(),
        }
    }

    /// 执行指令
    pub fn execute(&mut self, mc: &Mutation<'vm>) -> Result<NyarValue<'vm>> {
        while self.context.ip < self.code.len() {
            let instruction = &self.code[self.context.ip];
            self.context.ip += 1;

            match instruction {
                // 栈操作
                NyarInstruction::Push { constant_index } => {
                    let value = self.constants[*constant_index].clone();
                    self.context.push(value);
                }
                NyarInstruction::Pop => {
                    self.context.pop()?;
                }
                NyarInstruction::Dup => {
                    let value = self.context.peek()?.clone();
                    self.context.push(value);
                }
                NyarInstruction::Swap => {
                    let b = self.context.pop()?;
                    let a = self.context.pop()?;
                    self.context.push(b);
                    self.context.push(a);
                }

                // 变量操作
                NyarInstruction::LoadLocal { local_index } => {
                    let value = self.context.locals[*local_index].clone();
                    self.context.push(value);
                }
                NyarInstruction::StoreLocal { local_index } => {
                    let value = self.context.pop()?;
                    if *local_index >= self.context.locals.len() {
                        self.context.locals.resize(*local_index + 1, NyarValue::Null);
                    }
                    self.context.locals[*local_index] = value;
                }
                NyarInstruction::LoadGlobal { name_index } => {
                    let name = match &self.constants[*name_index] {
                        NyarValue::String(s) => s.to_string(),
                        _ => return Err(nyar_error::NyarError::custom("Expected string for global name")),
                    };
                    let value = self.context.globals.get(&name).cloned().unwrap_or(NyarValue::Null);
                    self.context.push(value);
                }
                NyarInstruction::StoreGlobal { name_index } => {
                    let name = match &self.constants[*name_index] {
                        NyarValue::String(s) => s.to_string(),
                        _ => return Err(nyar_error::NyarError::custom("Expected string for global name")),
                    };
                    let value = self.context.pop()?;
                    self.context.globals.insert(name, value);
                }

                // 控制流
                NyarInstruction::Jump { target } => {
                    self.context.ip = *target;
                }
                NyarInstruction::JumpIf { target } => {
                    let condition = self.context.pop()?;
                    if let NyarValue::Boolean(true) = condition {
                        self.context.ip = *target;
                    }
                }
                NyarInstruction::JumpIfNot { target } => {
                    let condition = self.context.pop()?;
                    if let NyarValue::Boolean(false) = condition {
                        self.context.ip = *target;
                    }
                }
                NyarInstruction::Call { arg_count } => {
                    let function_value = self.context.pop()?;

                    if let NyarValue::Function(function) = function_value {
                        // 保存当前执行状态
                        let return_address = self.context.ip;
                        let base_pointer = self.context.locals.len() - *arg_count;

                        self.context.call_stack.push(NyarFrame { return_address, base_pointer });

                        // 设置新的执行状态
                        self.context.current_function = Some(function.clone());
                        self.context.ip = 0;

                        // 切换代码段
                        let old_code = std::mem::replace(&mut self.code, function.instructions.clone());

                        // 执行函数
                        let result = self.execute(mc)?;

                        // 恢复原来的代码段
                        self.code = old_code;

                        // 压入返回值
                        self.context.push(result);
                    }
                    else {
                        return Err(nyar_error::NyarError::custom("Called value is not a function"));
                    }
                }
                NyarInstruction::Return => {
                    let return_value = self.context.pop()?;

                    if let Some(frame) = self.context.call_stack.pop() {
                        // 恢复调用者的执行状态
                        self.context.ip = frame.return_address;
                        self.context.locals.truncate(frame.base_pointer);
                        self.context.current_function = None;
                    }

                    return Ok(return_value);
                }

                // 对象操作
                NyarInstruction::NewObject { class_name_index } => {
                    let class_name = match &self.constants[*class_name_index] {
                        NyarValue::String(s) => s.to_string(),
                        _ => return Err(nyar_error::NyarError::custom("Expected string for class name")),
                    };
                    let object = NyarValue::new_object(mc, &class_name);
                    self.context.push(object);
                }
                NyarInstruction::GetProperty { property_name_index } => {
                    let prop_name = match &self.constants[*property_name_index] {
                        NyarValue::String(s) => s.to_string(),
                        _ => return Err(nyar_error::NyarError::custom("Expected string for property name")),
                    };

                    let object = self.context.pop()?;
                    if let NyarValue::Object(obj) = object {
                        let obj_ref = obj.read();
                        let value = obj_ref.properties.get(&prop_name).cloned().unwrap_or(NyarValue::Null);
                        self.context.push(value);
                    }
                    else {
                        return Err(nyar_error::NyarError::custom("Cannot get property of non-object"));
                    }
                }
                NyarInstruction::SetProperty { property_name_index } => {
                    let prop_name = match &self.constants[*property_name_index] {
                        NyarValue::String(s) => s.to_string(),
                        _ => return Err(nyar_error::NyarError::custom("Expected string for property name")),
                    };

                    let value = self.context.pop()?;
                    let object = self.context.pop()?;

                    if let NyarValue::Object(obj) = object {
                        let mut obj_ref = obj.write(mc);
                        obj_ref.properties.insert(prop_name, value.clone());
                        self.context.push(value);
                    }
                    else {
                        return Err(nyar_error::NyarError::custom("Cannot set property of non-object"));
                    }
                }

                // 异步操作
                NyarInstruction::Await => {
                    // 实际实现需要与tokio集成
                    // 这里只是一个简化的示例
                    let future_value = self.context.pop()?;
                    // 在实际实现中，这里应该暂停当前执行并等待future完成
                    self.context.push(NyarValue::Null); // 占位符
                }
                NyarInstruction::BlockOn => {
                    // 阻塞等待异步操作完成
                    // 实际实现需要与tokio集成
                    let future_value = self.context.pop()?;
                    // 在实际实现中，这里应该阻塞当前线程直到future完成
                    self.context.push(NyarValue::Null); // 占位符
                }

                // 其他指令...
                // TODO: 实现其他指令
                _ => {
                    return Err(nyar_error::NyarError::custom(format!("Unimplemented instruction: {:?}", instruction)));
                }
            }
        }

        // 如果执行完所有指令，返回栈顶值或null
        Ok(self.context.pop().unwrap_or(NyarValue::Null))
    }
}
