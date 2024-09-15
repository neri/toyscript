//! ToyScript Intermediate Representation Opcodes

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// ToyScript Intermediate Representation Opcodes
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    /// "unreachable"
    Unreachable,
    /// "nop"
    Nop,
    /// "block"
    Block,
    /// "loop"
    Loop,
    /// "end"
    End,
    /// "br"
    Br,
    /// "br_if"
    BrIf,
    /// "return"
    Return,
    /// "call"
    Call,
    /// "drop"
    Drop,
    /// "i32.const"
    I32Const,
    /// "i64.const"
    I64Const,
    /// "f32.const"
    F32Const,
    /// "f64.const"
    F64Const,
    /// "local.get"
    LocalGet,
    /// "local.set"
    LocalSet,
    /// "local.tee"
    LocalTee,
    /// "eqz"
    Eqz,
    /// "eq"
    Eq,
    /// "ne"
    Ne,
    /// "lt_s"
    LtS,
    /// "lt_u"
    LtU,
    /// "gt_s"
    GtS,
    /// "gt_u"
    GtU,
    /// "le_s"
    LeS,
    /// "le_u"
    LeU,
    /// "ge_s"
    GeS,
    /// "ge_u"
    GeU,
    /// "clz"
    Clz,
    /// "ctz"
    Ctz,
    /// "popcnt"
    Popcnt,
    /// "add"
    Add,
    /// "sub"
    Sub,
    /// "mul"
    Mul,
    /// "div_s"
    DivS,
    /// "div_u"
    DivU,
    /// "rem_s"
    RemS,
    /// "rem_u"
    RemU,
    /// "shl"
    Shl,
    /// "shr_s"
    ShrS,
    /// "shr_u"
    ShrU,
    /// "and"
    And,
    /// "or"
    Or,
    /// "xor"
    Xor,
    /// "rotl"
    Rotl,
    /// "rotr"
    Rotr,
    /// "not"
    Not,
    /// "inc"
    Inc,
    /// "dec"
    Dec,
    /// "neg"
    Neg,
    /// "unary_nop"
    UnaryNop,
    /// "drop_right"
    DropRight,
    /// "drop2"
    Drop2,
}

