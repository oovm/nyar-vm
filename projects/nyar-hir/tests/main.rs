use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI8, Ordering};
use std::thread;

// Tri-color states for GcMark.state
const WHITE: i8 = 0; // Object is not yet visited or is confirmed live after a cycle.
const GRAY: i8 = 1;  // Object is visited, but its children are not yet processed (in worklist).
const BLACK: i8 = 2; // Object is visited, and all its children are processed. Live.

// Placeholder for VmError
#[derive(Debug)]
pub enum VmError {
    ObjectNotFound,
    InvalidObjectState,
    TypeMismatch,
    DanglingPointerInSweep,
}

// Value enum - must provide a way to get child Gc pointers


impl Value {
    /// Iterates over Gc pointers contained within this Value.
    /// The visitor function `F` is called for each child `Gc<Value>`.
    pub fn visit_children<F>(&self, mut visitor: F)
    where
        F: FnMut(&Gc<Value>),
    {
        match self {
            Value::List(gc1, gc2) => {
                visitor(gc1);
                visitor(gc2);
            }
            Value::Number(_) | Value::String(_) => {
                // These types do not contain Gc children.
            }
            // Example: If Value had a List variant:
            // Value::List(items) => {
            //     for item_gc in items {
            //         visitor(item_gc);
            //     }
            // }
        }
    }

    /// Updates internal Gc pointers during compaction.
    /// `old_to_new_map` maps old Gc indices to new Gc indices.
    pub fn remap_children(&mut self, old_to_new_map: &HashMap<usize, usize>) -> Result<(), VmError> {
        match self {
            Value::List(ref mut gc1, ref mut gc2) => {
                gc1.index = *old_to_new_map.get(&gc1.index).ok_or(VmError::DanglingPointerInSweep)?;
                gc2.index = *old_to_new_map.get(&gc2.index).ok_or(VmError::DanglingPointerInSweep)?;
            }
            Value::Number(_) | Value::String(_) => {
                // No children to remap.
            }
            // Handle other variants with Gc children similarly.
        }
        Ok(())
    }
}

// Example From implementations for Value
impl From<i32> for Value { fn from(v: i32) -> Self { Value::Number(v) } }
impl From<String> for Value { fn from(v: String) -> Self { Value::String(v) } }
impl From<&str> for Value { fn from(v: &str) -> Self { Value::String(v.to_string()) } }

// A constructor for Pair to make example setup easier
impl Value {
    pub fn new_pair(v1: Gc<Value>, v2: Gc<Value>) -> Value {
        Value::List(v1, v2)
    }
}


pub struct GcMark {
    state: AtomicI8,
    value: Value,
}

pub struct Heap {
    roots: Arc<Mutex<HashSet<usize>>>,
    memory: Arc<Mutex<Vec<GcMark>>>,
    gray_worklist: Arc<Mutex<VecDeque<usize>>>, // Worklist for parallel marking
}

#[derive(Debug)]
pub struct Gc<T> {
    index: usize,
    phantom: PhantomData<T>,
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self { Gc { index: self.index, phantom: PhantomData } }
}
impl<T> Copy for Gc<T> {}

impl<T> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool { self.index == other.index }
}
impl<T> Eq for Gc<T> {}

impl<T> std::hash::Hash for Gc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.index.hash(state); }
}


