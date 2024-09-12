use super::*;

#[derive(Debug)]
pub struct Module {
    name: String,
    imports: Vec<Import>,
    functions: Vec<Function>,
}

impl Module {
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            imports: Vec::new(),
            functions: Vec::new(),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    #[inline]
    pub fn add_import(&mut self, import: Import) {
        self.imports.push(import);
    }

    #[inline]
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }

    #[inline]
    pub fn imports(&self) -> &[Import] {
        &self.imports
    }
}
