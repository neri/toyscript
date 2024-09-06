//! Minimal Code Optimizer

use super::*;
use core::{
    mem::transmute,
    ops::{BitAnd, BitOr, BitXor},
};

pub struct MinimalCodeOptimizer {
    positions: Vec<ArrayIndex>,
    codes: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum OptimizeError {
    /// An Internal Error
    OutOfPosition(usize),
    /// An Internal Error
    OutOfCodes(usize),

    InvalidBranch(usize, u32),

    InvalidParameter(usize, usize),

    OverwriteError(usize, usize, usize),

    InvalidDropChain(usize),

    RenameError(usize, usize, usize),
}

impl MinimalCodeOptimizer {
    pub fn optimize(codes: Vec<u32>) -> Result<Vec<u32>, OptimizeError> {
        let mut positions = Vec::with_capacity(codes.len() / 4);
        let mut index = 0;
        while let Some(len_opc) = codes.get(index) {
            let len_opc = *len_opc;
            let len = (len_opc >> 16) as usize;
            if len == 0 {
                break;
            }
            positions.push(ArrayIndex(index as u32));
            index += len;
        }

        let mut optimizer = Self { codes, positions };
        optimizer._optimize()?;

        Ok(optimizer.codes)
    }

    fn _optimize(&mut self) -> Result<(), OptimizeError> {
        {
            // Reduce unnecessary operations
            let mut ci = 0;
            let mut skip_until_end = false;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (len, opcode) = self.get_op(base)?;

                if skip_until_end {
                    if opcode != Op::End {
                        self.replace_nop(base)?;
                    } else {
                        skip_until_end = false;
                    }
                    continue;
                }

                match opcode {
                    Op::Br | Op::Return | Op::Unreachable => {
                        skip_until_end = true;
                    }

                    Op::BrIf => {
                        let block_index = self.param(base, len, 1)?;
                        let operand = self.array_index(CodeIndex(self.param(base, len, 2)?))?;
                        if let Some(const_val) = self.get_i32_const(operand)? {
                            if const_val == 0 {
                                self.replace_nop(operand)?;
                                self.replace_nop(base)?;
                            } else {
                                self.replace_nop(operand)?;
                                self.replace(base, Op::Br, &[block_index])?;
                                skip_until_end = true;
                            }
                        }
                    }

                    Op::Drop => {
                        let operand = CodeIndex(self.param(base, len, 1)?);
                        if self.chain_drop(operand)? {
                            self.replace_nop(base)?;
                        }
                    }

                    // binop
                    Op::Add
                    | Op::And
                    | Op::Div
                    | Op::Eq
                    | Op::Mul
                    | Op::Ne
                    | Op::Or
                    | Op::Rem
                    | Op::Rotl
                    | Op::Rotr
                    | Op::Shl
                    | Op::Shr
                    | Op::Sub
                    | Op::Xor => {
                        let block_index = CodeIndex(self.param(base, len, 1)?);
                        let operand1 = self.array_index(CodeIndex(self.param(base, len, 2)?))?;
                        let operand2 = self.array_index(CodeIndex(self.param(base, len, 3)?))?;
                        match (self.get_i32_const(operand1)?, self.get_i32_const(operand2)?) {
                            (Some(lhs), Some(rhs)) => {
                                self.replace_nop(operand1)?;
                                self.replace_nop(operand2)?;

                                let result = match opcode {
                                    Op::Eq => (lhs == rhs) as u32,
                                    Op::Ne => (lhs != rhs) as u32,

                                    Op::Add => lhs.wrapping_add(rhs),
                                    Op::Sub => lhs.wrapping_sub(rhs),
                                    Op::Mul => lhs.wrapping_mul(rhs),
                                    Op::Div => lhs.checked_div(rhs).unwrap_or(0),
                                    Op::Rem => lhs.checked_rem(rhs).unwrap_or(0),
                                    Op::And => lhs.bitand(rhs),
                                    Op::Or => lhs.bitor(rhs),
                                    Op::Xor => lhs.bitxor(rhs),
                                    Op::Shl => lhs.wrapping_shl(rhs),
                                    Op::Shr => lhs.wrapping_shr(rhs),
                                    Op::Rotl => lhs.rotate_left(rhs),
                                    Op::Rotr => lhs.rotate_right(rhs),

                                    _ => unreachable!(),
                                };

                                self.replace(base, Op::I32Const, &[block_index.0, result])?;
                            }
                            _ => {}
                        }
                    }

                    // unop
                    Op::Eqz => {
                        let result = CodeIndex(self.param(base, len, 1)?);
                        let target = self.array_index(CodeIndex(self.param(base, len, 2)?))?;
                        let (len2, op2) = self.get_op(target)?;
                        match op2 {
                            Op::Eqz => {
                                // self.replace_nop(target)?;
                                // self.replace_nop(base)?;
                            }
                            Op::I32Const => {
                                let const_val = self.param(target, len2, 2)?;
                                self.replace_nop(target)?;
                                self.replace(
                                    base,
                                    Op::I32Const,
                                    &[result.0, (const_val == 0) as u32],
                                )?;
                            }

                            Op::Eq => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(base, Op::Ne, &[result.0, lhs.0, rhs.0])?;
                            }
                            Op::Ne => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(base, Op::Eq, &[result.0, lhs.0, rhs.0])?;
                            }
                            Op::Lt => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(base, Op::Ge, &[result.0, lhs.0, rhs.0])?;
                            }
                            Op::Gt => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(base, Op::Le, &[result.0, lhs.0, rhs.0])?;
                            }
                            Op::Le => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(base, Op::Gt, &[result.0, lhs.0, rhs.0])?;
                            }
                            Op::Ge => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(base, Op::Lt, &[result.0, lhs.0, rhs.0])?;
                            }

