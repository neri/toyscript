//! ToyIR Assembler

use crate::*;
use core::error::Error;
use core::mem::transmute;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;
use error::AssembleError;
use opt::MinimalCodeOptimizer;

pub struct Function {
    func_idx: FuncTempIndex,
    signature: String,
    exports: Option<String>,
    dependencies: Vec<FuncTempIndex>,
    params: Vec<LocalVarDescriptor>,
    results: Vec<LocalVarDescriptor>,
    locals: Vec<LocalVarDescriptor>,
    codes: Arc<Vec<u32>>,
}

impl Function {
    pub fn new(
        func_idx: FuncTempIndex,
        signature: &str,
        exports: Option<&str>,
        results: Option<(&str, Primitive)>,
    ) -> Result<FunctionBuilder, AssembleError> {
        let mut results_ = Vec::new();
        if let Some((type_id, primitive_type)) = results {
            if primitive_type != Primitive::Void {
                results_.push(LocalVarDescriptor {
                    index: LocalIndex(0),
                    identifier: None,
                    high_context_type: type_id.to_owned(),
                    primitive_type,
                    is_mut: true,
                    assignment: None,
                    read_count: 0,
                    write_count: 0,
                });
            }
        }

        Ok(FunctionBuilder {
            func_idx,
            signature: signature.to_owned(),
            exports: exports.map(|v| v.to_owned()),
            results: results_,
            params: Vec::new(),
            locals: Vec::new(),
            buf: Vec::new(),
            ssa_index: AtomicU32::new(0),
            value_stack: Vec::new(),
            block_stack: Vec::new(),
            local_index: AtomicU32::new(0),
        })
    }

    #[inline]
    pub fn function_index(&self) -> FuncTempIndex {
        self.func_idx
    }

    #[inline]
    pub fn signature(&self) -> &str {
        &self.signature
    }

    #[inline]
    pub fn exports(&self) -> Option<&str> {
        self.exports.as_ref().map(|v| v.as_str())
    }

    #[inline]
    pub fn dependencies(&self) -> &[FuncTempIndex] {
        &self.dependencies
    }

    #[inline]
    pub fn params(&self) -> &[LocalVarDescriptor] {
        &self.params
    }

    #[inline]
    pub fn results(&self) -> &[LocalVarDescriptor] {
        &self.results
    }

    #[inline]
    pub fn locals(&self) -> &[LocalVarDescriptor] {
        &self.locals
    }

    #[inline]
    pub fn codes(&self) -> &Arc<Vec<u32>> {
        &self.codes
    }
}

impl core::fmt::Debug for Function {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("Function");
        let mut d = &mut d;

        d = d.field("index", &self.func_idx);
        d = d.field("signature", &self.signature);
        if let Some(exports) = self.exports.as_ref() {
            d = d.field("exports", &exports);
        }
        if self.dependencies.len() > 0 {
            d = d.field("dependencies", &self.dependencies);
        }
        d = d.field("params", &LocalIter(&self.params));
        d = d.field("results", &LocalIter(&self.results));
        d = d.field("locals", &LocalIter(&self.locals));
        d = d.field("codes", &CodeDumper(&self.codes));
        d.finish()
    }
}

pub struct FunctionBuilder {
    func_idx: FuncTempIndex,
    signature: String,
    exports: Option<String>,
    results: Vec<LocalVarDescriptor>,
    params: Vec<LocalVarDescriptor>,
    locals: Vec<LocalVarDescriptor>,

    value_stack: Vec<SsaIndex>,
    block_stack: Vec<(BlockIndex, usize)>,
    buf: Vec<u32>,
    ssa_index: AtomicU32,
    local_index: AtomicU32,
}

