//! 虚拟机核心模块，实现指令执行和协程调度

mod value_handler;
mod instruction_executor;
mod memory_manager;
mod environment;
mod coroutine;
mod effect_handler;

use nyar_error::NyarError;
use nyar_lir::{Gc, Heap, Instruction, NyarValue};

pub use self::{
    value_handler::ValueHandler,
    instruction_executor::InstructionExecutor,
    memory_manager::MemoryManager,
    environment::Environment,
    coroutine::CoroutineManager,
    effect_handler::EffectHandler,
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
    memory_manager: MemoryManager,
    /// 指令执行器
    instruction_executor: InstructionExecutor,
    /// 值处理器
    value_handler: ValueHandler,
    /// 环境管理
    environment: Environment,
    /// 协程管理器
    coroutine_manager: CoroutineManager,
    /// 效应处理器
    effect_handler: EffectHandler,
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

impl VirtualMachine {
    /// 创建一个新的虚拟机实例
    pub fn new() -> Self {
        Self {
            memory_manager: MemoryManager::new(),
            instruction_executor: InstructionExecutor::new(),
            value_handler: ValueHandler::new(),
            environment: Environment::new(),
            coroutine_manager: CoroutineManager::new(),
            effect_handler: EffectHandler::new(),
            state: VmState::Initial,
            value_stack: Vec::new(),
            instruction_pointer: 0,
            max_stack_depth: 1024,
            max_call_depth: 128,
        }
    }

    /// 执行指令序列
    pub fn execute(&mut self, instructions: Vec<Instruction>) -> Result<Gc<NyarValue>, NyarError> {
        self.state = VmState::Running;
        self.instruction_pointer = 0;
        
        while self.instruction_pointer < instructions.len() {
            let instruction = &instructions[self.instruction_pointer];
            if let Err(e) = self.instruction_executor.execute_instruction(self, instruction) {
                self.state = VmState::Failed(e.clone());
                return Err(e);
            }
            self.instruction_pointer += 1;
        }

        self.state = VmState::Completed;
        
        // 返回栈顶值，如果栈为空则返回空值
        Ok(self.value_stack.pop().unwrap_or_else(|| self.memory_manager.allocate(NyarValue::Null)))
    }
}

/// 执行状态，用于保存和恢复执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionState {
    instruction_pointer: usize,
    instructions: Vec<Instruction>,
    value_stack: Vec<Gc<NyarValue>>,
}