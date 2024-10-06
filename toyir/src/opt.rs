//! Minimal Code Optimizer

#[path = "./_generated/opt_cast.rs"]
mod opt_cast;

use super::*;
use core::{
    mem::transmute,
    ops::{BitAnd, BitOr, BitXor},
};
use error::OptimizeError;
use opt_cast::opt_cast;

pub struct MinimalCodeOptimizer {
    positions: Vec<ArrayIndex>,
    codes: Vec<u32>,
    params: Vec<LocalVarDescriptor>,
    locals: Vec<LocalVarDescriptor>,
    call_dependency_list: Vec<FuncTempIndex>,
}

impl MinimalCodeOptimizer {
    pub fn optimize(
        codes: Vec<u32>,
        params: &[LocalVarDescriptor],
        locals: &[LocalVarDescriptor],
    ) -> Result<(Vec<u32>, Vec<LocalVarDescriptor>, Vec<FuncTempIndex>), OptimizeError> {
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
            call_dependency_list: Vec::new(),
        };
        optimizer._optimize()?;

        Ok((
            optimizer.codes,
            optimizer.locals,
            optimizer.call_dependency_list,
        ))
    }

    fn _optimize(&mut self) -> Result<(), OptimizeError> {
        let mut will_remove_vars = Vec::new();

        for _ in 0..2 {
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
                            let local_index = self.param(base, len, 2)?;
                            if will_remove_vars.contains(unsafe { &LocalIndex::new(local_index) }) {
                                if self.chain_drop(operand_ci)? {
                                    self.replace_nop(base)?;
                                } else {
                                    self.replace(base, Op::Drop, &[operand_ci.0])?;
                                }
                            } else {
                                let operand_ai = self.array_index(operand_ci)?;
                                let const_val = self.get_const(operand_ai)?;
                                if let Some(const_val) = const_val {
                                    let var_desc =
                                        self.get_local_mut(local_index as usize).unwrap();
                                    if var_desc.is_const() {
                                        var_desc.assignment = Some(const_val);
                                        if self.chain_drop(operand_ci)? {
                                            self.replace_nop(base)?;
                                        }
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

                        Op::Cast => {
                            let target = self.array_index(CodeIndex(self.param(base, len, 2)?))?;
                            if let Some(const_val) = self.get_const(target)? {
                                let result = CodeIndex(self.param(base, len, 1)?);
                                let new_type_id = self.param(base, len, 3)?;
                                let old_type_id = self.param(base, len, 4)?;

                                let new_type = Primitive::from_type_id(new_type_id).ok_or(
                                    OptimizeError::TypeCastError(target.as_usize(), new_type_id),
                                )?;
                                let old_type = Primitive::from_type_id(old_type_id).ok_or(
                                    OptimizeError::TypeCastError(target.as_usize(), old_type_id),
                                )?;

                                opt_cast(
                                    self, old_type, new_type, const_val, base, target, result,
                                )?;
                            }
                        }

                        Op::Nop
                        | Op::UnaryNop
                        | Op::DropRight
                        | Op::Drop2
                        | Op::Block
                        | Op::Call
                        | Op::CallV
                        | Op::Dec
                        | Op::End
                        | Op::F32Const
                        | Op::F64Const
                        | Op::I32Const
                        | Op::I64Const
                        | Op::Inc
                        | Op::Loop
                        | Op::Not
                        | Op::Neg => {}
                    }
                }
            }

            for _ in 0..2 {
                // Reduce unnecessary blocks
                let mut block_freqs = BTreeMap::new();
                {
                    let mut last_block = None;
                    let mut block_empty_check = false;
                    let mut skip_until_end = false;
                    let mut ci = 0;
                    let mut block_stack = Vec::new();
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

                        if skip_until_end {
                            if opcode != Op::End {
                                self.replace_nop(base)?;
                            } else {
                                skip_until_end = false;
                            }
                            continue;
                        }

                        match opcode {
                            // Op::Nop => {}
                            Op::Block => {
                                let block_index = self.param(base, len, 1)?;
                                block_stack.push(block_index << 1);
                                block_freqs.insert(block_index, 0usize);
                                last_block = Some(block_index);
                                block_empty_check = true;
                            }
                            Op::Loop => {
                                let block_index = self.param(base, len, 1)?;
                                block_stack.push(block_index << 1 | 1);
                                block_freqs.insert(block_index, 0usize);
                            }
                            Op::End => {
                                let block_index = self.param(base, len, 1)?;
                                block_stack.pop().ok_or(OptimizeError::OutOfBlock(
                                    base.as_usize(),
                                    block_index,
                                ))?;
                            }
                            Op::Br => {
                                let block_index = self.param(base, len, 1)?;
                                let last_block = block_stack.last().ok_or(
                                    OptimizeError::InvalidBranch(base.as_usize(), block_index),
                                )?;
                                if *last_block == (block_index << 1) {
                                    self.replace_nop(base)?;
                                    skip_until_end = true;
                                } else {
                                    *block_freqs.get_mut(&block_index).ok_or(
                                        OptimizeError::InvalidBranch(base.as_usize(), block_index),
                                    )? += 1;
                                }
                            }
                            Op::BrIf => {
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
                                if *block_freqs.get(&block_index).ok_or(
                                    OptimizeError::InvalidBranch(base.as_usize(), block_index),
                                )? == 0
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

            // count local vars
            for item in self.params.iter_mut() {
                item.read_count = 0;
                item.write_count = 0;
            }
            for item in self.locals.iter_mut() {
                item.read_count = 0;
                item.write_count = 0;
            }

            let mut ci = 0;
            while let Ok(base) = self.array_index(CodeIndex(ci)) {
                ci += 1;
                let (len, opcode) = self.get_op(base)?;

                match opcode {
                    Op::LocalGet => {
                        let local_index = self.param(base, len, 2)?;
                        let var_desc = self.get_local_mut(local_index as usize).unwrap();
                        var_desc.read_count += 1;
                    }

                    Op::LocalSet => {
                        let local_index = self.param(base, len, 2)?;
                        let var_desc = self.get_local_mut(local_index as usize).unwrap();
                        var_desc.write_count += 1;
                    }

                    Op::LocalTee => {
                        let local_index = self.param(base, len, 2)?;
                        let var_desc = self.get_local_mut(local_index as usize).unwrap();
                        var_desc.read_count += 1;
                        var_desc.write_count += 1;
                    }

                    _ => {}
                }
            }

            // Mark local vars to be removed in the next turn.
            will_remove_vars.clear();
            for item in self.locals.iter() {
                if item.identifier().unwrap_or_default().starts_with("_") || item.read_count > 0 {
                } else {
                    will_remove_vars.push(item.index());
                }
            }
        }

        if true {
            // rename local vars
            let mut var_rename_table = BTreeMap::new();
            let mut new_locals = Vec::with_capacity(self.locals.len());
            for (index, _) in self.params.iter().enumerate() {
                var_rename_table.insert(index as u32, index as u32);
            }
            let index_base = self.params.len();

            let mut old_locals = Vec::new();
            core::mem::swap(&mut self.locals, &mut old_locals);

            // Sort by type, since wasm can generate code more efficiently by grouping variables of the same type together.
            old_locals.sort_by(|a, b| {
                let a = a.primitive_type().storage_type();
                let b = b.primitive_type().storage_type();
                match a.is_float().cmp(&b.is_float()) {
                    core::cmp::Ordering::Equal => a.bits_of().cmp(&b.bits_of()),
                    ord => ord,
                }
            });

            for mut item in old_locals.into_iter() {
                if item.read_count > 0 || item.write_count > 0 {
                    let old_index = item.index().as_u32();
                    let new_index = (index_base + new_locals.len()) as u32;
                    item.set_index(unsafe { LocalIndex::new(new_index) });
                    var_rename_table.insert(old_index, new_index);
                    new_locals.push(item);
                }
            }
            new_locals.shrink_to_fit();
            self.locals = new_locals;

            // compaction & renumber

            fn copy<F, R>(
                array: &[u32],
                base: ArrayIndex,
                dest: &mut Vec<u32>,
                kernel: F,
            ) -> Result<R, OptimizeError>
            where
                F: FnOnce(&mut [u32]) -> Result<R, OptimizeError>,
            {
                let si = base.as_usize();
                let len_opc = *array.get(si).ok_or(OptimizeError::OutOfCodes(si))?;
                let len = (len_opc >> 16) as usize;
                //let opcode = unsafe { transmute::<_, Op>((len_opc & 0xFFFF) as u8) };

                dest.push(len_opc);
                let di = dest.len();
                for i in 1..len {
                    dest.push(*array.get(si + i).unwrap());
                }
                let data = dest.get_mut(di..di + len - 1).unwrap();
                let result = kernel(data)?;

                Ok(result)
            }

            fn emit(array: &mut Vec<u32>, opcode: Op, params: &[u32]) {
                let len = params.len() + 1;
                let len_opc = ((len as u32) << 16) | (opcode as u32);
                array.push(len_opc);
                array.extend_from_slice(params);
            }

            let mut new_code = Vec::with_capacity(self.codes.len());
            let mut new_positions = Vec::with_capacity(self.positions.len());
            let mut block_stack = RenamingStack::new();
            let mut value_stack = RenamingStack::new();
            let mut ci = 0;
            let mut new_ci = 0;
            {
                let new_code = &mut new_code;
                while let Ok(base) = self.array_index(CodeIndex(ci)) {
                    ci += 1;
                    let (len, opcode) = self.get_op(base)?;
                    let mut dai = new_code.len();
                    match opcode {
                        Op::Nop => continue,

                        Op::Unreachable => {
                            copy(&self.codes, base, new_code, |_| Ok(()))?;
                        }

                        Op::Return => {
                            copy(&self.codes, base, new_code, |a| {
                                for item in a.iter_mut() {
                                    let new_value = value_stack.expect(base, *item)?;
                                    *item = new_value;
                                }
                                Ok(())
                            })?;
                        }

                        Op::Call | Op::CallV => {
                            let target = copy(&self.codes, base, new_code, |a| {
                                for item in a.iter_mut().skip(2).rev() {
                                    let old_value = *item;
                                    let new_value = value_stack.expect(base, old_value)?;
                                    *item = new_value;
                                }
                                if opcode == Op::Call {
                                    value_stack.push(a[0], new_ci);
                                }
                                a[0] = new_ci;

                                Ok(a[1])
                            })?;
                            self.add_dependency_list(FuncTempIndex::new(target));
                        }

                        Op::Drop => {
                            copy(&self.codes, base, new_code, |a| {
                                let operand = value_stack.expect(base, a[0])?;
                                a[0] = operand;
                                Ok(())
                            })?;
                        }

                        Op::Block | Op::Loop => {
                            copy(&self.codes, base, new_code, |a| {
                                let result = new_ci;
                                block_stack.push(a[0], result);
                                a[0] = result;
                                Ok(())
                            })?;
                        }

                        Op::End => {
                            copy(&self.codes, base, new_code, |a| {
                                let operand = block_stack.expect(base, a[0])?;
                                a[0] = operand;
                                Ok(())
                            })?;
                        }

                        Op::Br => {
                            copy(&self.codes, base, new_code, |a| {
                                let target = block_stack.lookup(base, a[0])?;
                                a[0] = target;
                                Ok(())
                            })?;
                        }

                        Op::BrIf => {
                            copy(&self.codes, base, new_code, |a| {
                                let target = block_stack.lookup(base, a[0])?;
                                let operand = value_stack.expect(base, a[1])?;
                                a[0] = target;
                                a[1] = operand;
                                Ok(())
                            })?;
                        }

                        Op::F32Const | Op::F64Const | Op::I32Const | Op::I64Const => {
                            copy(&self.codes, base, new_code, |a| {
                                let result = new_ci;
                                value_stack.push(a[0], result);
                                a[0] = result;
                                Ok(())
                            })?;
                        }

                        Op::LocalGet => {
                            copy(&self.codes, base, new_code, |a| {
                                let result = new_ci;
                                value_stack.push(a[0], result);
                                a[0] = result;
                                a[1] = *var_rename_table.get(&a[1]).unwrap();
                                Ok(())
                            })?;
                        }

                        Op::LocalSet => {
                            copy(&self.codes, base, new_code, |a| {
                                let operand = value_stack.expect(base, a[0])?;
                                a[0] = operand;
                                a[1] = *var_rename_table.get(&a[1]).unwrap();
                                Ok(())
                            })?;
                        }

                        Op::LocalTee => {
                            copy(&self.codes, base, new_code, |a| {
                                let operand = value_stack.expect(base, a[2])?;
                                let result = new_ci;
                                value_stack.push(a[0], result);
                                a[0] = result;
                                a[1] = *var_rename_table.get(&a[1]).unwrap();
                                a[2] = operand;
                                Ok(())
                            })?;
                        }

                        // unary
                        // %n = op %n
                        Op::Cast | Op::Dec | Op::Eqz | Op::Inc | Op::Not | Op::Neg => {
                            copy(&self.codes, base, new_code, |a| {
                                let operand = value_stack.expect(base, a[1])?;
                                let result = new_ci;
                                value_stack.push(a[0], result);
                                a[0] = result;
                                a[1] = operand;
                                Ok(())
                            })?;
                        }

                        // binary
                        // %n = op %n, %n
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
                        | Op::Shl
                        | Op::ShrS
                        | Op::ShrU
                        | Op::Sub
                        | Op::Xor => {
                            copy(&self.codes, base, new_code, |a| {
                                let rhs = value_stack.expect(base, a[2])?;
                                let lhs = value_stack.expect(base, a[1])?;
                                let result = new_ci;
                                value_stack.push(a[0], result);
                                a[0] = result;
                                a[1] = lhs;
                                a[2] = rhs;
                                Ok(())
                            })?;
                        }

                        Op::UnaryNop => {
                            let result = self.param(base, len, 1)?;
                            let operand = self.param(base, len, 2)?;
                            value_stack.expect(base, operand)?;
                            value_stack.push(result, new_ci);
                        }

                        Op::DropRight => {
                            let old_result = self.param(base, len, 1)?;
                            let lhs = self.param(base, len, 2)?;
                            let rhs = self.param(base, len, 3)?;

                            let rhs = value_stack.expect(base, rhs)?;
                            let lhs = value_stack.expect(base, lhs)?;
                            value_stack.push(old_result, lhs);

                            emit(new_code, Op::Drop, &[rhs]);
                        }

                        Op::Drop2 => {
                            let lhs = self.param(base, len, 2)?;
                            let rhs = self.param(base, len, 3)?;

                            let rhs = value_stack.expect(base, rhs)?;
                            let lhs = value_stack.expect(base, lhs)?;

                            emit(new_code, Op::Drop, &[rhs]);

                            new_positions.push(ArrayIndex(dai as u32));
                            new_ci += 1;
                            dai = new_code.len();

                            emit(new_code, Op::Drop, &[lhs]);
                        }
                    }

                    new_positions.push(ArrayIndex(dai as u32));
                    new_ci += 1;
                }
            }

            if block_stack.len() > 0 {
                return Err(OptimizeError::InvalidBlockStack(block_stack.len()));
            }
            if value_stack.len() > 0 {
                return Err(OptimizeError::InvalidValueStack(value_stack.len()));
            }
            new_code.shrink_to_fit();
            new_positions.shrink_to_fit();

            self.codes = new_code;
            self.positions = new_positions;
        }

        Ok(())
    }

    #[inline]
    fn add_dependency_list(&mut self, target: FuncTempIndex) {
        if !self.call_dependency_list.contains(&target) {
            self.call_dependency_list.push(target);
        }
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
            Op::UnaryNop | Op::Cast | Op::Dec | Op::Eqz | Op::Inc | Op::Not | Op::Neg => {
                let operand2 = self.param(base, len, 2)?;
                if self.chain_drop(CodeIndex(operand2))? {
                    self.replace_nop(base)?;
                } else {
                    self.replace(base, Op::Drop, &[operand2])?;
                }
                return Ok(true);
            }

            Op::CallV
            | Op::Drop2
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

                    _ => return Ok(false),
                };
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_i64_const(base, result, value)?;
                return Ok(true);
            }
            (Some(Constant::F32(lhs)), Some(Constant::F32(rhs))) => {
                let mut value = match opcode {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::DivS => lhs / rhs,
                    _ => return Ok(false),
                };
                if value.is_nan() {
                    // Correct for different NaN bit patterns on some platforms
                    value = f32::from_bits(0x7FC0_0000);
                }
                self.replace_nop(lhs_a)?;
                self.replace_nop(rhs_a)?;
                self.replace_f32_const(base, result, value)?;
                return Ok(true);
            }
            (Some(Constant::F64(lhs)), Some(Constant::F64(rhs))) => {
                let mut value = match opcode {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::DivS => lhs / rhs,
                    _ => return Ok(false),
                };
                if value.is_nan() {
                    // Correct for different NaN bit patterns on some platforms
                    value = f64::from_bits(0x7FF8_0000_0000_0000);
                }
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
                    Op::Add | Op::Sub | Op::Or | Op::Xor | Op::Shl | Op::ShrS | Op::ShrU => {
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
                    | Op::ShrU => {
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
                    Op::Add | Op::Sub | Op::Or | Op::Xor | Op::Shl | Op::ShrS | Op::ShrU => {
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

        match opcode {
            Op::Add | Op::Mul | Op::And | Op::Or | Op::Xor => {
                if let Some(Constant::I32(rhs_const)) = rhs_const {
                    let (len2, op2) = self.get_op(lhs_a)?;
                    if opcode == op2 {
                        let lhs2 = CodeIndex(self.param(lhs_a, len2, 2)?);
                        let rhs2 = CodeIndex(self.param(lhs_a, len2, 3)?);

                        let lhs_a2 = self.array_index(lhs2)?;
                        let lhs_const2 = self.get_const(lhs_a2)?;

                        if let Some(Constant::I32(lhs_const2)) = lhs_const2 {
                            let value = match opcode {
                                Op::Add => rhs_const.wrapping_add(lhs_const2),
                                Op::Mul => rhs_const.wrapping_mul(lhs_const2),
                                Op::And => rhs_const & lhs_const2,
                                Op::Or => rhs_const | lhs_const2,
                                Op::Xor => rhs_const ^ lhs_const2,
                                _ => unreachable!(),
                            };

                            self.replace_nop(lhs_a2)?;
                            self.replace_nop(lhs_a)?;
                            self.replace_i32_const(rhs_a, rhs, value)?;
                            self.replace(base, opcode, &[result.0, rhs2.0, rhs.0])?;
                            return Ok(true);
                        }

                        let rhs_a2 = self.array_index(rhs2)?;
                        let rhs_const2 = self.get_const(rhs_a2)?;

                        if let Some(Constant::I32(rhs_const2)) = rhs_const2 {
                            let value = match opcode {
                                Op::Add => rhs_const.wrapping_add(rhs_const2),
                                Op::Mul => rhs_const.wrapping_mul(rhs_const2),
                                Op::And => rhs_const & rhs_const2,
                                Op::Or => rhs_const | rhs_const2,
                                Op::Xor => rhs_const ^ rhs_const2,
                                _ => unreachable!(),
                            };

                            self.replace_nop(rhs_a2)?;
                            self.replace_nop(lhs_a)?;
                            self.replace_i32_const(rhs_a, rhs, value)?;
                            self.replace(base, opcode, &[result.0, lhs2.0, rhs.0])?;
                            return Ok(true);
                        }
                    }
                }
            }
            _ => {}
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
        let Some(p) = self.codes.get_mut(target.as_usize()) else {
            return Err(OptimizeError::OutOfCodes(target.as_usize()));
        };
        let len_opc = *p;
        *p = (len_opc & 0xFFFF_0000) | (opcode as u32);
        Ok(())
    }

    #[inline]
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

struct RenamingStack(Vec<(u32, u32)>);

impl RenamingStack {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, old_value: u32, new_value: u32) {
        self.0.push((old_value, new_value))
    }

    pub fn expect(&mut self, base: ArrayIndex, expected_value: u32) -> Result<u32, OptimizeError> {
        let (old_value, new_value) = self
            .0
            .pop()
            .ok_or(OptimizeError::OutOfStack(base.as_usize(), self.0.len()))?;

        if old_value != expected_value {
            return Err(OptimizeError::RenameError(
                base.as_usize(),
                old_value as usize,
                expected_value as usize,
            ));
        }

        Ok(new_value)
    }

    pub fn lookup(&self, base: ArrayIndex, expected_value: u32) -> Result<u32, OptimizeError> {
        for item in &self.0 {
            if item.0 == expected_value {
                return Ok(item.1);
            }
        }

        Err(OptimizeError::InvalidBranch(
            base.as_usize(),
            expected_value,
        ))
    }
}
