use crate::{Gc, Value, VmError};
use std::{
    collections::{HashSet},
    marker::PhantomData,
};

#[derive(Debug, Clone, Default)]
pub struct Heap {
    pub roots: HashSet<usize>,
    pub memory: Vec<GcCell>,
}
#[derive(Debug, Clone)]
pub struct GcCell {
    pub value: Value,
    state: GcState,
    forwarding_address: Option<usize>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcState {
    Unmarked,
    Marked,
}

impl Heap {
    pub fn alloc<T: Into<Value>>(&mut self, value: T, root: bool) -> Gc<T> {
        let index = self.memory.len();
        self.memory.push(GcCell::new(value.into()));
        if root {
            self.roots.insert(index);
        }
        Gc { pointer: index, phantom: PhantomData::default() }
    }

    pub fn dealloc<T: 'static>(&mut self, gc_ref: Gc<T>) {
        self.roots.remove(&gc_ref.pointer);
    }

    pub fn get_value(&self, index: usize) -> Option<&Value> {
        let cell = self.memory.get(index)?;
        Some(&cell.value)
    }

    pub fn collect(&mut self) {
        // 1. 重置所有 GC 追踪状态
        for marker in self.memory.iter_mut() {
            marker.state = GcState::Unmarked;
            marker.forwarding_address = None;
        }

        // 2. 标记所有可达对象
        let mut worklist = Vec::new();
        // 2.1 标记根对象
        for &root_index in &self.roots {
            match self.memory.get_mut(root_index) {
                Some(cell) => {
                    cell.state = GcState::Marked;
                    worklist.push(root_index);
                }
                None => eprintln!("Root object out of bounds: {}", root_index),
            }
        }
        // 2.2 递归标记其他可达对象
        let mut live_index = 0;
        while live_index < worklist.len() {
            let current_obj_index = worklist[live_index];
            live_index += 1;
            
            // 手机当前对象的所有子对象索引
            let children_target_indices = self.get_value(current_obj_index).map(|x| x.gc_trace()).unwrap_or_default();

            // Now mark the children
            for child_obj_index in children_target_indices {
                self.memory[child_obj_index].state = GcState::Marked;
                worklist.push(child_obj_index);
            }
        }
        
        // 3. 更新所有标记对象的转发地址
        let mut new_address: usize = 0;
        for old_index in 0..self.memory.len() {
            if self.memory[old_index].state == GcState::Marked {
                self.memory[old_index].forwarding_address = Some(new_address);
                new_address += 1;
            }
        }
        let live_object_count = new_address;

        // 4. Update pointers in live objects and roots
        //    Phase 4a: Collect new indices for children in lists
        let mut list_pointer_updates: Vec<(usize /* list_obj_idx */, Vec<usize> /* new_child_indices */)> = Vec::new();

        for old_idx in 0..self.memory.len() {
            if self.memory[old_idx].state == GcState::Marked {
                if let Value::List(ref children_gcs) = self.memory[old_idx].value {
                    // Immutable borrow
                    let mut new_indices_for_current_list = Vec::with_capacity(children_gcs.len());
                    for child_gc in children_gcs {
                        let pointed_obj_old_idx = child_gc.pointer;
                        if pointed_obj_old_idx < self.memory.len() && self.memory[pointed_obj_old_idx].state == GcState::Marked
                        {
                            // If the pointed-to object is live and marked, it must have a forwarding address.
                            match self.memory[pointed_obj_old_idx].forwarding_address {
                                Some(forwarded_idx) => {
                                    new_indices_for_current_list.push(forwarded_idx);
                                }
                                None => {
                                    // This should not happen if logic is correct: a marked object should have a forwarding address.
                                    panic!(
                                        "Internal GC Error: Marked object at index {} missing forwarding address during pointer update preparation.",
                                        pointed_obj_old_idx
                                    );
                                }
                            }
                        }
                        else {
                            // Child points to an unmarked/collected object or out of bounds.
                            // Keep the old index; it will be a "dangling" pointer.
                            new_indices_for_current_list.push(child_gc.pointer);
                        }
                    }
                    list_pointer_updates.push((old_idx, new_indices_for_current_list));
                }
            }
        }

        // Phase 4b: Apply the collected updates to list objects
        for (list_obj_idx, new_child_indices) in list_pointer_updates {
            // Ensure the object still exists and is a list (it should be, as it was marked)
            if list_obj_idx < self.memory.len() {
                if let Value::List(ref mut children_gcs_mut) = self.memory[list_obj_idx].value {
                    // Mutable borrow
                    for (i, new_idx) in new_child_indices.iter().enumerate() {
                        if i < children_gcs_mut.len() {
                            children_gcs_mut[i].pointer = *new_idx;
                        }
                    }
                }
            }
        }

        // Update roots
        let mut new_roots = HashSet::new();
        for &root_old_index in &self.roots {
            if root_old_index < self.memory.len() && self.memory[root_old_index].state == GcState::Marked {
                if let Some(forwarded_idx) = self.memory[root_old_index].forwarding_address {
                    new_roots.insert(forwarded_idx);
                }
                // else: Root pointed to something that wasn't live or an error.
                // It won't be included in new_roots.
            }
        }
        self.roots = new_roots;

        // 5. Compact phase: Move marked objects to new memory locations
        if live_object_count > 0 {
            // let mut new_memory: Vec<Gc<Value>> = Vec::with_capacity(live_object_count);
            // Create a temporary vector initialized to hold the new objects.
            // Using a vec of Option to build the new_memory, then unwrapping.
            // This is safer if GcMarker wasn't easily default-constructible for placeholder.
            let mut temp_compact_store: Vec<Option<GcCell>> = vec![None; live_object_count];

            for old_idx in 0..self.memory.len() {
                if self.memory[old_idx].state == GcState::Marked {
                    if let Some(new_idx) = self.memory[old_idx].forwarding_address {
                        let mut marker_to_move = self.memory[old_idx].clone(); // Value::List children's Gc.index are already updated
                        marker_to_move.state = GcState::Unmarked; // Reset state for the new heap
                        marker_to_move.forwarding_address = None; // Clear forwarding address

                        if new_idx < temp_compact_store.len() {
                            temp_compact_store[new_idx] = Some(marker_to_move);
                        }
                        else {
                            panic!(
                                "Internal GC Error: Forwarding address {} out of bounds ({}) during compaction.",
                                new_idx,
                                temp_compact_store.len()
                            );
                        }
                    }
                }
            }
            self.memory = temp_compact_store.into_iter().filter_map(|opt| opt).collect();

            if self.memory.len() != live_object_count {
                panic!(
                    "Internal GC Error: Mismatch in live object count ({}) after compaction, expected {}.",
                    self.memory.len(),
                    live_object_count
                );
            }
        }
        else {
            self.memory.clear();
            self.roots.clear(); // Should already be empty if no live objects
        }
    }
}

impl GcCell {
    fn new(value: Value) -> Self {
        GcCell { value, state: GcState::Unmarked, forwarding_address: None }
    }
}
impl<T> Gc<T> {
    pub fn deref<'vm, V>(&self, heap: &'vm Heap) -> Result<V, VmError>
    where
        V: TryFrom<&'vm Value, Error = VmError>,
    {
        if self.pointer >= heap.memory.len() {
            return Err(VmError::IndexOutOfBounds);
        }
        let marker = &heap.memory[self.pointer];
        // After GC, objects in 'memory' are live. Their GcState is reset to Unmarked.
        V::try_from(&marker.value)
    }
}
