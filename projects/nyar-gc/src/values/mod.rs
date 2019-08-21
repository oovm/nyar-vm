mod converts;

use crate::{Gc, VmError};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Number(i32),
    String(Box<str>),
    List(Vec<Gc<Value>>),
}

// Helper for tests
impl Value {
    pub fn as_number(&self) -> Option<i32> {
        if let Value::Number(n) = self { Some(*n) } else { None }
    }
    /// 获取所有直接引用对象
    pub fn gc_trace(&self) -> Vec<usize> {
        match self {
            Value::List(l) => l.iter().map(|v| v.pointer).collect(),
            _ => vec![],
        }
    }
}
