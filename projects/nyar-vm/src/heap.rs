//! 堆内存和垃圾回收模块

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use crate::error::VmError;
use crate::value::Value;

/// 堆内存，用于存储GC管理的对象
#[derive(Debug, Clone)]
pub struct Heap {
    /// 根对象集合，这些对象不会被GC回收
    pub roots: HashSet<usize>,
    /// 内存空间，存储所有对象
    pub memory: Vec<GcMarker>,
    /// 空闲内存索引
    free_indices: Vec<usize>,
}

/// GC标记，用于标记对象是否可达
#[derive(Debug, Clone)]
pub struct GcMarker {
    /// 对象值
    pub value: Value,
    /// 是否被标记（用于标记-清除GC算法）
    marked: bool,
}

/// GC指针，指向堆中的值
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gc<T: ?Sized> {
    /// 在堆中的索引
    pub index: usize,
    /// 类型标记
    pub phantom: PhantomData<T>,
}

impl<T> Display for Gc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc({})", self.index)
    }
}

impl<T> Gc<T> {
    /// 将GC指针转换为任意类型的GC指针
    pub fn as_any(&self) -> Gc<Value> {
        self.transmute()
    }

    /// 将GC指针转换为指定类型的GC指针
    pub fn transmute<U>(&self) -> Gc<U> {
        Gc { index: self.index, phantom: PhantomData::default() }
    }

    /// 解引用GC指针，获取指向的值
    pub fn deref<'gc>(&self, heap: &'gc Heap) -> Result<T, VmError>
    where
        T: TryFrom<&'gc Value, Error=VmError>,
    {
        match heap.memory.get(self.index) {
            Some(s) => {
                // 不能解引用已死亡的对象
                if !s.marked {
                    return Err(VmError::DeadObjectAccess(self.index));
                }
                (&s.value).try_into()
            }
            None => {
                Err(VmError::InvalidGcIndex(self.index))
            }
        }
    }
}

impl Heap {
    /// 创建新的堆
    pub fn new() -> Self {
        Self {
            roots: HashSet::new(),
            memory: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    /// 分配新对象
    pub fn allocate<T>(&mut self, value: Value) -> Gc<T> {
        let index = if let Some(index) = self.free_indices.pop() {
            // 使用空闲索引
            self.memory[index] = GcMarker { value, marked: true };
            index
        } else {
            // 分配新索引
            let index = self.memory.len();
            self.memory.push(GcMarker { value, marked: true });
            index
        };

        Gc { index, phantom: PhantomData::default() }
    }

    /// 添加根对象
    pub fn add_root(&mut self, gc: &Gc<Value>) {
        self.roots.insert(gc.index);
    }

    /// 移除根对象
    pub fn remove_root(&mut self, gc: &Gc<Value>) {
        self.roots.remove(&gc.index);
    }

    /// 执行垃圾回收
    pub fn collect_garbage(&mut self) {
        // 标记阶段：标记所有可达对象
        self.mark();

        // 清除阶段：清除所有不可达对象
        self.sweep();
    }

    /// 标记阶段
    fn mark(&mut self) {
        // 重置所有标记
        for marker in &mut self.memory {
            marker.marked = false;
        }

        // 标记所有根对象
        let mut stack = Vec::new();
        for &index in &self.roots {
            if let Some(marker) = self.memory.get_mut(index) {
                marker.marked = true;
                stack.push(index);
            }
        }

        // 标记所有可达对象
        while let Some(index) = stack.pop() {
            if let Some(marker) = self.memory.get(index) {
                // 根据值类型找到引用的其他对象
                match &marker.value {
                    Value::String(_) => {}, // 字符串不包含其他引用
                    Value::Array(array_gc) => {
                        for item in array_gc {
                            let item_index = item.index;
                            if let Some(item_marker) = self.memory.get_mut(item_index) {
                                if !item_marker.marked {
                                    item_marker.marked = true;
                                    stack.push(item_index);
                                }
                            }
                        }
                    },
                    Value::Object(object_gc) => {
                        if let Ok(object) = object_gc.deref(self) {
                            for (_, value) in object {
                                let value_index = value.index;
                                if let Some(value_marker) = self.memory.get_mut(value_index) {
                                    if !value_marker.marked {
                                        value_marker.marked = true;
                                        stack.push(value_index);
                                    }
                                }
                            }
                        }
                    },
                    Value::Function(function_gc) => {
                        if let Ok(function) = function_gc.deref(self) {
                            let env_index = function.environment.index;
                            if let Some(env_marker) = self.memory.get_mut(env_index) {
                                if !env_marker.marked {
                                    env_marker.marked = true;
                                    stack.push(env_index);
                                }
                            }
                        }
                    },
                    // 其他类型的处理...
                    _ => {},
                }
            }
        }
    }

    /// 清除阶段
    fn sweep(&mut self) {
        for i in 0..self.memory.len() {
            if let Some(marker) = self.memory.get(i) {
                if !marker.marked {
                    // 对象不可达，加入空闲列表
                    self.free_indices.push(i);
                }
            }
        }
    }
}

// 为Value类型实现TryFrom特性，用于从Value转换为特定类型
impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = VmError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s.to_string().as_str()),
            _ => Err(VmError::TypeMismatch {
                expected: "string",
                found: value.type_name(),
            }),
        }
    }
}

