use crate::*;

#[derive(Debug)]
pub struct Import {
    name: String,
    from: String,
    import_desc: ImportDescriptor,
    is_external: bool,
}

#[derive(Debug)]
pub enum ImportDescriptor {
    Function(ImportFunction),
}

#[derive(Debug)]
pub struct ImportFunction {
    func_idx: FuncTempIndex,
    signature: String,
    params: Vec<Primitive>,
    results: Vec<Primitive>,
}

impl Import {
    #[inline]
    pub fn func(
        func_idx: FuncTempIndex,
        signature: &str,
        name: &str,
        from: &str,
        params: &[Primitive],
        results: &[Primitive],
        is_external: bool,
    ) -> Self {
        Import {
            name: name.to_owned(),
            from: from.to_owned(),
            import_desc: ImportDescriptor::Function(ImportFunction {
                func_idx,
                signature: signature.to_owned(),
                params: params.to_vec(),
                results: results.to_vec(),
            }),
            is_external,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn from(&self) -> &str {
        &self.from
    }

    #[inline]
    pub fn import_desc(&self) -> &ImportDescriptor {
        &self.import_desc
    }

    #[inline]
    pub const fn is_external(&self) -> bool {
        self.is_external
    }
}

impl ImportFunction {
    #[inline]
    pub fn function_index(&self) -> FuncTempIndex {
        self.func_idx
    }

    #[inline]
    pub fn signature(&self) -> &str {
        &self.signature
    }

    #[inline]
    pub fn params(&self) -> &[Primitive] {
        &self.params
    }

    #[inline]
    pub fn results(&self) -> &[Primitive] {
        &self.results
    }
}
