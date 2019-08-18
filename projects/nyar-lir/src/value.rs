//! Nyar语言的值类型系统
//!
//! 实现了Nyar语言的基本值类型，包括null、bool、bigint和string，
//! 以及使用gc-arena进行内存管理的对象引用。

use gc_arena::{Arena, Collect, Gc, GcCell, Mutation, MutationContext};
use num::BigInt;
use std::fmt;

/// Nyar语言的值类型
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum Value<'gc> {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 大整数
    Integer(BigInt),
    /// 字符串
    String(Gc<'gc, String>),
    /// 对象引用
    Object(Gc<'gc, GcCell<'gc, Object<'gc>>>),
    /// 数组
    Array(Gc<'gc, GcCell<'gc, Vec<Value<'gc>>>>),
    /// 函数引用
    Function(Gc<'gc, Function<'gc>>),
}

/// 对象结构
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Object<'gc> {
    /// 类型名称
    pub class_name: Gc<'gc, String>,
    /// 属性表
    pub properties: std::collections::HashMap<String, Value<'gc>>,
}

/// 函数结构
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Function<'gc> {
    /// 函数名称
    pub name: Gc<'gc, String>,
    /// 函数体指令
    pub instructions: Vec<crate::instruction::Instruction>,
    /// 捕获的上下文
    pub captures: std::collections::HashMap<String, Value<'gc>>,
}

/// 值类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Boolean,
    Integer,
    String,
    Object,
    Array,
    Function,
}

impl<'vm> Value<'vm> {
    /// 获取值的类型
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Integer(_) => ValueType::Integer,
            Value::String(_) => ValueType::String,
            Value::Object(_) => ValueType::Object,
            Value::Array(_) => ValueType::Array,
            Value::Function(_) => ValueType::Function,
        }
    }

    /// 创建一个新的对象
    pub fn new_object(mc: Mutation<'vm>, class_name: &str) -> Self {
        let obj = Object { class_name: Gc::new(&mc, class_name.to_string()), properties: std::collections::HashMap::new() };
        Value::Object(Gc::new(&mc, GcCell::allocate(mc, obj)))
    }

    /// 创建一个新的数组
    pub fn new_array(mc: Mutation<'vm>) -> Self {
        Value::Array(Gc::new(&mc, GcCell::allocate(mc, Vec::new())))
    }
}

impl<'gc> fmt::Debug for Value<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Object(_) => write!(f, "<object>"),
            Value::Array(_) => write!(f, "<array>"),
            Value::Function(func) => write!(f, "<function: {}>", func.name),
        }
    }
}
