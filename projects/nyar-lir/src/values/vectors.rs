use super::*;

#[derive(Debug, Clone)]
pub struct NyarVector {
    list: VecDeque<Gc<NyarValue>>,
}
