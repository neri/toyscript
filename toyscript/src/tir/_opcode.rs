//! ToyScript Intermediate Representation Opcodes

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// ToyScript Intermediate Representation Opcodes
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    /// "add"
    Add,
    /// "and"
    And,
    /// "block"
    Block,
    /// "br"
    Br,
    /// "br_if"
    BrIf,
    /// "call"
    Call,
    /// "clz"
    Clz,
    /// "ctz"
    Ctz,
    /// "dec"
    Dec,
    /// "div"
    Div,
    /// "drop"
    Drop,
    /// "end"
    End,
    /// "eq"
    Eq,
    /// "eqz"
    Eqz,
    /// "ge"
    Ge,
    /// "gt"
    Gt,
    /// "i32.const"
    I32Const,
    /// "i64.const"
    I64Const,
    /// "inc"
    Inc,
    /// "le"
    Le,
    /// "local.get"
    LocalGet,
    /// "local.set"
    LocalSet,
    /// "local.tee"
    LocalTee,
    /// "loop"
    Loop,
    /// "lt"
    Lt,
    /// "mul"
    Mul,
    /// "ne"
    Ne,
    /// "neg"
    Neg,
    /// "nop"
    Nop,
    /// "or"
    Or,
    /// "popcnt"
    Popcnt,
    /// "rem"
    Rem,
    /// "return"
    Return,
    /// "rotl"
    Rotl,
    /// "rotr"
    Rotr,
    /// "shl"
    Shl,
    /// "shr"
    Shr,
    /// "sub"
    Sub,
    /// "unreachable"
    Unreachable,
    /// "xor"
    Xor,
}

impl Op {
    pub fn all_values() -> &'static [Self] {
        &[
            Self::Add,
            Self::And,
            Self::Block,
            Self::Br,
            Self::BrIf,
            Self::Call,
            Self::Clz,
            Self::Ctz,
            Self::Dec,
            Self::Div,
            Self::Drop,
            Self::End,
            Self::Eq,
            Self::Eqz,
            Self::Ge,
            Self::Gt,
            Self::I32Const,
            Self::I64Const,
            Self::Inc,
            Self::Le,
            Self::LocalGet,
            Self::LocalSet,
            Self::LocalTee,
            Self::Loop,
            Self::Lt,
            Self::Mul,
            Self::Ne,
            Self::Neg,
            Self::Nop,
            Self::Or,
            Self::Popcnt,
            Self::Rem,
            Self::Return,
            Self::Rotl,
            Self::Rotr,
            Self::Shl,
            Self::Shr,
            Self::Sub,
            Self::Unreachable,
            Self::Xor,
        ]
    }

    pub fn from_str(v: &str) -> Option<Self> {
        match v {
            "add" => Some(Self::Add),
            "and" => Some(Self::And),
            "block" => Some(Self::Block),
            "br" => Some(Self::Br),
            "br_if" => Some(Self::BrIf),
            "call" => Some(Self::Call),
            "clz" => Some(Self::Clz),
            "ctz" => Some(Self::Ctz),
            "dec" => Some(Self::Dec),
            "div" => Some(Self::Div),
            "drop" => Some(Self::Drop),
            "end" => Some(Self::End),
            "eq" => Some(Self::Eq),
            "eqz" => Some(Self::Eqz),
            "ge" => Some(Self::Ge),
            "gt" => Some(Self::Gt),
            "i32.const" => Some(Self::I32Const),
            "i64.const" => Some(Self::I64Const),
            "inc" => Some(Self::Inc),
            "le" => Some(Self::Le),
            "local.get" => Some(Self::LocalGet),
            "local.set" => Some(Self::LocalSet),
            "local.tee" => Some(Self::LocalTee),
            "loop" => Some(Self::Loop),
            "lt" => Some(Self::Lt),
            "mul" => Some(Self::Mul),
            "ne" => Some(Self::Ne),
            "neg" => Some(Self::Neg),
            "nop" => Some(Self::Nop),
            "or" => Some(Self::Or),
            "popcnt" => Some(Self::Popcnt),
            "rem" => Some(Self::Rem),
            "return" => Some(Self::Return),
            "rotl" => Some(Self::Rotl),
            "rotr" => Some(Self::Rotr),
            "shl" => Some(Self::Shl),
            "shr" => Some(Self::Shr),
            "sub" => Some(Self::Sub),
            "unreachable" => Some(Self::Unreachable),
            "xor" => Some(Self::Xor),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::And => "and",
            Self::Block => "block",
            Self::Br => "br",
            Self::BrIf => "br_if",
            Self::Call => "call",
            Self::Clz => "clz",
            Self::Ctz => "ctz",
            Self::Dec => "dec",
            Self::Div => "div",
            Self::Drop => "drop",
            Self::End => "end",
            Self::Eq => "eq",
            Self::Eqz => "eqz",
            Self::Ge => "ge",
            Self::Gt => "gt",
            Self::I32Const => "i32.const",
            Self::I64Const => "i64.const",
            Self::Inc => "inc",
            Self::Le => "le",
            Self::LocalGet => "local.get",
            Self::LocalSet => "local.set",
            Self::LocalTee => "local.tee",
            Self::Loop => "loop",
            Self::Lt => "lt",
            Self::Mul => "mul",
            Self::Ne => "ne",
            Self::Neg => "neg",
            Self::Nop => "nop",
            Self::Or => "or",
            Self::Popcnt => "popcnt",
            Self::Rem => "rem",
            Self::Return => "return",
            Self::Rotl => "rotl",
            Self::Rotr => "rotr",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Sub => "sub",
            Self::Unreachable => "unreachable",
            Self::Xor => "xor",
        }
    }
}

impl core::fmt::Display for Op {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::fmt::Debug for Op {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}
