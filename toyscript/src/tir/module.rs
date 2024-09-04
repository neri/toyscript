use super::*;

#[derive(Debug)]
pub struct Module {
    functions: Vec<Function>,
}

impl Module {
    #[inline]
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    #[inline]
    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }
}
