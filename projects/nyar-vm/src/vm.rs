//! 虚拟机核心模块，实现指令执行和协程调度

use std::collections::HashMap;
use crate::error::VmError;
use crate::heap::{Gc, Heap};
use crate::instruction::Instruction;
use crate::value::{Class, Coroutine, CoroutineState, EffectHandler, Enum, Function, Trait, Value};

/// 虚拟机状态
#[derive(Debug, Clone, PartialEq)]
pub enum VmState {
    /// 初始状态
    Initial,
    /// 运行中
    Running,
    /// 已暂停
    Suspended,
    /// 已完成
    Completed,
    /// 出错
    Failed(VmError),
}

/// 虚拟机结构体，负责执行指令和管理内存
#[derive(Debug)]
pub struct VirtualMachine {
    /// 堆内存
    pub heap: Heap,
    /// 当前状态
    pub state: VmState,
    /// 当前值栈
    pub value_stack: Vec<Gc<Value>>,
    /// 当前调用栈
    pub call_stack: Vec<Gc<Function>>,
    /// 当前环境栈
    pub environment_stack: Vec<Gc<HashMap<String, Gc<Value>>>>,
    /// 当前指令指针
    pub instruction_pointer: usize,
    /// 当前指令集
    pub instructions: Vec<Instruction>,
    /// 全局变量
    pub globals: Gc<HashMap<String, Gc<Value>>>,
    /// 当前协程
    pub current_coroutine: Option<Gc<Coroutine>>,
    /// 效应处理器栈
    pub effect_handlers: Vec<Gc<EffectHandler>>,
    /// 最大栈深度
    pub max_stack_depth: usize,
    /// 最大调用深度
    pub max_call_depth: usize,
}

impl VirtualMachine {
    /// 创建新的虚拟机
    pub fn new() -> Self {
        let mut heap = Heap::new();
        let globals = heap.allocate::<HashMap<String, Gc<Value>>>(Value::Object(Gc {
            index: 0,
            phantom: std::marker::PhantomData,
        }));
        
        Self {
            heap,
            state: VmState::Initial,
            value_stack: Vec::new(),
            call_stack: Vec::new(),
            environment_stack: Vec::new(),
            instruction_pointer: 0,
            instructions: Vec::new(),
            globals,
            current_coroutine: None,
            effect_handlers: Vec::new(),
            max_stack_depth: 1000,
            max_call_depth: 100,
        }
    }

    /// 执行指令集
    pub fn execute(&mut self, instructions: Vec<Instruction>) -> Result<Gc<Value>, VmError> {
        self.instructions = instructions;
        self.instruction_pointer = 0;
        self.state = VmState::Running;

        while self.instruction_pointer < self.instructions.len() {
            if let Err(err) = self.execute_instruction() {
                self.state = VmState::Failed(err.clone());
                return Err(err);
            }
        }

        self.state = VmState::Completed;
        
        // 返回栈顶值，如果栈为空则返回null
        if let Some(value) = self.value_stack.pop() {
            Ok(value)
        } else {
            Ok(self.heap.allocate(Value::Null))
        }
    }

