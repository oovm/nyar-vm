//! 值类型模块，定义了VM支持的所有值类型

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use crate::error::VmError;
use crate::heap::{Gc, Heap};

/// VM支持的所有值类型
#[derive(Debug, Clone)]
pub enum Value {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 大整数
    BigInt(i64),
    /// 字符串，存储在GC堆上
    String(Gc<String>),
    /// 数组，存储在GC堆上
    Array(Vec<Gc<Value>>),
    /// 对象，存储在GC堆上
    Object(HashMap<String, Gc<Value>>),
    /// 函数，包含函数体和闭包环境
    Function(Gc<Function>),
    /// 类定义
    Class(Gc<Class>),
    /// 特征/接口定义
    Trait(Gc<Trait>),
    /// 枚举定义
    Enum(Gc<Enum>),
    /// 协程
    Coroutine(Gc<Coroutine>),
}

impl Value {
    /// 获取值的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::BigInt(_) => "bigint",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
            Value::Class(_) => "class",
            Value::Trait(_) => "trait",
            Value::Enum(_) => "enum",
            Value::Coroutine(_) => "coroutine",
        }
    }

    /// 判断值是否为空
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// 判断值是否为真
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::BigInt(n) => *n != 0,
            Value::String(s) => !s.to_string().is_empty(),
            _ => true,
        }
    }
}

/// 函数定义，包含函数体和闭包环境
#[derive(Debug, Clone)]
pub struct Function {
    /// 函数名称
    pub name: Option<String>,
    /// 参数列表
    pub parameters: Vec<String>,
    /// 函数体指令
    pub body: Vec<crate::instruction::Instruction>,
    /// 闭包环境
    pub environment: Gc<HashMap<String, Gc<Value>>>,
}

/// 类定义
#[derive(Debug, Clone)]
pub struct Class {
    /// 类名称
    pub name: String,
    /// 父类
    pub parent: Option<Gc<Class>>,
    /// 实现的特征
    pub traits: Vec<Gc<Trait>>,
    /// 方法
    pub methods: HashMap<String, Gc<Function>>,
    /// 属性
    pub properties: HashMap<String, Gc<Value>>,
}

/// 特征/接口定义
#[derive(Debug, Clone)]
pub struct Trait {
    /// 特征名称
    pub name: String,
    /// 方法签名
    pub methods: HashMap<String, Vec<String>>,
}

/// 枚举定义
#[derive(Debug, Clone)]
pub struct Enum {
    /// 枚举名称
    pub name: String,
    /// 变体
    pub variants: HashMap<String, Gc<Value>>,
}

/// 协程状态
#[derive(Debug, Clone, PartialEq)]
pub enum CoroutineState {
    /// 初始状态
    Initial,
    /// 运行中
    Running,
    /// 已暂停
    Suspended,
    /// 已完成
    Completed,
    /// 出错
    Failed,
}

/// 协程定义
#[derive(Debug, Clone)]
pub struct Coroutine {
    /// 协程状态
    pub state: CoroutineState,
    /// 协程函数
    pub function: Gc<Function>,
    /// 当前指令指针
    pub instruction_pointer: usize,
    /// 当前值栈
    pub value_stack: Vec<Gc<Value>>,
    /// 当前调用栈
    pub call_stack: Vec<Gc<Function>>,
    /// 当前环境栈
    pub environment_stack: Vec<Gc<HashMap<String, Gc<Value>>>>,
    /// 当前效应处理器栈
    pub effect_handlers: Vec<Gc<EffectHandler>>,
}

/// 效应处理器
#[derive(Debug, Clone)]
pub struct EffectHandler {
    /// 效应名称
    pub name: String,
    /// 处理函数
    pub handler: Gc<Function>,
    /// 恢复点
    pub resume_point: Option<usize>,
}

impl<'gc> TryFrom<&'gc Value> for &'gc HashMap<String, Gc<Value>> {
    type Error = VmError;

    fn try_from(value: &'gc Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(VmError::TypeMismatch {
                expected: "object",
                found: value.type_name(),
            }),
        }
    }
}

impl<'gc> TryFrom<&'gc Value> for &'gc Vec<Gc<Value>> {
    type Error = VmError;

    fn try_from(value: &'gc Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(a) => Ok(a.as_ref()),
            _ => Err(VmError::TypeMismatch {
                expected: "array",
                found: value.type_name(),
            }),
        }
    }
}

impl<'gc> TryFrom<&'gc Value> for Function {
    type Error = VmError;

    fn try_from(value: &'gc Value) -> Result<Self, Self::Error> {
        todo!()
    }
}