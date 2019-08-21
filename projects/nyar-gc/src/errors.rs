
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    InvalidType,
    ObjectMovedOrCollected,
    IndexOutOfBounds,
}