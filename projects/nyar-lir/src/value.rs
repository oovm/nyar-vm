//! Nyar语言的值类型系统
//!
//! 实现了Nyar语言的基本值类型，包括null、bool、bigint和string，
//! 以及使用gc-arena进行内存管理的对象引用。

use crate::instruction::NyarInstruction;
use gc_arena::{ Collect, Gc, Mutation};
use num::BigInt;
use std::{collections::HashMap, fmt};

/// Nyar语言的值类型
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum NyarValue<'gc> {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 大整数
    Integer(Gc<'gc, BigInt>),
    /// 字符串
    String(Gc<'gc, String>),
    /// 对象引用
    Object(Gc<'gc, NyarObject<'gc>>),
    /// 数组
    Array(Gc<'gc, Vec<NyarValue<'gc>>>),
    /// 函数引用
    Function(Gc<'gc, Function<'gc>>),
}

/// 对象结构
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct NyarObject<'gc> {
    /// 类型名称
    pub class_name: Gc<'gc, String>,
    /// 属性表
    pub properties: HashMap<String, NyarValue<'gc>>,
}

/// 函数结构
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Function<'gc> {
    /// 函数名称
    pub name: Gc<'gc, String>,
    /// 函数体指令
    pub instructions: Vec<NyarInstruction>,
    /// 捕获的上下文
    pub captures: HashMap<String, NyarValue<'gc>>,
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

impl<'vm> NyarValue<'vm> {
    /// 获取值的类型
    pub fn get_type(&self) -> ValueType {
        match self {
            NyarValue::Null => ValueType::Null,
            NyarValue::Boolean(_) => ValueType::Boolean,
            NyarValue::Integer(_) => ValueType::Integer,
            NyarValue::String(_) => ValueType::String,
            NyarValue::Object(_) => ValueType::Object,
            NyarValue::Array(_) => ValueType::Array,
            NyarValue::Function(_) => ValueType::Function,
        }
    }

    /// 创建一个新的对象
    pub fn new_object(mc: &Mutation<'vm>, class_name: &str) -> Self {
        let obj = NyarObject { class_name: Gc::new(mc, class_name.to_string()), properties: HashMap::new() };
        NyarValue::Object(Gc::new(mc, obj))
    }

    /// 创建一个新的数组
    pub fn new_array(mc: &Mutation<'vm>) -> Self {
        NyarValue::Array(Gc::new(mc, Vec::new()))
    }
}

impl<'gc> fmt::Debug for NyarValue<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NyarValue::Null => write!(f, "null"),
            NyarValue::Boolean(b) => write!(f, "{}", b),
            NyarValue::Integer(i) => write!(f, "{}", i),
            NyarValue::String(s) => write!(f, "\"{}\"", s),
            NyarValue::Object(_) => write!(f, "<object>"),
            NyarValue::Array(_) => write!(f, "<array>"),
            NyarValue::Function(func) => write!(f, "<function: {}>", func.name),
        }
    }
}