impl FunctionBuilder {
    #[inline]
    pub fn assembler<'a>(&'a mut self) -> FunctionAssembler<'a> {
        FunctionAssembler::from_function(self)
    }

    pub fn declare_param(
        &mut self,
        identifier: &str,
        high_context_type: &str,
        primitive_type: Primitive,
    ) -> Result<LocalIndex, AssembleError> {
        let local_idx = LocalIndex(self.local_index.fetch_add(1, Ordering::SeqCst));
        let item = LocalVarDescriptor {
            index: local_idx,
            identifier: Some(identifier.to_owned()),
            high_context_type: high_context_type.to_owned(),
            primitive_type,
            is_mut: false,
            assignment: None,
            read_count: 0,
            write_count: 0,
        };
        self.params.push(item);
        Ok(local_idx)
    }

    pub fn declare_local(
        &mut self,
        index: LocalIndex,
        identifier: &str,
        high_context_type: &str,
        primitive_type: Primitive,
        is_mut: bool,
    ) -> Result<(), AssembleError> {
        let expected = LocalIndex((self.params.len() + self.locals.len()) as u32);
        if expected != index {
            return Err(AssembleError::InvalidParameter);
        }
        let item = LocalVarDescriptor {
            index,
            identifier: Some(identifier.to_owned()),
            high_context_type: high_context_type.to_owned(),
            primitive_type,
            is_mut,
            assignment: None,
            read_count: 0,
            write_count: 0,
        };
        self.locals.push(item);
        Ok(())
    }

    pub fn build(self) -> Result<Function, Box<dyn Error>> {
        if self.block_stack.len() != 0 {
            return Err(AssembleError::InvalidBlockStack.into());
        }
        if self.value_stack.len() != 0 {
            return Err(AssembleError::InvalidValueStack.into());
        }
        if (self.params.len() + self.locals.len())
            != self.local_index.load(Ordering::Relaxed) as usize
        {
            return Err(AssembleError::InvalidParameter.into());
        }

        let Self {
            func_idx,
            signature,
            exports,
            buf,
            ssa_index: _,
            value_stack: _,
            block_stack: _,
            local_index: _,
            results,
            params,
            locals,
        } = self;

        let (codes, locals, dependencies) = MinimalCodeOptimizer::optimize(buf, &params, &locals)?;

        Ok(Function {
            func_idx,
            signature,
            exports,
            params,
            results,
            locals,
            codes: Arc::new(codes),
            dependencies,
        })
    }
}

struct CodeDumper<'a>(&'a [u32]);

impl core::fmt::Debug for CodeDumper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(CodeStreamIter::new(self.0)).finish()
    }
}

pub struct CodeStreamIter<'a> {
    buf: &'a [u32],
    index: usize,
}

impl<'a> CodeStreamIter<'a> {
    #[inline]
    pub fn new(buf: &'a [u32]) -> Self {
        Self { buf, index: 0 }
    }
}

impl<'a> Iterator for CodeStreamIter<'a> {
    type Item = CodeFragment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let len_opc = self.buf.get(self.index)?;
            let len = (len_opc >> 16) as usize;
            if len > 0 {
                let opcode = unsafe { transmute::<u8, Op>((len_opc & 0xFFFF) as u8) };
                let params = self.buf.get(self.index + 1..self.index + len)?;
                self.index += len;
                if opcode == Op::Nop {
                    continue;
                }
                return Some(CodeFragment { opcode, params });
            } else {
                return None;
            }
        }
    }
}

pub struct FunctionAssembler<'a> {
    builder: &'a mut FunctionBuilder,
    base_stack_level: usize,
}