impl Heap {
    pub fn new() -> Self {
        Heap {
            roots: Arc::new(Mutex::new(HashSet::new())),
            memory: Arc::new(Mutex::new(Vec::new())),
            gray_worklist: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn alloc<T: Into<Value> + 'static>(&mut self, value: T, root: bool) -> Gc<T> {
        let mut mem_guard = self.memory.lock().unwrap();
        let index = mem_guard.len();
        mem_guard.push(GcMark { state: AtomicI8::new(WHITE), value: value.into() });

        if root {
            drop(mem_guard); // Release memory lock before acquiring roots lock
            self.roots.lock().unwrap().insert(index);
        }
        Gc { index, phantom: PhantomData::default() }
    }

    pub fn dealloc(&mut self, gc_ptr: Gc<impl Into<Value>>) {
        let index = gc_ptr.index;
        self.roots.lock().unwrap().remove(&index); // Remove from roots

        let mem_guard = self.memory.lock().unwrap();
        if let Some(obj_mark) = mem_guard.get(index) {
            // Mark as WHITE, making it a candidate for collection.
            obj_mark.state.store(WHITE, Ordering::Release);
        }
    }

    pub fn collect_garbage(&self, num_marker_threads: usize) {
        println!("[GC] Starting garbage collection cycle ({} marker threads).", num_marker_threads);

        self.initial_mark_roots();
        let initial_worklist_size = self.gray_worklist.lock().unwrap().len();
        println!("[GC] Initial root marking complete. Gray worklist size: {}", initial_worklist_size);

        if initial_worklist_size > 0 || num_marker_threads == 0 { // num_marker_threads == 0 implies single-threaded marking on main thread
            let mut marker_handles = vec![];
            let effective_threads = if num_marker_threads == 0 { 0 } else { num_marker_threads };


            for i in 0..effective_threads {
                let heap_clone = self.clone_for_threading();
                marker_handles.push(thread::spawn(move || {
                    // println!("[GC THREAD {}] Starting mark worker.", i);
                    heap_clone.concurrent_mark_worker();
                    // println!("[GC THREAD {}] Mark worker finished.", i);
                }));
            }
            if num_marker_threads == 0 { // If 0 threads specified, run marking on current thread
                // println!("[GC MAIN THREAD] Starting mark worker.");
                self.concurrent_mark_worker();
                // println!("[GC MAIN THREAD] Mark worker finished.");
            }


            for handle in marker_handles {
                handle.join().expect("GC marker thread panicked");
            }
        }

        let final_worklist_size = self.gray_worklist.lock().unwrap().len();
        if final_worklist_size == 0 {
            println!("[GC] Concurrent marking phase finished successfully.");
        } else {
            eprintln!("[GC] Error: Concurrent marking phase finished, but gray worklist is not empty (size: {}). This indicates a problem.", final_worklist_size);
        }


        println!("[GC] Starting sweep phase (compaction).");
        if let Err(e) = self.sweep_and_compact() {
            eprintln!("[GC] Error during sweep and compact phase: {:?}", e);
        }
        println!("[GC] Garbage collection cycle finished.");
    }

    fn initial_mark_roots(&self) {
        let roots_guard = self.roots.lock().unwrap();
        // Intend to lock memory and gray_worklist together to ensure consistent view for roots
        let memory_guard = self.memory.lock().unwrap();
        let mut gray_worklist_guard = self.gray_worklist.lock().unwrap();

        for &root_index in roots_guard.iter() {
            if root_index < memory_guard.len() { // Check bounds
                let obj_mark = &memory_guard[root_index]; // Safe due to bounds check
                if obj_mark.state.compare_exchange(WHITE, GRAY, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                    gray_worklist_guard.push_back(root_index);
                }
            } else {
                eprintln!("[GC] Stale root index detected during initial mark: {}", root_index);
            }
        }
    }

    fn concurrent_mark_worker(&self) {
        loop {
            let current_obj_index = {
                let mut worklist_guard = self.gray_worklist.lock().unwrap();
                worklist_guard.pop_front()
            };

            if let Some(obj_idx) = current_obj_index {
                let memory_guard = self.memory.lock().unwrap();

                let obj_mark = match memory_guard.get(obj_idx) {
                    Some(m) => m,
                    None => { continue; } // Object disappeared? Should be rare if index was valid.
                };

                // Attempt to transition from GRAY to BLACK.
                if obj_mark.state.compare_exchange(GRAY, BLACK, Ordering::AcqRel, Ordering::Acquire).is_err() {
                    // Already processed (e.g., became BLACK, or some other state). Skip.
                    continue;
                }

                let mut children_to_gray = Vec::new();
                obj_mark.value.visit_children(|child_gc| {
                    if child_gc.index < memory_guard.len() { // Check bounds for child
                        let child_mark = &memory_guard[child_gc.index]; // Safe due to bounds check
                        if child_mark.state.compare_exchange(WHITE, GRAY, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                            children_to_gray.push(child_gc.index);
                        }
                    }
                });

                drop(memory_guard); // Release memory lock before (potentially) locking gray_worklist

                if !children_to_gray.is_empty() {
                    let mut worklist_guard = self.gray_worklist.lock().unwrap();
                    for child_idx in children_to_gray {
                        worklist_guard.push_back(child_idx);
                    }
                }
            } else {
                break; // Worklist is empty
            }
        }
    }

    fn sweep_and_compact(&self) -> Result<(), VmError> {
        let mut memory_guard = self.memory.lock().unwrap();
        let old_len = memory_guard.len();
        let mut new_memory: Vec<GcMark> = Vec::with_capacity(old_len);
        let mut old_to_new_index: HashMap<usize, usize> = HashMap::with_capacity(old_len / 2 + 1); // Pre-allocate

        // Pass 1: Identify live objects and map old indices to new indices
        for (old_idx, obj_mark) in memory_guard.iter().enumerate() {
            let state = obj_mark.state.load(Ordering Acquire);
            if state == BLACK {
                old_to_new_index.insert(old_idx, new_memory.len());
                // Temporarily push a placeholder; value will be updated in Pass 2
                // For now, we just need to reserve space to know the new index.
                // This is inefficient; better to calculate all new indices first, then copy.
                // Let's adjust: calculate new indices, then copy.
                // The new_memory.len() when a BLACK object is encountered *is* its new index.
                new_memory.push(GcMark { state: AtomicI8::new(WHITE) /* Placeholder state */, value: Value::Number(0) /* Placeholder value */ });
            } else if state == GRAY {
                eprintln!("[GC SWEEP] Error: Object {} found in GRAY state during sweep. Treating as live.", old_idx);
                old_to_new_index.insert(old_idx, new_memory.len());
                new_memory.push(GcMark { state: AtomicI8::new(WHITE), value: Value::Number(0) });
            }
        }

        let live_object_count = new_memory.len();
        new_memory.clear(); // Clear placeholders, will re-fill with actual data.

        // Pass 2: Copy live objects to new_memory and remap their internal pointers.
        // Objects are reset to WHITE in their new locations.
        for (old_idx, obj_mark) in memory_guard.iter_mut().enumerate() { // iter_mut to potentially take ownership of value
            if old_to_new_index.contains_key(&old_idx) { // Is this object live?
                let new_idx = *old_to_new_index.get(&old_idx).unwrap(); // new_idx is its final position in new_memory

                // Take ownership of the value to modify it.
                // This requires GcMark.value to not be behind a shared ref if we are moving it.
                // For simplicity, we clone and then remap.
                let mut new_value = obj_mark.value.clone();
                new_value.remap_children(&old_to_new_index)?;

                // Ensure new_memory has space up to new_idx. This should be managed by pushing in order.
                // This part of the logic for Pass 1 & 2 needs refinement for direct fill.

                // Refined Pass 1 & 2 logic:
                // Pass 1: Just build old_to_new_index.
                // Pass 2: Iterate old memory. If object at old_idx is live, copy its GcMark.value,
                //         remap its children using old_to_new_index, set state to WHITE,
                //         and push to new_memory. The order of push determines the new index.
            }
        }

        // Corrected Pass 1 & 2 logic structure:
        old_to_new_index.clear(); // Reset for correct calculation
        let mut temp_live_objects_data : Vec<(Value, usize)> = Vec::new(); // Store (value, old_idx) for remapping

        for (old_idx, obj_mark) in memory_guard.iter().enumerate() {
            let state = obj_mark.state.load(Ordering::Acquire);
            if state == BLACK || (state == GRAY) { // Treat GRAY as live if found (error state)
                if state == GRAY {
                    eprintln!("[GC SWEEP] Error: Object {} found in GRAY state during sweep. Treating as live.", old_idx);
                }
                old_to_new_index.insert(old_idx, temp_live_objects_data.len());
                temp_live_objects_data.push((obj_mark.value.clone(), old_idx));
            }
        }

        for (new_idx, (mut value, _old_idx)) in temp_live_objects_data.into_iter().enumerate() {
            // old_idx might be useful if remapping requires knowing original parent index, but not directly here.
            value.remap_children(&old_to_new_index)?;
            new_memory.push(GcMark {
                state: AtomicI8::new(WHITE), // Reset to WHITE for the next cycle
                value,
            });
        }


        // Pass 3: Update roots
        let mut roots_guard = self.roots.lock().unwrap();
        let mut new_roots = HashSet::new();
        for &old_root_idx in roots_guard.iter() {
            if let Some(&new_root_idx) = old_to_new_index.get(&old_root_idx) {
                new_roots.insert(new_root_idx);
            }
        }
        *roots_guard = new_roots;

        // Replace old memory with compacted memory
        *memory_guard = new_memory;

        println!("[GC SWEEP] Compaction complete. Old size: {}, New size: {}. Roots: {}", old_len, memory_guard.len(), roots_guard.len());
        Ok(())
    }

    fn clone_for_threading(&self) -> Self {
        Heap {
            roots: Arc::clone(&self.roots),
            memory: Arc::clone(&self.memory),
            gray_worklist: Arc::clone(&self.gray_worklist),
        }
    }

    pub fn write_barrier(&self, parent_obj_gc: Gc<Value>, new_child_gc: Gc<Value>) {
        let memory_guard = self.memory.lock().unwrap();

        let parent_mark = match memory_guard.get(parent_obj_gc.index) {
            Some(m) => m, None => return,
        };
        let child_mark = match memory_guard.get(new_child_gc.index) {
            Some(m) => m, None => return,
        };

        let parent_state = parent_mark.state.load(Ordering::Relaxed);

        if parent_state == BLACK { // Only care if parent is black
            // Try to transition child from WHITE to GRAY
            if child_mark.state.compare_exchange(WHITE, GRAY, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                // Child was WHITE and is now GRAY. Add to worklist.
                drop(memory_guard); // Release memory lock before acquiring worklist lock

                let mut worklist_guard = self.gray_worklist.lock().unwrap();
                worklist_guard.push_back(new_child_gc.index);
            }
            // If CAS fails, child is no longer WHITE (e.g., already GRAY/BLACK). This is fine.
        }
    }
}

impl<T> Gc<T> {
    pub fn as_any(&self) -> Gc<Value> { self.transmute() }
    pub fn transmute<U>(&self) -> Gc<U> { Gc { index: self.index, phantom: PhantomData::default() } }

