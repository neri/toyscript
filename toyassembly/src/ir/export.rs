use crate::*;
use ir::{index::*, WasmSectionId};
use leb128::{Leb128Writer, WriteError, WriteLeb128};

#[derive(Default)]
pub struct Exports(pub(super) Vec<Export>);

#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Debug)]
pub enum ExportDesc {
    Func(FuncIndex),
    Table(TableIndex),
    Memory(MemoryIndex),
    Global(GlobalIndex),
}

impl Exports {
    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                writer.write(&item.name)?;
                match item.desc {
                    ExportDesc::Func(v) => {
                        writer.write(0)?;
                        writer.write(v.as_usize())?;
                    }
                    ExportDesc::Table(v) => {
                        writer.write(1)?;
                        writer.write(v.as_usize())?;
                    }
                    ExportDesc::Memory(v) => {
                        writer.write(2)?;
                        writer.write(v.as_usize())?;
                    }
                    ExportDesc::Global(v) => {
                        writer.write(3)?;
                        writer.write(v.as_usize())?;
                    }
                }
            }
        }
        Ok(WasmSectionId::Export)
    }

    pub fn export<S: ?Sized + ToString>(
        &mut self,
        name: &S,
        desc: ExportDesc,
    ) -> Result<(), AssembleError> {
        self.0.push(Export {
            name: name.to_string(),
            desc,
        });
        Ok(())
    }
}

impl core::fmt::Debug for Exports {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}
