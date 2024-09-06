//! ToyIR Assembler

use super::*;
use crate::types::index::LocalIndex;
use core::{
    mem::transmute,
    sync::atomic::{AtomicU32, Ordering},
};
use opt::MinimalCodeOptimizer;

pub struct Assembler {
    buf: Vec<u32>,
    ssa_index: AtomicU32,
    value_stack: Vec<SsaIndex>,
    block_stack: Vec<(BlockIndex, usize)>,
}

#[derive(Debug)]
pub enum AssembleError {
    InvalidParameter,
    OutOfBlockStack,
    OutOfValueStack,
    InvalidBlockStack,
    InvalidValueStack,
    InvalidBranchTarget,
}

pub struct CodeStream<'a> {
    codes: &'a mut Assembler,
    base_stack_level: usize,
}

impl Assembler {
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: Default::default(),
            ssa_index: AtomicU32::new(0),
            value_stack: Default::default(),
            block_stack: Default::default(),
        }
    }

    #[inline]
    pub fn stream<'a>(&'a mut self) -> CodeStream<'a> {
        CodeStream {
            codes: self,
            base_stack_level: 0,
        }
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = CodeFragment> + 'a {
        CodeStreamIter {
            buf: self.buf.as_slice(),
            index: 0,
        }
    }

    pub fn finalize(self) -> Result<Self, AssembleError> {
        if self.block_stack.len() != 0 {
            return Err(AssembleError::InvalidBlockStack);
        }
        if self.value_stack.len() != 0 {
            return Err(AssembleError::InvalidValueStack);
        }

        let Self {
            buf,
            ssa_index,
            value_stack: _,
            block_stack: _,
        } = self;

        let buf = MinimalCodeOptimizer::optimize(buf).unwrap();

        Ok(Self {
            buf,
            ssa_index,
            value_stack: Vec::new(),
            block_stack: Vec::new(),
        })
    }
}

impl CodeStream<'_> {
    #[inline]
    pub fn current_index(&self) -> SsaIndex {
        SsaIndex(self.codes.ssa_index.load(Ordering::Relaxed))
    }

    fn emit(&mut self, op: Op, operands: &[Operand]) {
        let mut buf = Vec::new();
        for operand in operands {
            match operand {
                Operand::SsaIndex(v) => buf.push(v.0),
                Operand::I32(v) => buf.push(*v as u32),
                Operand::U32(v) => buf.push(*v as u32),
                Operand::I64(v) => {
                    let u = *v as u64;
                    buf.push(u as u32);
                    buf.push((u >> 32) as u32);
                }
                Operand::U64(v) => {
                    let u = *v as u64;
                    buf.push(u as u32);
                    buf.push((u >> 32) as u32);
                }
            }
        }
        self.codes
            .buf
            .push((buf.len() as u32 + 1) << 16 | op as u32);
        self.codes.buf.extend(&buf);

        self.codes
            .ssa_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    }

    #[inline]
    fn pop(&mut self) -> Result<SsaIndex, AssembleError> {
        if self.codes.value_stack.len() > self.base_stack_level {
            self.codes
                .value_stack
                .pop()
                .ok_or(AssembleError::OutOfValueStack)
        } else {
            Err(AssembleError::OutOfValueStack)
        }
    }

    #[inline]
    fn push(&mut self, value: SsaIndex) {
        self.codes.value_stack.push(value);
    }

    #[inline]
    pub fn emit_binop(&mut self, op: Op) -> Result<(), AssembleError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let result = self.current_index();
        self.push(result);
        self.emit(op, &[result.into(), lhs.into(), rhs.into()]);
        Ok(())
    }

    #[inline]
    pub fn emit_unop(&mut self, op: Op) -> Result<(), AssembleError> {
        let operand = self.pop()?;
        let result = self.current_index();
        self.push(result);
        self.emit(op, &[result.into(), operand.into()]);
        Ok(())
    }

    #[inline]
    pub fn ir_block(&mut self) -> BlockIndex {
        self.begin_block(Op::Block)
    }

    #[inline]
    pub fn ir_loop(&mut self) -> BlockIndex {
        self.begin_block(Op::Loop)
    }

    fn begin_block(&mut self, op: Op) -> BlockIndex {
        let block = self.current_index();
        self.codes
            .block_stack
            .push((BlockIndex(block.0), self.base_stack_level));
        self.base_stack_level = self.codes.value_stack.len();
        self.emit(op, &[Operand::SsaIndex(block)]);
        BlockIndex(block.0)
    }

    pub fn ir_end(&mut self, index: BlockIndex) -> Result<(), AssembleError> {
        let (test_index, stack_level) = self
            .codes
            .block_stack
            .pop()
            .ok_or(AssembleError::OutOfBlockStack)?;
        if test_index != index {
            return Err(AssembleError::InvalidBlockStack);
        }
        self.base_stack_level = stack_level;
        self.emit(Op::End, &[index.0.into()]);
        Ok(())
    }

    #[inline]
    pub fn ir_bool_const(&mut self, value: bool) {
        self.ir_i32_const(value as i32)
    }

    #[inline]
    pub fn ir_i32_const(&mut self, value: i32) {
        let result = self.current_index();
        self.push(result);
        self.emit(Op::I32Const, &[result.into(), value.into()]);
    }

    #[inline]
    pub fn ir_i64_const(&mut self, value: i64) {
        let result = self.current_index();
        self.push(result);
        self.emit(Op::I64Const, &[result.into(), value.into()]);
    }

    #[inline]
    pub fn ir_br(&mut self, target: BlockIndex) -> Result<(), AssembleError> {
        if self
            .codes
            .block_stack
            .iter()
            .find(|v| v.0 == target)
            .is_none()
        {
            return Err(AssembleError::InvalidBranchTarget);
        }
        self.emit(Op::Br, &[Operand::U32(target.0)]);
        Ok(())
    }

    #[inline]
    pub fn ir_br_if(&mut self, target: BlockIndex) -> Result<(), AssembleError> {
        if self
            .codes
            .block_stack
            .iter()
            .find(|v| v.0 == target)
            .is_none()
        {
            return Err(AssembleError::InvalidBranchTarget);
        }
        let cc = self.pop()?;
        self.emit(Op::BrIf, &[Operand::U32(target.0), cc.into()]);
        Ok(())
    }

    #[inline]
    pub fn ir_local_get(&mut self, localidx: LocalIndex) {
        let result = self.current_index();
        self.push(result);
        self.emit(Op::LocalGet, &[result.into(), Operand::U32(localidx.get())]);
    }

    #[inline]
    pub fn ir_local_set(&mut self, localidx: LocalIndex) -> Result<(), AssembleError> {
        let result = self.pop()?;
        self.emit(Op::LocalSet, &[result.into(), Operand::U32(localidx.get())]);
        Ok(())
    }

    #[inline]
    pub fn ir_local_tee(&mut self, localidx: LocalIndex) -> Result<(), AssembleError> {
        let result = self.current_index();
        let ssa_index = self.pop()?;
        self.push(result);
        self.emit(
            Op::LocalTee,
            &[
                result.into(),
                Operand::U32(localidx.get()),
                ssa_index.into(),
            ],
        );
        Ok(())
    }

    pub fn ir_drop(&mut self) -> Result<(), AssembleError> {
        let result = self.pop()?;
        self.emit(Op::Drop, &[result.into()]);
        Ok(())
    }

    pub fn ir_call(
        &mut self,
        target: usize,
        params_len: usize,
        result_len: usize,
    ) -> Result<(), AssembleError> {
        let mut params = Vec::with_capacity(params_len);
        for _ in 0..params_len {
            params.push(Operand::from(self.pop()?));
        }
        let result = self.current_index();
        params.push(Operand::U32(target as u32));
        params.push(result.into());
        params.reverse();
        if result_len > 0 {
            self.push(result);
        }
        self.emit(Op::Call, &params);
        Ok(())
    }

    pub fn ir_return(&mut self, result_len: usize) -> Result<(), AssembleError> {
        if result_len > 0 {
            let result = self.pop()?;
            self.emit(Op::Return, &[result.into()]);
        } else {
            self.emit(Op::Return, &[]);
        }
        Ok(())
    }

    pub fn ir_invert(&mut self) -> Result<(), AssembleError> {
        let operand = self.pop()?;
        let result = self.current_index();
        self.push(result);
        self.emit(Op::Eqz, &[result.into(), operand.into(), 0.into()]);
        Ok(())
    }
}

