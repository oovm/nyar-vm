//! Nyar Low-level Intermediate Representation
//!
//! 这个模块实现了Nyar语言的低级中间表示，使用栈机模型执行指令。

mod value;
mod instruction;
mod vm;
mod object;
mod control_flow;

pub use crate::{
    value::{NyarValue, ValueType},
    instruction::{NyarInstruction, OpCode},
    vm::{NyarVM, ExecutionContext},
    object::{Class, Trait, Enum, NyarObject},
    control_flow::{ControlFlow, NyarHandler},
};

/// Nyar-LIR 的结果类型
pub type Result<T> = std::result::Result<T, nyar_error::NyarError>;