use super::*;
use crate::values::NyarObject;

impl<T> Display for Gc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc(0x{})", self.index)
    }
}

impl<T> Gc<T> {
    /// 将GC指针转换为指定类型的GC指针
    pub fn transmute<U>(&self) -> Gc<U> {
        Gc { index: self.index, phantom: PhantomData::default() }
    }
    /// 将GC指针转换为任意类型的GC指针
    pub fn as_any(&self) -> Gc<NyarValue> {
        self.transmute()
    }
    pub fn as_object<'gc>(&self, heap: &'gc mut Heap) -> Result<&'gc mut NyarObject> {
        match heap.view_mut(*self)? {
            NyarValue::Object(o) => Ok(o.as_mut()),
            _ => Err(NyarError::custom("Invalid type23".to_string())),
        }
    }
    /// 解引用 GC指针，获取 Value 类型的值
    pub fn unbox<'gc>(&self, heap: &'gc Heap) -> Result<T>
    where
        T: TryFrom<&'gc NyarValue, Error = NyarError>,
    {
        heap.view_ref(*self)?.try_into()
    }
    /// 解引用 GC指针，获取 Heap 类型的指针
    pub fn deref<'gc>(&self, heap: &'gc Heap) -> Result<&'gc T>
    where
        &'gc T: TryFrom<&'gc NyarValue, Error = NyarError>,
    {
        heap.view_ref(*self)?.try_into()
    }
}
