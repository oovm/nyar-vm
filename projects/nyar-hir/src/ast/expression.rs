//! 表达式模块，定义了各种表达式类型

use crate::ast::AstNode;
use nyar_error::NyarError;
use nyar_lir::{Gc, NyarFunction, NyarValue};
use std::collections::HashMap;

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    /// 字面量
    Literal(Literal),
    /// 变量引用
    Variable(String),
    /// 二元操作
    Binary(Box<BinaryExpression>),
    /// 一元操作
    Unary(Box<UnaryExpression>),
    /// 函数调用
    Call(Box<CallExpression>),
    /// Lambda表达式
    Lambda(Box<LambdaExpression>),
    /// 条件表达式
    Conditional(Box<ConditionalExpression>),
    /// 成员访问
    MemberAccess(Box<MemberAccessExpression>),
    /// 索引访问
    IndexAccess(Box<IndexAccessExpression>),
    /// 效应操作
    Effect(Box<EffectExpression>),
}

impl AstNode for Expression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        match self {
            Expression::Literal(literal) => literal.to_lir(),
            Expression::Variable(name) => {
                // 变量引用需要在运行时解析
                Err(NyarError::new(format!("变量 '{}' 需要在运行时解析", name)))
            }
            Expression::Binary(expr) => expr.to_lir(),
            Expression::Unary(expr) => expr.to_lir(),
            Expression::Call(expr) => expr.to_lir(),
            Expression::Lambda(expr) => expr.to_lir(),
            Expression::Conditional(expr) => expr.to_lir(),
            Expression::MemberAccess(expr) => expr.to_lir(),
            Expression::IndexAccess(expr) => expr.to_lir(),
            Expression::Effect(expr) => expr.to_lir(),
        }
    }
}

/// 字面量
#[derive(Debug, Clone)]
pub enum Literal {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 整数
    Integer(i64),
    /// 字符串
    String(String),
    /// 列表
    List(Vec<Expression>),
    /// 对象
    Object(HashMap<String, Expression>),
}

impl AstNode for Literal {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        match self {
            Literal::Null => Ok(NyarValue::Null),
            Literal::Boolean(value) => Ok(NyarValue::Boolean(*value)),
            Literal::Integer(value) => Ok(NyarValue::Integer(Box::new(value.clone().into()))),
            Literal::String(value) => Ok(NyarValue::String(Box::new(value.clone()))),
            Literal::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    let value = item.to_lir()?;
                    result.push(Gc::new(value));
                }
                Ok(NyarValue::List(result))
            }
            Literal::Object(properties) => {
                let mut result = indexmap::IndexMap::new();
                for (key, value) in properties {
                    let lir_value = value.to_lir()?;
                    result.insert(key.clone(), Gc::new(lir_value));
                }
                Ok(NyarValue::Object(result))
            }
        }
    }
}

/// 二元表达式
#[derive(Debug, Clone)]
pub struct BinaryExpression {
    /// 左操作数
    pub left: Expression,
    /// 操作符
    pub operator: String,
    /// 右操作数
    pub right: Expression,
}

impl AstNode for BinaryExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 二元表达式需要在运行时计算
        Err(NyarError::new("二元表达式需要在运行时计算".to_string()))
    }
}

/// 一元表达式
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    /// 操作符
    pub operator: String,
    /// 操作数
    pub operand: Expression,
}

impl AstNode for UnaryExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 一元表达式需要在运行时计算
        Err(NyarError::new("一元表达式需要在运行时计算".to_string()))
    }
}

/// 函数调用表达式
#[derive(Debug, Clone)]
pub struct CallExpression {
    /// 被调用的函数
    pub callee: Expression,
    /// 参数列表
    pub arguments: Vec<Expression>,
}

impl AstNode for CallExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 函数调用需要在运行时执行
        Err(NyarError::new("函数调用需要在运行时执行".to_string()))
    }
}

/// Lambda表达式
#[derive(Debug, Clone)]
pub struct LambdaExpression {
    /// 参数列表
    pub parameters: Vec<String>,
    /// 函数体
    pub body: Expression,
}

impl AstNode for LambdaExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 创建一个函数对象
        let function = NyarFunction {
            name: None, // Lambda是匿名函数
            parameters: self.parameters.clone(),
            body: Vec::new(), // 需要编译器将表达式转换为指令
            environment: Gc::new(HashMap::new()), // 空环境，需要在运行时捕获
        };
        
        Ok(NyarValue::Function(Box::new(function)))
    }
}

/// 条件表达式
#[derive(Debug, Clone)]
pub struct ConditionalExpression {
    /// 条件
    pub condition: Expression,
    /// 条件为真时的表达式
    pub then_branch: Expression,
    /// 条件为假时的表达式
    pub else_branch: Option<Expression>,
}

impl AstNode for ConditionalExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 条件表达式需要在运行时计算
        Err(NyarError::new("条件表达式需要在运行时计算".to_string()))
    }
}

/// 成员访问表达式
#[derive(Debug, Clone)]
pub struct MemberAccessExpression {
    /// 对象
    pub object: Expression,
    /// 成员名
    pub member: String,
}

impl AstNode for MemberAccessExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 成员访问需要在运行时解析
        Err(NyarError::new("成员访问需要在运行时解析".to_string()))
    }
}

/// 索引访问表达式
#[derive(Debug, Clone)]
pub struct IndexAccessExpression {
    /// 对象
    pub object: Expression,
    /// 索引
    pub index: Expression,
}

impl AstNode for IndexAccessExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 索引访问需要在运行时解析
        Err(NyarError::new("索引访问需要在运行时解析".to_string()))
    }
}

/// 效应表达式
#[derive(Debug, Clone)]
pub struct EffectExpression {
    /// 效应名称
    pub name: String,
    /// 效应参数
    pub arguments: Vec<Expression>,
}

impl AstNode for EffectExpression {
    fn to_lir(&self) -> Result<NyarValue, NyarError> {
        // 效应操作需要在运行时处理
        Err(NyarError::new("效应操作需要在运行时处理".to_string()))
    }
}