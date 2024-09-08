use crate::*;
use error::AssembleError;
use toyassembly::types::ValType;

pub struct Function {
    signature: String,
    exports: Option<String>,
    params: Vec<(String, String, Primitive, ValType)>,
    results: Vec<(String, String, Primitive, ValType)>,
    locals: Vec<(String, String, Primitive, ValType)>,
    codes: Assembler,
}

impl Function {
    #[inline]
    pub fn new(
        signature: &str,
        exports: Option<&str>,
        params: &[(String, String, Primitive)],
        results: &[(String, Primitive)],
        locals: &[(String, String, Primitive)],
        codes: Assembler,
    ) -> Result<Self, AssembleError> {
        let mut params_ = Vec::new();
        for (id, type_id, primitive) in params {
            let valtype = primitive
                .wasm_binding()
                .ok_or(AssembleError::InvalidPrimitive)?;
            params_.push((id.to_owned(), type_id.to_owned(), *primitive, valtype));
        }
        let mut results_ = Vec::new();
        for (type_id, primitive) in results {
            if *primitive == Primitive::Void {
                break;
            }
            let valtype = primitive
                .wasm_binding()
                .ok_or(AssembleError::InvalidPrimitive)?;
            results_.push(("".to_owned(), type_id.to_owned(), *primitive, valtype));
        }
        let mut locals_ = Vec::new();
        for (id, type_id, primitive) in locals {
            let valtype = primitive
                .wasm_binding()
                .ok_or(AssembleError::InvalidPrimitive)?;
            locals_.push((id.to_owned(), type_id.to_owned(), *primitive, valtype));
        }
        Ok(Self {
            signature: signature.to_owned(),
            exports: exports.map(|v| v.to_owned()),
            params: params_,
            results: results_,
            locals: locals_,
            codes,
        })
    }
}

impl core::fmt::Debug for Function {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Function")
            .field("signature", &self.signature)
            .field("exports", &self.exports)
            .field("params", &LocalIter(&self.params))
            .field("results", &LocalIter(&self.results))
            .field("locals", &LocalIter(&self.locals))
            .field("codes", &self.codes)
            .finish()
    }
}

struct LocalIter<'a>(&'a [(String, String, Primitive, ValType)]);

impl core::fmt::Debug for LocalIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "[")?;
        for item in self.0.iter() {
            if item.0.is_empty() {
                writeln!(
                    f,
                    "    {:?}, /* => Primitive({}) ValType({}) */",
                    item.1, item.2, item.3,
                )?;
            } else {
                writeln!(
                    f,
                    "    {:?}: {:?}, /* => Primitive({}) ValType({}) */",
                    item.0, item.1, item.2, item.3,
                )?;
            }
        }
        write!(f, "]")
    }
}
