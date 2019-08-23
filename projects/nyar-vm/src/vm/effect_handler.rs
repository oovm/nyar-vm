//! 效应处理器模块，负责处理代数效应

use nyar_error::NyarError;
use nyar_lir::{Gc, NyarValue, NyarHandler, NyarFunction};

use super::VirtualMachine;

/// 效应处理器，负责处理代数效应
#[derive(Debug)]
pub struct EffectHandler {
    /// 当前注册的效应处理器
    registered_handlers: Vec<(String, Gc<NyarFunction>)>,
}

impl EffectHandler {
    /// 创建一个新的效应处理器
    pub fn new() -> Self {
        Self {
            registered_handlers: Vec::new(),
        }
    }

    /// 注册效应处理器
    pub fn register_handler(&mut self, name: String, handler: Gc<NyarFunction>) {
        self.registered_handlers.push((name, handler));
    }

    /// 触发效应
    pub fn raise_effect(&mut self, vm: &mut VirtualMachine, name: &str, argument_count: usize) -> Result<(), NyarError> {
        // 检查是否有处理该效应的处理器
        let handler_index = self.find_handler(name);
        
        if handler_index.is_none() {
            return Err(NyarError::custom(format!("未处理的效应: {}", name)));
        }
        
        // 获取参数
        let mut args = Vec::with_capacity(argument_count);
        for _ in 0..argument_count {
            if let Some(arg) = vm.value_stack.pop() {
                args.push(arg);
            } else {
                return Err(NyarError::custom("栈为空，无法获取效应参数".to_string()));
            }
        }
        args.reverse(); // 反转参数顺序
        
        // 保存当前执行状态，以便稍后恢复
        let resume_point = vm.instruction_pointer + 1; // 效应处理完后应该继续执行的指令
        
        // 创建效应处理器对象
        let (_, handler_func) = &self.registered_handlers[handler_index.unwrap()];
        let handler = NyarHandler {
            name: name.to_string(),
            handler: handler_func.clone(),
            resume_point: Some(resume_point),
        };
        
        // 将处理器压入效应处理器栈
        let handler_gc = vm.memory_manager.allocate(NyarValue::Handler(Box::new(handler)));
        
        // 调用处理器函数
        // 这里简化处理，实际上需要更复杂的逻辑
        
        Ok(())
    }

    /// 处理效应
    pub fn handle_effect(&mut self, vm: &mut VirtualMachine, name: &str) -> Result<(), NyarError> {
        // 从栈上获取处理器函数
        let handler_func = vm.value_stack.pop()
            .ok_or_else(|| NyarError::custom("栈为空，无法获取效应处理器".to_string()))?;
        
        match &*handler_func {
            NyarValue::Function(func) => {
                // 注册效应处理器
                self.register_handler(name.to_string(), Gc::new(func.as_ref().clone()));
                Ok(())
            },
            _ => Err(NyarError::custom(format!("类型错误: {} 不是函数，无法作为效应处理器", handler_func.type_name())))
        }
    }

    /// 恢复效应
    pub fn resume_effect(&mut self, vm: &mut VirtualMachine, value_count: usize) -> Result<(), NyarError> {
        // 获取要返回的值
        let mut resume_values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            if let Some(value) = vm.value_stack.pop() {
                resume_values.push(value);
            } else {
                return Err(NyarError::custom("栈为空，无法获取恢复值".to_string()));
            }
        }
        resume_values.reverse(); // 反转顺序
        
        // 获取当前效应处理器
        let handler = vm.value_stack.pop()
            .ok_or_else(|| NyarError::custom("栈为空，无法获取效应处理器".to_string()))?;
        
        match &*handler {
            NyarValue::Handler(h) => {
                if let Some(resume_point) = h.resume_point {
                    // 恢复到效应触发点
                    vm.instruction_pointer = resume_point;
                    
                    // 将恢复值放入栈上
                    for value in resume_values {
                        vm.value_stack.push(value);
                    }
                    
                    Ok(())
                } else {
                    Err(NyarError::custom("效应处理器没有恢复点".to_string()))
                }
            },
            _ => Err(NyarError::custom(format!("类型错误: {} 不是效应处理器", handler.type_name())))
        }
    }

    /// 查找处理指定效应的处理器
    fn find_handler(&self, name: &str) -> Option<usize> {
        self.registered_handlers.iter()
            .position(|(handler_name, _)| handler_name == name)
    }
}