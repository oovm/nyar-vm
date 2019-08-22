use std::ops::Range;
use nyar_error::{ArcStr, NyarError};
use nyar_lir::Instruction;
use crate::NyarProgram;

pub struct NyarCompiler {
    errors: Vec<NyarError>,
}
pub struct NyarCompiled {
    bytecode: Vec<Instruction>,
    errors: Vec<NyarError>,
    span: Range<usize>,
    file: ArcStr,
}
impl NyarCompiler {
    pub fn compile(&mut self, ast: NyarProgram) -> nyar_error::Result<NyarCompiled> {
        todo!()
    }
}
