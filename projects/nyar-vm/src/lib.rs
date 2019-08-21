//! Nyar VM 是一个支持多种高级语言特性的AST解释器

mod value;
mod heap;
mod instruction;
mod vm;
mod error;

pub use value::*;
pub use heap::*;
pub use instruction::*;
pub use vm::*;
pub use error::*;