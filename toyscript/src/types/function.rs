use crate::*;
use ast::{block::Block, function::FunctionDeclaration, statement::Statement};
use core::sync::atomic::{AtomicU32, Ordering};
use keyword::ModifierFlag;
use token::TokenPosition;
use types::{index::FuncIndex, TypeDescriptor, TypeSystem};

pub const MAIN_FUNC_NAME: &str = "main";

pub struct FunctionDescriptor {
    index: AtomicU32,
    modifiers: ModifierFlag,
    signature: String,
    identifier: String,
    import_from: Option<(String, String)>,
    params: Vec<(Identifier, Arc<TypeDescriptor>)>,
    result_type: Arc<TypeDescriptor>,
    body: FunctionBody,
    position: TokenPosition,
}

impl FunctionDescriptor {
    pub fn parse(
        prefix: Option<&str>,
        id_override: Option<&str>,
        func_decl: &FunctionDeclaration,
        types: &TypeSystem,
        func_idx: FuncIndex,
    ) -> Result<Arc<Self>, CompileError> {
        let mut modifiers = func_decl.modifiers();

        let mut param_names = Vec::new();
        let mut param_types = Vec::new();
        for param in func_decl.parameters() {
            let id = param.identifier().clone();
            if param_names.contains(&id) {
                return Err(CompileError::duplicate_identifier(&id));
            }
            if let Some(v) = types.from_ast(param.type_decl()) {
                if v.is_special_type() {
                    return Err(CompileError::invalid_type(param.type_decl().identifier()));
                }
                param_types.push(v);
                param_names.push(id);
            } else {
                return Err(CompileError::identifier_not_found(
                    param.type_decl().identifier(),
                ));
            }
        }
        let params = param_names
            .iter()
            .zip(param_types.iter())
            .map(|v| (v.0.clone(), v.1.clone()))
            .collect::<Vec<_>>();

        let result_type = match func_decl.result_type() {
            Some(type_desc) => {
                if let Some(v) = types.from_ast(type_desc) {
                    v.clone()
                } else {
                    return Err(CompileError::identifier_not_found(&type_desc.identifier()));
                }
            }
            None => types.builtin_void(),
        };

        let identifier = TypeSystem::prefixed_identifier(
            prefix,
            id_override.unwrap_or(func_decl.identifier().as_str()),
            &[],
        );
        if identifier == MAIN_FUNC_NAME {
            modifiers.insert(ModifierFlag::EXPORT);
        }
        let signature = TypeSystem::mangled(prefix, id_override.unwrap_or(&identifier), &[]);

        Ok(Arc::new(FunctionDescriptor {
            index: AtomicU32::new(func_idx.as_u32()),
            modifiers,
            identifier: identifier.to_owned(),
            signature,
            import_from: func_decl
                .import_from()
                .map(|v| (v.0.to_owned(), v.1.to_owned())),
            params,
            result_type,
            body: FunctionBody::Block(func_decl.body().clone()),
            position: func_decl.position(),
        }))
    }

    pub fn from_name_and_body(
        prefix: Option<&str>,
        name: &str,
        modifiers: ModifierFlag,
        body: Vec<Statement>,
        types: &TypeSystem,
        func_idx: FuncIndex,
    ) -> Result<Arc<Self>, CompileError> {
        let result_type = types.builtin_void();

        let identifier = name.to_owned();
        let signature = TypeSystem::mangled(prefix, &identifier, &[]);

        Ok(Arc::new(FunctionDescriptor {
            index: AtomicU32::new(func_idx.as_u32()),
            modifiers,
            signature,
            identifier,
            import_from: None,
            params: [].to_vec(),
            result_type,
            body: FunctionBody::Block(Block::from_statements(body)),
            // TODO:
            position: TokenPosition::empty(),
        }))
    }

    pub fn instrinc_inline_op(
        name: &str,
        modifiers: ModifierFlag,
        param_types: &[Arc<TypeDescriptor>],
        result_type: &Arc<TypeDescriptor>,
        op: toyir::Op,
        func_idx: FuncIndex,
    ) -> Result<Arc<Self>, CompileError> {
        let identifier = name.to_owned();
        let signature = TypeSystem::mangled2(&identifier);

        let params = param_types
            .iter()
            .enumerate()
            .map(|(index, _ty)| (Identifier::new(&format!("${}", index)), _ty.clone()))
            .collect::<Vec<_>>();

        Ok(Arc::new(FunctionDescriptor {
            index: AtomicU32::new(func_idx.as_u32()),
            modifiers,
            signature,
            identifier,
            import_from: None,
            params,
            result_type: result_type.clone(),
            body: FunctionBody::InlineOp(op),
            // TODO:
            position: TokenPosition::empty(),
        }))
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
    pub fn import_from(&self) -> Option<(&str, &str)> {
        self.import_from
            .as_ref()
            .map(|v| (v.0.as_str(), v.1.as_str()))
    }

    #[inline]
    pub fn params(&self) -> &[(Identifier, Arc<TypeDescriptor>)] {
        &self.params
    }

    #[inline]
    pub fn param_len(&self) -> usize {
        self.params.len()
    }

    #[inline]
    pub fn result_type(&self) -> &Arc<TypeDescriptor> {
        &self.result_type
    }

    #[inline]
    pub fn body(&self) -> &FunctionBody {
        &self.body
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }
}

impl core::fmt::Debug for FunctionDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FunctionDescriptor")
            .field("index", &self.index)
            .field("modifiers", &self.modifiers)
            .field("signature", &self.signature)
            .field("identifier", &self.identifier)
            .field("import_from", &self.import_from)
            .field("params", &self.params)
            .field("result_type", &self.result_type)
            // .field("body", &self.body)
            .field("position", &self.position)
            .finish()
    }
}

#[derive(Debug)]
pub enum FunctionBody {
    Block(Block),
    InlineOp(toyir::Op),
}
