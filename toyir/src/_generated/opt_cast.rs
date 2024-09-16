//! Optimize constant casts

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

use super::*;

#[inline]
pub(super) fn opt_cast(
    opt: &mut MinimalCodeOptimizer,
    old_type: Primitive, 
    new_type: Primitive, 
    const_val: Constant,
    base: ArrayIndex, 
    target: ArrayIndex, 
    result: CodeIndex,
) -> Result<bool, OptimizeError> {
    match (old_type, new_type, const_val) {
        (Primitive::I8, Primitive::I8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i8) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::U8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i8) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::I16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i8) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::U16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i8) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::I32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i8) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::U32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i8) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::F32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as i8) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::I64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i8) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::U64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i8) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::I8, Primitive::F64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as i8) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::I8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u8) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::U8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u8) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::I16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u8) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::U16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u8) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::I32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u8) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::U32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u8) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::F32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as u8) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::I64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u8) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::U64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u8) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::U8, Primitive::F64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as u8) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::I8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i16) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::U8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i16) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::I16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i16) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::U16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i16) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::I32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i16) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::U32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i16) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::F32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as i16) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::I64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i16) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::U64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i16) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::I16, Primitive::F64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as i16) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::I8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u16) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::U8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u16) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::I16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u16) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::U16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u16) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::I32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u16) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::U32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u16) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::F32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as u16) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::I64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u16) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::U64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u16) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::U16, Primitive::F64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as u16) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::I8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i32) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::U8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i32) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::I16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i32) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::U16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i32) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::I32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i32) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::U32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i32) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::F32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as i32) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::I64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i32) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::U64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i32) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::I32, Primitive::F64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as i32) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::I8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u32) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::U8, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u32) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::I16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u32) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::U16, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u32) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::I32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u32) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::U32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u32) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::F32, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as u32) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::I64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u32) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::U64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u32) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::U32, Primitive::F64, Constant::I32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as u32) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::I8, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f32) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::U8, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f32) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::I16, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f32) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::U16, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f32) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::I32, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f32) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::U32, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f32) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::F32, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as f32) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::I64, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as f32) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::U64, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as f32) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::F32, Primitive::F64, Constant::F32(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as f32) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::I8, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i64) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::U8, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i64) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::I16, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i64) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::U16, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i64) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::I32, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i64) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::U32, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as i64) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::F32, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as i64) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::I64, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i64) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::U64, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as i64) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::I64, Primitive::F64, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as i64) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::I8, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u64) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::U8, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u64) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::I16, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u64) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::U16, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u64) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::I32, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u64) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::U32, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as u64) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::F32, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as u64) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::I64, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u64) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::U64, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as u64) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::U64, Primitive::F64, Constant::I64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as u64) as f64) as f64)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::I8, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f64) as i8) as i32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::U8, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f64) as u8) as i32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::I16, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f64) as i16) as i32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::U16, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f64) as u16) as i32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::I32, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f64) as i32) as i32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::U32, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i32_const(base, result, ((const_val as f64) as u32) as i32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::F32, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f32_const(base, result, ((const_val as f64) as f32) as f32)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::I64, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as f64) as i64) as i64)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::U64, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_i64_const(base, result, ((const_val as f64) as u64) as i64)?;
            Ok(true)
        },
        (Primitive::F64, Primitive::F64, Constant::F64(const_val)) => {
            opt.replace_nop(target)?;
            opt.replace_f64_const(base, result, ((const_val as f64) as f64) as f64)?;
            Ok(true)
        },
        _ => todo!()
    }
}
