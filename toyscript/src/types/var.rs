use token::TokenPosition;
use toyir::LocalIndex;
use types::{InferredType, TypeDescriptor};

use crate::*;

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
    pub fn from_parameter(id: &Identifier, type_desc: &Arc<TypeDescriptor>) -> Self {
        Self {
            index: LocalIndex::default(),
            identifier: id.clone(),
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
