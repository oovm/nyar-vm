use std::marker::PhantomData;
use nyar_gc::{Gc, Heap, Value};



#[test]
fn test_alloc_and_unbox() {
    let mut heap = Heap::default();
    let num_gc: Gc<Value> = heap.alloc(Value::Number(42), true); // Explicit type for Gc for unbox
    let val: i32 = num_gc.deref(&heap).unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_simple_gc_no_dead_objects() {
    let mut heap = Heap::default();
    let _num1_gc: Gc<Value> = heap.alloc(Value::Number(10), true);
    let _num2_gc: Gc<Value> = heap.alloc(Value::Number(20), true);

    assert_eq!(heap.memory.len(), 2);
    heap.collect();
    assert_eq!(heap.memory.len(), 2);
    assert_eq!(heap.roots.len(), 2);
}

#[test]
fn test_gc_collect_one_dead_object() {
    let mut heap = Heap::default();
    let num1_gc: Gc<Value> = heap.alloc(Value::Number(10), true);
    let _num2_gc: Gc<Value> = heap.alloc(Value::Number(20), false);

    assert_eq!(heap.memory.len(), 2);
    heap.collect();
    assert_eq!(heap.memory.len(), 1);
    assert_eq!(heap.roots.len(), 1);

    let root_idx = *heap.roots.iter().next().unwrap();
    assert_eq!(root_idx, 0); // num1_gc should now be at index 0

    // Create a new Gc pointing to the new location to unbox
    let unboxed_val: i32 = Gc::<Value>{index: root_idx, phantom: PhantomData}.deref(&heap).unwrap();
    assert_eq!(unboxed_val, 10);

    // The original num1_gc.index might be stale if not updated by user code.
    // GC only updates pointers *within* the heap.
}

#[test]
fn test_gc_list_and_compaction() {
    let mut heap = Heap::default();

    let _dead_gc: Gc<Value> = heap.alloc(Value::Number(100), false); // index 0
    let item1_gc: Gc<Value> = heap.alloc(Value::Number(10), false);  // index 1
    let item2_gc: Gc<Value> = heap.alloc(Value::Number(20), false);  // index 2
    let list_val = Value::List(vec![item1_gc.as_any(), item2_gc.as_any()]);
    let list_gc: Gc<Value> = heap.alloc(list_val, true);            // index 3, root

    assert_eq!(heap.memory.len(), 4);
    heap.collect();

    assert_eq!(heap.memory.len(), 3, "Heap should have 3 live objects");
    assert_eq!(heap.roots.len(), 1, "Heap should have 1 root");

    let root_idx = *heap.roots.iter().next().unwrap();
    assert_eq!(root_idx, 2, "Root list should be at new index 2"); // dead collected, item1->0, item2->1, list->2

    let list_ref: Vec<Gc<Value>> = Gc::<Value>{index: root_idx, phantom: PhantomData}.deref(&heap).unwrap();
    assert_eq!(list_ref.len(), 2, "List should contain 2 items");

    let unboxed_item1: i32 = list_ref[0].deref(&heap).unwrap();
    assert_eq!(unboxed_item1, 10);
    assert_eq!(list_ref[0].index, 0, "Pointer to item1 should be updated to 0");

    let unboxed_item2: i32 = list_ref[1].deref(&heap).unwrap();
    assert_eq!(unboxed_item2, 20);
    assert_eq!(list_ref[1].index, 1, "Pointer to item2 should be updated to 1");

    assert_eq!(heap.memory[0].value.as_number().unwrap(), 10);
    assert_eq!(heap.memory[1].value.as_number().unwrap(), 20);
    if let Value::List(l_final) = &heap.memory[2].value {
        assert_eq!(l_final[0].index, 0);
        assert_eq!(l_final[1].index, 1);
    } else {
        panic!("Root object is not a list after GC");
    }
}

#[test]
fn test_dealloc_root() {
    let mut heap = Heap::default();
    let num1_gc: Gc<Value> = heap.alloc(Value::Number(10), true);
    let _num2_gc: Gc<Value> = heap.alloc(Value::Number(20), true);

    assert_eq!(heap.roots.len(), 2);
    heap.dealloc(num1_gc.clone()); // num1_gc is no longer a root
    assert_eq!(heap.roots.len(), 1);

    heap.collect();
    assert_eq!(heap.memory.len(), 1);
    assert_eq!(heap.roots.len(), 1);

    let root_idx = *heap.roots.iter().next().unwrap();
    let val: i32 = Gc::<Value>{index: root_idx, phantom: PhantomData}.deref(&heap).unwrap();
    assert_eq!(val, 20);
}

#[test]
fn test_complex_list_scenario_with_shared_elements_and_cycles() {
    let mut heap = Heap::default();

    // Shared elements
    let shared_num_gc: Gc<Value> = heap.alloc(Value::Number(555), false); // idx 0

    // List 1 (root)
    let item1_l1_gc: Gc<Value> = heap.alloc(Value::Number(10), false);   // idx 1
    let list1_val = Value::List(vec![item1_l1_gc.as_any(), shared_num_gc.as_any()]);
    let list1_gc: Gc<Value> = heap.alloc(list1_val, true); // idx 2, root

    // List 2 (root)
    let item1_l2_gc: Gc<Value> = heap.alloc(Value::Number(20), false);   // idx 3
    let list2_val = Value::List(vec![item1_l2_gc.as_any(), shared_num_gc.as_any()]);
    let list2_gc: Gc<Value> = heap.alloc(list2_val, true); // idx 4, root

    // Unrooted object
    let _dead_gc: Gc<Value> = heap.alloc(Value::Number(999), false);    // idx 5

    // Create a cycle: list1 contains a pointer to list2
    if let Value::List(ref mut children_l1) = heap.memory[list1_gc.index].value {
        children_l1.push(list2_gc.as_any());
    }

    // State before GC:
    // 0: Number(555) (shared)
    // 1: Number(10)  (item_l1)
    // 2: List([Gc(1), Gc(0), Gc(4)]) (list1, root)
    // 3: Number(20)  (item_l2)
    // 4: List([Gc(3), Gc(0)]) (list2, root)
    // 5: Number(999) (dead)
    // Roots: {2, 4}

    assert_eq!(heap.memory.len(), 6);
    heap.collect();

    // Expected after GC: dead_gc (idx 5) is collected. Others survive.
    // Order in new_memory: 555, 10, list1, 20, list2 (or some permutation)
    // New indices:
    // shared_num_gc -> 0
    // item1_l1_gc   -> 1
    // list1_gc      -> 2
    // item1_l2_gc   -> 3
    // list2_gc      -> 4

    assert_eq!(heap.memory.len(), 5, "Heap should have 5 live objects");
    assert_eq!(heap.roots.len(), 2, "Heap should have 2 roots"); // list1 and list2

    // Find new indices for list1_gc and list2_gc from the roots
    // This assumes specific new indices based on a plausible compaction order.
    // A more robust test would iterate roots and check values.
    let mut new_list1_idx_opt = None;
    let mut new_list2_idx_opt = None;

    for &root_idx in &heap.roots {
        if let Value::List(ref children) = heap.memory[root_idx].value {
            // Check if it's list1 (contains item1_l1_gc which has value 10, and shared_num_gc 555)
            // and potentially list2_gc
            let first_val_opt: Option<i32> = if !children.is_empty() { children[0].deref(&heap).ok() } else {None};

            if children.len() >=2 && first_val_opt == Some(10) { // Likely list1
                new_list1_idx_opt = Some(root_idx);
            } else if children.len() >= 2 && first_val_opt == Some(20) { // Likely list2
                new_list2_idx_opt = Some(root_idx);
            }
        }
    }

    assert!(new_list1_idx_opt.is_some(), "list1 not found in roots");
    assert!(new_list2_idx_opt.is_some(), "list2 not found in roots");

    let new_list1_idx = new_list1_idx_opt.unwrap();
    let new_list2_idx = new_list2_idx_opt.unwrap();

    // Unbox list1
    let list1_children: Vec<Gc<Value>> = Gc::<Value>{index: new_list1_idx, phantom: PhantomData}.deref(&heap).unwrap();
    assert_eq!(list1_children.len(), 3); // item1_l1, shared_num, list2

    // Check list1's children
    let l1_child1_val: i32 = list1_children[0].deref(&heap).unwrap(); // item1_l1
    let l1_child2_val: i32 = list1_children[1].deref(&heap).unwrap(); // shared_num
    assert_eq!(l1_child1_val, 10);
    assert_eq!(l1_child2_val, 555);

    // Check that the Gc for shared_num in list1 points to the new shared_num location
    // This requires knowing the new index of shared_num. Let's assume it's 0.
    let expected_shared_idx = 0; // Based on original allocation order of live objects
    assert_eq!(list1_children[1].index, expected_shared_idx);

    // Check the Gc for list2 within list1 points to new_list2_idx
    assert_eq!(list1_children[2].index, new_list2_idx);


    // Unbox list2
    let list2_children: Vec<Gc<Value>> = Gc::<Value>{index: new_list2_idx, phantom: PhantomData}.deref(&heap).unwrap();
    assert_eq!(list2_children.len(), 2); // item1_l2, shared_num

    // Check list2's children
    let l2_child1_val: i32 = list2_children[0].deref(&heap).unwrap(); // item1_l2
    let l2_child2_val: i32 = list2_children[1].deref(&heap).unwrap(); // shared_num
    assert_eq!(l2_child1_val, 20);
    assert_eq!(l2_child2_val, 555);
    assert_eq!(list2_children[1].index, expected_shared_idx); // Points to the same shared_num
}
