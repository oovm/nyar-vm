use super::*;

impl<T> Display for Gc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc(0x{})", self.index)
    }
}

impl<T> Gc<T> {
    /// 将GC指针转换为任意类型的GC指针
    pub fn as_any(&self) -> Gc<NyarValue> {
        self.transmute()
    }

    /// 将GC指针转换为指定类型的GC指针
    pub fn transmute<U>(&self) -> Gc<U> {
        Gc { index: self.index, phantom: PhantomData::default() }
    }

    /// 解引用 GC指针，获取 Value 类型的值
    pub fn unbox<'gc>(&self, heap: &'gc Heap) -> Result<T>
    where
        T: TryFrom<&'gc NyarValue, Error = NyarError>,
    {
        self.view_ref(heap)?.try_into()
    }

    /// 解引用 GC指针，获取 Heap 类型的指针
    pub fn deref<'gc>(&self, heap: &'gc Heap) -> Result<&'gc T>
    where
        &'gc T: TryFrom<&'gc NyarValue, Error = NyarError>,
    {
        self.view_ref(heap)?.try_into()
    }

    pub fn view_ref<'gc>(&self, heap: &'gc Heap) -> Result<&'gc NyarValue> {
        match heap.memory.get(self.index) {
            Some(s) => {
                // 不能访问已回收的对象
                if !s.dead {
                    return Err(todo!());
                }
                Ok(&s.value)
            }
            None => Err(todo!()),
        }
    }

    pub fn view_mut<'gc>(&self, heap: &'gc mut Heap) -> Result<&'gc mut NyarValue> {
        match heap.memory.get_mut(self.index) {
            Some(s) => {
                // 不能解引用已死亡的对象
                if !s.dead {
                    return Err(todo!());
                }
                Ok(&mut s.value)
            }
            None => Err(todo!()),
        }
    }
}
