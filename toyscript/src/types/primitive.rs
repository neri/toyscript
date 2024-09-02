//! ToyScript Primitive Types

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// ToyScript Primitive Types
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Primitive {
    /// "bool"
    Bool,
    /// "f32"
    F32,
    /// "f64"
    F64,
    /// "i16"
    I16,
    /// "i32"
    I32,
    /// "i64"
    I64,
    /// "i8"
    I8,
    /// "u16"
    U16,
    /// "u32"
    U32,
    /// "u64"
    U64,
    /// "u8"
    U8,
    /// "void"
    Void,
}

impl Primitive {
    pub fn all_values() -> &'static [Self] {
        &[
            Self::Bool,
            Self::F32,
            Self::F64,
            Self::I16,
            Self::I32,
            Self::I64,
            Self::I8,
            Self::U16,
            Self::U32,
            Self::U64,
            Self::U8,
            Self::Void,
        ]
    }

    pub fn from_str(v: &str) -> Option<Self> {
        match v {
            "bool" => Some(Self::Bool),
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            "i16" => Some(Self::I16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "i8" => Some(Self::I8),
            "u16" => Some(Self::U16),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            "u8" => Some(Self::U8),
            "void" => Some(Self::Void),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U8 => "u8",
            Self::Void => "void",
        }
    }
}

impl core::fmt::Display for Primitive {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::fmt::Debug for Primitive {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}