    /// 执行单条指令
    fn execute_instruction(&mut self) -> Result<(), VmError> {
        if self.value_stack.len() > self.max_stack_depth {
            return Err(VmError::StackOverflow);
        }

        if self.call_stack.len() > self.max_call_depth {
            return Err(VmError::StackOverflow);
        }

        let instruction = self.instructions[self.instruction_pointer].clone();
        self.instruction_pointer += 1;

        match instruction {
            Instruction::PushConstant { value } => {
                let gc_value = self.heap.allocate(value);
                self.value_stack.push(gc_value);
            }
            Instruction::PushVariable { name } => {
                // 先从当前环境查找变量
                let value = if let Some(env) = self.environment_stack.last() {
                    if let Ok(env_map) = env.deref(&self.heap) {
                        env_map.get(&name).cloned()
                    } else {
                        None
                    }
                } else {
                    None
                };

                // 如果当前环境没有，则从全局环境查找
                let value = if value.is_none() {
                    if let Ok(globals) = self.globals.deref(&self.heap) {
                        globals.get(&name).cloned()
                    } else {
                        None
                    }
                } else {
                    value
                };

                if let Some(value) = value {
                    self.value_stack.push(value);
                } else {
                    return Err(VmError::UndefinedVariable(name));
                }
            }
            Instruction::StoreVariable { name } => {
                if self.value_stack.is_empty() {
                    return Err(VmError::StackUnderflow);
                }

                let value = self.value_stack.pop().unwrap();

                // 先检查当前环境
                if let Some(env) = self.environment_stack.last() {
                    if let Ok(mut env_map) = env.deref(&self.heap) {
                        env_map.insert(name, value);
                        return Ok(());
                    }
                }

                // 如果当前环境不存在，则存储到全局环境
                if let Ok(mut globals) = self.globals.deref(&self.heap) {
                    globals.insert(name, value);
                }
            }
            Instruction::Call { argument_count } => {
                if self.value_stack.len() < argument_count + 1 {
                    return Err(VmError::StackUnderflow);
                }

                // 获取参数
                let mut args = Vec::new();
                for _ in 0..argument_count {
                    args.push(self.value_stack.pop().unwrap());
                }
                args.reverse(); // 反转参数顺序

                // 获取函数
                let function_value = self.value_stack.pop().unwrap();
                let function = match function_value.deref(&self.heap)? {
                    Value::Function(f) => f.clone(),
                    other => return Err(VmError::NotCallable(other.type_name())),
                };

                // 检查参数数量
                if let Ok(func) = function.deref(&self.heap) {
                    if func.parameters.len() != argument_count {
                        return Err(VmError::ArgumentCountMismatch {
                            expected: func.parameters.len(),
                            found: argument_count,
                        });
                    }

                    // 保存当前执行状态
                    let current_ip = self.instruction_pointer;
                    let current_instructions = self.instructions.clone();

                    // 创建新的环境
                    let mut new_env = HashMap::new();
                    for (param, arg) in func.parameters.iter().zip(args) {
                        new_env.insert(param.clone(), arg);
                    }

                    // 将闭包环境合并到新环境
                    if let Ok(closure_env) = func.environment.deref(&self.heap) {
                        for (name, value) in closure_env.iter() {
                            new_env.insert(name.clone(), value.clone());
                        }
                    }

                    let new_env_gc = self.heap.allocate(Value::Object(Gc {
                        index: 0,
                        phantom: std::marker::PhantomData,
                    }));

                    // 切换到新环境和函数体
                    self.environment_stack.push(new_env_gc);
                    self.call_stack.push(function);
                    self.instructions = func.body.clone();
                    self.instruction_pointer = 0;

                    // 执行函数体
                    while self.instruction_pointer < self.instructions.len() {
                        let current_instruction = self.instructions[self.instruction_pointer].clone();
                        if let Instruction::Return = current_instruction {
                            break;
                        }
                        if let Err(err) = self.execute_instruction() {
                            // 恢复执行状态
                            self.instructions = current_instructions;
                            self.instruction_pointer = current_ip;
                            self.environment_stack.pop();
                            self.call_stack.pop();
                            return Err(err);
                        }
                    }

                    // 恢复执行状态
                    self.instructions = current_instructions;
                    self.instruction_pointer = current_ip;
                    self.environment_stack.pop();
                    self.call_stack.pop();
                }
            }
            Instruction::CreateFunction { name, parameter_count, body_size } => {
                // 获取函数体
                let mut body = Vec::new();
                let start = self.instruction_pointer;
                let end = start + body_size;

                if end > self.instructions.len() {
                    return Err(VmError::InvalidJumpTarget);
                }

                for i in start..end {
                    body.push(self.instructions[i].clone());
                }

                // 跳过函数体
                self.instruction_pointer = end;

                // 获取参数名称
                let mut parameters = Vec::new();
                for _ in 0..parameter_count {
                    if self.value_stack.is_empty() {
                        return Err(VmError::StackUnderflow);
                    }
                    let param_value = self.value_stack.pop().unwrap();
                    if let Ok(Value::String(s)) = param_value.deref(&self.heap) {
                        parameters.push(s.to_string());
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "string",
                            found: "non-string",
                        });
                    }
                }
                parameters.reverse(); // 反转参数顺序

                // 创建当前环境的快照作为闭包环境
                let mut environment = HashMap::new();
                if let Some(env) = self.environment_stack.last() {
                    if let Ok(env_map) = env.deref(&self.heap) {
                        for (name, value) in env_map.iter() {
                            environment.insert(name.clone(), value.clone());
                        }
                    }
                }

                let env_gc = self.heap.allocate(Value::Object(Gc {
                    index: 0,
                    phantom: std::marker::PhantomData,
                }));

                // 创建函数对象
                let function = Function {
                    name,
                    parameters,
                    body,
                    environment: env_gc,
                };

                let function_gc = self.heap.allocate(Value::Function(Gc {
                    index: 0,
                    phantom: std::marker::PhantomData,
                }));

                self.value_stack.push(function_gc);
            }
            Instruction::CreateCoroutine => {
                if self.value_stack.is_empty() {
                    return Err(VmError::StackUnderflow);
                }

                let function_value = self.value_stack.pop().unwrap();
                let function = match function_value.deref(&self.heap)? {
                    Value::Function(f) => f.clone(),
                    other => return Err(VmError::NotCallable(other.type_name())),
                };

                // 创建协程对象
                let coroutine = Coroutine {
                    state: CoroutineState::Initial,
                    function,
                    instruction_pointer: 0,
                    value_stack: Vec::new(),
                    call_stack: Vec::new(),
                    environment_stack: Vec::new(),
                    effect_handlers: Vec::new(),
                };

                let coroutine_gc = self.heap.allocate(Value::Coroutine(Gc {
                    index: 0,
                    phantom: std::marker::PhantomData,
                }));

                self.value_stack.push(coroutine_gc);
            }
            Instruction::ResumeCoroutine => {
                if self.value_stack.is_empty() {
                    return Err(VmError::StackUnderflow);
                }

                let coroutine_value = self.value_stack.pop().unwrap();
                let coroutine = match coroutine_value.deref(&self.heap)? {
                    Value::Coroutine(c) => c.clone(),
                    other => return Err(VmError::TypeMismatch {
                        expected: "coroutine",
                        found: other.type_name(),
                    }),
                };

                // 保存当前执行状态
                let current_state = self.save_execution_state();

                // 恢复协程执行状态
                if let Ok(co) = coroutine.deref(&self.heap) {
                    if co.state == CoroutineState::Completed || co.state == CoroutineState::Failed {
                        return Err(VmError::CoroutineError("Cannot resume completed or failed coroutine".to_string()));
                    }

                    // 加载协程状态
                    self.load_coroutine_state(co);

                    // 执行协程直到暂停或完成
                    self.execute_coroutine()?;

                    // 保存协程状态
                    self.save_coroutine_state(&coroutine);

                    // 恢复当前执行状态
                    self.restore_execution_state(current_state);

                    // 如果协程完成，将结果压入栈
                    if let Ok(updated_co) = coroutine.deref(&self.heap) {
                        if updated_co.state == CoroutineState::Completed && !updated_co.value_stack.is_empty() {
                            self.value_stack.push(updated_co.value_stack[0].clone());
                        }
                    }
                }
            }
            Instruction::YieldCoroutine { value_count } => {
                if self.value_stack.len() < value_count {
                    return Err(VmError::StackUnderflow);
                }

                // 如果不在协程中执行，则报错
                if self.current_coroutine.is_none() {
                    return Err(VmError::CoroutineError("Cannot yield outside of a coroutine".to_string()));
                }

                // 标记当前协程为暂停状态
                if let Some(coroutine) = &self.current_coroutine {
                    if let Ok(mut co) = coroutine.deref(&self.heap) {
                        co.state = CoroutineState::Suspended;
                    }
                }

                // 暂停执行
                return Ok(());
            }
            Instruction::RaiseEffect { name, argument_count } => {
                if self.value_stack.len() < argument_count {
                    return Err(VmError::StackUnderflow);
                }

                // 查找效应处理器
                let handler = self.find_effect_handler(&name)?;

                // 获取参数
                let mut args = Vec::new();
                for _ in 0..argument_count {
                    args.push(self.value_stack.pop().unwrap());
                }
                args.reverse(); // 反转参数顺序

                // 保存当前恢复点
                if let Ok(mut handler_obj) = handler.deref(&self.heap) {
                    handler_obj.resume_point = Some(self.instruction_pointer);
                }

                // 调用处理器函数
                if let Ok(handler_obj) = handler.deref(&self.heap) {
                    let function = handler_obj.handler.clone();
                    
                    // 保存当前执行状态
                    let current_ip = self.instruction_pointer;
                    let current_instructions = self.instructions.clone();

                    // 创建新的环境
                    let mut new_env = HashMap::new();
                    if let Ok(func) = function.deref(&self.heap) {
                        for (param, arg) in func.parameters.iter().zip(args) {
                            new_env.insert(param.clone(), arg);
                        }

                        // 将闭包环境合并到新环境
                        if let Ok(closure_env) = func.environment.deref(&self.heap) {
                            for (name, value) in closure_env.iter() {
                                new_env.insert(name.clone(), value.clone());
                            }
                        }

                        let new_env_gc = self.heap.allocate(Value::Object(Gc {
                            index: 0,
                            phantom: std::marker::PhantomData,
                        }));

                        // 切换到新环境和函数体
                        self.environment_stack.push(new_env_gc);
                        self.call_stack.push(function);
                        self.instructions = func.body.clone();
                        self.instruction_pointer = 0;

                        // 执行处理器函数
                        while self.instruction_pointer < self.instructions.len() {
                            let current_instruction = self.instructions[self.instruction_pointer].clone();
                            if let Instruction::ResumeEffect { .. } = current_instruction {
                                break;
                            }
                            if let Err(err) = self.execute_instruction() {
                                // 恢复执行状态
                                self.instructions = current_instructions;
                                self.instruction_pointer = current_ip;
                                self.environment_stack.pop();
                                self.call_stack.pop();
                                return Err(err);
                            }
                        }

                        // 恢复执行状态
                        self.instructions = current_instructions;
                        self.instruction_pointer = current_ip;
                        self.environment_stack.pop();
                        self.call_stack.pop();
                    }
                }
            }
            Instruction::HandleEffect { name } => {
                if self.value_stack.is_empty() {
                    return Err(VmError::StackUnderflow);
                }

                let handler_value = self.value_stack.pop().unwrap();
                let handler_function = match handler_value.deref(&self.heap)? {
                    Value::Function(f) => f.clone(),
                    other => return Err(VmError::NotCallable(other.type_name())),
                };

                // 创建效应处理器
                let handler = EffectHandler {
                    name: name.clone(),
                    handler: handler_function,
                    resume_point: None,
                };

                let handler_gc = self.heap.allocate(Value::Object(Gc {
                    index: 0,
                    phantom: std::marker::PhantomData,
                }));

                // 添加到效应处理器栈
                self.effect_handlers.push(handler_gc);
            }
            Instruction::ResumeEffect { value_count } => {
                if self.value_stack.len() < value_count {
                    return Err(VmError::StackUnderflow);
                }

                // 获取恢复值
                let mut values = Vec::new();
                for _ in 0..value_count {
                    values.push(self.value_stack.pop().unwrap());
                }
                values.reverse(); // 反转值顺序

                // 获取当前效应处理器
                if let Some(handler) = self.effect_handlers.last() {
                    if let Ok(handler_obj) = handler.deref(&self.heap) {
                        if let Some(resume_point) = handler_obj.resume_point {
                            // 恢复到效应触发点
                            self.instruction_pointer = resume_point;

                            // 将恢复值压入栈
                            for value in values {
                                self.value_stack.push(value);
                            }

                            // 移除当前效应处理器
                            self.effect_handlers.pop();
                            return Ok(());
                        }
                    }
                }

                return Err(VmError::RuntimeError("No effect to resume".to_string()));
            }
            Instruction::Jump { offset } => {
                let new_ip = (self.instruction_pointer as isize + offset) as usize;
                if new_ip >= self.instructions.len() {
                    return Err(VmError::InvalidJumpTarget);
                }
                self.instruction_pointer = new_ip;
            }
            Instruction::JumpIfFalse { offset } => {
                if self.value_stack.is_empty() {
                    return Err(VmError::StackUnderflow);
                }

                let condition = self.value_stack.pop().unwrap();
                if let Ok(value) = condition.deref(&self.heap) {
                    if !value.is_truthy() {
                        let new_ip = (self.instruction_pointer as isize + offset) as usize;
                        if new_ip >= self.instructions.len() {
                            return Err(VmError::InvalidJumpTarget);
                        }
                        self.instruction_pointer = new_ip;
                    }
                }
            }
            Instruction::Return => {
                // 在函数调用中处理
                return Ok(());
            }
            // 其他指令的实现...
            _ => {
                // 暂时不实现的指令
                return Err(VmError::RuntimeError(format!("Instruction not implemented: {:?}", instruction)));
            }
        }

        Ok(())
    }

    /// 保存当前执行状态
    fn save_execution_state(&self) -> ExecutionState {
        ExecutionState {
            instruction_pointer: self.instruction_pointer,
            instructions: self.instructions.clone(),
            value_stack: self.value_stack.clone(),
            call_stack: self.call_stack.clone(),
            environment_stack: self.environment_stack.clone(),
            effect_handlers: self.effect_handlers.clone(),
        }
    }

    /// 恢复执行状态
    fn restore_execution_state(&mut self, state: ExecutionState) {
        self.instruction_pointer = state.instruction_pointer;
        self.instructions = state.instructions;
        self.value_stack = state.value_stack;
        self.call_stack = state.call_stack;
        self.environment_stack = state.environment_stack;
        self.effect_handlers = state.effect_handlers;
    }

    /// 加载协程状态
    fn load_coroutine_state(&mut self, coroutine: &Coroutine) {
        self.current_coroutine = Some(Gc {
            index: 0, // 这里需要实际的索引
            phantom: std::marker::PhantomData,
        });
        self.instruction_pointer = coroutine.instruction_pointer;
        if let Ok(func) = coroutine.function.deref(&self.heap) {
            self.instructions = func.body.clone();
        }
        self.value_stack = coroutine.value_stack.clone();
        self.call_stack = coroutine.call_stack.clone();
        self.environment_stack = coroutine.environment_stack.clone();
        self.effect_handlers = coroutine.effect_handlers.clone();
    }

    /// 保存协程状态
    fn save_coroutine_state(&self, coroutine: &Gc<Coroutine>) {
        if let Ok(mut co) = coroutine.deref(&self.heap) {
            co.instruction_pointer = self.instruction_pointer;
            co.value_stack = self.value_stack.clone();
            co.call_stack = self.call_stack.clone();
            co.environment_stack = self.environment_stack.clone();
            co.effect_handlers = self.effect_handlers.clone();
        }
    }

    /// 执行协程
    fn execute_coroutine(&mut self) -> Result<(), VmError> {
        if let Some(coroutine) = &self.current_coroutine {
            if let Ok(mut co) = coroutine.deref(&self.heap) {
                co.state = CoroutineState::Running;
            }
        }

        while self.instruction_pointer < self.instructions.len() {
            let instruction = self.instructions[self.instruction_pointer].clone();
            
            // 如果是YieldCoroutine指令，暂停执行
            if let Instruction::YieldCoroutine { .. } = instruction {
                if let Err(err) = self.execute_instruction() {
                    if let Some(coroutine) = &self.current_coroutine {
                        if let Ok(mut co) = coroutine.deref(&self.heap) {
                            co.state = CoroutineState::Failed;
                        }
                    }
                    return Err(err);
                }
                return Ok(());
            }
            
            // 如果是Return指令，完成执行
            if let Instruction::Return = instruction {
                if let Some(coroutine) = &self.current_coroutine {
                    if let Ok(mut co) = coroutine.deref(&self.heap) {
                        co.state = CoroutineState::Completed;
                    }
                }
                return Ok(());
            }
            
            if let Err(err) = self.execute_instruction() {
                if let Some(coroutine) = &self.current_coroutine {
                    if let Ok(mut co) = coroutine.deref(&self.heap) {
                        co.state = CoroutineState::Failed;
                    }
                }
                return Err(err);
            }
        }

        // 如果执行完所有指令，标记为完成
        if let Some(coroutine) = &self.current_coroutine {
            if let Ok(mut co) = coroutine.deref(&self.heap) {
                co.state = CoroutineState::Completed;
            }
        }

        Ok(())
    }

    /// 查找效应处理器
    fn find_effect_handler(&self, name: &str) -> Result<Gc<EffectHandler>, VmError> {
        for handler in self.effect_handlers.iter().rev() {
            if let Ok(h) = handler.deref(&self.heap) {
                if h.name == name {
                    return Ok(handler.clone());
                }
            }
        }
        Err(VmError::UnhandledEffect(name.to_string()))
    }
}

/// 执行状态，用于保存和恢复执行上下文
#[derive(Debug, Clone)]
struct ExecutionState {
    instruction_pointer: usize,
    instructions: Vec<Instruction>,
    value_stack: Vec<Gc<Value>>,
    call_stack: Vec<Gc<Function>>,
    environment_stack: Vec<Gc<HashMap<String, Gc<Value>>>>,
    effect_handlers: Vec<Gc<EffectHandler>>,
}