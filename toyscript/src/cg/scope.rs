use crate::*;
use token::TokenPosition;
use toyir::{BlockIndex, LocalIndex};
use types::{var::VariableDescriptor, InferredType};

#[derive(Debug)]
pub struct VariableStorage<'a> {
    types: &'a TypeSystem,
    variables: RefCell<Vec<VariableDescriptor>>,
}

pub struct Scope<'a> {
    storage: &'a VariableStorage<'a>,
    parent: Option<&'a Scope<'a>>,
    variables: Vec<(String, LocalIndex)>,
    defer_index: Option<BlockIndex>,
    break_index: Option<BlockIndex>,
    continue_index: Option<BlockIndex>,
}

impl<'a> VariableStorage<'a> {
    #[inline]
    pub fn new(types: &'a TypeSystem) -> Self {
        Self {
            types,
            variables: RefCell::new(Vec::new()),
        }
    }
}

impl VariableStorage<'_> {
    #[inline]
    pub fn into_vars(self) -> Vec<VariableDescriptor> {
        self.variables.into_inner()
    }

    #[inline]
    pub fn root_scope<'a>(&'a self) -> Scope<'a> {
        Scope::root(self)
    }

    fn alloc_local(&self, var_desc: VariableDescriptor) {
        let mut variables = self.variables.borrow_mut();
        variables.push(var_desc);
        drop(variables);
    }

    pub fn get_desc_local(&self, index: LocalIndex) -> VariableDescriptor {
        let variables = self.variables.borrow();
        let var = unsafe { variables.get_unchecked(index.as_usize()).clone() };
        drop(variables);
        var
    }

    pub fn edit_local<F, R>(&self, index: LocalIndex, kernel: F) -> R
    where
        F: FnOnce(&mut VariableDescriptor) -> R,
    {
        let mut variables = self.variables.borrow_mut();
        let var_ref = unsafe { variables.get_unchecked_mut(index.as_usize()) };
        let result = kernel(var_ref);
        drop(variables);
        result
    }
}

impl<'a> Scope<'a> {
    #[inline]
    pub fn root(storage: &'a VariableStorage) -> Self {
        Self {
            storage,
            parent: None,
            variables: Vec::new(),
            defer_index: None,
            break_index: None,
            continue_index: None,
        }
    }

    #[inline]
    pub fn scoped(
        &'a self,
        break_index: Option<BlockIndex>,
        continue_index: Option<BlockIndex>,
    ) -> Self {
        Self {
            storage: self.storage,
            parent: Some(self),
            variables: Vec::new(),
            defer_index: self.defer_index,
            break_index: break_index.or(self.break_index),
            continue_index: continue_index.or(self.continue_index),
        }
    }
}

impl Scope<'_> {
    #[inline]
    pub fn types(&self) -> &TypeSystem {
        self.storage.types
    }

    #[inline]
    pub fn parent(&self) -> Option<&Self> {
        self.parent
    }

    #[inline]
    pub fn defer_index(&self) -> Option<BlockIndex> {
        self.defer_index
    }

    #[inline]
    pub fn break_index(&self) -> Option<BlockIndex> {
        self.break_index
    }

    #[inline]
    pub fn continue_index(&self) -> Option<BlockIndex> {
        self.continue_index
    }

    #[inline]
    pub fn storage(&self) -> &VariableStorage {
        self.storage
    }

    fn get_local(&self, identifier: &str) -> Option<LocalIndex> {
        self.variables
            .iter()
            .find(|v| v.0 == identifier)
            .map(|v| v.1)
    }

    pub fn get_desc_local(&self, index: LocalIndex) -> VariableDescriptor {
        self.storage().get_desc_local(index)
    }

    #[inline]
    pub fn resolve_local(&self, identifier: &str) -> Option<LocalIndex> {
        let mut current = self;
        loop {
            match current.get_local(identifier) {
                Some(v) => return Some(v),
                None => {}
            }
            match current.parent {
                Some(v) => current = v,
                None => return None,
            }
        }
    }

    pub fn declare_local(&mut self, var_desc: VariableDescriptor) -> Result<(), CompileError> {
        if self
            .storage
            .types
            .global_name(var_desc.identifier().as_str())
            .is_some()
        {
            return Err(CompileError::duplicate_identifier(var_desc.identifier()));
        }
        match self.get_local(var_desc.identifier().as_str()) {
            Some(_) => Err(CompileError::duplicate_identifier(var_desc.identifier())),
            None => {
                self.storage.alloc_local(var_desc.clone());
                self.variables
                    .push((var_desc.identifier().to_string(), var_desc.index()));
                Ok(())
            }
        }
    }

    pub fn infer_local(
        &self,
        index: LocalIndex,
        inferred_to: &InferredType,
        position: TokenPosition,
    ) -> Result<InferredType, CompileError> {
        self.storage().edit_local(index, |v| {
            let mut lhs = inferred_to.clone();
            self.storage
                .types
                .infer_each(&mut lhs, v.inferred_type_mut(), position)
                .map(|_| lhs)
        })
    }
}
