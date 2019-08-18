//! Nyar语言编译器
//!
//! 负责将Nyar语言的高级中间表示(HIR)编译为低级中间表示(LIR)指令。

use nyar_error::NyarError;
use nyar_lir::{Instruction, OpCode, Value};
use std::collections::HashMap;

/// 编译器结构
pub struct Compiler {
    /// 常量池
    constants: Vec<Value<'static>>,
    /// 局部变量表
    locals: HashMap<String, usize>,
    /// 当前指令列表
    instructions: Vec<Instruction>,
    /// 跳转表（用于解析标签）
    jumps: HashMap<String, usize>,
}

/// 高级中间表示（AST）
/// 注意：实际实现中，这应该来自nyar-hir项目
pub struct Hir {
    /// AST节点
    pub nodes: Vec<HirNode>,
}

/// HIR节点类型
#[derive(Debug, Clone)]
pub enum HirNode {
    /// 字面量
    Literal(LiteralValue),
    /// 变量声明
    VarDeclaration { name: String, initializer: Box<HirNode> },
    /// 变量引用
    VarReference(String),
    /// 二元操作
    BinaryOp { op: BinaryOperator, left: Box<HirNode>, right: Box<HirNode> },
    /// 函数声明
    FunctionDeclaration { name: String, params: Vec<String>, body: Vec<HirNode> },
    /// 函数调用
    FunctionCall { callee: Box<HirNode>, args: Vec<HirNode> },
    /// 条件语句
    IfStatement { condition: Box<HirNode>, then_branch: Vec<HirNode>, else_branch: Option<Vec<HirNode>> },
    /// 循环语句
    LoopStatement { body: Vec<HirNode> },
    /// 返回语句
    ReturnStatement { value: Option<Box<HirNode>> },
    /// 类声明
    ClassDeclaration { name: String, methods: Vec<HirNode> },
    /// 特征声明
    TraitDeclaration { name: String, methods: Vec<HirNode> },
    /// 枚举声明
    EnumDeclaration { name: String, variants: Vec<EnumVariant> },
}

/// 字面量值
#[derive(Debug, Clone)]
pub enum LiteralValue {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 整数
    Integer(i64),
    /// 字符串
    String(String),
}

/// 二元操作符
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

/// 枚举变体
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// 变体名
    pub name: String,
    /// 字段列表
    pub fields: Vec<String>,
}

