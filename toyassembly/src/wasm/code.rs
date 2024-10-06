//! Normal code block

use crate::*;
use code_tir::FromToyIR;
use ir::{index::*, Module};
use leb128::{Leb128Writer, WriteLeb128};
use toyir::CodeStreamIter;
use types::ValType;
use wasm::opcode::WasmOpcode;

mod code_tir;

#[derive(Debug)]
pub enum Code {
    Binary(Binary),
    ToyIr(ToyIr),
}

pub struct Binary {
    bytes: Vec<u8>,
}

pub struct ToyIr(pub Arc<Vec<u32>>);

impl Binary {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl core::fmt::Debug for Binary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        DumpHex(&self.bytes).fmt(f)
    }
}

impl core::fmt::Debug for ToyIr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(CodeStreamIter::new(&self.0))
            .finish()
    }
}

impl Code {
    pub fn assemble(
        &mut self,
        module: &Module,
        results: &[ValType],
        locals: &[ValType],
        _local_ids: &BTreeMap<String, LocalIndex>,
        params_and_locals: &[ValType],
    ) -> Result<(), AssembleError> {
        match self {
            Code::Binary(_) => Ok(()),
            Code::ToyIr(tir) => {
                FromToyIR::assemble(&tir.0, module, results, locals, params_and_locals)
                    .map(|bytes| *self = Code::Binary(Binary { bytes }))
            }
        }
    }

    #[inline]
    fn assemble_locals(locals: &[ValType], writer: &mut Leb128Writer) {
        let mut locals = locals.iter().map(|v| v.as_bytecode());
        let mut total_local_num = 0;
        let mut local_writer = Leb128Writer::new();
        if let Some(first) = locals.next() {
            let mut current_num = 1usize;
            let mut current_type = first;
            for next in locals {
                if current_type == next {
                    current_num += 1;
                } else {
                    local_writer.write(current_num).unwrap();
                    local_writer.write(current_type).unwrap();
                    total_local_num += 1;
                    current_num = 1;
                    current_type = next;
                }
            }
            local_writer.write(current_num).unwrap();
            local_writer.write(current_type).unwrap();
            total_local_num += 1;
        }
        writer.write(total_local_num).unwrap();
        writer.write_bytes(&local_writer.into_vec()).unwrap();
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockInstType {
    Block,
    Loop,
    If,
    Else,
}

impl BlockInstType {
    #[inline]
    pub fn as_wasm(&self) -> WasmOpcode {
        match self {
            BlockInstType::Block => WasmOpcode::Block,
            BlockInstType::Loop => WasmOpcode::Loop,
            BlockInstType::If => WasmOpcode::If,
            BlockInstType::Else => WasmOpcode::Else,
        }
    }
}
