use crate::{Gc, NyarValue};
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct NyarObject {
    dict: IndexMap<Gc<String>, Gc<NyarValue>>,
}