impl core::fmt::Debug for Assembler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

struct CodeStreamIter<'a> {
    buf: &'a [u32],
    index: usize,
}

impl<'a> Iterator for CodeStreamIter<'a> {
    type Item = CodeFragment;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let len_opc = self.buf.get(self.index)?;
            self.index += 1;
            let len = (len_opc >> 16) as usize;
            if len > 0 {
                let opcode = unsafe { transmute::<u8, Op>((len_opc & 0xFFFF) as u8) };
                // if opcode == Op::Nop {
                //     self.index += len - 1;
                //     continue;
                // }
                let mut params = Vec::with_capacity(len);
                for _ in 1..len {
                    params.push(*self.buf.get(self.index)?);
                    self.index += 1;
                }
                return Some(CodeFragment { opcode, params });
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    SsaIndex(SsaIndex),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
}

impl From<SsaIndex> for Operand {
    #[inline]
    fn from(value: SsaIndex) -> Self {
        Self::SsaIndex(value)
    }
}

impl From<bool> for Operand {
    #[inline]
    fn from(value: bool) -> Self {
        Self::I32(value as i32)
    }
}

impl From<i32> for Operand {
    #[inline]
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<u32> for Operand {
    #[inline]
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<i64> for Operand {
    #[inline]
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for Operand {
    #[inline]
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

pub struct CodeFragment {
    opcode: Op,
    params: Vec<u32>,
}

impl core::fmt::Debug for CodeFragment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "/* {:04x} {:04x} */ {} {:?}",
            self.params.len() + 1,
            self.opcode as usize,
            self.opcode,
            self.params
        )
    }
}

macro_rules! ir_no_params {
    {$( $func_name:ident: $op:ident; )*} => {
        impl CodeStream<'_> {
            $(
                #[inline]
                pub fn $func_name(&mut self) {
                    self.emit(Op::$op, &[])
                }
            )*
        }
    };
}

ir_no_params! {
    ir_unreachable: Unreachable;
    ir_nop: Nop;
}

macro_rules! ir_unops {
    {$( $func_name:ident: $op:ident; )*} => {
        impl CodeStream<'_> {
            $(
                #[inline]
                pub fn $func_name(&mut self) -> Result<(), AssembleError> {
                    self.emit_unop(Op::$op)
                }
            )*
        }
    };
}

ir_unops! {
    ir_neg: Neg;
    ir_not: Not;
    ir_inc: Inc;
    ir_dec: Dec;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SsaIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockIndex(u32);

impl BlockIndex {
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}
