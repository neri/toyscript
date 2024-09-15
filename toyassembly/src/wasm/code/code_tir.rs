//! ToyIr to WASM Assembler

use super::{BlockInstType, Code};
use crate::*;
use ir::Module;
use leb128::{Leb128Writer, WriteLeb128};
use toyir::CodeStreamIter;
use types::ValType;
use wasm::opcode::WasmOpcode;

use toyir::Op as TIR;

pub(super) struct FromToyIR;

impl FromToyIR {
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
                TIR::Nop => {}

                TIR::Unreachable => {
                    writer.write(WasmOpcode::Unreachable).unwrap();
                }

                TIR::Block | TIR::Loop => {
                    let inst_type = match opcode {
                        TIR::Block => BlockInstType::Block,
                        TIR::Loop => BlockInstType::Loop,
                        // TIR::If => BlockInstType::If,
                        _ => unreachable!(),
                    };
                    let mnemonic = inst_type.as_wasm();

                    let label = BlockIndex(Self::get_params(tir.params(), 0)?);

                    block_stack.push(label, BlockStackEntry { inst_type })?;

                    writer.write(mnemonic).unwrap();
                    writer.write_byte(0x40).unwrap();
                }

                TIR::End => match block_stack.pop() {
                    Some(_) => writer.write(WasmOpcode::End).unwrap(),
                    None => {
                        return Err(AssembleError::out_of_bounds(
                            "Too many 'end'",
                            ErrorPosition::Unspecified,
                        ))
                    }
                },

                TIR::Br => {
                    let index = BlockIndex(Self::get_params(tir.params(), 0)?);
                    let target = block_stack.solve(index)?;
                    writer.write(WasmOpcode::Br).unwrap();
                    writer.write(target).unwrap();
                }

                TIR::BrIf => {
                    let index = BlockIndex(Self::get_params(tir.params(), 0)?);
                    let ssa_index = Self::get_params(tir.params(), 1)?;
                    let target = block_stack.solve(index)?;
                    value_stack.expect_type(ssa_index, ValType::I32)?;

                    writer.write(WasmOpcode::BrIf).unwrap();
                    writer.write(target).unwrap();
                }

                TIR::Call => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let target = Self::get_params(tir.params(), 1)?;

                    let target_func = module.get_func_by_index(target).ok_or(
                        AssembleError::internal_inconsistency(
                            &format!("bad function ${}", target),
                            ErrorPosition::Unspecified,
                        ),
                    )?;
                    let target_type = module.get_type(target_func.0);
                    if target_type.params().len() != tir.params().len() - 2 {
                        return Err(AssembleError::internal_inconsistency(
                            &format!(
                                "this function takes {} arguments but {} arguments was supplied",
                                target_type.params().len(),
                                tir.params().len() - 2,
                            ),
                            ErrorPosition::Unspecified,
                        ));
                    }

                    let mut index = tir.params().len() - 1;
                    for param in target_type.params().iter().rev() {
                        let arg = Self::get_params(tir.params(), index)?;
                        value_stack.expect_type(arg, *param)?;
                        index -= 1;
                    }
                    if let Some(result_type) = target_type.results().iter().next() {
                        value_stack.push(result, *result_type);
                    }

