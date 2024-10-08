//! ToyScript Intermediate Representation Opcodes

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// ToyScript Intermediate Representation Opcodes
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
    /// "call_v"
    CallV,
    /// "cast"
    Cast,
    /// "dec"
    Dec,
    /// "div_s"
    DivS,
    /// "div_u"
    DivU,
    /// "drop"
    Drop,
    /// "drop2"
    Drop2,
    /// "drop_right"
    DropRight,
    /// "end"
    End,
    /// "eq"
    Eq,
    /// "eqz"
    Eqz,
    /// "f32.const"
    F32Const,
    /// "f64.const"
    F64Const,
    /// "ge_s"
    GeS,
    /// "ge_u"
    GeU,
    /// "gt_s"
    GtS,
    /// "gt_u"
    GtU,
    /// "i32.const"
    I32Const,
    /// "i64.const"
    I64Const,
    /// "inc"
    Inc,
    /// "le_s"
    LeS,
    /// "le_u"
    LeU,
    /// "local.get"
    LocalGet,
    /// "local.set"
    LocalSet,
    /// "local.tee"
    LocalTee,
    /// "loop"
    Loop,
    /// "lt_s"
    LtS,
    /// "lt_u"
    LtU,
    /// "mul"
    Mul,
    /// "ne"
    Ne,
    /// "neg"
    Neg,
    /// "nop"
    Nop,
    /// "not"
    Not,
    /// "or"
    Or,
    /// "rem_s"
    RemS,
    /// "rem_u"
    RemU,
    /// "return"
    Return,
    /// "shl"
    Shl,
    /// "shr_s"
    ShrS,
    /// "shr_u"
    ShrU,
    /// "sub"
    Sub,
    /// "unary_nop"
    UnaryNop,
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
            Self::CallV,
            Self::Cast,
            Self::Dec,
            Self::DivS,
            Self::DivU,
            Self::Drop,
            Self::Drop2,
            Self::DropRight,
            Self::End,
            Self::Eq,
            Self::Eqz,
            Self::F32Const,
            Self::F64Const,
            Self::GeS,
            Self::GeU,
            Self::GtS,
            Self::GtU,
            Self::I32Const,
            Self::I64Const,
            Self::Inc,
            Self::LeS,
            Self::LeU,
            Self::LocalGet,
            Self::LocalSet,
            Self::LocalTee,
            Self::Loop,
            Self::LtS,
            Self::LtU,
            Self::Mul,
            Self::Ne,
            Self::Neg,
            Self::Nop,
            Self::Not,
            Self::Or,
            Self::RemS,
            Self::RemU,
            Self::Return,
            Self::Shl,
            Self::ShrS,
            Self::ShrU,
            Self::Sub,
            Self::UnaryNop,
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
            "call_v" => Some(Self::CallV),
            "cast" => Some(Self::Cast),
            "dec" => Some(Self::Dec),
            "div_s" => Some(Self::DivS),
            "div_u" => Some(Self::DivU),
            "drop" => Some(Self::Drop),
            "drop2" => Some(Self::Drop2),
            "drop_right" => Some(Self::DropRight),
            "end" => Some(Self::End),
            "eq" => Some(Self::Eq),
            "eqz" => Some(Self::Eqz),
            "f32.const" => Some(Self::F32Const),
            "f64.const" => Some(Self::F64Const),
            "ge_s" => Some(Self::GeS),
            "ge_u" => Some(Self::GeU),
            "gt_s" => Some(Self::GtS),
            "gt_u" => Some(Self::GtU),
            "i32.const" => Some(Self::I32Const),
            "i64.const" => Some(Self::I64Const),
            "inc" => Some(Self::Inc),
            "le_s" => Some(Self::LeS),
            "le_u" => Some(Self::LeU),
            "local.get" => Some(Self::LocalGet),
            "local.set" => Some(Self::LocalSet),
            "local.tee" => Some(Self::LocalTee),
            "loop" => Some(Self::Loop),
            "lt_s" => Some(Self::LtS),
            "lt_u" => Some(Self::LtU),
            "mul" => Some(Self::Mul),
            "ne" => Some(Self::Ne),
            "neg" => Some(Self::Neg),
            "nop" => Some(Self::Nop),
            "not" => Some(Self::Not),
            "or" => Some(Self::Or),
            "rem_s" => Some(Self::RemS),
            "rem_u" => Some(Self::RemU),
            "return" => Some(Self::Return),
            "shl" => Some(Self::Shl),
            "shr_s" => Some(Self::ShrS),
            "shr_u" => Some(Self::ShrU),
            "sub" => Some(Self::Sub),
            "unary_nop" => Some(Self::UnaryNop),
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
            Self::CallV => "call_v",
            Self::Cast => "cast",
            Self::Dec => "dec",
            Self::DivS => "div_s",
            Self::DivU => "div_u",
            Self::Drop => "drop",
            Self::Drop2 => "drop2",
            Self::DropRight => "drop_right",
            Self::End => "end",
            Self::Eq => "eq",
            Self::Eqz => "eqz",
            Self::F32Const => "f32.const",
            Self::F64Const => "f64.const",
            Self::GeS => "ge_s",
            Self::GeU => "ge_u",
            Self::GtS => "gt_s",
            Self::GtU => "gt_u",
            Self::I32Const => "i32.const",
            Self::I64Const => "i64.const",
            Self::Inc => "inc",
            Self::LeS => "le_s",
            Self::LeU => "le_u",
            Self::LocalGet => "local.get",
            Self::LocalSet => "local.set",
            Self::LocalTee => "local.tee",
            Self::Loop => "loop",
            Self::LtS => "lt_s",
            Self::LtU => "lt_u",
            Self::Mul => "mul",
            Self::Ne => "ne",
            Self::Neg => "neg",
            Self::Nop => "nop",
            Self::Not => "not",
            Self::Or => "or",
            Self::RemS => "rem_s",
            Self::RemU => "rem_u",
            Self::Return => "return",
            Self::Shl => "shl",
            Self::ShrS => "shr_s",
            Self::ShrU => "shr_u",
            Self::Sub => "sub",
            Self::UnaryNop => "unary_nop",
            Self::Unreachable => "unreachable",
            Self::Xor => "xor",
        }
    }

    pub fn to_identifier(&self) -> &str {
        match self {
            Self::Add => "Add",
            Self::And => "And",
            Self::Block => "Block",
            Self::Br => "Br",
            Self::BrIf => "BrIf",
            Self::Call => "Call",
            Self::CallV => "CallV",
            Self::Cast => "Cast",
            Self::Dec => "Dec",
            Self::DivS => "DivS",
            Self::DivU => "DivU",
            Self::Drop => "Drop",
            Self::Drop2 => "Drop2",
            Self::DropRight => "DropRight",
            Self::End => "End",
            Self::Eq => "Eq",
            Self::Eqz => "Eqz",
            Self::F32Const => "F32Const",
            Self::F64Const => "F64Const",
            Self::GeS => "GeS",
            Self::GeU => "GeU",
            Self::GtS => "GtS",
            Self::GtU => "GtU",
            Self::I32Const => "I32Const",
            Self::I64Const => "I64Const",
            Self::Inc => "Inc",
            Self::LeS => "LeS",
            Self::LeU => "LeU",
            Self::LocalGet => "LocalGet",
            Self::LocalSet => "LocalSet",
            Self::LocalTee => "LocalTee",
            Self::Loop => "Loop",
            Self::LtS => "LtS",
            Self::LtU => "LtU",
            Self::Mul => "Mul",
            Self::Ne => "Ne",
            Self::Neg => "Neg",
            Self::Nop => "Nop",
            Self::Not => "Not",
            Self::Or => "Or",
            Self::RemS => "RemS",
            Self::RemU => "RemU",
            Self::Return => "Return",
            Self::Shl => "Shl",
            Self::ShrS => "ShrS",
            Self::ShrU => "ShrU",
            Self::Sub => "Sub",
            Self::UnaryNop => "UnaryNop",
            Self::Unreachable => "Unreachable",
            Self::Xor => "Xor",
        }
    }

    pub fn class(&self) -> OpClass {
        match self {
            Self::Add => OpClass::BinOp,
            Self::And => OpClass::BinOp,
            Self::Block => OpClass::Block,
            Self::Br => OpClass::Control,
            Self::BrIf => OpClass::Control,
            Self::Call => OpClass::Control,
            Self::CallV => OpClass::Control,
            Self::Cast => OpClass::Control,
            Self::Dec => OpClass::UnOp,
            Self::DivS => OpClass::BinOp,
            Self::DivU => OpClass::BinOp,
            Self::Drop => OpClass::Control,
            Self::Drop2 => OpClass::Control,
            Self::DropRight => OpClass::Control,
            Self::End => OpClass::Block,
            Self::Eq => OpClass::Cmp,
            Self::Eqz => OpClass::UnOp,
            Self::F32Const => OpClass::Const,
            Self::F64Const => OpClass::Const,
            Self::GeS => OpClass::Cmp,
            Self::GeU => OpClass::Cmp,
            Self::GtS => OpClass::Cmp,
            Self::GtU => OpClass::Cmp,
            Self::I32Const => OpClass::Const,
            Self::I64Const => OpClass::Const,
            Self::Inc => OpClass::UnOp,
            Self::LeS => OpClass::Cmp,
            Self::LeU => OpClass::Cmp,
            Self::LocalGet => OpClass::Local,
            Self::LocalSet => OpClass::Local,
            Self::LocalTee => OpClass::Local,
            Self::Loop => OpClass::Block,
            Self::LtS => OpClass::Cmp,
            Self::LtU => OpClass::Cmp,
            Self::Mul => OpClass::BinOp,
            Self::Ne => OpClass::Cmp,
            Self::Neg => OpClass::UnOp,
            Self::Nop => OpClass::NoParam,
            Self::Not => OpClass::UnOp,
            Self::Or => OpClass::BinOp,
            Self::RemS => OpClass::BinOp,
            Self::RemU => OpClass::BinOp,
            Self::Return => OpClass::Control,
            Self::Shl => OpClass::BinOp,
            Self::ShrS => OpClass::BinOp,
            Self::ShrU => OpClass::BinOp,
            Self::Sub => OpClass::BinOp,
            Self::UnaryNop => OpClass::Control,
            Self::Unreachable => OpClass::NoParam,
            Self::Xor => OpClass::BinOp,
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpClass {
    NoParam,
    Block,
    Control,
    Const,
    Local,
    UnOp,
    Cmp,
    BinOp,
}