impl Compiler {
    /// 创建一个新的编译器
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            locals: HashMap::new(),
            instructions: Vec::new(),
            jumps: HashMap::new(),
        }
    }

    /// 解析源代码为HIR
    pub fn parse(&self, source: &str) -> crate::Result<Hir> {
        // 实际实现中，这里应该调用nyar-hir的解析器
        // 这里只是一个占位实现
        Err(NyarError::custom("解析器尚未实现"))
    }

    /// 编译HIR为LIR指令
    pub fn compile(&mut self, hir: &Hir) -> crate::Result<Vec<Instruction>> {
        // 清空之前的状态
        self.constants.clear();
        self.locals.clear();
        self.instructions.clear();
        self.jumps.clear();

        // 编译每个节点
        for node in &hir.nodes {
            self.compile_node(node)?;
        }

        Ok(self.instructions.clone())
    }

    /// 编译单个HIR节点
    fn compile_node(&mut self, node: &HirNode) -> crate::Result<()> {
        match node {
            HirNode::Literal(lit) => self.compile_literal(lit),
            HirNode::VarDeclaration { name, initializer } => self.compile_var_declaration(name, initializer),
            HirNode::VarReference(name) => self.compile_var_reference(name),
            HirNode::BinaryOp { op, left, right } => self.compile_binary_op(op, left, right),
            HirNode::FunctionDeclaration { name, params, body } => self.compile_function_declaration(name, params, body),
            HirNode::FunctionCall { callee, args } => self.compile_function_call(callee, args),
            HirNode::IfStatement { condition, then_branch, else_branch } => {
                self.compile_if_statement(condition, then_branch, else_branch.as_deref())
            },
            HirNode::LoopStatement { body } => self.compile_loop_statement(body),
            HirNode::ReturnStatement { value } => self.compile_return_statement(value.as_deref()),
            HirNode::ClassDeclaration { name, methods } => self.compile_class_declaration(name, methods),
            HirNode::TraitDeclaration { name, methods } => self.compile_trait_declaration(name, methods),
            HirNode::EnumDeclaration { name, variants } => self.compile_enum_declaration(name, variants),
        }
    }

    /// 编译字面量
    fn compile_literal(&mut self, literal: &LiteralValue) -> crate::Result<()> {
        let value = match literal {
            LiteralValue::Null => Value::Null,
            LiteralValue::Boolean(b) => Value::Boolean(*b),
            LiteralValue::Integer(i) => Value::Integer(num::BigInt::from(*i)),
            LiteralValue::String(s) => {
                // 在实际实现中，这里需要使用gc-arena分配字符串
                // 这里只是一个简化的示例
                Value::Null
            }
        };

        // 添加到常量池并生成Push指令
        let const_idx = self.add_constant(value);
        self.emit(Instruction::new(OpCode::Push, vec![const_idx]));

        Ok(())
    }

    /// 编译变量声明
    fn compile_var_declaration(&mut self, name: &str, initializer: &HirNode) -> crate::Result<()> {
        // 编译初始化表达式
        self.compile_node(initializer)?;

        // 分配局部变量
        let local_idx = self.locals.len();
        self.locals.insert(name.to_string(), local_idx);

        // 生成存储指令
        self.emit(Instruction::new(OpCode::StoreLocal, vec![local_idx]));

        Ok(())
    }

    /// 编译变量引用
    fn compile_var_reference(&mut self, name: &str) -> crate::Result<()> {
        if let Some(local_idx) = self.locals.get(name) {
            // 局部变量
            self.emit(Instruction::new(OpCode::LoadLocal, vec![*local_idx]));
        } else {
            // 全局变量
            let name_idx = self.add_constant(Value::Null); // 实际应该是字符串常量
            self.emit(Instruction::new(OpCode::LoadGlobal, vec![name_idx]));
        }

        Ok(())
    }

    /// 编译二元操作
    fn compile_binary_op(&mut self, op: &BinaryOperator, left: &HirNode, right: &HirNode) -> crate::Result<()> {
        // 编译左右操作数
        self.compile_node(left)?;
        self.compile_node(right)?;

        // 生成操作指令
        let opcode = match op {
            BinaryOperator::Add => OpCode::Add,
            BinaryOperator::Sub => OpCode::Sub,
            BinaryOperator::Mul => OpCode::Mul,
            BinaryOperator::Div => OpCode::Div,
            BinaryOperator::Mod => OpCode::Mod,
            BinaryOperator::Eq => OpCode::Equal,
            BinaryOperator::NotEq => OpCode::NotEqual,
            BinaryOperator::Lt => OpCode::Less,
            BinaryOperator::LtEq => OpCode::LessEqual,
            BinaryOperator::Gt => OpCode::Greater,
            BinaryOperator::GtEq => OpCode::GreaterEqual,
            BinaryOperator::And => OpCode::And,
            BinaryOperator::Or => OpCode::Or,
        };

        self.emit(Instruction::simple(opcode));

        Ok(())
    }

    /// 编译函数声明
    fn compile_function_declaration(&mut self, name: &str, params: &[String], body: &[HirNode]) -> crate::Result<()> {
        // 保存当前编译状态
        let old_instructions = std::mem::take(&mut self.instructions);
        let old_locals = self.locals.clone();

        // 设置新的局部变量表（参数）
        self.locals.clear();
        for (i, param) in params.iter().enumerate() {
            self.locals.insert(param.clone(), i);
        }

        // 编译函数体
        for node in body {
            self.compile_node(node)?;
        }

        // 确保函数有返回值
        if !body.iter().any(|node| matches!(node, HirNode::ReturnStatement { .. })) {
            self.emit(Instruction::simple(OpCode::Push)); // 推入null
            self.emit(Instruction::simple(OpCode::Return));
        }

        // 创建函数对象（实际实现中需要使用gc-arena）
        let function_instructions = std::mem::replace(&mut self.instructions, old_instructions);
        
        // 恢复原来的局部变量表
        self.locals = old_locals;

        // 将函数添加为常量并存储为全局变量
        let func_idx = self.add_constant(Value::Null); // 实际应该是函数对象
        let name_idx = self.add_constant(Value::Null); // 实际应该是字符串常量

        self.emit(Instruction::new(OpCode::Push, vec![func_idx]));
        self.emit(Instruction::new(OpCode::StoreGlobal, vec![name_idx]));

        Ok(())
    }

    /// 编译函数调用
    fn compile_function_call(&mut self, callee: &HirNode, args: &[HirNode]) -> crate::Result<()> {
        // 编译参数（从右到左压栈）
        for arg in args.iter().rev() {
            self.compile_node(arg)?;
        }

        // 编译被调用者
        self.compile_node(callee)?;

        // 生成调用指令
        self.emit(Instruction::new(OpCode::Call, vec![args.len()]));

        Ok(())
    }

    /// 编译条件语句
    fn compile_if_statement(
        &mut self,
        condition: &HirNode,
        then_branch: &[HirNode],
        else_branch: Option<&[HirNode]>,
    ) -> crate::Result<()> {
        // 编译条件表达式
        self.compile_node(condition)?;

        // 生成条件跳转指令（先用占位符）
        let jump_if_not_idx = self.instructions.len();
        self.emit(Instruction::new(OpCode::JumpIfNot, vec![0])); // 占位符

        // 编译then分支
        for node in then_branch {
            self.compile_node(node)?;
        }

        if let Some(else_branch) = else_branch {
            // 生成跳过else分支的跳转指令（先用占位符）
            let jump_idx = self.instructions.len();
            self.emit(Instruction::new(OpCode::Jump, vec![0])); // 占位符

            // 更新条件跳转指令的目标地址
            let else_start = self.instructions.len();
            self.instructions[jump_if_not_idx] = Instruction::new(OpCode::JumpIfNot, vec![else_start]);

            // 编译else分支
            for node in else_branch {
                self.compile_node(node)?;
            }

            // 更新跳过else分支的跳转指令的目标地址
            let after_else = self.instructions.len();
            self.instructions[jump_idx] = Instruction::new(OpCode::Jump, vec![after_else]);
        } else {
            // 没有else分支，直接更新条件跳转指令的目标地址
            let after_then = self.instructions.len();
            self.instructions[jump_if_not_idx] = Instruction::new(OpCode::JumpIfNot, vec![after_then]);
        }

        Ok(())
    }

    /// 编译循环语句
    fn compile_loop_statement(&mut self, body: &[HirNode]) -> crate::Result<()> {
        // 记录循环开始位置
        let loop_start = self.instructions.len();

        // 生成循环开始指令
        self.emit(Instruction::simple(OpCode::Loop));

        // 编译循环体
        for node in body {
            self.compile_node(node)?;
        }

        // 生成跳回循环开始的指令
        self.emit(Instruction::new(OpCode::Jump, vec![loop_start]));

        // 记录循环结束位置（用于break指令）
        let loop_end = self.instructions.len();

        // 更新所有break指令的目标地址
        // 实际实现中需要维护一个break指令列表

        Ok(())
    }

    /// 编译返回语句
    fn compile_return_statement(&mut self, value: Option<&HirNode>) -> crate::Result<()> {
        if let Some(value) = value {
            // 编译返回值表达式
            self.compile_node(value)?;
        } else {
            // 没有返回值，默认返回null
            self.emit(Instruction::new(OpCode::Push, vec![self.add_constant(Value::Null)]));
        }

        // 生成返回指令
        self.emit(Instruction::simple(OpCode::Return));

        Ok(())
    }

    /// 编译类声明
    fn compile_class_declaration(&mut self, name: &str, methods: &[HirNode]) -> crate::Result<()> {
        // 创建类对象
        let class_name_idx = self.add_constant(Value::Null); // 实际应该是字符串常量
        self.emit(Instruction::new(OpCode::NewObject, vec![class_name_idx]));

        // 编译方法
        for method in methods {
            if let HirNode::FunctionDeclaration { name: method_name, .. } = method {
                // 编译方法并添加到类
                self.compile_node(method)?;
                
                // 获取方法并设置为类的属性
                let method_name_idx = self.add_constant(Value::Null); // 实际应该是字符串常量
                self.emit(Instruction::new(OpCode::SetProperty, vec![method_name_idx]));
            }
        }

        // 存储类为全局变量
        let name_idx = self.add_constant(Value::Null); // 实际应该是字符串常量
        self.emit(Instruction::new(OpCode::StoreGlobal, vec![name_idx]));

        Ok(())
    }

    /// 编译特征声明
    fn compile_trait_declaration(&mut self, name: &str, methods: &[HirNode]) -> crate::Result<()> {
        // 特征编译类似于类，但不生成实现代码，只生成方法签名
        // 实际实现中需要与运行时集成
        Ok(())
    }

    /// 编译枚举声明
    fn compile_enum_declaration(&mut self, name: &str, variants: &[EnumVariant]) -> crate::Result<()> {
        // 枚举编译类似于类，但需要为每个变体生成构造函数
        // 实际实现中需要与运行时集成
        Ok(())
    }

    /// 添加常量并返回索引
    fn add_constant(&mut self, value: Value<'static>) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// 发出指令
    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}