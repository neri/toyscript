//! ToyScript Type System

use crate::*;
use ast::{integer::Integer, statement::Statement};
use function::FunctionDescriptor;
use index::FuncIndex;
use keyword::ModifierFlag;
use token::TokenPosition;
use toyir::Primitive;

pub mod function;
pub mod index;
// pub mod string;

pub const BUILTIN_BOOLEAN: &str = "boolean";
pub const BUILTIN_CHAR: &str = "char";
pub const BUILTIN_HANDLE: &str = "__builtin_handle";
pub const BUILTIN_INT: &str = "int";
pub const BUILTIN_ISIZE: &str = "isize";
pub const BUILTIN_NEVER: &str = "never";
pub const BUILTIN_NUMBER: &str = "number";
pub const BUILTIN_STRING: &str = "string";
pub const BUILTIN_UINT: &str = "uint";
pub const BUILTIN_USIZE: &str = "usize";
pub const BUILTIN_VOID: &str = "void";

/// ToyScript Type System
#[derive(Debug)]
pub struct TypeSystem {
    name: String,
    global_objects: BTreeMap<String, GlobalObjectIndex>,
    functions: Vec<Arc<FunctionDescriptor>>,
    types: BTreeMap<String, Arc<TypeDescriptor>>,

    integer_bits: usize,
    pointer_bits: usize,
    type_int: Primitive,
    type_uint: Primitive,
    type_isize: Primitive,
    type_usize: Primitive,
}

