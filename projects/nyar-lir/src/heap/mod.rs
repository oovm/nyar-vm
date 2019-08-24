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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Gc<T: ?Sized> {
    /// 在堆中的索引
    pub index: usize,
    /// 类型标记
    pub phantom: PhantomData<T>,
}

impl<T> Copy for Gc<T> {}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc { index: 0, phantom: PhantomData }
    }
}

impl Heap {
    /// 创建新的堆
    pub fn new() -> Self {
        Self { roots: HashSet::new(), memory: Vec::new(), free_indices: Vec::new() }
    }
    pub fn allocate<T>(&mut self, value: T) -> Gc<NyarValue>
    where
        T: Into<NyarValue>,
    {
        let value = GcValue { dead: false, value: value.into() };
        match self.free_indices.pop() {
            Some(index) => {
                let gc = Gc { index, phantom: PhantomData };
                self.memory[index] = value;
                gc
            }
            None => {
                let gc = Gc { index: self.roots.len(), phantom: PhantomData };
                self.memory.push(value);
                gc
            }
        }
    }
    
    pub fn view_ref<T>(&self, index: Gc<T>) -> Result<&NyarValue> {
        match self.memory.get(index.index) {
            Some(s) if s.dead => Err(NyarError::use_after_free(index.index)),
            Some(s) => Ok(&s.value),
            None => Err(NyarError::use_after_free(index.index)),
        }
    }
    
    pub fn view_mut<T>(&mut self, index: Gc<T>) -> Result<&mut NyarValue> {
        match self.memory.get_mut(index.index) {
            Some(s) if s.dead => Err(NyarError::use_after_free(index.index)),
            Some(s) => Ok(&mut s.value),
            None => Err(NyarError::use_after_free(index.index)),
        }
    }
}
