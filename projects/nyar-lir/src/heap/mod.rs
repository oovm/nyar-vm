//! 堆内存和垃圾回收模块

use crate::values::NyarValue;
use nyar_error::{NyarError, Result};
use std::{
    collections::HashSet,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};

mod gc_ptr;

/// 堆内存，用于存储GC管理的对象
#[derive(Debug, Clone)]
pub struct Heap {
    /// 根对象集合，这些对象不会被GC回收
    roots: HashSet<usize>,
    /// 内存空间，存储所有对象
    memory: Vec<GcValue>,
    /// 空闲内存索引
    free_indices: Vec<usize>,
}

/// GC标记，用于标记对象是否可达
#[derive(Debug, Clone)]
pub struct GcValue {
    /// 标记为已回收
    dead: bool,
    /// 对象值
    value: NyarValue,
}
/// GC指针，指向堆中的值
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gc<T: ?Sized> {
    /// 在堆中的索引
    pub index: usize,
    /// 类型标记
    pub phantom: PhantomData<T>,
}

impl Heap {
    /// 创建新的堆
    pub fn new() -> Self {
        Self { roots: HashSet::new(), memory: Vec::new(), free_indices: Vec::new() }
    }
}
