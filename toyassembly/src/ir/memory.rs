use crate::*;
use core::num::NonZeroU32;
use ir::{Module, WasmSectionId};
use leb128::{Leb128Writer, WriteError, WriteLeb128};

#[derive(Default)]
pub struct Memories(pub(super) Vec<Memory>);

#[derive(Debug)]
pub struct Memory {
    pub min: u32,
    pub max: Option<NonZeroU32>,
}

impl Memories {
    pub(super) fn process_tir(module: &mut Module) -> Result<(), AssembleError> {
        // default memory
        if module.memories.0.len() == 0 {
            module.memories.0.push(Memory { min: 1, max: None });
        }

        Ok(())
    }

    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
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
        Ok(WasmSectionId::Memory)
    }
}

impl core::fmt::Debug for Memories {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}