impl TypeSystem {
    pub fn new(name: &str, ast: &ast::Ast) -> Result<Self, CompileError> {
        let mut system = Self {
            name: name.to_owned(),
            global_objects: BTreeMap::new(),
            types: BTreeMap::new(),
            functions: Vec::new(),

            integer_bits: 0,
            pointer_bits: 0,
            type_int: Primitive::Void,
            type_uint: Primitive::Void,
            type_isize: Primitive::Void,
            type_usize: Primitive::Void,
        };

        for primitive in Primitive::all_values() {
            let desc = TypeDescriptor::from_primitive(*primitive);
            system.types.insert(desc.identifier().to_string(), desc);
        }

        system.set_use(32).unwrap();

        system
            .make_primitive_alias(BUILTIN_NEVER, Primitive::Void)
            .unwrap();
        system
            .make_primitive_alias(BUILTIN_NUMBER, Primitive::F64)
            .unwrap();
        system
            .make_primitive_alias(BUILTIN_CHAR, Primitive::U32)
            .unwrap();
        system
            .make_primitive_alias(BUILTIN_BOOLEAN, Primitive::Bool)
            .unwrap();

        // system.make_reference(
        //     &Identifier::new(BUILTIN_HANDLE, TokenPosition::empty()),
        //     &system.builtin_void(),
        // )?;
        // system.make_simple_alias(BUILTIN_HANDLE, BUILTIN_USIZE)?;

        system.make_reference(
            &Identifier::new(BUILTIN_STRING, TokenPosition::empty()),
            &system.builtin_void(),
        )?;

        let mut will_waiting_ids = Vec::new();
        let mut waiting_type_list = Vec::new();
        for statement in ast.program() {
            match statement {
                Statement::TypeAlias(identifier, type_desc) => {
                    will_waiting_ids.push(identifier.to_string());
                    match system.make_alias(identifier, type_desc) {
                        Ok(_) => (),
                        Err(err) => match err.kind() {
                            CompileErrorKind::IdentifierNotFound => {
                                waiting_type_list.push((identifier.clone(), type_desc.clone()));
                            }
                            _ => return Err(err),
                        },
                    }
                }
                _ => (),
            }
        }

        // Attempt to resolve forward references
        for _ in 0..8 {
            let mut waiting_list2 = Vec::new();
            for item in waiting_type_list.iter().rev() {
                let (identifier, type_desc) = item;
                if !will_waiting_ids.contains(&type_desc.as_string()) {
                    return Err(CompileError::identifier_not_found(&type_desc.identifier()));
                }
                match system.make_alias(identifier, type_desc) {
                    Ok(_) => (),
                    Err(err) => match err.kind() {
                        CompileErrorKind::IdentifierNotFound => {
                            waiting_list2.push((identifier.clone(), type_desc.clone()));
                        }
                        _ => return Err(err),
                    },
                }
            }
            waiting_type_list = waiting_list2;
            let mut waiting_list2 = Vec::new();
            for item in waiting_type_list.iter() {
                let (identifier, type_desc) = item;
                if !will_waiting_ids.contains(&type_desc.as_string()) {
                    return Err(CompileError::identifier_not_found(&type_desc.identifier()));
                }
                match system.make_alias(identifier, type_desc) {
                    Ok(_) => (),
                    Err(err) => match err.kind() {
                        CompileErrorKind::IdentifierNotFound => {
                            waiting_list2.push((identifier.clone(), type_desc.clone()));
                        }
                        _ => return Err(err),
                    },
                }
            }
            waiting_type_list = waiting_list2;
        }
        if let Some(item) = waiting_type_list.first() {
            return Err(CompileError::identifier_not_found_looping(
                &item.1.identifier(),
            ));
        }

        for is_import in &[true, false] {
            for statement in ast.program() {
                match statement {
                    Statement::Function(func_decl) => {
                        if *is_import == func_decl.modifiers().contains(ModifierFlag::IMPORT) {
                            let func = FunctionDescriptor::parse(
                                func_decl,
                                &system,
                                FuncIndex(system.functions.len() as u32),
                            )?;
                            if system.global_object(func.identifier().as_str()).is_none() {
                                system.global_objects.insert(
                                    func.identifier().as_string().clone(),
                                    GlobalObjectIndex::Funtion(func.function_index()),
                                );
                                system.functions.push(Arc::new(func));
                            } else {
                                return Err(CompileError::duplicate_identifier(func.identifier()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(system)
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn integer_bits(&self) -> usize {
        self.integer_bits
    }

    #[inline]
    pub fn pointer_bits(&self) -> usize {
        self.pointer_bits
    }

    #[inline]
    pub fn all_types(&self) -> impl Iterator<Item = &Arc<TypeDescriptor>> {
        self.types.values()
    }

    #[inline]
    pub fn get(&self, identifier: &str) -> Option<&Arc<TypeDescriptor>> {
        self.types.get(identifier)
    }

    #[inline]
    pub fn from_ast(&self, ast_type: &ast::class::TypeDescriptor) -> Option<&Arc<TypeDescriptor>> {
        self.get(&ast_type.as_string())
    }

    #[inline]
    pub fn primitive_type(&self, primitive: Primitive) -> Arc<TypeDescriptor> {
        self.resolve(primitive.as_str()).unwrap()
    }

    #[inline]
    pub fn builtin_boolean(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_BOOLEAN).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_char(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_CHAR).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_handle(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_HANDLE).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_int(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_INT).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_uint(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_UINT).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_isize(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_ISIZE).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_usize(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_USIZE).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_never(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_NEVER).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_number(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_NUMBER).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_string(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_STRING).map(|v| v.clone()).unwrap()
    }

    #[inline]
    pub fn builtin_void(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_VOID).map(|v| v.clone()).unwrap()
    }

    #[inline]
    fn set_use(&mut self, value: usize) -> Result<(), ()> {
        match value {
            8 => self.set_data_model(8, 16),
            16 => self.set_data_model(16, 16),
            32 => self.set_data_model(32, 32),
            64 => self.set_data_model(32, 64),
            _ => Err(()),
        }
    }

    /// Set the actual primitive types of `int`, `uint`, `isize`, and `usize` according to the specified data model.
    ///
    /// This function only works correctly once.
    #[inline]
    fn set_data_model(&mut self, integer_bits: usize, pointer_bits: usize) -> Result<(), ()> {
        let type_int = Primitive::int_for_bits(integer_bits)?;
        let type_uint = Primitive::uint_for_bits(integer_bits)?;
        let type_isize = Primitive::int_for_bits(pointer_bits)?;
        let type_usize = Primitive::uint_for_bits(pointer_bits)?;

        self.make_primitive_alias(BUILTIN_INT, type_int)?;
        self.make_primitive_alias(BUILTIN_UINT, type_uint)?;
        self.make_primitive_alias(BUILTIN_ISIZE, type_isize)?;
        self.make_primitive_alias(BUILTIN_USIZE, type_usize)?;

        self.integer_bits = integer_bits;
        self.pointer_bits = pointer_bits;
        self.type_int = type_int;
        self.type_uint = type_uint;
        self.type_isize = type_isize;
        self.type_usize = type_usize;

        Ok(())
    }

    #[inline]
    fn make_primitive_alias(&mut self, identifier: &str, primitive: Primitive) -> Result<(), ()> {
        if self.get(identifier).is_some() {
            return Err(());
        }
        let type_desc = TypeDescriptor::make_alias(identifier, &self.primitive_type(primitive));
        self.types
            .insert(identifier.to_string(), Arc::new(type_desc));
        Ok(())
    }

    #[inline]
    fn _make_simple_alias(&mut self, identifier: &str, target: &str) -> Result<(), CompileError> {
        self.make_alias(
            &Identifier::new(identifier, TokenPosition::empty()),
            &ast::class::TypeDescriptor::Simple(Identifier::new(target, TokenPosition::empty())),
        )
    }

    pub fn make_alias(
        &mut self,
        identifier: &Identifier,
        type_desc: &ast::class::TypeDescriptor,
    ) -> Result<(), CompileError> {
        if self.get(identifier.as_str()).is_some() {
            return Err(CompileError::duplicate_identifier(identifier));
        }
        if let Some(desc) = self.from_ast(type_desc) {
            if desc.is_special_type() {
                return Err(CompileError::invalid_type(&type_desc.identifier()));
            }
            let desc = TypeDescriptor::make_alias(identifier.as_str(), &desc);
            self.types.insert(identifier.to_string(), Arc::new(desc));
        } else {
            return Err(CompileError::identifier_not_found(&type_desc.identifier()));
        }
        Ok(())
    }

    #[inline]
    pub fn make_reference(
        &mut self,
        identifier: &Identifier,
        target: &Arc<TypeDescriptor>,
    ) -> Result<(), CompileError> {
        if self.get(identifier.as_str()).is_some() {
            return Err(CompileError::duplicate_identifier(identifier));
        }
        let desc = TypeDescriptor::make_reference(identifier.as_str(), target);
        self.types.insert(identifier.to_string(), Arc::new(desc));

        Ok(())
    }

    #[inline]
    pub fn mangled<'a, T>(
        identifier: &str,
        _param_types: T,
        _result_type: &Arc<TypeDescriptor>,
    ) -> String
    where
        T: Iterator<Item = &'a TypeDescriptor>,
    {
        format!(
            "${}",
            identifier,
            // param_types.map(|v| v.mangled()).collect::<String>(),
            // result_type.mangled(),
        )
    }

    pub fn infer_as(
        &self,
        lhs: &InferredType,
        rhs: &Arc<TypeDescriptor>,
        position: TokenPosition,
    ) -> Result<InferredType, CompileError> {
        let mut lhs = lhs.clone();
        let mut rhs = InferredType::Inferred(rhs.clone());
        self.infer_each(&mut lhs, &mut rhs, position).map(|_| lhs)
    }

    pub fn infer_each(
        &self,
        lhs: &mut InferredType,
        rhs: &mut InferredType,
        position: TokenPosition,
    ) -> Result<(), CompileError> {
        match (&lhs, &rhs) {
            (InferredType::Inferred(lt), InferredType::Inferred(rt)) => {
                if lt != rt {
                    return Err(CompileError::type_mismatch(&lt, &rt, position));
                }
            }
            (InferredType::Inferred(lt), InferredType::Unknown)
            | (InferredType::Inferred(lt), InferredType::Maybe(_)) => {
                *rhs = InferredType::Inferred(lt.clone());
            }
            (InferredType::Unknown, InferredType::Inferred(rt))
            | (InferredType::Maybe(_), InferredType::Inferred(rt)) => {
                *lhs = InferredType::Inferred(rt.clone());
            }
            (InferredType::Maybe(lt), InferredType::Unknown) => {
                *rhs = InferredType::Maybe(lt.clone());
            }
            (InferredType::Unknown, InferredType::Maybe(rt)) => {
                *lhs = InferredType::Maybe(rt.clone());
            }
            _ => {}
        }
        Ok(())
    }

    pub fn infer_integer(
        &self,
        value: &Integer,
        inferred_to: &InferredType,
        position: TokenPosition,
    ) -> Result<(Integer, InferredType), CompileError> {
        match inferred_to {
            InferredType::Inferred(inferred_to) => {
                if let Some(primitive) = inferred_to.primitive_type() {
                    match value.try_convert_to(primitive) {
                        Ok(v) => return Ok((v, InferredType::Inferred(inferred_to.clone()))),
                        Err(_) => {
                            return Err(CompileError::literal_overflow(&inferred_to, position))
                        }
                    }
                }
                Err(CompileError::type_mismatch(
                    &inferred_to,
                    &self.primitive_type(value.primitive_type()),
                    position,
                ))
            }
            InferredType::Maybe(inferred_to) => {
                if let Some(primitive) = inferred_to.primitive_type() {
                    match value.try_convert_to(primitive) {
                        Ok(v) => return Ok((v, InferredType::Maybe(inferred_to.clone()))),
                        Err(_) => {
                            return Err(CompileError::literal_overflow(&inferred_to, position))
                        }
                    }
                }
                Ok((value.clone(), InferredType::Maybe(inferred_to.clone())))
            }
            InferredType::Unknown => {
                self.infer_integer(&value, &InferredType::Maybe(self.builtin_int()), position)
            }
        }
    }

    #[inline]
    pub fn canonical(&self, ty: Option<&Arc<TypeDescriptor>>) -> Arc<TypeDescriptor> {
        ty.map(|v| v.clone()).unwrap_or(self.builtin_void())
    }

    pub fn global_object(&self, identifier: &str) -> Option<GlobalObjectIndex> {
        self.global_objects.get(&identifier.to_string()).map(|v| *v)
    }

    pub fn function(&self, identifier: &str) -> Option<&Arc<FunctionDescriptor>> {
        let funcidx = match self.global_object(identifier)? {
            GlobalObjectIndex::Funtion(v) => v,
        };
        self.functions.get(funcidx.as_usize())
    }
}

pub trait Resolve<T: ?Sized> {
    fn resolve(&self, v: &T) -> Option<Arc<TypeDescriptor>>;

    fn resolve_primitive(&self, v: &T) -> Option<Primitive>;
}

impl Resolve<ast::class::TypeDescriptor> for TypeSystem {
    fn resolve(&self, desc: &ast::class::TypeDescriptor) -> Option<Arc<TypeDescriptor>> {
        self.resolve(desc.as_string().as_str())
    }

    fn resolve_primitive(&self, desc: &ast::class::TypeDescriptor) -> Option<Primitive> {
        self.resolve(desc)
            .and_then(|ref v| self.resolve_primitive(v))
    }
}

impl Resolve<str> for TypeSystem {
    fn resolve(&self, identifier: &str) -> Option<Arc<TypeDescriptor>> {
        self.get(identifier).and_then(|v| self.resolve(v))
    }

    fn resolve_primitive(&self, identifier: &str) -> Option<Primitive> {
        self.resolve(identifier)
            .and_then(|ref v| self.resolve_primitive(v))
    }
}

impl Resolve<Arc<TypeDescriptor>> for TypeSystem {
    fn resolve(&self, desc: &Arc<TypeDescriptor>) -> Option<Arc<TypeDescriptor>> {
        match desc.kind() {
            TypeKind::Primitive(_) => Some(desc.clone()),
            TypeKind::Alias(alias) => self.resolve(alias),
            TypeKind::Reference(_) => Some(desc.clone()),
            TypeKind::Optional(_) => Some(desc.clone()),
        }
    }

    fn resolve_primitive(&self, desc: &Arc<TypeDescriptor>) -> Option<Primitive> {
        match desc.kind() {
            TypeKind::Primitive(primitive) => Some(*primitive),
            TypeKind::Alias(alias) => self.resolve_primitive(alias),
            TypeKind::Reference(_) => Some(self.type_usize),
            TypeKind::Optional(_) => Some(self.type_usize),
        }
    }
}

#[derive(Debug)]
pub struct TypeDescriptor {
    identifier: String,
    kind: TypeKind,
}

impl TypeDescriptor {
    #[inline]
    pub fn from_primitive(primitive: Primitive) -> Arc<Self> {
        Arc::new(Self {
            identifier: primitive.as_str().to_owned(),
            kind: TypeKind::Primitive(primitive),
        })
    }

    #[inline]
    fn make_alias(identifier: &str, alias: &Arc<TypeDescriptor>) -> Self {
        Self {
            identifier: identifier.to_string(),
            kind: TypeKind::Alias(alias.clone()),
        }
    }

    #[inline]
    fn make_reference(identifier: &str, target: &Arc<TypeDescriptor>) -> Self {
        Self {
            identifier: identifier.to_string(),
            kind: TypeKind::Reference(target.clone()),
        }
    }

    #[inline]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    #[inline]
    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    #[inline]
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, TypeKind::Primitive(_))
    }

    #[inline]
    fn primitive_type(&self) -> Option<Primitive> {
        match self.kind() {
            TypeKind::Primitive(v) => Some(*v),
            TypeKind::Alias(v) => v.primitive_type(),
            _ => None,
        }
    }

    #[inline]
    pub fn is_refernce(&self) -> bool {
        matches!(self.kind, TypeKind::Reference(_))
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        self.identifier() == BUILTIN_BOOLEAN
    }

    #[inline]
    pub fn is_never(&self) -> bool {
        self.identifier() == BUILTIN_NEVER
    }

    #[inline]
    pub fn is_void(&self) -> bool {
        self.identifier() == BUILTIN_VOID
    }

    #[inline]
    pub fn is_special_type(&self) -> bool {
        self.is_void() || self.is_never()
    }

    #[inline]
    pub fn mangled(&self) -> String {
        let ch = match self.identifier() {
            BUILTIN_BOOLEAN => 'b',
            BUILTIN_CHAR => 'w',
            BUILTIN_INT => 'i',
            BUILTIN_UINT => 'j',
            BUILTIN_ISIZE => 'l',
            BUILTIN_USIZE => 'm',
            _ => '\0',
        };
        if ch != '\0' {
            return ch.to_string();
        }
        match self.kind() {
            TypeKind::Primitive(pritive) => {
                let ch = match pritive {
                    Primitive::Bool => 'b',
                    Primitive::F32 => 'f',
                    Primitive::F64 => 'd',
                    Primitive::Void => 'v',
                    _ => '\0',
                };
                if ch != '\0' {
                    return ch.to_string();
                }
            }
            TypeKind::Reference(p) => return format!("P{}", p.mangled()),
            _ => (),
        }
        format!("{}{}", self.identifier.len(), self.identifier)
    }
}

impl PartialEq for TypeDescriptor {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

pub enum TypeKind {
    Primitive(Primitive),
    Alias(Arc<TypeDescriptor>),
    Reference(Arc<TypeDescriptor>),
    Optional(Arc<TypeDescriptor>),
    //Class(T),
    //Function(T),
}

impl core::fmt::Debug for TypeKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Primitive(arg0) => f.debug_tuple("Primitive").field(arg0).finish(),
            Self::Alias(arg0) => f.debug_tuple("Alias").field(&arg0.identifier()).finish(),
            Self::Reference(arg0) => f
                .debug_tuple("Reference")
                .field(&arg0.identifier())
                .finish(),
            Self::Optional(arg0) => f.debug_tuple("Optional").field(&arg0.identifier()).finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InferredType {
    Unknown,
    Inferred(Arc<TypeDescriptor>),
    Maybe(Arc<TypeDescriptor>),
}

impl InferredType {
    #[inline]
    pub fn from_type_opt(value: Option<&Arc<TypeDescriptor>>) -> Self {
        match value {
            Some(v) => Self::Inferred(v.clone()),
            None => Self::Unknown,
        }
    }

    #[inline]
    pub fn optimistic_type(&self) -> Option<&Arc<TypeDescriptor>> {
        match self {
            InferredType::Inferred(v) => Some(v),
            InferredType::Maybe(v) => Some(v),
            InferredType::Unknown => None,
        }
    }

    #[inline]
    pub fn strict_type(&self) -> Option<&Arc<TypeDescriptor>> {
        match self {
            InferredType::Inferred(v) => Some(v),
            InferredType::Maybe(_) => None,
            InferredType::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlobalObjectIndex {
    Funtion(FuncIndex),
}