    /// Unboxes the Gc pointer to get a reference to the value.
    /// This version is highly unsafe if used carelessly during concurrent GC phases.
    /// It's intended for use when the heap is stable (e.g., STW or by GC-internal logic).
    /// The lifetime 'heap_borrow is bound to the &Heap, implying the caller ensures safety.
    pub fn unbox_value<'heap_borrow>(&self, heap: &'heap_borrow Heap) -> Result<&'heap_borrow Value, VmError> {
        let memory_guard = heap.memory.lock().unwrap(); // Lock stays for the lifetime of the returned borrow

        match memory_guard.get(self.index) {
            Some(gc_mark) => {
                let state = gc_mark.state.load(Ordering::Acquire);
                if state == BLACK || state == WHITE { // Valid states to read
                    // This is the tricky part: returning a reference from within the MutexGuard.
                    // To make this sound, the reference must not outlive the guard.
                    // One common way is to use `std::mem::transmute` to extend the lifetime,
                    // but this is `unsafe` and relies on the caller to uphold invariants.
                    let value_ref: &'heap_borrow Value = unsafe {
                        // Transmute the lifetime of the reference to 'heap_borrow.
                        // This is only safe if memory_guard (and the data it protects)
                        // effectively lives as long as 'heap_borrow from the caller's perspective.
                        // This typically means the caller is not holding onto the reference
                        // across GC cycles or mutations of this specific object.
                        std::mem::transmute::<&Value, &'heap_borrow Value>(&gc_mark.value)
                    };
                    Ok(value_ref)
                } else {
                    Err(VmError::InvalidObjectState)
                }
            }
            None => Err(VmError::ObjectNotFound),
        }
    }

    // Helper to get a clone of the value, safer than direct ref in concurrent scenarios
    pub fn get_cloned_value(&self, heap: &Heap) -> Result<Value, VmError> {
        let memory_guard = heap.memory.lock().unwrap();
        match memory_guard.get(self.index) {
            Some(gc_mark) => {
                let state = gc_mark.state.load(Ordering::Acquire);
                if state == BLACK || state == WHITE {
                    Ok(gc_mark.value.clone())
                } else {
                    Err(VmError::InvalidObjectState)
                }
            }
            None => Err(VmError::ObjectNotFound),
        }
    }
}