impl FunctionAssembler<'_> {
    #[inline]
    pub fn from_function<'a>(builder: &'a mut FunctionBuilder) -> FunctionAssembler<'a> {
        FunctionAssembler {
            builder,
            base_stack_level: 0,
        }
    }

    pub fn alloc_local(&mut self) -> LocalIndex {
        LocalIndex(self.builder.local_index.fetch_add(1, Ordering::SeqCst))
    }

    #[inline]
    pub fn current_index(&self) -> SsaIndex {
        SsaIndex(self.builder.ssa_index.load(Ordering::Relaxed))
    }

    fn emit(&mut self, op: Op, operands: &[Operand]) {
        let mut buf = Vec::with_capacity(1 + operands.len());
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
        self.builder
            .buf
            .push((buf.len() as u32 + 1) << 16 | op as u32);
        self.builder.buf.extend(&buf);

        self.builder.ssa_index.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn pop_vs(&mut self) -> Result<SsaIndex, AssembleError> {
        if self.builder.value_stack.len() > self.base_stack_level {
            self.builder
                .value_stack
                .pop()
                .ok_or(AssembleError::OutOfValueStack)
        } else {
            Err(AssembleError::OutOfValueStack)
        }
    }

    #[inline]
    fn push_vs(&mut self, value: SsaIndex) {
        self.builder.value_stack.push(value);
    }

    /// %result = binop %operand1, %operand2
    #[inline]
    pub fn emit_binop(&mut self, op: Op) -> Result<(), AssembleError> {
        let rhs = self.pop_vs()?;
        let lhs = self.pop_vs()?;
        let result = self.current_index();
        self.push_vs(result);
        self.emit(op, &[result.into(), lhs.into(), rhs.into()]);
        Ok(())
    }

    /// %result = unop %operand1
    #[inline]
    pub fn emit_unop(&mut self, op: Op) -> Result<(), AssembleError> {
        let operand = self.pop_vs()?;
        let result = self.current_index();
        self.push_vs(result);
        self.emit(op, &[result.into(), operand.into()]);
        Ok(())
    }

    /// Parameter independent instructions
    #[inline]
    pub fn emit_independent(&mut self, op: Op) -> Result<(), AssembleError> {
        self.emit(op, &[]);
        Ok(())
    }

    /// %block = block
    #[inline]
    pub fn ir_block(&mut self) -> BlockIndex {
        self.begin_block(Op::Block)
    }

    /// %block = loop
    #[inline]
    pub fn ir_loop(&mut self) -> BlockIndex {
        self.begin_block(Op::Loop)
    }

    fn begin_block(&mut self, op: Op) -> BlockIndex {
        let block = self.current_index();
        self.builder
            .block_stack
            .push((BlockIndex(block.0), self.base_stack_level));
        self.base_stack_level = self.builder.value_stack.len();
        self.emit(op, &[Operand::SsaIndex(block)]);
        BlockIndex(block.0)
    }

    /// end %block
    pub fn ir_end(&mut self, index: BlockIndex) -> Result<(), AssembleError> {
        let (test_index, stack_level) = self
            .builder
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

    /// %result = const $i
    #[inline]
    pub fn ir_bool_const(&mut self, value: bool) {
        self.ir_i32_const(value as i32)
    }

    /// %result = const $i
    #[inline]
    pub fn ir_i32_const(&mut self, value: i32) {
        let result = self.current_index();
        self.push_vs(result);
        self.emit(Op::I32Const, &[result.into(), value.into()]);
    }

    /// %result = const $i
    #[inline]
    pub fn ir_i64_const(&mut self, value: i64) {
        let result = self.current_index();
        self.push_vs(result);
        self.emit(Op::I64Const, &[result.into(), value.into()]);
    }

    /// %result = const $i
    #[inline]
    pub fn ir_f32_const(&mut self, value: f32) {
        let result = self.current_index();
        self.push_vs(result);
        self.emit(Op::F32Const, &[result.into(), value.to_bits().into()]);
    }

    /// %result = const $i
    #[inline]
    pub fn ir_f64_const(&mut self, value: f64) {
        let result = self.current_index();
        self.push_vs(result);
        self.emit(Op::F64Const, &[result.into(), value.to_bits().into()]);
    }

    /// br %block
    #[inline]
    pub fn ir_br(&mut self, target: BlockIndex) -> Result<(), AssembleError> {
        if self
            .builder
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

    /// br %block, %cond
    #[inline]
    pub fn ir_br_if(&mut self, target: BlockIndex) -> Result<(), AssembleError> {
        if self
            .builder
            .block_stack
            .iter()
            .find(|v| v.0 == target)
            .is_none()
        {
            return Err(AssembleError::InvalidBranchTarget);
        }
        let cond = self.pop_vs()?;
        self.emit(Op::BrIf, &[Operand::U32(target.0), cond.into()]);
        Ok(())
    }

    /// %result = local_get $idx
    #[inline]
    pub fn ir_local_get(&mut self, localidx: LocalIndex) {
        let result = self.current_index();
        self.push_vs(result);
        self.emit(
            Op::LocalGet,
            &[result.into(), localidx.as_u32().into(), 0.into()],
        );
    }

    /// local_set %value, $idx
    #[inline]
    pub fn ir_local_set(&mut self, localidx: LocalIndex) -> Result<(), AssembleError> {
        let ssa_index = self.pop_vs()?;
        self.emit(Op::LocalSet, &[ssa_index.into(), localidx.as_u32().into()]);
        Ok(())
    }

    /// %result = local_tee $idx, %value
    #[inline]
    pub fn ir_local_tee(&mut self, localidx: LocalIndex) -> Result<(), AssembleError> {
        let result = self.current_index();
        let ssa_index = self.pop_vs()?;
        self.push_vs(result);
        self.emit(
            Op::LocalTee,
            &[result.into(), localidx.as_u32().into(), ssa_index.into()],
        );
        Ok(())
    }

    /// drop %value
    pub fn ir_drop(&mut self) -> Result<(), AssembleError> {
        let result = self.pop_vs()?;
        self.emit(Op::Drop, &[result.into()]);
        Ok(())
    }

    /// %result = call $params, ...
    pub fn ir_call(
        &mut self,
        target: usize,
        params_len: usize,
        result_len: usize,
    ) -> Result<(), AssembleError> {
        let mut params = Vec::with_capacity(params_len);
        for _ in 0..params_len {
            params.push(Operand::from(self.pop_vs()?));
        }
        let result = self.current_index();
        params.push(Operand::U32(target as u32));
        params.push(result.into());
        params.reverse();
        if result_len > 0 {
            self.push_vs(result);
            self.emit(Op::Call, &params);
        } else {
            self.emit(Op::CallV, &params);
        }
        Ok(())
    }

    /// return %value
    pub fn ir_return(&mut self) -> Result<(), AssembleError> {
        let mut results = Vec::new();
        for _ in 0..self.builder.results.len() {
            let result = self.pop_vs()?;
            results.push(Operand::SsaIndex(result))
        }
        self.emit(Op::Return, &results);
        Ok(())
    }

    /// %result = cast %value, dest_type
    pub fn ir_cast(&mut self, dest_type: u32, src_type: u32) -> Result<(), AssembleError> {
        let operand = self.pop_vs()?;
        let result = self.current_index();
        self.push_vs(result);
        self.emit(
            Op::Cast,
            &[
                result.into(),
                operand.into(),
                dest_type.into(),
                src_type.into(),
            ],
        );
        Ok(())
    }

    /// %result = invert %value, 0
    pub fn ir_invert(&mut self) -> Result<(), AssembleError> {
        let operand = self.pop_vs()?;
        let result = self.current_index();
        self.push_vs(result);
        self.emit(Op::Eqz, &[result.into(), operand.into(), 0.into()]);
        Ok(())
    }

    /// Pseudo-instruction to reverse the sign
    pub fn ir_neg<F, R, E>(&mut self, kernel: F) -> Result<R, E>
    where
        F: FnOnce(&mut Self) -> Result<(Primitive, R), E>,
        E: From<AssembleError>,
    {
        let patch_point = self.builder.buf.len();
        self.ir_i64_const(0);

        let result = kernel(self)?;

        let patch_op = match result.0 {
            Primitive::I8
            | Primitive::U8
            | Primitive::I16
            | Primitive::U16
            | Primitive::I32
            | Primitive::U32 => Op::I32Const,

            Primitive::I64 | Primitive::U64 => Op::I64Const,

            Primitive::F32 => Op::F32Const,

            Primitive::F64 => Op::F64Const,

            Primitive::Void => return Err(AssembleError::InvalidPrimitive.into()),
        };

        self.emit_binop(Op::Sub)?;

        self.builder.buf[patch_point] =
            (self.builder.buf[patch_point] & 0xFFFF_0000) | (patch_op as u32);

        Ok(result.1)
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

pub struct CodeFragment<'a> {
    opcode: Op,
    params: &'a [u32],
}

impl CodeFragment<'_> {
    #[inline]
    pub const fn opcode(&self) -> Op {
        self.opcode
    }

    #[inline]
    pub fn params(&self) -> &[u32] {
        &self.params
    }
}

impl core::fmt::Debug for CodeFragment<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.params.len() + 1;
        match self.opcode {
            Op::I32Const => {
                let const_val = self.params[1];
                match const_val {
                    0x20..=0x7E => {
                        write!(
                            f,
                            "/* {:04x} {:04x} */ %{} = {} {} /* {:?} {:#x} */",
                            len,
                            self.opcode as usize,
                            self.params[0],
                            self.opcode,
                            const_val,
                            const_val as u8 as char,
                            const_val,
                        )
                    }
                    _ => {
                        write!(
                            f,
                            "/* {:04x} {:04x} */ %{} = {} {} /* {:#x} */",
                            len,
                            self.opcode as usize,
                            self.params[0],
                            self.opcode,
                            const_val as i32,
                            const_val,
                        )
                    }
                }
            }
            Op::I64Const => {
                let pl = self.params[1];
                let ph = self.params[2];
                let const_val = (pl as u64) + ((ph as u64) << 32);
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} {} /* {:#x}_{:#x} */",
                    len, self.opcode as usize, self.params[0], self.opcode, const_val, ph, pl,
                )
            }
            Op::F32Const => {
                let const_val = f32::from_bits(self.params[1]);
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} {} /* {:#x} */",
                    len,
                    self.opcode as usize,
                    self.params[0],
                    self.opcode,
                    const_val,
                    self.params[1],
                )
            }
            Op::F64Const => {
                let pl = self.params[1];
                let ph = self.params[2];
                let const_val = f64::from_bits((pl as u64) + ((ph as u64) << 32));
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} {} /* {:#x}_{:#x} */",
                    len, self.opcode as usize, self.params[0], self.opcode, const_val, ph, pl,
                )
            }
            Op::LocalGet => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} ${}",
                    len, self.opcode as usize, self.params[0], self.opcode, self.params[1],
                )
            }
            Op::LocalSet => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ {} %{}, ${}",
                    len, self.opcode as usize, self.opcode, self.params[0], self.params[1],
                )
            }
            Op::LocalTee => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} ${}, %{}",
                    len,
                    self.opcode as usize,
                    self.params[0],
                    self.opcode,
                    self.params[1],
                    self.params[2],
                )
            }
            Op::Cast => {
                let new_type = Primitive::from_type_id(self.params[2])
                    .map(|v| v.as_str())
                    .unwrap_or("???");
                let old_type = Primitive::from_type_id(self.params[3])
                    .map(|v| v.as_str())
                    .unwrap_or("???");
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} %{}, {} as {} /* {}, {} */",
                    len,
                    self.opcode as usize,
                    self.params[0],
                    self.opcode,
                    self.params[1],
                    old_type,
                    new_type,
                    self.params[2],
                    self.params[3],
                )
            }
            Op::Call => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {} ${} /* {:#x} */ (",
                    len,
                    self.opcode as usize,
                    self.params[0],
                    self.opcode,
                    self.params[1],
                    self.params[1],
                )?;
                for (index, param) in self.params.iter().skip(2).enumerate() {
                    if index == 0 {
                        write!(f, "%{}", param,)?;
                    } else {
                        write!(f, ", %{}", param,)?;
                    }
                }
                write!(f, ")")
            }
            Op::CallV => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ {} ${} /* {:#x} */ (",
                    len, self.opcode as usize, self.opcode, self.params[1], self.params[1],
                )?;
                for (index, param) in self.params.iter().skip(2).enumerate() {
                    if index == 0 {
                        write!(f, "%{}", param,)?;
                    } else {
                        write!(f, ", %{}", param,)?;
                    }
                }
                write!(f, ")")
            }
            Op::Br | Op::End => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ {} %{}",
                    len, self.opcode as usize, self.opcode, self.params[0],
                )
            }
            Op::Block | Op::Loop => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ %{} = {}",
                    len, self.opcode as usize, self.params[0], self.opcode,
                )
            }
            Op::BrIf => {
                write!(
                    f,
                    "/* {:04x} {:04x} */ {} %{}, %{}",
                    len, self.opcode as usize, self.opcode, self.params[0], self.params[1],
                )
            }
            Op::Return => {
                if len > 0 {
                    write!(
                        f,
                        "/* {:04x} {:04x} */ {} %{}",
                        len, self.opcode as usize, self.opcode, self.params[0],
                    )
                } else {
                    write!(
                        f,
                        "/* {:04x} {:04x} */ {}",
                        len, self.opcode as usize, self.opcode,
                    )
                }
            }
            _ => match self.opcode.class() {
                OpClass::UnOp => {
                    write!(
                        f,
                        "/* {:04x} {:04x} */ %{} = {} %{}",
                        len, self.opcode as usize, self.params[0], self.opcode, self.params[1],
                    )
                }
                OpClass::Cmp | OpClass::BinOp => {
                    write!(
                        f,
                        "/* {:04x} {:04x} */ %{} = {} %{}, %{}",
                        len,
                        self.opcode as usize,
                        self.params[0],
                        self.opcode,
                        self.params[1],
                        self.params[2],
                    )
                }
                _ => {
                    write!(
                        f,
                        "/* {:04x} {:04x} */ {} {:?}",
                        len, self.opcode as usize, self.opcode, self.params
                    )
                }
            },
        }
    }
}