impl Op {
    pub fn all_values() -> &'static [Self] {
        &[
            Self::Unreachable,
            Self::Nop,
            Self::Block,
            Self::Loop,
            Self::End,
            Self::Br,
            Self::BrIf,
            Self::Return,
            Self::Call,
            Self::Drop,
            Self::I32Const,
            Self::I64Const,
            Self::F32Const,
            Self::F64Const,
            Self::LocalGet,
            Self::LocalSet,
            Self::LocalTee,
            Self::Eqz,
            Self::Eq,
            Self::Ne,
            Self::LtS,
            Self::LtU,
            Self::GtS,
            Self::GtU,
            Self::LeS,
            Self::LeU,
            Self::GeS,
            Self::GeU,
            Self::Clz,
            Self::Ctz,
            Self::Popcnt,
            Self::Add,
            Self::Sub,
            Self::Mul,
            Self::DivS,
            Self::DivU,
            Self::RemS,
            Self::RemU,
            Self::Shl,
            Self::ShrS,
            Self::ShrU,
            Self::And,
            Self::Or,
            Self::Xor,
            Self::Rotl,
            Self::Rotr,
            Self::Not,
            Self::Inc,
            Self::Dec,
            Self::Neg,
            Self::UnaryNop,
            Self::DropRight,
            Self::Drop2,
        ]
    }

    pub fn from_str(v: &str) -> Option<Self> {
        match v {
            "unreachable" => Some(Self::Unreachable),
            "nop" => Some(Self::Nop),
            "block" => Some(Self::Block),
            "loop" => Some(Self::Loop),
            "end" => Some(Self::End),
            "br" => Some(Self::Br),
            "br_if" => Some(Self::BrIf),
            "return" => Some(Self::Return),
            "call" => Some(Self::Call),
            "drop" => Some(Self::Drop),
            "i32.const" => Some(Self::I32Const),
            "i64.const" => Some(Self::I64Const),
            "f32.const" => Some(Self::F32Const),
            "f64.const" => Some(Self::F64Const),
            "local.get" => Some(Self::LocalGet),
            "local.set" => Some(Self::LocalSet),
            "local.tee" => Some(Self::LocalTee),
            "eqz" => Some(Self::Eqz),
            "eq" => Some(Self::Eq),
            "ne" => Some(Self::Ne),
            "lt_s" => Some(Self::LtS),
            "lt_u" => Some(Self::LtU),
            "gt_s" => Some(Self::GtS),
            "gt_u" => Some(Self::GtU),
            "le_s" => Some(Self::LeS),
            "le_u" => Some(Self::LeU),
            "ge_s" => Some(Self::GeS),
            "ge_u" => Some(Self::GeU),
            "clz" => Some(Self::Clz),
            "ctz" => Some(Self::Ctz),
            "popcnt" => Some(Self::Popcnt),
            "add" => Some(Self::Add),
            "sub" => Some(Self::Sub),
            "mul" => Some(Self::Mul),
            "div_s" => Some(Self::DivS),
            "div_u" => Some(Self::DivU),
            "rem_s" => Some(Self::RemS),
            "rem_u" => Some(Self::RemU),
            "shl" => Some(Self::Shl),
            "shr_s" => Some(Self::ShrS),
            "shr_u" => Some(Self::ShrU),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "xor" => Some(Self::Xor),
            "rotl" => Some(Self::Rotl),
            "rotr" => Some(Self::Rotr),
            "not" => Some(Self::Not),
            "inc" => Some(Self::Inc),
            "dec" => Some(Self::Dec),
            "neg" => Some(Self::Neg),
            "unary_nop" => Some(Self::UnaryNop),
            "drop_right" => Some(Self::DropRight),
            "drop2" => Some(Self::Drop2),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unreachable => "unreachable",
            Self::Nop => "nop",
            Self::Block => "block",
            Self::Loop => "loop",
            Self::End => "end",
            Self::Br => "br",
            Self::BrIf => "br_if",
            Self::Return => "return",
            Self::Call => "call",
            Self::Drop => "drop",
            Self::I32Const => "i32.const",
            Self::I64Const => "i64.const",
            Self::F32Const => "f32.const",
            Self::F64Const => "f64.const",
            Self::LocalGet => "local.get",
            Self::LocalSet => "local.set",
            Self::LocalTee => "local.tee",
            Self::Eqz => "eqz",
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::LtS => "lt_s",
            Self::LtU => "lt_u",
            Self::GtS => "gt_s",
            Self::GtU => "gt_u",
            Self::LeS => "le_s",
            Self::LeU => "le_u",
            Self::GeS => "ge_s",
            Self::GeU => "ge_u",
            Self::Clz => "clz",
            Self::Ctz => "ctz",
            Self::Popcnt => "popcnt",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::DivS => "div_s",
            Self::DivU => "div_u",
            Self::RemS => "rem_s",
            Self::RemU => "rem_u",
            Self::Shl => "shl",
            Self::ShrS => "shr_s",
            Self::ShrU => "shr_u",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Rotl => "rotl",
            Self::Rotr => "rotr",
            Self::Not => "not",
            Self::Inc => "inc",
            Self::Dec => "dec",
            Self::Neg => "neg",
            Self::UnaryNop => "unary_nop",
            Self::DropRight => "drop_right",
            Self::Drop2 => "drop2",
        }
    }

    pub fn to_identifier(&self) -> &str {
        match self {
            Self::Unreachable => "Unreachable",
            Self::Nop => "Nop",
            Self::Block => "Block",
            Self::Loop => "Loop",
            Self::End => "End",
            Self::Br => "Br",
            Self::BrIf => "BrIf",
            Self::Return => "Return",
            Self::Call => "Call",
            Self::Drop => "Drop",
            Self::I32Const => "I32Const",
            Self::I64Const => "I64Const",
            Self::F32Const => "F32Const",
            Self::F64Const => "F64Const",
            Self::LocalGet => "LocalGet",
            Self::LocalSet => "LocalSet",
            Self::LocalTee => "LocalTee",
            Self::Eqz => "Eqz",
            Self::Eq => "Eq",
            Self::Ne => "Ne",
            Self::LtS => "LtS",
            Self::LtU => "LtU",
            Self::GtS => "GtS",
            Self::GtU => "GtU",
            Self::LeS => "LeS",
            Self::LeU => "LeU",
            Self::GeS => "GeS",
            Self::GeU => "GeU",
            Self::Clz => "Clz",
            Self::Ctz => "Ctz",
            Self::Popcnt => "Popcnt",
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::DivS => "DivS",
            Self::DivU => "DivU",
            Self::RemS => "RemS",
            Self::RemU => "RemU",
            Self::Shl => "Shl",
            Self::ShrS => "ShrS",
            Self::ShrU => "ShrU",
            Self::And => "And",
            Self::Or => "Or",
            Self::Xor => "Xor",
            Self::Rotl => "Rotl",
            Self::Rotr => "Rotr",
            Self::Not => "Not",
            Self::Inc => "Inc",
            Self::Dec => "Dec",
            Self::Neg => "Neg",
            Self::UnaryNop => "UnaryNop",
            Self::DropRight => "DropRight",
            Self::Drop2 => "Drop2",
        }
    }

    pub fn class(&self) -> OpClass {
        match self {
            Self::Unreachable => OpClass::Control,
            Self::Nop => OpClass::Control,
            Self::Block => OpClass::Block,
            Self::Loop => OpClass::Block,
            Self::End => OpClass::Block,
            Self::Br => OpClass::Control,
            Self::BrIf => OpClass::Control,
            Self::Return => OpClass::Control,
            Self::Call => OpClass::Control,
            Self::Drop => OpClass::Control,
            Self::I32Const => OpClass::Const,
            Self::I64Const => OpClass::Const,
            Self::F32Const => OpClass::Const,
            Self::F64Const => OpClass::Const,
            Self::LocalGet => OpClass::Local,
            Self::LocalSet => OpClass::Local,
            Self::LocalTee => OpClass::Local,
            Self::Eqz => OpClass::UnOp,
            Self::Eq => OpClass::Cmp,
            Self::Ne => OpClass::Cmp,
            Self::LtS => OpClass::Cmp,
            Self::LtU => OpClass::Cmp,
            Self::GtS => OpClass::Cmp,
            Self::GtU => OpClass::Cmp,
            Self::LeS => OpClass::Cmp,
            Self::LeU => OpClass::Cmp,
            Self::GeS => OpClass::Cmp,
            Self::GeU => OpClass::Cmp,
            Self::Clz => OpClass::UnOp,
            Self::Ctz => OpClass::UnOp,
            Self::Popcnt => OpClass::UnOp,
            Self::Add => OpClass::BinOp,
            Self::Sub => OpClass::BinOp,
            Self::Mul => OpClass::BinOp,
            Self::DivS => OpClass::BinOp,
            Self::DivU => OpClass::BinOp,
            Self::RemS => OpClass::BinOp,
            Self::RemU => OpClass::BinOp,
            Self::Shl => OpClass::BinOp,
            Self::ShrS => OpClass::BinOp,
            Self::ShrU => OpClass::BinOp,
            Self::And => OpClass::BinOp,
            Self::Or => OpClass::BinOp,
            Self::Xor => OpClass::BinOp,
            Self::Rotl => OpClass::BinOp,
            Self::Rotr => OpClass::BinOp,
            Self::Not => OpClass::UnOp,
            Self::Inc => OpClass::UnOp,
            Self::Dec => OpClass::UnOp,
            Self::Neg => OpClass::UnOp,
            Self::UnaryNop => OpClass::Control,
            Self::DropRight => OpClass::Control,
            Self::Drop2 => OpClass::Control,
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

#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpClass {
    Control,
    Block,
    Const,
    Local,
    UnOp,
    Cmp,
    BinOp,
}
