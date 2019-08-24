//! 虚拟机核心模块，实现指令执行和协程调度

mod coroutine;
mod effect_handler;
mod environment;
mod instruction_executor;
mod value_handler;

use nyar_error::NyarError;
use nyar_lir::{Gc, Heap, Instruction, NyarValue};

pub use self::{
    coroutine::CoroutineManager, effect_handler::EffectHandler, environment::Environment,
    instruction_executor::InstructionExecutor, value_handler::ValueHandler,
};

/// 虚拟机状态
#[derive(Debug, Clone, PartialEq)]
pub enum VmState {
    /// 初始状态
    Initial,
    /// 运行中
    Running,
    /// 已暂停
    Suspended,
    /// 已完成
    Completed,
    /// 出错
    Failed(NyarError),
}

/// 虚拟机结构体，负责执行指令和管理内存
#[derive(Debug)]
pub struct VirtualMachine {
    /// 堆内存管理器
    memory: Heap,
    /// 当前指令指针
    instruction_pointer: usize,
    /// 最大栈深度
    max_stack_depth: usize,
    /// 最大调用深度
    max_call_depth: usize,
}

impl VirtualMachine {
    /// 创建一个新的虚拟机实例
    pub fn new() -> Self {
        Self { memory: Heap::default(), instruction_pointer: 0, max_stack_depth: 1024, max_call_depth: 128 }
    }

    /// 执行指令序列
    pub fn execute(&mut self, instructions: Vec<Instruction>) -> Result<Gc<NyarValue>, NyarError> {
        todo!()
    }
}

/// 执行状态，用于保存和恢复执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionState {
    instruction_pointer: usize,
    instructions: Vec<Instruction>,
    value_stack: Vec<Gc<NyarValue>>,
}