macro_rules! ir_no_params {
    {$( $func_name:ident: $op:ident; )*} => {
        impl FunctionAssembler<'_> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SsaIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockIndex(u32);

impl BlockIndex {
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct LocalIndex(u32);

impl LocalIndex {
    #[inline]
    pub const unsafe fn new(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl core::fmt::Debug for LocalIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LocalIndex({})", self.0)
    }
}

struct LocalIter<'a>(&'a [LocalVarDescriptor]);

impl core::fmt::Debug for LocalIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "[")?;
        for item in self.0.iter() {
            if let Some(identifier) = item.identifier.as_ref() {
                writeln!(
                    f,
                    "    ${}: {} /* {:?}: {:?} => {} r({}) w({}) */",
                    item.index.as_u32(),
                    item.primitive_type.storage_type(),
                    identifier,
                    item.high_context_type,
                    item.primitive_type,
                    item.read_count,
                    item.write_count,
                )?;
            } else {
                writeln!(
                    f,
                    "    ${}: {} /* {} => {} */",
                    item.index.as_u32(),
                    item.primitive_type.storage_type(),
                    item.high_context_type,
                    item.primitive_type,
                )?;
            }
        }
        write!(f, "]")
    }
}

#[derive(Debug)]
pub struct LocalVarDescriptor {
    index: LocalIndex,
    identifier: Option<String>,
    high_context_type: String,
    primitive_type: Primitive,
    is_mut: bool,
    pub(super) assignment: Option<Constant>,
    pub(super) read_count: usize,
    pub(super) write_count: usize,
}

