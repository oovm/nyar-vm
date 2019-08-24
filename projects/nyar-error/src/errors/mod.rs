use arcstr::ArcStr;
use std::ops::Range;

mod display;
mod from_num;
mod from_std;
pub type Result<T> = std::result::Result<T, NyarError>;

#[derive(Clone, PartialEq)]
pub struct NyarError {
    kind: Box<NyarErrorKind>,
    span: Range<usize>,
    file: ArcStr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NyarErrorKind {
    Decode {
        format: String,
        message: String,
    },
    Encode {
        format: String,
        message: String,
    },
    Custom {
        message: String,
    },
    /// 堆内存错误
    UseAfterFree {
        /// 错误类型
        address: usize,
    },
}

/// 堆内存错误类型
#[derive(Clone, Debug, PartialEq)]
pub enum HeapErrorKind {
    /// 访问已释放的内存
    DeadMemoryAccess,
    /// 访问无效的内存地址
    InvalidMemoryAccess,
}

impl NyarError {
    pub fn custom(message: impl ToString) -> NyarError {
        NyarErrorKind::Custom { message: message.to_string() }.into()
    }

    pub fn use_after_free(index: usize) -> NyarError {
        NyarErrorKind::UseAfterFree { address: index }.into()
    }
}
