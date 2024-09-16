use crate::*;
use ast::function::Parameter;
use token::TokenPosition;
use toyir::{BlockIndex, LocalIndex};
use types::{InferredType, TypeDescriptor};

#[derive(Debug)]
pub struct VariableStorage<'a> {
    types: &'a TypeSystem,
    variables: RefCell<Vec<VariableDescriptor>>,
}

pub struct Scope<'a> {
    storage: &'a VariableStorage<'a>,
    parent: Option<&'a Scope<'a>>,
    variables: Vec<(String, LocalIndex)>,
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
        self.get_local(identifier)
            .or_else(|| self.parent.and_then(|v| v.resolve_local(identifier)))
    }

    pub fn declare_local(&mut self, var_desc: VariableDescriptor) -> Result<(), CompileError> {
        if self
            .storage
            .types
            .global_object(var_desc.identifier().as_str())
            .is_some()
        {
            return Err(CompileError::duplicate_identifier(&var_desc.identifier));
        }
        match self.get_local(var_desc.identifier().as_str()) {
            Some(_) => Err(CompileError::duplicate_identifier(&var_desc.identifier)),
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

#[derive(Debug)]
pub struct VariableDescriptor {
    identifier: Identifier,
    inferred_type: InferredType,
    index: LocalIndex,
    is_mutable: bool,
}

impl VariableDescriptor {
    #[inline]
    pub fn from_var_decl(
        var_decl: &ast::variable::Variable,
        type_desc: Option<&Arc<TypeDescriptor>>,
    ) -> Self {
        Self {
            index: LocalIndex::default(),
            identifier: var_decl.identifier().clone(),
            inferred_type: InferredType::from_type_opt(type_desc),
            is_mutable: var_decl.is_mutable(),
        }
    }

    #[inline]
    pub fn from_parameter(param: &Parameter, type_desc: &Arc<TypeDescriptor>) -> Self {
        Self {
            index: LocalIndex::default(),
            identifier: param.identifier().clone(),
            inferred_type: InferredType::Inferred(type_desc.clone()),
            is_mutable: true,
        }
    }

    pub fn infer(
        &mut self,
        inferred_type: &Arc<TypeDescriptor>,
        position: TokenPosition,
    ) -> Result<(), CompileError> {
        if inferred_type.is_special_type() {
            return Err(CompileError::invalid_type2(
                inferred_type.identifier(),
                position,
            ));
        }
        match &self.inferred_type {
            InferredType::Inferred(old_id) => {
                if old_id.identifier() != inferred_type.identifier() {
                    return Err(CompileError::type_mismatch(
                        &old_id,
                        &inferred_type,
                        position,
                    ));
                }
            }
            InferredType::Maybe(_) | InferredType::Unknown => {
                self.inferred_type = InferredType::Inferred(inferred_type.clone())
            }
        }
        Ok(())
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn inferred_type(&self) -> &InferredType {
        &self.inferred_type
    }

    #[inline]
    pub fn inferred_type_mut(&mut self) -> &mut InferredType {
        &mut self.inferred_type
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
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}

impl Clone for VariableDescriptor {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            identifier: self.identifier.clone(),
            inferred_type: self.inferred_type.clone(),
            is_mutable: self.is_mutable(),
        }
    }
}
