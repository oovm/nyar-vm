use crate::{NyarError, NyarErrorKind};
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

impl Error for NyarError {}

impl Display for NyarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}
impl Debug for NyarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for NyarErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NyarErrorKind::Decode { format, message } => {
                write!(f, "Decode error: {}: {}", format, message)
            }
            NyarErrorKind::Encode { format, message } => {
                write!(f, "Encode error: {}: {}", format, message)
            }
            NyarErrorKind::Custom { message } => {
                write!(f, "Custom error: {}", message)
            }
            NyarErrorKind::UseAfterFree { address } => {
                write!(f, "Use after free error: {}", address)
            }
        }
    }
}
