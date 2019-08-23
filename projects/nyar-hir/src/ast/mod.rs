//! AST模块，定义了抽象语法树的结构

use nyar_error::NyarError;
use nyar_lir::{Gc, NyarValue};
use std::collections::HashMap;

mod class;
mod enum_def;
mod expression;
mod function;
mod statement;
mod trait_def;

pub use class::ClassDefinition;
pub use enum_def::EnumDefinition;
pub use expression::Expression;
pub use function::FunctionDefinition;
pub use statement::Statement;
pub use trait_def::TraitDefinition;

/// AST节点特征，所有AST节点都应实现此特征
pub trait AstNode {
    /// 将AST节点转换为LIR值
    fn to_lir(&self) -> Result<NyarValue, NyarError>;
}

/// 程序，由多个语句组成
#[derive(Debug, Clone)]
pub struct Program {
    /// 语句列表
    pub statements: Vec<Statement>,
}

impl Program {
    /// 创建一个新的程序
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    /// 添加语句
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl AstNode for Program {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 将程序转换为对象，包含所有语句的执行结果
        let mut results = Vec::new();
        
        for statement in &self.statements {
            let result = statement.to_lir()?;
            results.push(Gc::new(result));
        }
        
        Ok(NyarValue::List(results))
    }
}