use crate::*;
use ast::function::FunctionDeclaration;
use core::sync::atomic::{AtomicU32, Ordering};
use keyword::ModifierFlag;
use types::{index::FuncIndex, TypeDescriptor, TypeSystem};

pub const MAIN_NAME: &str = "main";

#[derive(Debug)]
pub struct FunctionDescriptor {
    index: AtomicU32,
    modifiers: ModifierFlag,
    signature: String,
    identifier: String,
    param_types: Box<[Arc<TypeDescriptor>]>,
    result_type: Arc<TypeDescriptor>,
}

impl FunctionDescriptor {
    pub fn parse(
        prefix: Option<&str>,
        id_override: Option<&str>,
        decl: &FunctionDeclaration,
        types: &TypeSystem,
        func_idx: FuncIndex,
    ) -> Result<Arc<Self>, CompileError> {
        let mut modifiers = decl.modifiers();

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

        let identifier = id_override.unwrap_or(decl.identifier().as_str());
        let identifier = TypeSystem::prefixed_identifier(prefix, identifier);
        let signature;
        if identifier == MAIN_NAME {
            signature = format!("${}", identifier);
            modifiers.insert(ModifierFlag::EXPORT);
        } else {
            signature = TypeSystem::mangled(
                prefix,
                id_override.unwrap_or(&identifier),
                param_types.iter().map(|v| v.as_ref()),
                &result_type,
            );
        }

        Ok(Arc::new(FunctionDescriptor {
            index: AtomicU32::new(func_idx.as_u32()),
            modifiers,
            identifier: identifier.to_owned(),
            signature,
            param_types: param_types.into_boxed_slice(),
            result_type,
        }))
    }

    #[inline]
    pub fn intrinsic(
        index: FuncIndex,
        modifiers: ModifierFlag,
        prefix: Option<&str>,
        identifier: &str,
        param_types: Vec<Arc<TypeDescriptor>>,
        result_type: Arc<TypeDescriptor>,
    ) -> Self {
        let signature = TypeSystem::mangled(
            prefix,
            identifier,
            param_types.iter().map(|v| v.as_ref()),
            &result_type,
        );
        Self {
            index: AtomicU32::new(index.as_u32()),
            modifiers,
            signature,
            identifier: identifier.to_owned(),
            param_types: param_types.into_boxed_slice(),
            result_type,
        }
    }

    #[inline]
    pub fn index(&self) -> FuncIndex {
        FuncIndex(self.index.load(Ordering::Relaxed))
    }

    #[inline]
    pub(super) fn set_index(&self, new_value: FuncIndex) {
        self.index.store(new_value.as_u32(), Ordering::SeqCst)
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
    pub fn identifier(&self) -> &str {
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
