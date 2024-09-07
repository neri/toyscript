//! ToyIR: ToyScript Intermediate Representation

mod _opcode;
mod asm;
mod function;
mod module;
pub use _opcode::*;
pub use asm::*;
pub use function::*;
pub use module::*;

pub mod error;
pub mod opt;

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
