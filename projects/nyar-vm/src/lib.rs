//! Nyar虚拟机实现
//!
//! 整合nyar-error、nyar-hir和nyar-lir，提供完整的Nyar语言执行环境。

mod runtime;
mod compiler;
mod interpreter;

pub use crate::{
    runtime::Runtime,
    compiler::Compiler,
    interpreter::Interpreter,
};

/// Nyar虚拟机结果类型
pub type Result<T> = std::result::Result<T, nyar_error::NyarError>;

/// Nyar虚拟机
pub struct NyarVM {
    /// 运行时环境
    runtime: runtime::Runtime,
    /// 编译器
    compiler: compiler::Compiler,
    /// 解释器
    interpreter: interpreter::Interpreter,
}

impl NyarVM {
    /// 创建一个新的Nyar虚拟机
    pub fn new() -> Self {
        Self {
            runtime: runtime::Runtime::new(),
            compiler: compiler::Compiler::new(),
            interpreter: interpreter::Interpreter::new(),
        }
    }

    /// 执行Nyar代码
    pub fn execute(&mut self, source: &str) -> Result<nyar_lir::Value<'static>> {
        // 1. 解析源代码为HIR（假设已经由外部解析器完成）
        let hir = self.compiler.parse(source)?;
        
        // 2. 编译HIR为LIR指令
        let instructions = self.compiler.compile(&hir)?;
        
        // 3. 执行LIR指令
        self.interpreter.execute(instructions)
    }

    /// 执行Nyar文件
    pub fn execute_file(&mut self, file_path: &str) -> Result<nyar_lir::Value<'static>> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| nyar_error::NyarError::custom(format!("Failed to read file: {}", e)))?;
        self.execute(&source)
    }

    /// 获取运行时环境
    pub fn runtime(&self) -> &runtime::Runtime {
        &self.runtime
    }

    /// 获取运行时环境（可变）
    pub fn runtime_mut(&mut self) -> &mut runtime::Runtime {
        &mut self.runtime
    }
}

/// 默认实例
impl Default for NyarVM {
    fn default() -> Self {
        Self::new()
    }
}