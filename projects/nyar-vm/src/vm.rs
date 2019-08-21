//! 虚拟机核心模块，实现指令执行和协程调度

use nyar_error::NyarError;
use nyar_lir::{Gc, Heap, Instruction, NyarValue};

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
    /// 堆内存
    heap: Heap,
    /// 当前状态
    state: VmState,
    /// 当前值栈
    value_stack: Vec<Gc<NyarValue>>,
    /// 当前指令指针
    instruction_pointer: usize,
    /// 最大栈深度
    max_stack_depth: usize,
    /// 最大调用深度
    max_call_depth: usize,
}

/// 执行状态，用于保存和恢复执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionState {
    instruction_pointer: usize,
    instructions: Vec<Instruction>,
    value_stack: Vec<Gc<NyarValue>>,
}
