//! 值类型模块，定义了VM支持的所有值类型

use crate::heap::Gc;
pub use self::objects::NyarObject;
pub use self::vectors::NyarVector;
use indexmap::IndexMap;
use num::BigInt;
use std::{collections::HashMap, fmt::Debug};
use std::collections::VecDeque;

mod objects;
mod vectors;

/// VM支持的所有值类型
#[derive(Debug, Clone)]
pub enum NyarValue {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 大整数
    Integer(Box<BigInt>),
    /// 字符串，存储在GC堆上
    String(Box<String>),
    /// 数组，存储在GC堆上
    Vector(Box<NyarVector>),
    /// 对象，存储在GC堆上
    Object(Box<NyarObject>),
    /// 函数，包含函数体和闭包环境
    Function(Box<NyarFunction>),
    /// 类定义
    Class(Box<NyarClass>),
    /// 特征/接口定义
    Trait(Box<NyarTrait>),
    /// 枚举定义
    Enum(Box<NyarEnum>),
    /// 协程
    Coroutine(Box<NyarCoroutine>),
    /// Effect handler
    Handler(Box<NyarHandler>),
}

impl NyarValue {
    /// 获取值的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            NyarValue::Null => "null",
            NyarValue::Boolean(_) => "boolean",
            NyarValue::Integer(_) => "bigint",
            NyarValue::String(_) => "string",
            NyarValue::Vector(_) => "array",
            NyarValue::Object(_) => "object",
            NyarValue::Function(_) => "function",
            NyarValue::Class(_) => "class",
            NyarValue::Trait(_) => "trait",
            NyarValue::Enum(_) => "enum",
            NyarValue::Coroutine(_) => "coroutine",
            NyarValue::Handler(_) => "handler",
        }
    }

    /// 判断值是否为空
    pub fn is_null(&self) -> bool {
        matches!(self, NyarValue::Null)
    }
}

/// 函数定义，包含函数体和闭包环境
#[derive(Debug, Clone)]
pub struct NyarFunction {
    /// 函数名称, maybe None for lambda(anonymous function)
    pub name: Option<String>,
    /// 参数列表
    pub parameters: Vec<String>,
    /// 函数体指令
    pub body: Vec<crate::instruction::Instruction>,
    /// 闭包环境
    pub environment: Gc<HashMap<String, Gc<NyarValue>>>,
}

/// 类定义
#[derive(Debug, Clone)]
pub struct NyarClass {
    /// 类名称
    pub name: String,
    /// 父类
    pub parent: Option<Gc<NyarClass>>,
    /// 实现的特征
    pub traits: Vec<Gc<NyarTrait>>,
    /// 方法
    pub methods: HashMap<String, Gc<NyarFunction>>,
    /// 属性
    pub properties: HashMap<String, Gc<NyarValue>>,
}

/// 特征/接口定义
#[derive(Debug, Clone)]
pub struct NyarTrait {
    /// 特征名称
    pub name: String,
    /// 方法签名
    pub methods: HashMap<String, Vec<String>>,
}

/// 枚举定义
#[derive(Debug, Clone)]
pub struct NyarEnum {
    /// 枚举名称
    pub name: String,
    /// 变体
    pub variants: HashMap<String, Gc<NyarValue>>,
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
pub struct NyarCoroutine {
    /// 协程状态
    pub state: CoroutineState,
    /// 协程函数
    pub function: Gc<NyarFunction>,
    /// 当前指令指针
    pub instruction_pointer: usize,
    /// 当前值栈
    pub value_stack: Vec<Gc<NyarValue>>,
    /// 当前调用栈
    pub call_stack: Vec<Gc<NyarFunction>>,
    /// 当前环境栈
    pub environment_stack: Vec<Gc<HashMap<String, Gc<NyarValue>>>>,
    /// 当前效应处理器栈
    pub effect_handlers: Vec<Gc<NyarHandler>>,
}

/// 效应处理器
#[derive(Debug, Clone)]
pub struct NyarHandler {
    /// 效应名称
    pub name: String,
    /// 处理函数
    pub handler: Gc<NyarFunction>,
    /// 恢复点
    pub resume_point: Option<usize>,
}
