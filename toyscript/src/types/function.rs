use crate::*;
use ast::function::FunctionDeclaration;
use keyword::ModifierFlag;
use types::{index::FuncIndex, TypeDescriptor, TypeSystem};

pub const MAIN_NAME: &str = "main";

#[derive(Debug)]
pub struct FunctionDescriptor {
    index: FuncIndex,
    modifiers: ModifierFlag,
    signature: String,
    identifier: Identifier,
    param_types: Box<[Arc<TypeDescriptor>]>,
    result_type: Arc<TypeDescriptor>,
}

impl FunctionDescriptor {
    pub fn parse(
        decl: &FunctionDeclaration,
        types: &TypeSystem,
        func_idx: FuncIndex,
    ) -> Result<Self, CompileError> {
        let mut modifiers = decl.modifiers();
        let identifier = decl.identifier().clone();

        let mut param_types = Vec::new();
        for param in decl.parameters() {
            if let Some(v) = types.from_ast(param.type_decl()) {
                if v.is_special_type() {
                    return Err(CompileError::invalid_type(param.type_decl().identifier()));
                }
                param_types.push(v.clone());
            } else {
                return Err(CompileError::identifier_not_found(
                    param.type_decl().identifier(),
                ));
            }
        }

        let result_type = match decl.result_type() {
            Some(type_desc) => {
                if let Some(v) = types.from_ast(type_desc) {
                    v.clone()
                } else {
                    return Err(CompileError::identifier_not_found(&type_desc.identifier()));
                }
            }
            None => types.builtin_void(),
        };

        let signature;
        if identifier.as_str() == MAIN_NAME {
            signature = format!("${}", identifier.as_str());
            modifiers.insert(ModifierFlag::EXPORT);
        } else {
            signature = TypeSystem::mangled(
                identifier.as_str(),
                param_types.iter().map(|v| v.as_ref()),
                &result_type,
            );
        }

        Ok(FunctionDescriptor {
            index: func_idx,
            modifiers,
            identifier,
            signature,
            param_types: param_types.into_boxed_slice(),
            result_type,
        })
    }

    #[inline]
    pub fn intrinsic(
        index: FuncIndex,
        modifiers: ModifierFlag,
        identifier: Identifier,
        param_types: Vec<Arc<TypeDescriptor>>,
        result_type: Arc<TypeDescriptor>,
    ) -> Self {
        let signature = TypeSystem::mangled(
            identifier.as_str(),
            param_types.iter().map(|v| v.as_ref()),
            &result_type,
        );
        Self {
            index,
            modifiers,
            signature,
            identifier,
            param_types: param_types.into_boxed_slice(),
            result_type,
        }
    }

    #[inline]
    pub fn index(&self) -> FuncIndex {
        self.index
    }

    #[inline]
    pub fn modifiers(&self) -> ModifierFlag {
        self.modifiers
    }

    #[inline]
    pub fn signature(&self) -> &str {
        &self.signature
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn param_types(&self) -> &[Arc<TypeDescriptor>] {
        &self.param_types
    }

    #[inline]
    pub fn result_type(&self) -> &Arc<TypeDescriptor> {
        &self.result_type
    }
}
