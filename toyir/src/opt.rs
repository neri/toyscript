//! Minimal Code Optimizer

use super::*;
use core::{
    mem::transmute,
    ops::{BitAnd, BitOr, BitXor},
};
use error::OptimizeError;

pub struct MinimalCodeOptimizer {
    positions: Vec<ArrayIndex>,
    codes: Vec<u32>,
    params: Vec<LocalVarDescriptor>,
    locals: Vec<LocalVarDescriptor>,
}

impl MinimalCodeOptimizer {
    pub fn optimize(
        codes: Vec<u32>,
        params: &[LocalVarDescriptor],
        locals: &[LocalVarDescriptor],
    ) -> Result<Vec<u32>, OptimizeError> {
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

        let mut optimizer = Self {
            codes,
            positions,
            params: params.to_vec(),
            locals: locals.to_vec(),
        };
        optimizer._optimize()?;

        Ok(optimizer.codes)
    }

    fn _optimize(&mut self) -> Result<(), OptimizeError> {
        {
            // Reduce wasted instructions that lead to DROP instructions first to shorten optimization time.
            let mut ci = 0;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (len, opcode) = self.get_op(base)?;

                match opcode {
                    Op::Drop => {
                        let operand = CodeIndex(self.param(base, len, 1)?);
                        if self.chain_drop(operand)? {
                            self.replace_nop(base)?;
                        }
                    }

                    _ => {}
                }
            }
        }

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

                    // cmp
                    Op::Eq
                    | Op::GeS
                    | Op::GeU
                    | Op::GtS
                    | Op::GtU
                    | Op::LeS
                    | Op::LeU
                    | Op::LtS
                    | Op::LtU => {
                        let result = CodeIndex(self.param(base, len, 1)?);
                        let lhs = CodeIndex(self.param(base, len, 2)?);
                        let rhs = CodeIndex(self.param(base, len, 3)?);
                        self.reduce_cmp(base, opcode, result, lhs, rhs)?;
                    }

                    // binop
                    Op::Add
                    | Op::And
                    | Op::DivS
                    | Op::DivU
                    | Op::Mul
                    | Op::Ne
                    | Op::Or
                    | Op::RemS
                    | Op::RemU
                    | Op::Rotl
                    | Op::Rotr
                    | Op::Shl
                    | Op::ShrS
                    | Op::ShrU
                    | Op::Sub
                    | Op::Xor => {
                        let result = CodeIndex(self.param(base, len, 1)?);
                        let lhs = CodeIndex(self.param(base, len, 2)?);
                        let rhs = CodeIndex(self.param(base, len, 3)?);
                        self.reduce_binop(base, opcode, result, lhs, rhs)?;
                    }

                    // unop
                    Op::Eqz => {
                        let result = CodeIndex(self.param(base, len, 1)?);
                        let target = self.array_index(CodeIndex(self.param(base, len, 2)?))?;
                        let (len2, op2) = self.get_op(target)?;
                        match op2 {
                            Op::Eqz => {
                                // self.replace_opcode(target, Op::UnaryNop)?;
                                // self.replace_opcode(base, Op::UnaryNop)?;
                            }
                            Op::I32Const => {
                                let const_val = self.param(target, len2, 2)?;
                                self.replace_nop(target)?;
                                self.replace_i32_const(base, result, (const_val == 0) as i32)?;
                            }

                            Op::Eq
                            | Op::Ne
                            | Op::LtS
                            | Op::GtS
                            | Op::LeS
                            | Op::GeS
                            | Op::LtU
                            | Op::GtU
                            | Op::LeU
                            | Op::GeU => {
                                let lhs = CodeIndex(self.param(target, len2, 2)?);
                                let rhs = CodeIndex(self.param(target, len2, 3)?);
                                self.replace_nop(target)?;
                                self.replace(
                                    base,
                                    op2.inverted_condition(),
                                    &[result.0, lhs.0, rhs.0],
                                )?;
                            }

                            _ => {}
                        }
                    }

