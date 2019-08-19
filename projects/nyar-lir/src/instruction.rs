//! Nyar语言的指令集系统
//!
//! 实现了Nyar语言的栈机模型指令集，包括基本操作和控制流指令。

use crate::value::NyarValue;
use gc_arena::Collect;

/// 指令枚举
#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub enum NyarInstruction {
    // 栈操作
    Push { constant_index: usize }, // 将常量池中的值压入栈
    Pop,                          // 弹出栈顶值
    Dup,                          // 复制栈顶值
    Swap,                         // 交换栈顶两个值

    // 变量操作
    LoadLocal { local_index: usize },    // 加载局部变量
    StoreLocal { local_index: usize },   // 存储局部变量
    LoadGlobal { name_index: usize },   // 加载全局变量
    StoreGlobal { name_index: usize },  // 存储全局变量

    // 逻辑运算
    And, // 逻辑与
    Or,  // 逻辑或
    Not, // 逻辑非

    // 比较运算
    Equal,        // 相等
    NotEqual,     // 不等
    Less,         // 小于
    LessEqual,    // 小于等于
    Greater,      // 大于
    GreaterEqual, // 大于等于

    // 控制流
    Jump { target: usize },      // 无条件跳转
    JumpIf { target: usize },    // 条件跳转
    JumpIfNot { target: usize }, // 条件不成立跳转
    Call { arg_count: usize }, // 调用函数
    Return,                    // 函数返回
    Yield,                     // 生成器产出值

    // 对象操作
    NewObject { class_name_index: usize },   // 创建新对象
    GetProperty { property_name_index: usize }, // 获取属性
    SetProperty { property_name_index: usize }, // 设置属性
    GetMethod { method_name_index: usize },   // 获取方法
    CallMethod { method_name_index: usize, arg_count: usize },  // 调用方法

    // 数组操作
    NewArray { size: usize }, // 创建新数组
    GetIndex,               // 获取数组元素
    SetIndex,               // 设置数组元素

    // 异步操作
    Await,   // 等待异步操作完成
    BlockOn, // 阻塞等待异步操作

    // 效果系统
    Raise,  // 抛出效果
    Handle, // 处理效果
    Resume, // 恢复效果处理

    // 循环控制
    Loop,     // 循环开始
    Break,    // 跳出循环
    Continue, // 继续下一次循环

    // 其他
    Nop, // 空操作
}

/// 代码块结构
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// 指令列表
    pub instructions: Vec<NyarInstruction>,
    /// 常量池
    pub constants: Vec<NyarValue<'static>>,
    /// 局部变量名称
    pub locals: Vec<String>,
}

impl CodeBlock {
    /// 创建一个新的代码块
    pub fn new() -> Self {
        Self { instructions: Vec::new(), constants: Vec::new(), locals: Vec::new() }
    }

    /// 添加一个指令
    pub fn add_instruction(&mut self, instruction: NyarInstruction) {
        self.instructions.push(instruction);
    }

    /// 添加一个常量并返回其索引
    pub fn add_constant(&mut self, value: NyarValue<'static>) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// 添加一个局部变量并返回其索引
    pub fn add_local(&mut self, name: &str) -> usize {
        self.locals.push(name.to_string());
        self.locals.len() - 1
    }
}
