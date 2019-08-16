use crate::{NyarError, NyarErrorKind};
use std::str::ParseBoolError;

impl From<NyarErrorKind> for NyarError {
    fn from(value: NyarErrorKind) -> Self {
        NyarError { kind: Box::new(value) }
    }
}

impl From<ParseBoolError> for NyarError {
    fn from(error: ParseBoolError) -> Self {
        NyarErrorKind::Decode { format: "Boolean".to_string(), message: error.to_string() }.into()
    }
}
