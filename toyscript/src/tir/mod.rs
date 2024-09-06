//! ToyIR: ToyScript Intermediate Representation

mod _opcode;
mod asm;
mod function;
mod module;
pub use _opcode::*;
pub use asm::*;
pub use function::*;
pub use module::*;
pub mod opt;

#[allow(unused)]
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