                            _ => {}
                        }
                    }

                    Op::Nop
                    | Op::Block
                    | Op::Call
                    | Op::Clz
                    | Op::Ctz
                    | Op::Dec
                    | Op::End
                    | Op::F32Const
                    | Op::F64Const
                    | Op::Ge
                    | Op::Gt
                    | Op::I32Const
                    | Op::I64Const
                    | Op::Inc
                    | Op::Le
                    | Op::LocalGet
                    | Op::LocalSet
                    | Op::LocalTee
                    | Op::Loop
                    | Op::Lt
                    | Op::Neg
                    | Op::Not
                    | Op::Popcnt => {}
                }
            }
        }

        let mut block_freqs = BTreeMap::new();
        {
            // Reduce unnecessary blocks - step 1
            let mut last_block = None;
            let mut block_empty_check = false;
            let mut ci = 0;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (len, opcode) = self.get_op(base)?;

                if block_empty_check {
                    match opcode {
                        Op::Nop => {}
                        Op::Br => {
                            let block_index = self.param(base, len, 1)?;
                            if let Some(last_block) = last_block {
                                if last_block == block_index {
                                    self.replace_nop(base)?;
                                    continue;
                                }
                            }
                            last_block = None;
                            block_empty_check = false;
                        }
                        Op::Loop => {}
                        _ => {
                            last_block = None;
                            block_empty_check = false;
                        }
                    }
                }

                match opcode {
                    // Op::Nop => {}
                    Op::Block => {
                        let block_index = self.param(base, len, 1)?;
                        block_freqs.insert(block_index, 0usize);
                        last_block = Some(block_index);
                        block_empty_check = true;
                    }
                    Op::Loop => {
                        let block_index = self.param(base, len, 1)?;
                        block_freqs.insert(block_index, 0usize);
                    }
                    Op::Br | Op::BrIf => {
                        let block_index = self.param(base, len, 1)?;
                        *block_freqs
                            .get_mut(&block_index)
                            .ok_or(OptimizeError::InvalidBranch(base.as_usize(), block_index))? +=
                            1;
                    }

                    _ => {}
                }
            }
        }
        {
            // Reduce unnecessary blocks - step 2
            let mut ci = 0;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (len, opcode) = self.get_op(base)?;
                match opcode {
                    Op::Block | Op::Loop | Op::End => {
                        let block_index = self.param(base, len, 1)?;
                        if *block_freqs
                            .get(&block_index)
                            .ok_or(OptimizeError::InvalidBranch(base.as_usize(), block_index))?
                            == 0
                        {
                            self.replace_nop(base)?;
                        }
                    }
                    _ => {}
                }
            }
        }
        drop(block_freqs);

        {
            // Reduce unnecessary operations - step 2
            let mut ci = 0;
            let mut skip_until_end = false;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (_len, opcode) = self.get_op(base)?;

                if skip_until_end {
                    if opcode != Op::End {
                        self.replace_nop(base)?;
                    } else {
                        skip_until_end = false;
                    }
                    continue;
                }

                match opcode {
                    Op::Br | Op::Return | Op::Unreachable => {
                        skip_until_end = true;
                    }

                    _ => {}
                }
            }
        }

        {
            // compaction & renumber
            let mut ren_tbl = BTreeMap::new();
            let mut new_positions = Vec::new();
            let mut new_ci = 0;
            let mut oi = 0;
            for (i, ai) in self.positions.iter().enumerate() {
                let ci = CodeIndex(i as u32);
                let ai = *ai;
                let (len, opc) = self.get_op(ai)?;
                if opc == Op::Nop {
                    continue;
                }
                ren_tbl.insert(ci, CodeIndex(new_ci));
                new_positions.push(ArrayIndex(oi as u32));
                for i in 0..len {
                    self.codes[oi + i] = self.codes[ai.as_usize() + i];
                }
                oi += len;
                new_ci += 1;
            }
            self.codes.resize(oi, 0xCCCC_CCCC);
            self.codes.shrink_to_fit();
            self.positions = new_positions;

            let mut ci = 0;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (len, opcode) = self.get_op(base)?;
                match opcode {
                    Op::Nop | Op::Unreachable => {}

                    Op::Return => {
                        for i in 1..len {
                            self.rename(&ren_tbl, base, len, i)?;
                        }
                    }

                    Op::Call => {
                        for i in 1..len {
                            if i == 2 {
                                continue;
                            }
                            self.rename(&ren_tbl, base, len, i)?;
                        }
                    }

                    // %n = op [n, ...]
                    Op::Block
                    | Op::Br
                    | Op::Drop
                    | Op::End
                    | Op::F32Const
                    | Op::F64Const
                    | Op::I32Const
                    | Op::I64Const
                    | Op::LocalGet
                    | Op::LocalSet
                    | Op::LocalTee
                    | Op::Loop => {
                        self.rename(&ren_tbl, base, len, 1)?;
                    }

                    // unary
                    // %n = op %n
                    Op::BrIf
                    | Op::Clz
                    | Op::Ctz
                    | Op::Dec
                    | Op::Eqz
                    | Op::Inc
                    | Op::Neg
                    | Op::Not
                    | Op::Popcnt => {
                        for i in 1..=2 {
                            self.rename(&ren_tbl, base, len, i)?;
                        }
                    }

                    // binary
                    // %n = op %n, %n
                    Op::Add
                    | Op::And
                    | Op::Div
                    | Op::Eq
                    | Op::Ge
                    | Op::Gt
                    | Op::Le
                    | Op::Lt
                    | Op::Mul
                    | Op::Ne
                    | Op::Or
                    | Op::Rem
                    | Op::Rotl
                    | Op::Rotr
                    | Op::Shl
                    | Op::Shr
                    | Op::Sub
                    | Op::Xor => {
                        for i in 1..=3 {
                            self.rename(&ren_tbl, base, len, i)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn rename(
        &mut self,
        ren_tbl: &BTreeMap<CodeIndex, CodeIndex>,
        base: ArrayIndex,
        len: usize,
        index: usize,
    ) -> Result<(), OptimizeError> {
        if index >= len {
            return Err(OptimizeError::InvalidParameter(base.as_usize(), index));
        }
        let addr = base.as_usize() + index;
        let p = self
            .codes
            .get_mut(addr)
            .ok_or(OptimizeError::OutOfCodes(addr))?;

        let old = CodeIndex(*p);
        let new = ren_tbl.get(&old).ok_or(OptimizeError::RenameError(
            base.as_usize(),
            index,
            old.as_usize(),
        ))?;
        *p = new.0;

        Ok(())
    }

    fn chain_drop(&mut self, index: CodeIndex) -> Result<bool, OptimizeError> {
        let base = self.array_index(index)?;
        let (len, opcode) = self.get_op(base)?;

        match opcode {
            Op::Nop | Op::F32Const | Op::F64Const | Op::I32Const | Op::I64Const | Op::LocalGet => {
                self.replace_nop(base)?;
                return Ok(true);
            }

            Op::Call => Ok(false),

            Op::LocalTee => {
                self.replace_opcode(base, Op::LocalSet)?;
                Ok(true)
            }

            // binop
            Op::Add
            | Op::And
            | Op::Div
            | Op::Eq
            | Op::Ge
            | Op::Gt
            | Op::Le
            | Op::Lt
            | Op::Mul
            | Op::Ne
            | Op::Or
            | Op::Rem
            | Op::Rotl
            | Op::Rotr
            | Op::Shl
            | Op::Shr
            | Op::Sub
            | Op::Xor => {
                self.chain_drop(CodeIndex(self.param(base, len, 2)?))?;
                self.chain_drop(CodeIndex(self.param(base, len, 3)?))?;
                self.replace_nop(base)?;
                return Ok(true);
            }

            // unop
            Op::Clz | Op::Ctz | Op::Dec | Op::Eqz | Op::Inc | Op::Neg | Op::Not | Op::Popcnt => {
                self.chain_drop(CodeIndex(self.param(base, len, 2)?))?;
                self.replace_nop(base)?;
                return Ok(true);
            }

            Op::Block
            | Op::Loop
            | Op::Br
            | Op::BrIf
            | Op::LocalSet
            | Op::Return
            | Op::Unreachable
            | Op::Drop
            | Op::End => Err(OptimizeError::InvalidDropChain(index.as_usize())),
        }
    }

    #[inline]
    fn param(&self, base: ArrayIndex, len: usize, index: usize) -> Result<u32, OptimizeError> {
        if index >= len {
            return Err(OptimizeError::InvalidParameter(base.as_usize(), index));
        }
        let addr = base.as_usize() + index;
        self.codes
            .get(addr)
            .map(|v| *v)
            .ok_or(OptimizeError::OutOfCodes(addr))
    }

    #[inline]
    fn array_index(&self, index: CodeIndex) -> Result<ArrayIndex, OptimizeError> {
        self.positions
            .get(index.as_usize())
            .map(|v| *v)
            .ok_or(OptimizeError::OutOfPosition(index.as_usize()))
    }

    fn get_op(&self, index: ArrayIndex) -> Result<(usize, Op), OptimizeError> {
        let Some(len_opc) = self.codes.get(index.as_usize()) else {
            return Err(OptimizeError::OutOfCodes(index.as_usize()));
        };

        let len_opc = *len_opc;
        let len = (len_opc >> 16) as usize;
        let opcode = unsafe { transmute::<u8, Op>((len_opc & 0xFFFF) as u8) };

        Ok((len, opcode))
    }

    fn get_i32_const(&self, index: ArrayIndex) -> Result<Option<u32>, OptimizeError> {
        let (len, opcode) = self.get_op(index)?;
        if matches!(opcode, Op::I32Const) {
            let i = self.param(index, len, 2)?;
            Ok(Some(i))
        } else {
            Ok(None)
        }
    }

    fn replace(
        &mut self,
        target: ArrayIndex,
        opcode: Op,
        params: &[u32],
    ) -> Result<(), OptimizeError> {
        let new_len = params.len() + 1;
        let (len, _) = self.get_op(target)?;
        if len < new_len {
            return Err(OptimizeError::OverwriteError(
                target.as_usize(),
                len,
                new_len,
            ));
        }
        self.codes[target.as_usize()] = ((new_len as u32) << 16) | (opcode as u32);
        for (i, v) in params.iter().enumerate() {
            self.codes[target.as_usize() + i + 1] = *v;
        }
        for i in new_len..len {
            self.codes[target.as_usize() + i] = 0x0001_0000 | (Op::Nop as u32);
        }
        Ok(())
    }

    fn replace_opcode(&mut self, target: ArrayIndex, opcode: Op) -> Result<(), OptimizeError> {
        let (len, _) = self.get_op(target)?;
        self.codes[target.as_usize()] = ((len as u32) << 16) | (opcode as u32);
        Ok(())
    }

    fn replace_nop(&mut self, target: ArrayIndex) -> Result<(), OptimizeError> {
        self.replace_opcode(target, Op::Nop)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CodeIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ArrayIndex(u32);

impl CodeIndex {
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl ArrayIndex {
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