                    Op::LocalSet => {
                        let operand_ci = CodeIndex(self.param(base, len, 1)?);
                        let operand = self.array_index(operand_ci)?;
                        let const_val = self.get_const(operand)?;
                        if let Some(const_val) = const_val {
                            let local_index = self.param(base, len, 2)?;
                            let var_desc = self.get_local_mut(local_index as usize).unwrap();
                            if var_desc.is_const() {
                                var_desc.assignment = Some(const_val);
                                if self.chain_drop(operand_ci)? {
                                    self.replace_nop(base)?;
                                }
                            }
                        }
                    }

                    Op::LocalTee => {
                        let operand = self.array_index(CodeIndex(self.param(base, len, 3)?))?;
                        let const_val = self.get_const(operand)?;
                        if let Some(const_val) = const_val {
                            let local_index = self.param(base, len, 2)?;
                            let var_desc = self.get_local_mut(local_index as usize).unwrap();
                            if var_desc.is_const() {
                                var_desc.assignment = Some(const_val);
                            }
                        }
                    }

                    Op::LocalGet => {
                        let result = CodeIndex(self.param(base, len, 1)?);
                        let local_index = self.param(base, len, 2)?;
                        let var_desc = self.get_local(local_index as usize).unwrap();
                        if var_desc.is_const() {
                            if let Some(value) = var_desc.assignment {
                                self.replace_const(base, result, value)?;
                            }
                        }
                    }

