use crate::{Gc, NyarValue, values::NyarVector};
use indexmap::IndexMap;

#[derive(Debug, Clone, Default)]
pub struct NyarObject {
    dict: IndexMap<Gc<String>, Gc<NyarValue>>,
}

impl NyarObject {
    pub fn insert(&mut self, name: Gc<String>, value: Gc<NyarValue>) -> Option<Gc<NyarValue>> {
        self.dict.insert(name, value)
    }
}

impl From<NyarObject> for NyarValue {
    fn from(value: NyarObject) -> Self {
        NyarValue::Object(Box::new(value))
    }
}
