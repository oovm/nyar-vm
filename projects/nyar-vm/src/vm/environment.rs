//! 环境管理模块，负责管理变量环境和作用域

use std::collections::HashMap;
use nyar_error::NyarError;
use nyar_lir::{Gc, NyarValue};

/// 环境管理器，负责管理变量环境和作用域
#[derive(Debug)]
pub struct Environment {
    /// 环境栈，每个作用域一个环境
    environments: Vec<NyarObject>,
    /// 全局环境
    global_environment: Gc<NyarObject>,
}

impl Environment {
    /// 创建一个新的环境管理器
    pub fn new() -> Self {
        let global_env = Gc::new(HashMap::new());
        Self {
            environments: vec![global_env.clone()],
            global_environment: global_env,
        }
    }

    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Result<Option<Gc<NyarValue>>, NyarError> {
        // 从当前作用域向上查找变量
        for env in self.environments.iter().rev() {
            if let Some(value) = env.get(name) {
                return Ok(Some(value.clone()));
            }
        }
        
        // 变量未找到
        Ok(None)
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Gc<NyarValue>) {
        // 在当前作用域中设置变量
        if let Some(current_env) = self.environments.last_mut() {
            current_env.as_mut().insert(name, value);
        }
    }

    /// 定义全局变量
    pub fn define_global(&mut self, name: String, value: Gc<NyarValue>) {
        self.global_environment.as_mut().insert(name, value);
    }

    /// 进入新的作用域
    pub fn enter_scope(&mut self) {
        let new_env = Gc::new(HashMap::new());
        self.environments.push(new_env);
    }

    /// 退出当前作用域
    pub fn exit_scope(&mut self) -> Result<(), NyarError> {
        if self.environments.len() <= 1 {
            return Err(NyarError::custom("无法退出全局作用域".to_string()));
        }
        
        self.environments.pop();
        Ok(())
    }

    /// 创建闭包环境
    pub fn create_closure_environment(&self, captured_variables: &[String]) -> Gc<HashMap<String, Gc<NyarValue>>> {
        let mut closure_env = HashMap::new();
        
        // 捕获指定的变量
        for var_name in captured_variables {
            if let Ok(Some(value)) = self.get_variable(var_name) {
                closure_env.insert(var_name.clone(), value);
            }
        }
        
        Gc::new(closure_env)
    }
}