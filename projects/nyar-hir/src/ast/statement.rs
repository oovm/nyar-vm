//! 语句模块，定义了各种语句类型

use crate::ast::{AstNode, Expression};
use nyar_error::NyarError;
use nyar_lir::{Gc, NyarValue};

/// 语句
#[derive(Debug, Clone)]
pub enum Statement {
    /// 表达式语句
    Expression(Expression),
    /// 变量声明
    VariableDeclaration(VariableDeclaration),
    /// 赋值语句
    Assignment(Assignment),
    /// 条件语句
    If(IfStatement),
    /// 循环语句
    Loop(LoopStatement),
    /// 返回语句
    Return(Option<Expression>),
    /// 块语句
    Block(Vec<Statement>),
    /// 函数定义
    FunctionDeclaration(crate::ast::FunctionDefinition),
    /// 类定义
    ClassDeclaration(crate::ast::ClassDefinition),
    /// 特征定义
    TraitDeclaration(crate::ast::TraitDefinition),
    /// 枚举定义
    EnumDeclaration(crate::ast::EnumDefinition),
    /// 效应处理器定义
    EffectHandler(EffectHandlerDefinition),
    /// 导入语句
    Import(ImportStatement),
    /// 导出语句
    Export(ExportStatement),
    /// 尝试-捕获语句
    TryCatch(TryCatchStatement),
    /// 抛出语句
    Throw(Expression),
    /// 断言语句
    Assert(Expression, Option<String>),
}

impl AstNode for Statement {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        match self {
            Statement::Expression(expr) => expr.to_lir(),
            Statement::VariableDeclaration(decl) => decl.to_lir(),
            Statement::Assignment(assign) => assign.to_lir(),
            Statement::If(if_stmt) => if_stmt.to_lir(),
            Statement::Loop(loop_stmt) => loop_stmt.to_lir(),
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    e.to_lir()
                } else {
                    Ok(NyarValue::Null)
                }
            },
            Statement::Block(statements) => {
                let mut results = Vec::new();
                for stmt in statements {
                    let result = stmt.to_lir()?;
                    results.push(Gc::new(result));
                }
                Ok(NyarValue::List(results))
            },
            Statement::FunctionDeclaration(func) => func.to_lir(),
            Statement::ClassDeclaration(class) => class.to_lir(),
            Statement::TraitDeclaration(trait_def) => trait_def.to_lir(),
            Statement::EnumDeclaration(enum_def) => enum_def.to_lir(),
            Statement::EffectHandler(handler) => handler.to_lir(),
            Statement::Import(import) => import.to_lir(),
            Statement::Export(export) => export.to_lir(),
            Statement::TryCatch(try_catch) => try_catch.to_lir(),
            Statement::Throw(expr) => expr.to_lir(),
            Statement::Assert(expr, _) => expr.to_lir(),
        }
    }
}

/// 变量声明
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    /// 变量名
    pub name: String,
    /// 类型注解（可选）
    pub type_annotation: Option<String>,
    /// 初始值（可选）
    pub initializer: Option<Expression>,
    /// 是否为常量
    pub is_constant: bool,
}

impl AstNode for VariableDeclaration {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        if let Some(initializer) = &self.initializer {
            initializer.to_lir()
        } else {
            Ok(NyarValue::Null)
        }
    }
}

/// 赋值语句
#[derive(Debug, Clone)]
pub struct Assignment {
    /// 左值
    pub target: Expression,
    /// 右值
    pub value: Expression,
}

impl AstNode for Assignment {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 赋值语句需要在运行时执行
        self.value.to_lir()
    }
}

/// 条件语句
#[derive(Debug, Clone)]
pub struct IfStatement {
    /// 条件
    pub condition: Expression,
    /// 条件为真时执行的语句
    pub then_branch: Vec<Statement>,
    /// 条件为假时执行的语句（可选）
    pub else_branch: Option<Vec<Statement>>,
}

impl AstNode for IfStatement {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 条件语句需要在运行时执行
        Err(NyarError::new("条件语句需要在运行时执行".to_string()))
    }
}

/// 循环语句
#[derive(Debug, Clone)]
pub enum LoopStatement {
    /// While循环
    While {
        /// 条件
        condition: Expression,
        /// 循环体
        body: Vec<Statement>,
    },
    /// For循环
    For {
        /// 初始化
        initializer: Box<Statement>,
        /// 条件
        condition: Expression,
        /// 更新
        update: Box<Statement>,
        /// 循环体
        body: Vec<Statement>,
    },
    /// ForEach循环
    ForEach {
        /// 迭代变量
        variable: String,
        /// 迭代对象
        iterable: Expression,
        /// 循环体
        body: Vec<Statement>,
    },
    /// 无限循环
    Infinite {
        /// 循环体
        body: Vec<Statement>,
    },
}

impl AstNode for LoopStatement {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 循环语句需要在运行时执行
        Err(NyarError::new("循环语句需要在运行时执行".to_string()))
    }
}

/// 效应处理器定义
#[derive(Debug, Clone)]
pub struct EffectHandlerDefinition {
    /// 效应名称
    pub name: String,
    /// 处理函数
    pub handler: crate::ast::FunctionDefinition,
}

impl AstNode for EffectHandlerDefinition {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 效应处理器需要在运行时注册
        self.handler.to_lir()
    }
}

/// 导入语句
#[derive(Debug, Clone)]
pub struct ImportStatement {
    /// 导入的模块路径
    pub path: String,
    /// 导入的符号列表
    pub symbols: Vec<String>,
    /// 是否为全部导入
    pub is_all: bool,
    /// 别名
    pub alias: Option<String>,
}

impl AstNode for ImportStatement {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 导入语句需要在运行时处理
        Ok(NyarValue::Null)
    }
}

/// 导出语句
#[derive(Debug, Clone)]
pub struct ExportStatement {
    /// 导出的声明
    pub declaration: Box<Statement>,
}

impl AstNode for ExportStatement {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 导出语句需要在运行时处理
        self.declaration.to_lir()
    }
}

/// 尝试-捕获语句
#[derive(Debug, Clone)]
pub struct TryCatchStatement {
    /// 尝试块
    pub try_block: Vec<Statement>,
    /// 捕获块
    pub catch_blocks: Vec<CatchBlock>,
    /// 最终块
    pub finally_block: Option<Vec<Statement>>,
}

/// 捕获块
#[derive(Debug, Clone)]
pub struct CatchBlock {
    /// 错误类型
    pub error_type: Option<String>,
    /// 错误变量名
    pub error_variable: String,
    /// 处理块
    pub handler: Vec<Statement>,
}

impl AstNode for TryCatchStatement {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 尝试-捕获语句需要在运行时执行
        Err(NyarError::new("尝试-捕获语句需要在运行时执行".to_string()))
    }
}