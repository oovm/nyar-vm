use crate::{Gc, VmError};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Number(i32),
    String(Box<str>),
    List(Vec<Gc<Value>>),
}

// Example TryFrom implementations (ensure these are present in your actual code)
impl<'a> TryFrom<&'a Value> for i32 {
    type Error = VmError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::Number(n) = value { Ok(*n) } else { Err(VmError::InvalidType) }
    }
}

impl<'a> TryFrom<&'a Value> for Box<str> {
    type Error = VmError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value { Ok(s.clone()) } else { Err(VmError::InvalidType) }
    }
}

impl<'a> TryFrom<&'a Value> for Vec<Gc<Value>> {
    type Error = VmError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::List(l) = value { Ok(l.clone()) } else { Err(VmError::InvalidType) }
    }
}
impl<'a> TryFrom<&'a Value> for Value {
    // For unboxing Gc<Value> directly
    type Error = VmError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Ok(value.clone())
    }
}

// Helper for tests
impl Value {
    pub fn as_number(&self) -> Option<i32> {
        if let Value::Number(n) = self { Some(*n) } else { None }
    }
}
