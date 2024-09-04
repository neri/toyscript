//! TotAssembly Types

mod _valtype;
pub use _valtype::*;

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