                    writer.write(WasmOpcode::Call).unwrap();
                    writer.write(target).unwrap();
                }

                TIR::Drop => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    value_stack.expect(ssa_index)?;
                    writer.write(WasmOpcode::Drop).unwrap();
                }

                TIR::Return => {
                    if let Some(result_type) = results.iter().next() {
                        let ssa_index = Self::get_params(tir.params(), 0)?;
                        value_stack.expect_type(ssa_index, *result_type)?;
                    }

                    writer.write(WasmOpcode::Return).unwrap();
                }

                TIR::I32Const => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let const_val = Self::get_params(tir.params(), 1)?;
                    value_stack.push(ssa_index, ValType::I32);

                    writer.write(WasmOpcode::I32Const).unwrap();
                    writer.write(const_val as i32).unwrap();
                }

                TIR::I64Const => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let const_low = Self::get_params(tir.params(), 1)?;
                    let const_hi = Self::get_params(tir.params(), 2)?;
                    let const_val = ((const_hi as u64) << 32) | (const_low as u64);
                    value_stack.push(ssa_index, ValType::I64);

                    writer.write(WasmOpcode::I64Const).unwrap();
                    writer.write(const_val as i64).unwrap();
                }
                TIR::F32Const => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let const_val = Self::get_params(tir.params(), 1)?;
                    value_stack.push(ssa_index, ValType::F64);

                    writer.write(WasmOpcode::F32Const).unwrap();
                    writer.write(f32::from_bits(const_val)).unwrap();
                }
                TIR::F64Const => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let const_low = Self::get_params(tir.params(), 1)?;
                    let const_hi = Self::get_params(tir.params(), 2)?;
                    let const_val = ((const_hi as u64) << 32) | (const_low as u64);
                    value_stack.push(ssa_index, ValType::F64);

                    writer.write(WasmOpcode::F64Const).unwrap();
                    writer.write(f64::from_bits(const_val)).unwrap();
                }

                TIR::LocalGet => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let local_idx = Self::get_params(tir.params(), 1)?;
                    let local_type = *local_and_params.get(local_idx as usize).unwrap();
                    value_stack.push(ssa_index, local_type);

                    writer.write(WasmOpcode::LocalGet).unwrap();
                    writer.write(local_idx as u32).unwrap();
                }
                TIR::LocalSet => {
                    let ssa_index = Self::get_params(tir.params(), 0)?;
                    let local_idx = Self::get_params(tir.params(), 1)?;
                    let local_type = *local_and_params.get(local_idx as usize).unwrap();
                    value_stack.expect_type(ssa_index, local_type)?;

                    writer.write(WasmOpcode::LocalSet).unwrap();
                    writer.write(local_idx as u32).unwrap();
                }
                TIR::LocalTee => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let local_idx = Self::get_params(tir.params(), 1)?;
                    let operand = Self::get_params(tir.params(), 2)?;
                    let local_type = *local_and_params.get(local_idx as usize).unwrap();
                    value_stack.expect_type(operand, local_type)?;
                    value_stack.push(result, local_type);

                    writer.write(WasmOpcode::LocalTee).unwrap();
                    writer.write(local_idx as u32).unwrap();
                }

                // binop
                TIR::Add
                | TIR::Sub
                | TIR::Mul
                | TIR::DivS
                | TIR::DivU
                | TIR::RemS
                | TIR::RemU
                | TIR::Shl
                | TIR::ShrS
                | TIR::ShrU
                | TIR::And
                | TIR::Or
                | TIR::Xor
                | TIR::Rotl
                | TIR::Rotr => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let lhs_i = Self::get_params(tir.params(), 1)?;
                    let rhs_i = Self::get_params(tir.params(), 2)?;

                    let rhs_t = value_stack.expect(rhs_i)?;
                    let lhs_t = value_stack.expect_type(lhs_i, rhs_t)?;
                    value_stack.push(result, lhs_t);

                    let mnemonic = WasmOpcode::from_tir(opcode, lhs_t).ok_or(
                        AssembleError::internal_inconsistency(
                            &format!("invalid operator '{}' for type '{}'", opcode, lhs_t),
                            ErrorPosition::Unspecified,
                        ),
                    )?;
                    writer.write(mnemonic).unwrap();
                }

                TIR::DropRight => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let lhs_i = Self::get_params(tir.params(), 1)?;
                    let rhs_i = Self::get_params(tir.params(), 2)?;

                    let _rhs_t = value_stack.expect(rhs_i)?;
                    let lhs_t = value_stack.expect(lhs_i)?;
                    value_stack.push(result, lhs_t);

                    writer.write(WasmOpcode::Drop).unwrap();
                }

                TIR::Drop2 => {
                    let lhs_i = Self::get_params(tir.params(), 1)?;
                    let rhs_i = Self::get_params(tir.params(), 2)?;
                    let _rhs_t = value_stack.expect(rhs_i)?;
                    let _lhs_t = value_stack.expect(lhs_i)?;

                    writer.write(WasmOpcode::Drop).unwrap();
                    writer.write(WasmOpcode::Drop).unwrap();
                }

                // cmp
                TIR::Eq
                | TIR::Ne
                | TIR::LtS
                | TIR::LtU
                | TIR::GtS
                | TIR::GtU
                | TIR::LeS
                | TIR::LeU
                | TIR::GeS
                | TIR::GeU => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let lhs_i = Self::get_params(tir.params(), 1)?;
                    let rhs_i = Self::get_params(tir.params(), 2)?;

                    let rhs_t = value_stack.expect(rhs_i)?;
                    let lhs_t = value_stack.expect_type(lhs_i, rhs_t)?;
                    value_stack.push(result, ValType::I32);

                    let mnemonic = WasmOpcode::from_tir(opcode, lhs_t).ok_or(
                        AssembleError::internal_inconsistency(
                            &format!("invalid operator '{}' for type '{}'", opcode, lhs_t),
                            ErrorPosition::Unspecified,
                        ),
                    )?;
                    writer.write(mnemonic).unwrap();
                }

                // unop
                TIR::Eqz | TIR::Clz | TIR::Ctz | TIR::Popcnt | TIR::Neg => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let operand = Self::get_params(tir.params(), 1)?;
                    let val_type = value_stack.expect(operand)?;
                    value_stack.push(result, val_type);

                    let mnemonic = WasmOpcode::from_tir(opcode, val_type).ok_or(
                        AssembleError::internal_inconsistency(
                            &format!("invalid operator '{}' for type '{}'", opcode, val_type),
                            ErrorPosition::Unspecified,
                        ),
                    )?;
                    writer.write(mnemonic).unwrap();
                }

                // special unop
                TIR::UnaryNop | TIR::Not | TIR::Inc | TIR::Dec => {
                    let result = Self::get_params(tir.params(), 0)?;
                    let operand = Self::get_params(tir.params(), 1)?;
                    let val_type = value_stack.expect(operand)?;
                    value_stack.push(result, val_type);

                    match (opcode, val_type) {
                        (TIR::UnaryNop, _) => {}

                        (TIR::Inc, ValType::I32) => {
                            writer.write(WasmOpcode::I32Const).unwrap();
                            writer.write(1).unwrap();
                            writer.write(WasmOpcode::I32Add).unwrap();
                        }
                        (TIR::Dec, ValType::I32) => {
                            writer.write(WasmOpcode::I32Const).unwrap();
                            writer.write(-1).unwrap();
                            writer.write(WasmOpcode::I32Add).unwrap();
                        }
                        (TIR::Not, ValType::I32) => {
                            writer.write(WasmOpcode::I32Const).unwrap();
                            writer.write(-1).unwrap();
                            writer.write(WasmOpcode::I32And).unwrap();
                        }

                        (TIR::Inc, ValType::I64) => {
                            writer.write(WasmOpcode::I64Const).unwrap();
                            writer.write(1).unwrap();
                            writer.write(WasmOpcode::I64Add).unwrap();
                        }
                        (TIR::Dec, ValType::I64) => {
                            writer.write(WasmOpcode::I64Const).unwrap();
                            writer.write(-1).unwrap();
                            writer.write(WasmOpcode::I64Add).unwrap();
                        }
                        (TIR::Not, ValType::I64) => {
                            writer.write(WasmOpcode::I64Const).unwrap();
                            writer.write(-1).unwrap();
                            writer.write(WasmOpcode::I64And).unwrap();
                        }

                        (TIR::Inc, ValType::F32) => {
                            writer.write(WasmOpcode::F32Const).unwrap();
                            writer.write(1.0f32).unwrap();
                            writer.write(WasmOpcode::F32Add).unwrap();
                        }
                        (TIR::Dec, ValType::F32) => {
                            writer.write(WasmOpcode::F32Const).unwrap();
                            writer.write(1.0f32).unwrap();
                            writer.write(WasmOpcode::F32Sub).unwrap();
                        }

                        (TIR::Inc, ValType::F64) => {
                            writer.write(WasmOpcode::F64Const).unwrap();
                            writer.write(1.0f64).unwrap();
                            writer.write(WasmOpcode::F64Add).unwrap();
                        }
                        (TIR::Dec, ValType::F64) => {
                            writer.write(WasmOpcode::F64Const).unwrap();
                            writer.write(1.0f64).unwrap();
                            writer.write(WasmOpcode::F64Sub).unwrap();
                        }

                        _ => {
                            return Err(AssembleError::internal_inconsistency(
                                &format!("invalid operator '{}' for type '{}'", opcode, val_type),
                                ErrorPosition::Unspecified,
                            ))
                        }
                    }
                }

                #[allow(unreachable_patterns)]
                _ => {
                    return Err(AssembleError::internal_inconsistency(
                        &format!("Unknown opcoe {}", opcode),
                        ErrorPosition::Unspecified,
                    ))
                }
            }
        }

        if block_stack.len() > 0 {
            return Err(AssembleError::internal_inconsistency(
                "End required",
                ErrorPosition::Unspecified,
            ));
        }
        if value_stack.len() > 0 {
            return Err(AssembleError::internal_inconsistency(
                "Value Stack level mismatch",
                ErrorPosition::Unspecified,
            ));
        }

        writer.write(WasmOpcode::End).unwrap();

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

struct BlockStackEntry {
    #[allow(unused)]
    inst_type: BlockInstType,
    // stack_level: usize,
}

#[derive(Default)]
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
    pub fn len(&self) -> usize {
        self.0.len()
    }

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