impl LocalVarDescriptor {
    #[inline]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_ref().map(|v| v.as_str())
    }

    #[inline]
    pub fn primitive_type(&self) -> Primitive {
        self.primitive_type
    }

    #[inline]
    pub fn index(&self) -> LocalIndex {
        self.index
    }

    #[inline]
    pub fn set_index(&mut self, index: LocalIndex) {
        self.index = index;
    }

    #[inline]
    pub const fn is_mut(&self) -> bool {
        self.is_mut
    }

    #[inline]
    pub const fn is_const(&self) -> bool {
        !self.is_mut
    }
}

impl Clone for LocalVarDescriptor {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            identifier: self.identifier.clone(),
            high_context_type: self.high_context_type.clone(),
            primitive_type: self.primitive_type.clone(),
            is_mut: self.is_mut.clone(),
            assignment: self.assignment.clone(),
            read_count: self.read_count,
            write_count: self.write_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constant {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Constant {
    pub fn is_same_type(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::I32(_), Self::I32(_))
                | (Self::I64(_), Self::I64(_))
                | (Self::F32(_), Self::F32(_))
                | (Self::F64(_), Self::F64(_))
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Self::I32(_) | Self::I64(_))
    }

    pub fn int_value(&self) -> Option<i64> {
        match self {
            Self::I32(v) => Some(*v as i64),
            Self::I64(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FuncTempIndex(u32);

impl FuncTempIndex {
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl core::fmt::Debug for FuncTempIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}
