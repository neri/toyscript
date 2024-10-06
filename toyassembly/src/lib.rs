//! A Lightweight toy language environment with high affinity for WebAssembly
#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[cfg(test)]
pub mod tests;

pub mod error;
pub mod ir;
pub mod types;
pub mod wasm;

#[allow(unused)]
pub(crate) use alloc::{
    borrow::Cow,
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
#[allow(unused)]
pub(crate) use core::{convert::Infallible, ops::ControlFlow};
pub(crate) use error::*;

pub struct ToyAssembly;

pub trait WasmBinding<T, E> {
    fn wasm_binding(&self) -> Result<T, E>;
}

pub struct DumpHex<'a, T: core::fmt::Debug + core::fmt::LowerHex>(&'a [T]);

impl<T: core::fmt::Debug + core::fmt::LowerHex> core::fmt::Debug for DumpHex<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        let mut needs_newline = false;
        for line in self.0.chunks(8) {
            write!(f, "\n    ")?;
            for item in line {
                write!(f, "0x{:x}, ", item)?;
            }
            needs_newline = true;
        }
        if needs_newline {
            writeln!(f, "")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
