use nyar_error::ArcStr;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct NyarProgram {
    statements: NyarStatement,
    span: Range<usize>,
    file: ArcStr,
}

#[derive(Clone, Debug)]
pub enum NyarStatement {}
