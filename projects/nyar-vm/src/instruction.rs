//! 指令集模块，定义了VM支持的所有指令

use crate::heap::Gc;
use crate::value::Value;

/// VM指令集
#[derive(Debug, Clone)]
pub enum Instruction {
    /// 将常量压入栈
    PushConstant { value: Value },
    /// 将变量压入栈
    PushVariable { name: String },
    /// 弹出栈顶值并存储到变量
    StoreVariable { name: String },
    /// 获取数组索引
    GetIndex { index: usize },
    /// 设置数组索引
    SetIndex { index: usize },
    /// 获取对象属性
    GetProperty { name: String },
    /// 设置对象属性
    SetProperty { name: String },
    /// 调用函数
    Call { argument_count: usize },
    /// 创建函数
    CreateFunction { name: Option<String>, parameter_count: usize, body_size: usize },
    /// 创建闭包
    CreateClosure { captured_variables: Vec<String> },
    /// 创建数组
    CreateArray { size: usize },
    /// 创建对象
    CreateObject { property_count: usize },
    /// 创建类
    CreateClass { name: String, method_count: usize, property_count: usize },
    /// 创建特征
    CreateTrait { name: String, method_count: usize },
    /// 创建枚举
    CreateEnum { name: String, variant_count: usize },
    /// 跳转
    Jump { offset: isize },
    /// 条件跳转
    JumpIfFalse { offset: isize },
    /// 循环开始
    LoopStart { label: Option<String> },
    /// 循环结束
    LoopEnd { label: Option<String> },
    /// 跳出循环
    Break { label: Option<String> },
    /// 继续循环
    Continue { label: Option<String> },
    /// 匹配开始
    MatchStart,
    /// 匹配条件
    MatchCase { fall_through: bool },
    /// 匹配结束
    MatchEnd,
    /// 返回
    Return,
    /// 创建协程
    CreateCoroutine,
    /// 恢复协程
    ResumeCoroutine,
    /// 暂停协程
    YieldCoroutine { value_count: usize },
    /// 等待异步操作
    Await,
    /// 阻塞等待异步操作完成
    BlockOn,
    /// 触发异步操作但不等待结果
    FireThenIgnore,
    /// 触发效应
    RaiseEffect { name: String, argument_count: usize },
    /// 处理效应
    HandleEffect { name: String },
    /// 恢复效应
    ResumeEffect { value_count: usize },
}