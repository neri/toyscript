use crate::*;
use core::num::NonZeroU32;
use ir::WasmSectionId;
use leb128::{Leb128Writer, WriteError, WriteLeb128};

#[derive(Default)]
pub struct Tables(pub(super) Vec<Table>);

#[derive(Debug)]
pub struct Table {
    pub min: u32,
    pub max: Option<NonZeroU32>,
    pub reftype: RefType,
}

impl Tables {
    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                item.reftype.write_to_wasm(writer)?;
                if let Some(max) = item.max {
                    writer.write(1)?;
                    writer.write(item.min)?;
                    writer.write(max.get())?;
                } else {
                    writer.write(0)?;
                    writer.write(item.min)?;
                }
            }
        }
        Ok(WasmSectionId::Table)
    }
}

impl core::fmt::Debug for Tables {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl RefType {
    #[inline]
    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<(), WriteError> {
        match self {
            RefType::FuncRef => writer.write_byte(0x70),
            RefType::ExternRef => writer.write_byte(0x6F),
        }
    }
}
