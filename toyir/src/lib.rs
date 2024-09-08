//! ToyIR: ToyScript Intermediate Representation
#![cfg_attr(not(test), no_std)]

#[path = "opcode/_irop.rs"]
mod _irop;
pub use _irop::*;

#[path = "primitive/_primitive.rs"]
mod _primitive;
pub use _primitive::*;

mod asm;
mod function;
mod module;
pub use asm::*;
pub use function::*;
pub use module::*;

pub mod error;
pub mod opt;

#[cfg(test)]
pub mod tests;

extern crate alloc;

#[allow(unused)]
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

impl Op {
    pub fn inverted_condition(&self) -> Op {
        match self {
            Op::Eq => Op::Ne,
            Op::Ne => Op::Eq,
            Op::LtS => Op::GeS,
            Op::GtS => Op::LeS,
            Op::LeS => Op::GtS,
            Op::GeS => Op::LtS,
            Op::LtU => Op::GeU,
            Op::GtU => Op::LeU,
            Op::LeU => Op::GtU,
            Op::GeU => Op::LtU,
            _ => unreachable!(),
        }
    }
}
