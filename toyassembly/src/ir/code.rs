use super::Module;
use crate::*;
use ir::{index::LocalIndex, WasmSectionId};
use leb128::{Leb128Writer, WriteError, WriteLeb128};
use types::ValType;
use wasm::code::Code;

#[derive(Default)]
pub struct Codes(pub(super) Vec<FuncCode>);

impl Codes {
    #[inline]
    pub fn drain(&mut self) -> Vec<FuncCode> {
        let mut result = Vec::new();
        core::mem::swap(&mut self.0, &mut result);
        result
    }

    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                match item.content {
                    Code::Source(_) | Code::ToyIr(_) => panic!("panics"),
                    Code::Binary(ref bin) => {
                        writer.write_blob(bin.as_bytes())?;
                    }
                }
            }
        }
        Ok(WasmSectionId::Code)
    }
}

impl core::fmt::Debug for Codes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

pub struct FuncCode {
    results: Vec<ValType>,
    locals: Vec<ValType>,
    local_ids: BTreeMap<String, LocalIndex>,
    local_and_params: Vec<ValType>,
    content: Code,
}

impl FuncCode {
    #[inline]
    pub fn new(
        results: Vec<ValType>,
        locals: Vec<ValType>,
        local_ids: BTreeMap<String, LocalIndex>,
        local_and_params: Vec<ValType>,
        content: Code,
    ) -> Self {
        Self {
            results,
            locals,
            local_ids,
            local_and_params,
            content,
        }
    }

    #[inline]
    pub fn assemble(&mut self, module: &Module) -> Result<(), AssembleError> {
        self.content.assemble(
            module,
            &self.results,
            &self.locals,
            &self.local_ids,
            &self.local_and_params,
        )
    }

    #[inline]
    pub fn content(&self) -> &Code {
        &self.content
    }
}

impl core::fmt::Debug for FuncCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncCode")
            .field("results", &self.results)
            // .field("locals", &self.locals)
            // .field("local_ids", &self.local_ids)
            .field("local_and_params", &self.local_and_params)
            .field("content", &self.content)
            .finish()
    }
}
