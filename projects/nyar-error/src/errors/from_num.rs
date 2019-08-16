use crate::{NyarError, NyarErrorKind};
use num::bigint::ParseBigIntError;

impl From<ParseBigIntError> for NyarError {
    fn from(error: ParseBigIntError) -> Self {
        NyarErrorKind::Decode { format: "Integer".to_string(), message: error.to_string() }.into()
    }
}
