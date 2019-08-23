//! Nyar Low-level Intermediate Representation
//!
//! 这个模块实现了Nyar语言的低级中间表示，使用栈机模型执行指令。

mod heap;
mod instruction;
pub mod values;

pub use crate::{
    heap::{Gc, Heap},
    instruction::Instruction,
    values::{CoroutineState, NyarCoroutine, NyarFunction, NyarHandler, NyarValue},
};
