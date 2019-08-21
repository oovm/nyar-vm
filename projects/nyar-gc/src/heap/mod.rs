use crate::{Gc, Value};
use std::{cell::RefCell, collections::HashSet, marker::PhantomData};

#[derive(Debug, Clone, Default)]
pub struct Heap {
    pub roots: HashSet<usize>,
    pub memory: Vec<GcCell>,
}
#[derive(Debug, Clone)]
pub struct GcCell {
    value: RefCell<Value>,
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
        Gc { index, phantom: PhantomData::default() }
    }

    pub fn dealloc<T: 'static>(&mut self, gc_ref: Gc<T>) {
        self.roots.remove(&gc_ref.index);
    }

    pub fn collect(&mut self) {
        // 1. Reset state for all existing objects
        for marker in self.memory.iter_mut() {
            marker.state = GcState::Unmarked;
            marker.forwarding_address = None;
        }

        // 2. Mark phase: Find all reachable objects
        let mut worklist = Vec::new();

        for &root_index in &self.roots {
            if root_index < self.memory.len() {
                if self.memory[root_index].state == GcState::Unmarked {
                    self.memory[root_index].state = GcState::Marked;
                    worklist.push(root_index);
                }
            }
        }

        let mut worklist_idx = 0;
        while worklist_idx < worklist.len() {
            let current_obj_index = worklist[worklist_idx];
            worklist_idx += 1;

            // Collect children indices first to avoid nested mutable/immutable borrows of self.memory
            let mut children_target_indices = Vec::new();
            if let Value::List(ref children_gcs) = self.memory[current_obj_index].value {
                for child_gc in children_gcs {
                    children_target_indices.push(child_gc.index);
                }
            }

            // Now mark the children
            for child_obj_index in children_target_indices {
                if child_obj_index < self.memory.len() && self.memory[child_obj_index].state == GcState::Unmarked {
                    self.memory[child_obj_index].state = GcState::Marked;
                    worklist.push(child_obj_index);
                }
            }
        }

        // 3. Compute forwarding addresses for marked (live) objects
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
                        let pointed_obj_old_idx = child_gc.index;
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
                            new_indices_for_current_list.push(child_gc.index);
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
                            children_gcs_mut[i].index = *new_idx;
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
            let mut new_memory: Vec<Gc<Value>> = Vec::with_capacity(live_object_count);
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
        GcCell { value: RefCell::new(value), state: GcState::Unmarked, forwarding_address: None }
    }
}
