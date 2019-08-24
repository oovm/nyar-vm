//! 效应处理器模块，负责处理代数效应

use super::VirtualMachine;
use nyar_error::{NyarError, Result};
use nyar_lir::{Gc, Heap, NyarFunction, NyarHandler, NyarValue, values::NyarObject};

/// 效应处理器，负责处理代数效应
#[derive(Debug)]
pub struct EffectHandler {
    /// Registered Handlers
    handlers: Gc<NyarObject>,
}

impl EffectHandler {
   
}
