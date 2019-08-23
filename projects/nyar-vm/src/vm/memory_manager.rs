//! 内存管理器模块，负责管理堆内存和垃圾回收

use nyar_lir::{Gc, Heap, NyarValue};

/// 内存管理器，负责管理堆内存和垃圾回收
#[derive(Debug)]
pub struct MemoryManager {
    /// 堆内存
    heap: Heap,
}

impl MemoryManager {
    /// 创建一个新的内存管理器
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
        }
    }

    /// 分配一个值到堆上
    pub fn allocate(&mut self, value: NyarValue) -> Gc<NyarValue> {
        self.heap.allocate(value)
    }

    /// 手动触发垃圾回收
    pub fn collect_garbage(&mut self) {
        self.heap.collect_garbage();
    }

    /// 获取当前堆使用情况
    pub fn heap_stats(&self) -> HeapStats {
        HeapStats {
            allocated_objects: self.heap.allocated_count(),
            total_memory: self.heap.total_memory(),
        }
    }
}

/// 堆内存统计信息
#[derive(Debug, Clone, Copy)]
pub struct HeapStats {
    /// 已分配对象数量
    pub allocated_objects: usize,
    /// 总内存使用量（字节）
    pub total_memory: usize,
}