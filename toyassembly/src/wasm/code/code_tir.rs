//! ToyIr to WASM Assembler

use super::{BlockInstType, Code};
use crate::*;
use ir::Module;
use leb128::{Leb128Writer, WriteLeb128};
use toyir::CodeStreamIter;
use types::ValType;
use wasm::opcode::WasmOpcode;

use toyir::Op as TOP;

pub(super) struct TirToWasm;

impl TirToWasm {
    /// Perform assembly from toyir
    pub fn assemble(
        src: &[u32],
        module: &Module,
        results: &[ValType],
        locals: &[ValType],
        local_and_params: &[ValType],
    ) -> Result<Vec<u8>, AssembleError> {
        let mut writer = Leb128Writer::new();

        Code::assemble_locals(locals, &mut writer);

        let mut value_stack = ValueStack::default();
        let mut block_stack = BlockStack::default();

        for tir in CodeStreamIter::new(src) {
            let opcode = tir.opcode();
            match opcode {
                TOP::Nop => {}

                TOP::Unreachable => {
                    writer
                        .write_byte(WasmOpcode::Unreachable.leading_byte())
                        .unwrap();
                }

                TOP::Block | TOP::Loop => {
                    let inst_type = match opcode {
                        TOP::Block => BlockInstType::Block,
                        TOP::Loop => BlockInstType::Loop,
                        // TOP::If => BlockInstType::If,
                        _ => unreachable!(),
                    };
                    let mnemonic = inst_type.as_wasm();

                    let label = BlockIndex(Self::get_params(tir.params(), 0)?);

                    block_stack.push(label, BlockStackEntry { inst_type })?;

                    writer.write_byte(mnemonic.leading_byte()).unwrap();
                    writer.write_byte(0x40).unwrap();
                }

                TOP::End => match block_stack.pop() {
                    Some(_) => writer.write_byte(WasmOpcode::End.leading_byte()).unwrap(),
                    None => {
                        return Err(AssembleError::out_of_bounds(
                            "Too many 'end'",
                            ErrorPosition::Unspecified,
                        ))
                    }
                },

                TOP::Br => {
                    let index = BlockIndex(Self::get_params(tir.params(), 0)?);
                    let target = block_stack.solve(index)?;
                    writer.write_byte(WasmOpcode::Br.leading_byte()).unwrap();
                    writer.write(target).unwrap();
                }

                TOP::BrIf => {
                    // TODO: stack check
                    let index = BlockIndex(Self::get_params(tir.params(), 0)?);
                    let ssa_index = Self::get_params(tir.params(), 1)?;
                    let target = block_stack.solve(index)?;
                    value_stack.expect_type(ssa_index, ValType::I32)?;

                    writer.write_byte(WasmOpcode::BrIf.leading_byte()).unwrap();
                    writer.write(target).unwrap();
                }

                TOP::Call => {
                    todo!()
                }

                TOP::UnaryNop => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let operand = Self::get_params(tir.params(), 1)?;
                    let val_type = value_stack.expect(operand)?;
                    value_stack.push(result, val_type);
                }

                TOP::Drop => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    value_stack.expect(ssa_index)?;
                    writer.write_byte(WasmOpcode::Drop.leading_byte()).unwrap();
                }

                TOP::Return => {
                    if let Some(result_type) = results.iter().next() {
                        let ssa_index = Self::get_params(tir.params(), 0)?;
                        value_stack.expect_type(ssa_index, *result_type)?;
                    }

                    writer
                        .write_byte(WasmOpcode::Return.leading_byte())
                        .unwrap();
                }

                TOP::I32Const => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let const_val = Self::get_params(tir.params(), 1)?;
                    value_stack.push(ssa_index, ValType::I32);

                    writer
                        .write_byte(WasmOpcode::I32Const.leading_byte())
                        .unwrap();
                    writer.write(const_val as i32).unwrap();
                }

                TOP::I64Const => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let const_low = Self::get_params(tir.params(), 1)?;
                    let const_hi = Self::get_params(tir.params(), 2)?;
                    let const_val = ((const_hi as u64) << 32) | (const_low as u64);
                    value_stack.push(ssa_index, ValType::I64);

