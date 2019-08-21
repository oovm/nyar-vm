use std::{cell::RefCell, collections::HashSet, marker::PhantomData};

mod errors;
mod heap;
mod values;

pub use crate::{
    errors::VmError,
    heap::{GcCell, GcState, Heap},
    values::Value,
};

#[derive(Debug, Clone)]
pub struct Gc<T> {
    pub index: usize,
    pub phantom: PhantomData<T>,
}

impl<T> Gc<T> {
    pub fn as_any(&self) -> Gc<Value> {
        self.transmute()
    }

    pub fn transmute<U>(&self) -> Gc<U> {
        Gc { index: self.index, phantom: PhantomData::default() }
    }

    pub fn deref<'vm, V>(&self, heap: &'vm Heap) -> Result<V, VmError>
    where
        V: TryFrom<&'vm Value, Error = VmError>,
    {
        if self.index >= heap.memory.len() {
            return Err(VmError::IndexOutOfBounds);
        }
        let marker = &heap.memory[self.index];
        // After GC, objects in 'memory' are live. Their GcState is reset to Unmarked.
        V::try_from(&marker.value)
    }
}
