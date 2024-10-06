//! Constant Expression

use crate::*;
use leb128::{Leb128Writer, WriteError, WriteLeb128};
use wasm::opcode::WasmOpcode;

#[derive(Debug)]
pub struct ConstExpr(Vec<ConstInstr>);

#[derive(Debug, Clone, Copy)]
pub enum ConstInstr {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
}

impl ConstExpr {
    #[inline]
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<(), WriteError> {
        for instr in self.0.iter() {
            match instr {
                ConstInstr::I32Const(v) => {
                    writer.write_byte(WasmOpcode::I32Const.leading_byte())?;
                    writer.write(*v)?;
                }
                ConstInstr::I64Const(v) => {
                    writer.write_byte(WasmOpcode::I64Const.leading_byte())?;
                    writer.write(*v)?;
                }
                ConstInstr::F32Const(v) => {
                    writer.write_byte(WasmOpcode::F32Const.leading_byte())?;
                    writer.write(*v)?;
                }
                ConstInstr::F64Const(v) => {
                    writer.write_byte(WasmOpcode::F64Const.leading_byte())?;
                    writer.write(*v)?;
                }
            }
        }
        writer.write_byte(WasmOpcode::End.leading_byte())?;
        Ok(())
    }
}
