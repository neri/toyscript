use super::*;

#[derive(Debug)]
pub struct Module {
    name: String,
    functions: Vec<Function>,
}

impl Module {
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
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
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
}
