use super::*;
use crate::types::Primitive;

#[derive(Debug)]
#[allow(unused)]
pub struct Function {
    signature: String,
    identifier: Option<String>,
    params: Vec<Primitive>,
    results: Vec<Primitive>,
    locals: Vec<Primitive>,
    codes: CodeBuilder,
}

impl Function {
    #[inline]
    pub fn new(
        signature: &str,
        identifier: Option<&str>,
        params: &[Primitive],
        results: &[Primitive],
        locals: &[Primitive],
        codes: CodeBuilder,
    ) -> Self {
        Self {
            signature: signature.to_owned(),
            identifier: identifier.map(|v| v.to_owned()),
            params: params.to_vec(),
            results: results.to_vec(),
            locals: locals.to_vec(),
            codes,
        }
    }
}
