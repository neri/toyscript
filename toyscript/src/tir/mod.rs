//! ToyScript Intermediate Representation

mod _opcode;
mod code;
mod function;
mod module;
pub use _opcode::*;
pub use code::*;
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
