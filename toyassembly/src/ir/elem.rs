use crate::*;
use ir::{index::FuncIndex, WasmSectionId};
use leb128::{Leb128Writer, WriteError, WriteLeb128};
use wasm::expr::ConstExpr;

#[derive(Default)]
pub struct Elems(pub(super) Vec<Elem>);

impl Elems {
    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                writer.write(0)?;
                item.offset.write_to_wasm(writer)?;
                writer.write(item.elemlist.len())?;
                for elem in item.elemlist.iter() {
                    writer.write(elem.as_usize())?;
                }
            }
        }
        Ok(WasmSectionId::Element)
    }
}

impl core::fmt::Debug for Elems {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

#[derive(Debug)]
pub struct Elem {
    offset: ConstExpr,
    elemlist: Vec<FuncIndex>,
}
