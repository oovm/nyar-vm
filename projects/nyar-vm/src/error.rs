//! 错误处理模块

use std::fmt::{Display, Formatter};

/// 虚拟机错误类型
#[derive(Debug, Clone)]
pub enum VmError {
    /// 类型不匹配
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },
    /// 未定义的变量
    UndefinedVariable(String),
    /// 未定义的属性
    UndefinedProperty {
        object_type: &'static str,
        property: String,
    },
    /// 索引越界
    IndexOutOfBounds {
        index: usize,
        length: usize,
    },
    /// 无效的GC索引
    InvalidGcIndex(usize),
    /// 访问已死亡对象
    DeadObjectAccess(usize),
    /// 调用非函数值
    NotCallable(&'static str),
    /// 参数数量不匹配
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
    },
    /// 未处理的效应
    UnhandledEffect(String),
    /// 无效的跳转目标
    InvalidJumpTarget,
    /// 无效的标签
    InvalidLabel(String),
    /// 栈溢出
    StackOverflow,
    /// 栈下溢
    StackUnderflow,
    /// 协程错误
    CoroutineError(String),
    /// 运行时错误
    RuntimeError(String),
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::TypeMismatch { expected, found } => {
                write!(f, "类型不匹配: 期望 {}, 实际为 {}", expected, found)
            }
            VmError::UndefinedVariable(name) => {
                write!(f, "未定义的变量: {}", name)
            }
            VmError::UndefinedProperty { object_type, property } => {
                write!(f, "未定义的属性: {} 上不存在属性 {}", object_type, property)
            }
            VmError::IndexOutOfBounds { index, length } => {
                write!(f, "索引越界: 索引 {} 超出长度 {}", index, length)
            }
            VmError::InvalidGcIndex(index) => {
                write!(f, "无效的GC索引: {}", index)
            }
            VmError::DeadObjectAccess(index) => {
                write!(f, "访问已死亡对象: {}", index)
            }
            VmError::NotCallable(type_name) => {
                write!(f, "不可调用: {} 不是函数", type_name)
            }
            VmError::ArgumentCountMismatch { expected, found } => {
                write!(f, "参数数量不匹配: 期望 {}, 实际为 {}", expected, found)
            }
            VmError::UnhandledEffect(name) => {
                write!(f, "未处理的效应: {}", name)
            }
            VmError::InvalidJumpTarget => {
                write!(f, "无效的跳转目标")
            }
            VmError::InvalidLabel(label) => {
                write!(f, "无效的标签: {}", label)
            }
            VmError::StackOverflow => {
                write!(f, "栈溢出")
            }
            VmError::StackUnderflow => {
                write!(f, "栈下溢")
            }
            VmError::CoroutineError(msg) => {
                write!(f, "协程错误: {}", msg)
            }
            VmError::RuntimeError(msg) => {
                write!(f, "运行时错误: {}", msg)
            }
        }
    }
}

impl std::error::Error for VmError {}