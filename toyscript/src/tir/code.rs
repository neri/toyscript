use super::*;
use crate::types::index::LocalIndex;
use core::{mem::transmute, sync::atomic::AtomicU32};
use leb128::{Leb128Reader, Leb128Writer, ReadLeb128, WriteLeb128};

pub struct CodeBuilder {
    writer: Leb128Writer,
    ssa_index: AtomicU32,
    value_stack: Vec<SsaIndex>,
    block_stack: Vec<(BlockIndex, usize)>,
}

#[derive(Debug)]
pub enum CodeBuildError {
    InvalidParameter,
    OutOfBlockStack,
    OutOfValueStack,
    InvalidBlockStack,
    InvalidValueStack,
}

pub struct CodeStream<'a> {
    codes: &'a mut CodeBuilder,
    base_stack_level: usize,
}

impl CodeBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            writer: Leb128Writer::new(),
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
        CodeStreamIter(Leb128Reader::from_slice(self.writer.as_slice()))
    }

    pub fn finalize(self) -> Result<Self, CodeBuildError> {
        if self.block_stack.len() != 0 {
            return Err(CodeBuildError::InvalidBlockStack);
        }
        if self.value_stack.len() != 0 {
            return Err(CodeBuildError::InvalidValueStack);
        }
        Ok(self)
    }
}

impl CodeStream<'_> {
    #[inline]
    pub fn next_index(&self) -> SsaIndex {
        SsaIndex(
            self.codes
                .ssa_index
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst),
        )
    }

    fn emit(&mut self, op: Op, operands: &[Operand]) {
        self.codes.writer.write(operands.len() + 1).unwrap();
        self.codes.writer.write(op as usize).unwrap();
        for operand in operands {
            match operand {
                Operand::SsaIndex(v) => self.codes.writer.write(v.0).unwrap(),
                Operand::Byte(v) => self.codes.writer.write_byte(*v).unwrap(),
                Operand::I32(v) => self.codes.writer.write(*v).unwrap(),
                Operand::U32(v) => self.codes.writer.write(*v).unwrap(),
                Operand::I64(v) => self.codes.writer.write(*v).unwrap(),
                Operand::U64(v) => self.codes.writer.write(*v).unwrap(),
            }
        }
    }

    #[inline]
    fn pop(&mut self) -> Result<SsaIndex, CodeBuildError> {
        if self.codes.value_stack.len() > self.base_stack_level {
            self.codes
                .value_stack
                .pop()
                .ok_or(CodeBuildError::OutOfValueStack)
        } else {
            Err(CodeBuildError::OutOfValueStack)
        }
    }

    #[inline]
    fn push(&mut self, value: SsaIndex) {
        self.codes.value_stack.push(value);
    }

    #[inline]
    pub fn emit_binop(&mut self, op: Op) -> Result<(), CodeBuildError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let result = self.next_index();
        self.push(result);
        self.emit(op, &[result.into(), lhs.into(), rhs.into()]);
        Ok(())
    }

    #[inline]
    pub fn emit_unop(&mut self, op: Op) -> Result<(), CodeBuildError> {
        let operand = self.pop()?;
        let result = self.next_index();
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
        let block = self.next_index();
        self.codes
            .block_stack
            .push((BlockIndex(block.0), self.base_stack_level));
        self.base_stack_level = self.codes.value_stack.len();
        self.emit(op, &[Operand::SsaIndex(block)]);
        BlockIndex(block.0)
    }

    pub fn ir_end(&mut self, block_index: BlockIndex) -> Result<(), CodeBuildError> {
        let (test_block_index, stack_level) = self
            .codes
            .block_stack
            .pop()
            .ok_or(CodeBuildError::OutOfBlockStack)?;
        if test_block_index != block_index {
            return Err(CodeBuildError::InvalidBlockStack);
        }
        self.base_stack_level = stack_level;
        self.emit(Op::End, &[block_index.0.into()]);
        Ok(())
    }

    #[inline]
    pub fn ir_i32_const(&mut self, value: i32) {
        let result = self.next_index();
        self.push(result);
        self.emit(Op::I32Const, &[result.into(), value.into()]);
    }

    #[inline]
    pub fn ir_i64_const(&mut self, value: i64) {
        let result = self.next_index();
        self.push(result);
        self.emit(Op::I64Const, &[result.into(), value.into()]);
    }

    #[inline]
    pub fn ir_br(&mut self, target: BlockIndex) {
        self.emit(Op::Br, &[Operand::U32(target.0)]);
    }

    #[inline]
    pub fn ir_br_if(&mut self, target: BlockIndex) -> Result<(), CodeBuildError> {
        let cc = self.pop()?;
        self.emit(Op::BrIf, &[Operand::U32(target.0), cc.into()]);
        Ok(())
    }

    #[inline]
    pub fn ir_local_get(&mut self, localidx: LocalIndex) {
        let result = self.next_index();
        self.push(result);
        self.emit(Op::LocalGet, &[result.into(), Operand::U32(localidx.get())]);
    }

    #[inline]
    pub fn ir_local_set(&mut self, localidx: LocalIndex) -> Result<(), CodeBuildError> {
        let result = self.pop()?;
        self.emit(Op::LocalSet, &[result.into(), Operand::U32(localidx.get())]);
        Ok(())
    }

    #[inline]
    pub fn ir_local_tee(&mut self, localidx: LocalIndex) -> Result<(), CodeBuildError> {
        let operand = self.pop()?;
        let result = self.next_index();
        self.push(result);
        self.emit(
            Op::LocalGet,
            &[result.into(), Operand::U32(localidx.get()), operand.into()],
        );
        Ok(())
    }

    pub fn ir_drop(&mut self) -> Result<(), CodeBuildError> {
        let result = self.pop()?;
        self.emit(Op::Drop, &[result.into()]);
        Ok(())
    }

    pub fn ir_call(
        &mut self,
        target: usize,
        params_len: usize,
        result_len: usize,
    ) -> Result<(), CodeBuildError> {
        let mut params = Vec::with_capacity(params_len);
        for _ in 0..params_len {
            params.push(Operand::from(self.pop()?));
        }
        let result = self.next_index();
        params.push(Operand::U32(target as u32));
        params.push(result.into());
        params.reverse();
        if result_len > 0 {
            self.push(result);
        }
        self.emit(Op::Call, &params);
        Ok(())
    }

    pub fn ir_return(&mut self, result_len: usize) -> Result<(), CodeBuildError> {
        if result_len > 0 {
            let result = self.pop()?;
            self.emit(Op::Return, &[result.into()]);
        } else {
            self.emit(Op::Return, &[]);
        }
        Ok(())
    }
}

impl core::fmt::Debug for CodeBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

struct CodeStreamIter<'a>(Leb128Reader<'a>);

impl<'a> Iterator for CodeStreamIter<'a> {
    type Item = CodeFragment;

    fn next(&mut self) -> Option<Self::Item> {
        let count: usize = self.0.read().ok()?;
        if count > 0 {
            let opcode: usize = self.0.read().ok()?;
            let opcode = unsafe { transmute::<u8, Op>(opcode as u8) };
            let mut params = Vec::with_capacity(count);
            for _ in 1..count {
                params.push(self.0.read().ok()?);
            }
            Some(CodeFragment { opcode, params })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    SsaIndex(SsaIndex),
    Byte(u8),
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
    params: Vec<usize>,
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
                pub fn $func_name(&mut self) -> Result<(), CodeBuildError> {
                    self.emit_unop(Op::$op)
                }
            )*
        }
    };
}

ir_unops! {
    ir_neg: Neg;
    ir_not: Eqz;
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
