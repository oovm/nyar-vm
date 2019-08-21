mod display;
mod from_num;
mod from_std;
pub type Result<T> = std::result::Result<T, NyarError>;

#[derive(Clone, PartialEq)]
pub struct NyarError {
    kind: Box<NyarErrorKind>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NyarErrorKind {
    Decode { format: String, message: String },
    Encode { format: String, message: String },
    Custom { message: String },
}

impl NyarError {
    pub fn custom(message: impl ToString) -> NyarError {
        NyarErrorKind::Custom { message: message.to_string() }.into()
    }
}