                    Op::Nop
                    | Op::UnaryNop
                    | Op::DropRight
                    | Op::Drop2
                    | Op::Block
                    | Op::Call
                    | Op::Clz
                    | Op::Ctz
                    | Op::Dec
                    | Op::End
                    | Op::F32Const
                    | Op::F64Const
                    | Op::I32Const
                    | Op::I64Const
                    | Op::Inc
                    | Op::Loop
                    | Op::Not
                    | Op::Popcnt
                    | Op::Neg => {}
                }
            }
        }

        {
            // Reduce unnecessary blocks
            let mut block_freqs = BTreeMap::new();
            {
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
                            *block_freqs.get_mut(&block_index).ok_or(
                                OptimizeError::InvalidBranch(base.as_usize(), block_index),
                            )? += 1;
                        }

                        _ => {}
                    }
                }
            }

            {
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
        }

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

                    Op::LocalTee => {
                        self.rename(&ren_tbl, base, len, 1)?;
                        self.rename(&ren_tbl, base, len, 3)?;
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
                    | Op::Loop => {
                        self.rename(&ren_tbl, base, len, 1)?;
                    }

                    // unary
                    // %n = op %n
                    Op::UnaryNop
                    | Op::BrIf
                    | Op::Clz
                    | Op::Ctz
                    | Op::Dec
                    | Op::Eqz
                    | Op::Inc
                    | Op::Not
                    | Op::Popcnt
                    | Op::Neg => {
                        for i in 1..=2 {
                            self.rename(&ren_tbl, base, len, i)?;
                        }
                    }

                    // binary
                    // %n = op %n, %n
                    Op::DropRight
                    | Op::Add
                    | Op::And
                    | Op::DivS
                    | Op::DivU
                    | Op::Eq
                    | Op::GeS
                    | Op::GeU
                    | Op::GtS
                    | Op::GtU
                    | Op::LeS
                    | Op::LeU
                    | Op::LtS
                    | Op::LtU
                    | Op::Mul
                    | Op::Ne
                    | Op::Or
                    | Op::RemS
                    | Op::RemU
                    | Op::Rotl
                    | Op::Rotr
                    | Op::Shl
                    | Op::ShrS
                    | Op::ShrU
                    | Op::Sub
                    | Op::Xor => {
                        for i in 1..=3 {
                            self.rename(&ren_tbl, base, len, i)?;
                        }
                    }

                    Op::Drop2 => {
                        for i in 2..=3 {
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

    /// Drops a droppable instruction chained to the specified instruction
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if dropable, `Ok(false)` if retained.
    fn chain_drop(&mut self, index: CodeIndex) -> Result<bool, OptimizeError> {
        let base = self.array_index(index)?;
        let (len, opcode) = self.get_op(base)?;

        match opcode {
            Op::F32Const | Op::F64Const | Op::I32Const | Op::I64Const | Op::LocalGet => {
                self.replace_nop(base)?;
                return Ok(true);
            }

            Op::Call => Ok(false),

            Op::DropRight => {
                let lhs = self.param(base, len, 2)?;
                let rhs = self.param(base, len, 3)?;
                if self.chain_drop(CodeIndex(lhs))? {
                    self.replace(base, Op::Drop, &[rhs])?;
                }
                return Ok(true);
            }

            Op::LocalTee => {
                let operand2 = self.param(base, len, 2)?;
                let operand3 = self.param(base, len, 3)?;
                self.replace(base, Op::LocalSet, &[operand3, operand2])?;
                Ok(true)
            }

            // binop
            Op::Add
            | Op::And
            | Op::DivS
            | Op::DivU
            | Op::Eq
            | Op::GeS
            | Op::GeU
            | Op::GtS
            | Op::GtU
            | Op::LeS
            | Op::LeU
            | Op::LtS
            | Op::LtU
            | Op::Mul
            | Op::Ne
            | Op::Or
            | Op::RemS
            | Op::RemU
            | Op::Rotl
            | Op::Rotr
            | Op::Shl
            | Op::ShrS
            | Op::ShrU
            | Op::Sub
            | Op::Xor => {
                let result = self.param(base, len, 1)?;
                let lhs = self.param(base, len, 2)?;
                let rhs = self.param(base, len, 3)?;
                let lhs_droppable = self.chain_drop(CodeIndex(lhs))?;
                let rhs_droppable = self.chain_drop(CodeIndex(rhs))?;
                match (lhs_droppable, rhs_droppable) {
                    (true, true) => {
                        self.replace_nop(base)?;
                        return Ok(true);
                    }
                    (true, false) => {
                        self.replace(base, Op::Drop, &[rhs])?;
                        return Ok(true);
                    }
                    (false, true) => {
                        self.replace(base, Op::UnaryNop, &[result, lhs])?;
                        return Ok(false);
                    }
                    (false, false) => {
                        self.replace_opcode(base, Op::Drop2)?;
                        return Ok(true);
                    }
                }
            }

            // unop
            Op::UnaryNop
            | Op::Clz
            | Op::Ctz
            | Op::Dec
            | Op::Eqz
            | Op::Inc
            | Op::Not
            | Op::Popcnt
            | Op::Neg => {
                let operand2 = self.param(base, len, 2)?;
                if self.chain_drop(CodeIndex(operand2))? {
                    self.replace_nop(base)?;
                } else {
                    self.replace(base, Op::Drop, &[operand2])?;
                }
                return Ok(true);
            }

            Op::Drop2
            | Op::Block
            | Op::Br
            | Op::BrIf
            | Op::Drop
            | Op::End
            | Op::LocalSet
            | Op::Loop
            | Op::Nop
            | Op::Return
            | Op::Unreachable => Err(OptimizeError::InvalidDropChain(index.as_usize())),
        }
    }

    fn reduce_binop(
        &mut self,
        base: ArrayIndex,
        opcode: Op,
        result: CodeIndex,
        lhs: CodeIndex,
        rhs: CodeIndex,
    ) -> Result<bool, OptimizeError> {
        let lhs_a = self.array_index(lhs)?;
        let rhs_a = self.array_index(rhs)?;
        let lhs_const = self.get_const(lhs_a)?;
        let rhs_const = self.get_const(rhs_a)?;

        // binop (const, const)
        match (lhs_const, rhs_const) {
            (Some(Constant::I32(lhs)), Some(Constant::I32(rhs))) => {
                let value = match opcode {
                    Op::Add => lhs.wrapping_add(rhs),
                    Op::Sub => lhs.wrapping_sub(rhs),
                    Op::Mul => lhs.wrapping_mul(rhs),
                    Op::DivS => match lhs.checked_div(rhs) {
                        Some(v) => v,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::RemS => match lhs.checked_rem(rhs) {
                        Some(v) => v,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::DivU => match (lhs as u32).checked_div(rhs as u32) {
                        Some(v) => v as i32,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::RemU => match (lhs as u32).checked_rem(rhs as u32) {
                        Some(v) => v as i32,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::And => lhs.bitand(rhs),
                    Op::Or => lhs.bitor(rhs),
                    Op::Xor => lhs.bitxor(rhs),
                    Op::Shl => lhs.wrapping_shl(rhs as u32),
                    Op::ShrS => lhs.wrapping_shr(rhs as u32),
                    Op::ShrU => (lhs as u32).wrapping_shr(rhs as u32) as i32,
                    Op::Rotl => lhs.rotate_left(rhs as u32),
                    Op::Rotr => lhs.rotate_right(rhs as u32),

                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i32_const(base, result, value)?;
                return Ok(true);
            }
            (Some(Constant::I64(lhs)), Some(Constant::I64(rhs))) => {
                let value = match opcode {
                    Op::Add => lhs.wrapping_add(rhs),
                    Op::Sub => lhs.wrapping_sub(rhs),
                    Op::Mul => lhs.wrapping_mul(rhs),
                    Op::DivS => match lhs.checked_div(rhs) {
                        Some(v) => v,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::RemS => match lhs.checked_rem(rhs) {
                        Some(v) => v,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::DivU => match (lhs as u64).checked_div(rhs as u64) {
                        Some(v) => v as i64,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::RemU => match (lhs as u64).checked_rem(rhs as u64) {
                        Some(v) => v as i64,
                        // cannot convert NaN
                        None => return Ok(false),
                    },
                    Op::And => lhs.bitand(rhs),
                    Op::Or => lhs.bitor(rhs),
                    Op::Xor => lhs.bitxor(rhs),
                    Op::Shl => lhs.wrapping_shl(rhs as u32),
                    Op::ShrS => lhs.wrapping_shr(rhs as u32),
                    Op::ShrU => (lhs as u64).wrapping_shr(rhs as u32) as i64,
                    Op::Rotl => lhs.rotate_left(rhs as u32),
                    Op::Rotr => lhs.rotate_right(rhs as u32),

                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i64_const(base, result, value)?;
                return Ok(true);
            }
            (Some(Constant::F32(lhs)), Some(Constant::F32(rhs))) => {
                let value = match opcode {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::DivS => lhs / rhs,
                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_f32_const(base, result, value)?;
                return Ok(true);
            }
            (Some(Constant::F64(lhs)), Some(Constant::F64(rhs))) => {
                let value = match opcode {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::DivS => lhs / rhs,
                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_f64_const(base, result, value)?;
                return Ok(true);
            }
            _ => {}
        }

        if let Some(Constant::I32(rhs_const)) = rhs_const {
            // binop (???, 0)
            if rhs_const == 0 {
                match opcode {
                    Op::Add
                    | Op::Sub
                    | Op::Or
                    | Op::Xor
                    | Op::Shl
                    | Op::ShrS
                    | Op::ShrU
                    | Op::Rotl
                    | Op::Rotr => {
                        self.replace_nop(rhs_a)?;
                        self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                        return Ok(true);
                    }
                    Op::Mul | Op::And => {
                        if self.chain_drop(lhs)? {
                            self.replace_nop(rhs_a)?;
                        } else {
                            self.replace(rhs_a, Op::Drop, &[lhs.0])?;
                        }
                        self.replace_i32_const(base, result, 0)?;
                        return Ok(true);
                    }
                    // TODO: cannot convert NaN
                    Op::DivS | Op::DivU | Op::RemS | Op::RemU => return Ok(false),
                    _ => {}
                }
            }

            {
                // mul or div (???, const (power of two))
                let rhs_const = rhs_const as u32;
                if rhs_const.next_power_of_two() == rhs_const {
                    match opcode {
                        Op::Mul => {
                            if rhs_const == 1 {
                                self.replace_nop(rhs_a)?;
                                self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                            } else {
                                self.replace_i32_const(
                                    rhs_a,
                                    rhs,
                                    rhs_const.trailing_zeros() as i32,
                                )?;
                                self.replace_opcode(base, Op::Shl)?;
                            }
                            return Ok(true);
                        }
                        Op::DivU => {
                            if rhs_const == 1 {
                                self.replace_nop(rhs_a)?;
                                self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                            } else {
                                self.replace_i32_const(
                                    rhs_a,
                                    rhs,
                                    rhs_const.trailing_zeros() as i32,
                                )?;
                                self.replace_opcode(base, Op::ShrU)?;
                            }
                            return Ok(true);
                        }
                        Op::RemU => {
                            self.replace_i32_const(rhs_a, rhs, rhs_const.wrapping_sub(1) as i32)?;
                            self.replace_opcode(base, Op::And)?;
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(Constant::I32(lhs_const)) = lhs_const {
            // binop (0, ???)
            if lhs_const == 0 {
                match opcode {
                    Op::Add | Op::Or | Op::Xor => {
                        self.replace_nop(lhs_a)?;
                        self.replace(base, Op::UnaryNop, &[result.0, rhs.0])?;
                        return Ok(true);
                    }
                    Op::Mul
                    | Op::DivS
                    | Op::DivU
                    | Op::RemS
                    | Op::RemU
                    | Op::And
                    | Op::Shl
                    | Op::ShrS
                    | Op::ShrU
                    | Op::Rotl
                    | Op::Rotr => {
                        if self.chain_drop(rhs)? {
                            self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                        } else {
                            self.replace_opcode(base, Op::DropRight)?;
                        }
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }

        if let Some(Constant::I64(rhs_const)) = rhs_const {
            // binop (???, 0)
            if rhs_const == 0 {
                match opcode {
                    Op::Add
                    | Op::Sub
                    | Op::Or
                    | Op::Xor
                    | Op::Shl
                    | Op::ShrS
                    | Op::ShrU
                    | Op::Rotl
                    | Op::Rotr => {
                        self.replace_nop(rhs_a)?;
                        self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                        return Ok(true);
                    }
                    Op::Mul | Op::And => {
                        if self.chain_drop(lhs)? {
                            self.replace_nop(rhs_a)?;
                        } else {
                            self.replace(rhs_a, Op::Drop, &[lhs.0])?;
                        }
                        self.replace_i64_const(base, result, 0)?;
                        return Ok(true);
                    }
                    // TODO: cannot convert NaN
                    Op::DivS | Op::DivU | Op::RemS | Op::RemU => return Ok(false),
                    _ => {}
                }
            }

            {
                // mul or div (???, const (power of two))
                let rhs_const = rhs_const as u64;
                if rhs_const.next_power_of_two() == rhs_const {
                    match opcode {
                        Op::Mul => {
                            if rhs_const == 1 {
                                self.replace_nop(rhs_a)?;
                                self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                            } else {
                                self.replace_i64_const(
                                    rhs_a,
                                    rhs,
                                    rhs_const.trailing_zeros() as i64,
                                )?;
                                self.replace_opcode(base, Op::Shl)?;
                            }
                            return Ok(true);
                        }
                        Op::DivU => {
                            if rhs_const == 1 {
                                self.replace_nop(rhs_a)?;
                                self.replace(base, Op::UnaryNop, &[result.0, lhs.0])?;
                            } else {
                                self.replace_i64_const(
                                    rhs_a,
                                    rhs,
                                    rhs_const.trailing_zeros() as i64,
                                )?;
                                self.replace_opcode(base, Op::ShrU)?;
                            }
                            return Ok(true);
                        }
                        Op::RemU => {
                            self.replace_i64_const(rhs_a, rhs, rhs_const.wrapping_sub(1) as i64)?;
                            self.replace_opcode(base, Op::And)?;
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(Constant::F32(lhs_const)) = lhs_const {
            // binop (0, ???)
            if lhs_const == 0.0 {
                match opcode {
                    Op::Sub => {
                        self.replace_nop(lhs_a)?;
                        self.replace(base, Op::Neg, &[result.0, rhs.0])?;
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }

        if let Some(Constant::F64(lhs_const)) = lhs_const {
            // binop (0, ???)
            if lhs_const == 0.0 {
                match opcode {
                    Op::Sub => {
                        self.replace_nop(lhs_a)?;
                        self.replace(base, Op::Neg, &[result.0, rhs.0])?;
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }

        Ok(false)
    }

    fn reduce_cmp(
        &mut self,
        base: ArrayIndex,
        opcode: Op,
        result: CodeIndex,
        lhs: CodeIndex,
        rhs: CodeIndex,
    ) -> Result<bool, OptimizeError> {
        let lhs_a = self.array_index(lhs)?;
        let rhs_a = self.array_index(rhs)?;
        let lhs_const = self.get_const(lhs_a)?;
        let rhs_const = self.get_const(rhs_a)?;

        // binop (const, const)
        match (lhs_const, rhs_const) {
            (Some(Constant::I32(lhs)), Some(Constant::I32(rhs))) => {
                let value = match opcode {
                    Op::Eq => lhs == rhs,
                    Op::Ne => lhs != rhs,
                    Op::GeS => lhs >= rhs,
                    Op::GeU => (lhs as u32) >= (rhs as u32),
                    Op::GtS => lhs > rhs,
                    Op::GtU => (lhs as u32) > (rhs as u32),
                    Op::LeS => lhs <= rhs,
                    Op::LeU => (lhs as u32) <= (rhs as u32),
                    Op::LtS => lhs < rhs,
                    Op::LtU => (lhs as u32) < (rhs as u32),
                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i32_const(base, result, value as i32)?;
                return Ok(true);
            }
            (Some(Constant::I64(lhs)), Some(Constant::I64(rhs))) => {
                let value = match opcode {
                    Op::Eq => lhs == rhs,
                    Op::Ne => lhs != rhs,
                    Op::GeS => lhs >= rhs,
                    Op::GeU => (lhs as u64) >= (rhs as u64),
                    Op::GtS => lhs > rhs,
                    Op::GtU => (lhs as u64) > (rhs as u64),
                    Op::LeS => lhs <= rhs,
                    Op::LeU => (lhs as u64) <= (rhs as u64),
                    Op::LtS => lhs < rhs,
                    Op::LtU => (lhs as u64) < (rhs as u64),
                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i32_const(base, result, value as i32)?;
                return Ok(true);
            }
            (Some(Constant::F32(lhs)), Some(Constant::F32(rhs))) => {
                let value = match opcode {
                    Op::Eq => lhs == rhs,
                    Op::Ne => lhs != rhs,
                    Op::GeS => lhs >= rhs,
                    Op::GtS => lhs > rhs,
                    Op::LeS => lhs <= rhs,
                    Op::LtS => lhs < rhs,
                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i32_const(base, result, value as i32)?;
                return Ok(true);
            }
            (Some(Constant::F64(lhs)), Some(Constant::F64(rhs))) => {
                let value = match opcode {
                    Op::Eq => lhs == rhs,
                    Op::Ne => lhs != rhs,
                    Op::GeS => lhs >= rhs,
                    Op::GtS => lhs > rhs,
                    Op::LeS => lhs <= rhs,
                    Op::LtS => lhs < rhs,
                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i32_const(base, result, value as i32)?;
                return Ok(true);
            }
            _ => {}
        }

        if let Some(Constant::I32(rhs_const)) = rhs_const {
            // binop (???, 0)
            if rhs_const == 0 {
                match opcode {
                    Op::Eq => {
                        self.replace_nop(rhs_a)?;
                        self.replace(base, Op::Eqz, &[result.0, lhs.0])?;
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }

        Ok(false)
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

    fn get_const(&self, index: ArrayIndex) -> Result<Option<Constant>, OptimizeError> {
        let (len, opcode) = self.get_op(index)?;
        match opcode {
            Op::I32Const => {
                let i = self.param(index, len, 2)?;
                Ok(Some(Constant::I32(i as i32)))
            }
            Op::I64Const => {
                let l = self.param(index, len, 2)?;
                let h = self.param(index, len, 3)?;
                let i = ((h as u64) << 32) + (l as u64);
                Ok(Some(Constant::I64(i as i64)))
            }
            Op::F32Const => {
                let i = self.param(index, len, 2)?;
                Ok(Some(Constant::F32(f32::from_bits(i))))
            }
            Op::F64Const => {
                let l = self.param(index, len, 2)?;
                let h = self.param(index, len, 3)?;
                let i = ((h as u64) << 32) + (l as u64);
                Ok(Some(Constant::F64(f64::from_bits(i))))
            }
            _ => Ok(None),
        }
    }

    fn get_i32_const(&self, index: ArrayIndex) -> Result<Option<i32>, OptimizeError> {
        self.get_const(index).map(|v| match v {
            Some(v) => match v {
                Constant::I32(v) => Some(v),
                _ => None,
            },
            None => None,
        })
    }

    fn get_local(&self, index: usize) -> Option<&LocalVarDescriptor> {
        if index < self.params.len() {
            self.params.get(index)
        } else {
            self.locals.get(index - self.params.len())
        }
    }

    fn get_local_mut(&mut self, index: usize) -> Option<&mut LocalVarDescriptor> {
        if index < self.params.len() {
            self.params.get_mut(index)
        } else {
            self.locals.get_mut(index - self.params.len())
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

    fn replace_const(
        &mut self,
        target: ArrayIndex,
        result: CodeIndex,
        value: Constant,
    ) -> Result<(), OptimizeError> {
        match value {
            Constant::I32(value) => self.replace_i32_const(target, result, value),
            Constant::I64(value) => self.replace_i64_const(target, result, value),
            Constant::F32(value) => self.replace_f32_const(target, result, value),
            Constant::F64(value) => self.replace_f64_const(target, result, value),
        }
    }

    fn replace_i32_const(
        &mut self,
        target: ArrayIndex,
        result: CodeIndex,
        value: i32,
    ) -> Result<(), OptimizeError> {
        self.replace(target, Op::I32Const, &[result.0, value as u32])
    }

    fn replace_i64_const(
        &mut self,
        target: ArrayIndex,
        result: CodeIndex,
        value: i64,
    ) -> Result<(), OptimizeError> {
        let value = value as u64;
        let l = value as u32;
        let h = (value >> 32) as u32;
        self.replace(target, Op::I64Const, &[result.0, l, h])
    }

    fn replace_f32_const(
        &mut self,
        target: ArrayIndex,
        result: CodeIndex,
        value: f32,
    ) -> Result<(), OptimizeError> {
        self.replace(target, Op::F32Const, &[result.0, value.to_bits()])
    }

    fn replace_f64_const(
        &mut self,
        target: ArrayIndex,
        result: CodeIndex,
        value: f64,
    ) -> Result<(), OptimizeError> {
        let value = value.to_bits();
        let l = value as u32;
        let h = (value >> 32) as u32;
        self.replace(target, Op::F64Const, &[result.0, l, h])
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
