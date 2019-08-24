//! 环境管理模块，负责管理变量环境和作用域

use nyar_lir::{Gc, values::NyarObject};

/// 环境管理器，负责管理变量环境和作用域
#[derive(Debug)]
pub struct Environment {
    /// 环境栈，每个作用域一个环境
    environments: Vec<Gc<NyarObject>>,
}

impl Environment {
    /// 创建一个新的环境管理器
    pub fn new(builtin: Gc<NyarObject>) -> Self {
        Self { environments: vec![builtin] }
    }
}