                    writer
                        .write_byte(WasmOpcode::I64Const.leading_byte())
                        .unwrap();
                    writer.write(const_val as i64).unwrap();
                }
                // TOP::F32Const => todo!(),
                // TOP::F64Const => todo!(),
                //
                TOP::LocalGet => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let local_idx = Self::get_params(tir.params(), 1)?;
                    let local_type = *local_and_params.get(local_idx as usize).unwrap();
                    value_stack.push(ssa_index, local_type);

                    writer
                        .write_byte(WasmOpcode::LocalGet.leading_byte())
                        .unwrap();
                    writer.write(local_idx as i32).unwrap();
                }
                TOP::LocalSet => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let local_idx = Self::get_params(tir.params(), 1)?;
                    let local_type = *local_and_params.get(local_idx as usize).unwrap();
                    value_stack.expect_type(ssa_index, local_type)?;

                    writer
                        .write_byte(WasmOpcode::LocalSet.leading_byte())
                        .unwrap();
                    writer.write(local_idx as i32).unwrap();
                }
                TOP::LocalTee => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let local_idx = Self::get_params(tir.params(), 1)?;
                    let operand = Self::get_params(tir.params(), 2)?;
                    let local_type = *local_and_params.get(local_idx as usize).unwrap();
                    value_stack.expect_type(operand, local_type)?;
                    value_stack.push(result, local_type);

                    writer
                        .write_byte(WasmOpcode::LocalTee.leading_byte())
                        .unwrap();
                    writer.write(local_idx as i32).unwrap();
                }

                // binop
                TOP::Add
                | TOP::Sub
                | TOP::Mul
                | TOP::DivS
                | TOP::DivU
                | TOP::RemS
                | TOP::RemU
                | TOP::Shl
                | TOP::ShrS
                | TOP::ShrU
                | TOP::And
                | TOP::Or
                | TOP::Xor
                | TOP::Rotl
                | TOP::Rotr => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let lhs_i = Self::get_params(tir.params(), 1)?;
                    let rhs_i = Self::get_params(tir.params(), 2)?;

                    let rhs_t = value_stack.expect(rhs_i)?;
                    let lhs_t = value_stack.expect_type(lhs_i, rhs_t)?;
                    value_stack.push(result, lhs_t);

                    writer
                        .write_byte(WasmOpcode::Unreachable.leading_byte())
                        .unwrap();
                }

                // cmp
                TOP::Eq
                | TOP::Ne
                | TOP::LtS
                | TOP::LtU
                | TOP::GtS
                | TOP::GtU
                | TOP::LeS
                | TOP::LeU
                | TOP::GeS
                | TOP::GeU => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let lhs_i = Self::get_params(tir.params(), 1)?;
                    let rhs_i = Self::get_params(tir.params(), 2)?;

                    let rhs_t = value_stack.expect(rhs_i)?;
                    let _lhs_t = value_stack.expect_type(lhs_i, rhs_t)?;
                    value_stack.push(result, ValType::I32);

                    writer
                        .write_byte(WasmOpcode::Unreachable.leading_byte())
                        .unwrap();
                }

                // unop
                TOP::Eqz | TOP::Clz | TOP::Ctz | TOP::Popcnt | TOP::Not | TOP::Inc | TOP::Dec => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let operand = Self::get_params(tir.params(), 1)?;
                    let val_type = value_stack.expect(operand)?;
                    value_stack.push(result, val_type);

                    writer
                        .write_byte(WasmOpcode::Unreachable.leading_byte())
                        .unwrap();
                }

                _ => {
                    // return Err(AssembleError::internal_inconsistency(
                    //     &format!("Unknown opcoe {}", opcode),
                    //     ErrorPosition::Unspecified,
                    // ))
                }
            }
        }

        Ok(writer.into_vec())
    }

    fn get_params(slice: &[u32], index: usize) -> Result<u32, AssembleError> {
        slice
            .get(index)
            .map(|v| *v)
            .ok_or(AssembleError::internal_inconsistency(
                &format!("invalid index of {}", index),
                ErrorPosition::Unspecified,
            ))
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct BlockIndex(u32);

#[derive(Debug)]
struct BlockStackEntry {
    inst_type: BlockInstType,
    // stack_level: usize,
}

#[derive(Debug, Default)]
struct BlockStack {
    items: Vec<BlockStackEntry>,
    labels: Vec<BlockIndex>,
}

impl BlockStack {
    pub fn push(&mut self, label: BlockIndex, value: BlockStackEntry) -> Result<(), AssembleError> {
        if self.labels.contains(&label) {
            return Err(AssembleError::internal_inconsistency(
                &format!("duplicate block $label_{}", label.0),
                ErrorPosition::Unspecified,
            ));
        }
        self.items.push(value);
        self.labels.push(label);
        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<BlockStackEntry> {
        self.items.pop().map(|v| {
            let _ = self.labels.pop();
            v
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn solve(&self, label: BlockIndex) -> Result<u32, AssembleError> {
        for (index, target) in self.labels.iter().rev().enumerate() {
            if *target == label {
                return Ok(index as u32);
            }
        }
        Err(AssembleError::undefined_identifier(
            &format!("$label_{}", label.0),
            ErrorPosition::Unspecified,
        ))
    }
}

#[derive(Default)]
struct ValueStack(Vec<ValueStackEntry>);

struct ValueStackEntry {
    ssa_index: u32,
    val_type: ValType,
}

impl ValueStack {
    #[inline]
    pub fn push(&mut self, ssa_index: u32, val_type: ValType) {
        self.0.push(ValueStackEntry {
            ssa_index,
            val_type,
        });
    }

    pub fn expect(&mut self, ssa_index: u32) -> Result<ValType, AssembleError> {
        if let Some(entry) = self.0.pop() {
            if entry.ssa_index == ssa_index {
                Ok(entry.val_type)
            } else {
                Err(AssembleError::internal_inconsistency(
                    &format!(
                        "Value stack LEVEL mismatch {} expected {}",
                        entry.ssa_index, ssa_index
                    ),
                    ErrorPosition::Unspecified,
                ))
            }
        } else {
            Err(AssembleError::internal_inconsistency(
                &format!("Out of value stack"),
                ErrorPosition::Unspecified,
            ))
        }
    }

    pub fn expect_type(
        &mut self,
        ssa_index: u32,
        val_type: ValType,
    ) -> Result<ValType, AssembleError> {
        self.expect(ssa_index).and_then(|v| {
            if v == val_type {
                Ok(val_type)
            } else {
                Err(AssembleError::internal_inconsistency(
                    &format!("Value stack TYPE mismatch {} expect {}", v, val_type),
                    ErrorPosition::Unspecified,
                ))
            }
        })
    }
}
