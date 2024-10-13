//! WebAssembly Types

#[path = "../_generated/valtype.rs"]
mod _valtype;
pub use _valtype::*;

use crate::WasmBinding;
use toyir::Primitive;

impl ValType {
    #[inline]
    pub const fn bits_of(&self) -> usize {
        match self {
            ValType::I32 | ValType::F32 => 32,
            ValType::I64 | ValType::F64 => 64,
        }
    }

    #[inline]
    pub const fn size_of(&self) -> usize {
        match self {
            ValType::I32 | ValType::F32 => 4,
            ValType::I64 | ValType::F64 => 8,
        }
    }

    #[inline]
    pub const fn align_of(&self) -> usize {
        self.size_of()
    }

    #[inline]
    pub fn int_for_bits(bits: usize) -> Result<ValType, ()> {
        match bits {
            32 => Ok(ValType::I32),
            64 => Ok(ValType::I64),
            _ => Err(()),
        }
    }

    #[inline]
    pub fn as_bytecode(&self) -> isize {
        match self {
            ValType::I32 => -1,
            ValType::I64 => -2,
            ValType::F32 => -3,
            ValType::F64 => -4,
        }
    }

    #[inline]
    pub fn signature(&self) -> &str {
        match self {
            ValType::I32 => "i",
            ValType::I64 => "l",
            ValType::F32 => "f",
            ValType::F64 => "d",
        }
    }
}

impl WasmBinding<ValType, ()> for Primitive {
    fn wasm_binding(&self) -> Result<ValType, ()> {
        match self {
            Primitive::I8
            | Primitive::U8
            | Primitive::I16
            | Primitive::U16
            | Primitive::I32
            | Primitive::U32 => Ok(ValType::I32),

            Primitive::I64 | Primitive::U64 => Ok(ValType::I64),

            Primitive::F32 => Ok(ValType::F32),

            Primitive::F64 => Ok(ValType::F64),

            // Primitive::Void
            _ => Err(()),
        }
    }
}
