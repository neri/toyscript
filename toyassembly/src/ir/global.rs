use crate::*;
use ir::WasmSectionId;
use leb128::{Leb128Writer, WriteError, WriteLeb128};
use types::ValType;
use wasm::expr::ConstExpr;

#[derive(Default)]
pub struct Globals(pub(super) Vec<Global>);

#[derive(Debug)]
pub struct Global {
    global_type: GlobalType,
    expr: ConstExpr,
}

impl Globals {
    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                item.global_type.write_to_wasm(writer)?;
                item.expr.write_to_wasm(writer)?;
            }
        }
        Ok(WasmSectionId::Global)
    }
}

impl core::fmt::Debug for Globals {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalType {
    pub valtype: ValType,
    pub is_mut: bool,
}

impl GlobalType {
    #[inline]
    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<(), WriteError> {
        writer.write(self.valtype.as_bytecode())?;
        writer.write_byte(self.is_mut as u8)?;
        Ok(())
    }
}
