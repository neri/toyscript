//! ToyScript Type System

use crate::*;
use ast::{class::TypeDeclaration, float::Float, integer::Integer, statement::Statement, Ast};
use class::ClassDescriptor;
use function::FunctionDescriptor;
use index::{ClassIndex, FuncIndex};
use keyword::ModifierFlag;
use string::{StringIndex, StringTable};
use token::TokenPosition;
use toyir::{FunctionAssembler, Primitive};

pub mod class;
pub mod function;
pub mod index;
pub mod string;
pub mod var;
// pub mod vtable;

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
    global_names: BTreeMap<String, GlobalNameIndex>,
    classes: Vec<Arc<ClassDescriptor>>,
    functions: Vec<Arc<FunctionDescriptor>>,
    types: RefCell<BTreeMap<String, Arc<TypeDescriptor>>>,
    string_table: StringTable,

    integer_bits: usize,
    pointer_bits: usize,
    type_int: Primitive,
    type_uint: Primitive,
    type_isize: Primitive,
    type_usize: Primitive,
}

impl TypeSystem {
    pub fn new(name: &str, ast: Ast) -> Result<Self, CompileError> {
        let mut system = Self {
            name: name.to_owned(),
            global_names: BTreeMap::new(),
            types: RefCell::new(BTreeMap::new()),
            functions: Vec::new(),
            classes: Vec::new(),
            string_table: StringTable::new(),

            integer_bits: 0,
            pointer_bits: 0,
            type_int: Primitive::Void,
            type_uint: Primitive::Void,
            type_isize: Primitive::Void,
            type_usize: Primitive::Void,
        };

        for primitive in Primitive::all_values() {
            // if primitive.is_float() {
            //     continue;
            // }
            let desc = TypeDescriptor::from_primitive(*primitive);
            system
                .types
                .borrow_mut()
                .insert(desc.identifier().to_string(), desc);
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
            .make_primitive_alias(BUILTIN_BOOLEAN, Primitive::U8)
            .unwrap();

        // system.make_reference(
        //     &Identifier::new(BUILTIN_HANDLE, TokenPosition::empty()),
        //     &system.builtin_void(),
        // )?;
        // system.make_simple_alias(BUILTIN_HANDLE, BUILTIN_USIZE)?;

        system.make_reference(&Identifier::new(BUILTIN_STRING), &system.builtin_void())?;

        let mut main_body = Vec::new();
        let mut func_decls = Vec::new();

        {
            let mut will_waiting_ids = Vec::new();
            let mut waiting_type_list = Vec::new();
            let mut waiting_class_list = Vec::new();

            for statement in ast.into_module() {
                match statement {
                    Statement::Eof(_) => {}

                    Statement::TypeAlias(identifier, type_desc) => {
                        will_waiting_ids.push(identifier.to_string());

                        match system.make_alias(&identifier, &type_desc) {
                            Ok(_) => (),
                            Err(err) => match err.kind() {
                                CompileErrorKind::IdentifierNotFound(_) => {
                                    waiting_type_list.push((identifier.clone(), type_desc.clone()));
                                }
                                _ => return Err(err),
                            },
                        }
                    }

                    Statement::Class(class_decl) => {
                        will_waiting_ids.push(class_decl.identifier().to_string());

                        match ClassDescriptor::parse(
                            &class_decl,
                            &system,
                            ClassIndex(system.classes.len() as u32),
                        ) {
                            Ok(class) => system.add_class(class)?,
                            Err(err) => match err.kind() {
                                CompileErrorKind::IdentifierNotFound(_) => {
                                    waiting_class_list.push(class_decl.clone());
                                }
                                _ => return Err(err),
                            },
                        }
                    }

                    Statement::Function(func_decl) => {
                        func_decls.push(func_decl);
                    }

                    Statement::Expression(ref expr) => {
                        if expr.is_empty() || expr.is_constant() {
                            // ignored
                        } else {
                            main_body.push(statement);
                        }
                    }

                    Statement::Block(_)
                    | Statement::Variable(_)
                    | Statement::IfStatement(_)
                    | Statement::ForStatement(_)
                    | Statement::WhileStatement(_, _)
                    | Statement::Break(_)
                    | Statement::Continue(_) => {
                        main_body.push(statement);
                    }

                    Statement::ReturnStatement(_expr, position) => {
                        return Err(CompileError::out_of_context("", position))
                    }

                    Statement::Enum(_) => {
                        return Err(CompileError::out_of_context(
                            format!("{:#?}", statement).as_str(),
                            TokenPosition::empty(),
                        ))
                    }
                }
            }

            let mut last_class_errors = Vec::new();

            // Attempt to resolve forward references
            for _ in 0..8 {
                let mut waiting_list2 = Vec::new();
                for item in waiting_type_list.iter().rev() {
                    let (identifier, type_desc) = item;
                    if !will_waiting_ids.contains(&type_desc.to_string()) {
                        return Err(CompileError::identifier_not_found(&type_desc.identifier()));
                    }
                    match system.make_alias(identifier, type_desc) {
                        Ok(_) => {}
                        Err(err) => match err.kind() {
                            CompileErrorKind::IdentifierNotFound(_) => {
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
                    if !will_waiting_ids.contains(&type_desc.to_string()) {
                        return Err(CompileError::identifier_not_found(&type_desc.identifier()));
                    }
                    match system.make_alias(identifier, type_desc) {
                        Ok(_) => {}
                        Err(err) => match err.kind() {
                            CompileErrorKind::IdentifierNotFound(_) => {
                                waiting_list2.push((identifier.clone(), type_desc.clone()));
                            }
                            _ => return Err(err),
                        },
                    }
                }
                waiting_type_list = waiting_list2;

                let mut waiting_list2 = Vec::new();
                for class_decl in waiting_class_list.iter().rev() {
                    match ClassDescriptor::parse(
                        &class_decl,
                        &system,
                        ClassIndex(system.classes.len() as u32),
                    ) {
                        Ok(class) => system.add_class(class)?,
                        Err(err) => match err.kind() {
                            CompileErrorKind::IdentifierNotFound(_) => {
                                waiting_list2.push(class_decl.clone());
                            }
                            _ => return Err(err),
                        },
                    }
                }
                waiting_class_list = waiting_list2;
                last_class_errors.clear();
                let mut waiting_list2 = Vec::new();
                for class_decl in waiting_class_list.iter() {
                    match ClassDescriptor::parse(
                        &class_decl,
                        &system,
                        ClassIndex(system.classes.len() as u32),
                    ) {
                        Ok(class) => system.add_class(class)?,
                        Err(err) => match err.kind() {
                            CompileErrorKind::IdentifierNotFound(_) => {
                                waiting_list2.push(class_decl.clone());
                                last_class_errors.push(err);
                            }
                            _ => return Err(err),
                        },
                    }
                }
                waiting_class_list = waiting_list2;
            }
            if let Some(item) = waiting_type_list.first() {
                return Err(CompileError::identifier_not_found_looping(
                    item.1.identifier(),
                ));
            }
            if let Some(item) = last_class_errors.first() {
                match item.kind() {
                    CompileErrorKind::IdentifierNotFound(id) => {
                        return Err(CompileError::identifier_not_found_looping(id));
                    }
                    _ => return Err(item.clone()),
                }
            }
        }

        // auto main
        if main_body.len() > 0 {
            let func = FunctionDescriptor::from_name_and_body(
                None,
                function::MAIN_FUNC_NAME,
                ModifierFlag::EXPORT,
                main_body,
                &system,
                FuncIndex(system.functions.len() as u32),
            )?;
            system.add_function(&func).unwrap();
        }

        for func_decl in &func_decls {
            let func = FunctionDescriptor::parse(
                None,
                None,
                func_decl,
                &system,
                FuncIndex(system.functions.len() as u32),
            )?;
            system
                .add_function(&func)
                .ok_or(CompileError::duplicate_identifier(func_decl.identifier()))?;
        }

        {
            let mut classes2 = Vec::new();
            core::mem::swap(&mut classes2, &mut system.classes);
            for class in &classes2 {
                for member in class.members() {
                    match member.1 {
                        class::ClassMember::Field(_, _) => {}
                        class::ClassMember::Method(func_desc) => {
                            func_desc.set_index(FuncIndex(system.functions.len() as u32));
                            system.add_function(func_desc).unwrap();
                        }
                    }
                }
            }
            system.classes = classes2;
        }

        {
            let func = FunctionDescriptor::intrinsic_inline_op(
                "unreachable",
                ModifierFlag::empty(),
                &[],
                &system.builtin_never(),
                toyir::Op::Unreachable,
                FuncIndex(system.functions.len() as u32),
            )?;
            system.add_function(&func).unwrap();
        }

        Ok(system)
    }

    pub fn add_class(&mut self, class: ClassDescriptor) -> Result<(), CompileError> {
        if self.get(class.identifier().as_str()).is_some() {
            Err(CompileError::duplicate_identifier(class.identifier()))
        } else {
            self.global_names.insert(
                class.identifier().to_string().clone(),
                GlobalNameIndex::Class(class.index()),
            );
            let class = Arc::new(class);
            self.classes.push(class.clone());
            let class_type = Arc::new(TypeDescriptor {
                identifier: class.identifier().to_string(),
                kind: TypeKind::Class(class.clone()),
            });
            self.types
                .borrow_mut()
                .insert(class.identifier().to_string(), class_type.clone());

            // let optional = Arc::new(TypeDescriptor::make_optional(&class_type));
            // self.types
            //     .borrow_mut()
            //     .insert(optional.identifier().to_string(), optional.clone());

            Ok(())
        }
    }

    pub fn add_function(&mut self, func: &Arc<FunctionDescriptor>) -> Option<()> {
        if self.global_name(func.identifier()).is_none() {
            self.global_names.insert(
                func.identifier().to_string().clone(),
                GlobalNameIndex::Function(func.index()),
            );
            self.functions.push(func.clone());
            Some(())
        } else {
            None
        }
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
    pub fn get(&self, identifier: &str) -> Option<Arc<TypeDescriptor>> {
        self.types.borrow().get(identifier).map(|v| v.clone())
    }

    #[inline]
    pub fn from_ast(&self, ast_type: &TypeDeclaration) -> Option<Arc<TypeDescriptor>> {
        match ast_type {
            TypeDeclaration::Simple(id) => self.get(&id.to_string()),
            TypeDeclaration::Optional(id) => {
                if let Some(v) = self.get(&ast_type.to_string()) {
                    return Some(v);
                }
                let mut types = self.types.borrow_mut();
                let Some(target) = types.get(id.as_str()) else {
                    return None;
                };
                let type_desc = Arc::new(TypeDescriptor::make_optional(target));
                types.insert(type_desc.identifier().to_string(), type_desc.clone());
                drop(types);
                Some(type_desc)
            }
            TypeDeclaration::TypeParameter(_) => None,
        }
    }

    #[inline]
    pub fn primitive_type(&self, primitive: Primitive) -> Arc<TypeDescriptor> {
        self.get(primitive.as_str()).unwrap()
    }

    #[inline]
    pub fn builtin_boolean(&self) -> Arc<TypeDescriptor> {
        self.get(BUILTIN_BOOLEAN).unwrap()
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
    pub fn storage_type(&self, type_desc: &Arc<TypeDescriptor>) -> Primitive {
        let mut target = type_desc;
        loop {
            match target.kind() {
                TypeKind::Primitive(primitive) => return *primitive,
                TypeKind::Alias(arc) => target = arc,
                TypeKind::Reference(_)
                | TypeKind::Class(_)
                | TypeKind::Function(_)
                | TypeKind::Optional(_) => return self.type_usize,
            }
        }
    }

    #[inline]
    fn make_primitive_alias(&mut self, identifier: &str, primitive: Primitive) -> Result<(), ()> {
        if self.get(identifier).is_some() {
            return Err(());
        }
        let type_desc = TypeDescriptor::make_alias(identifier, &self.primitive_type(primitive));
        self.types
            .borrow_mut()
            .insert(identifier.to_string(), Arc::new(type_desc));
        Ok(())
    }

    #[inline]
    fn _make_simple_alias(&mut self, identifier: &str, target: &str) -> Result<(), CompileError> {
        self.make_alias(
            &Identifier::new(identifier),
            &TypeDeclaration::Simple(Identifier::new(target)),
        )
    }

    pub fn make_alias(
        &mut self,
        identifier: &Identifier,
        type_decl: &TypeDeclaration,
    ) -> Result<(), CompileError> {
        if self.get(identifier.as_str()).is_some() {
            return Err(CompileError::duplicate_identifier(identifier));
        }
        if let Some(desc) = self.from_ast(type_decl) {
            if desc.is_special_type() {
                return Err(CompileError::invalid_type(&type_decl.identifier()));
            }
            let desc = TypeDescriptor::make_alias(identifier.as_str(), &desc);
            self.types
                .borrow_mut()
                .insert(identifier.to_string(), Arc::new(desc));
        } else {
            return Err(CompileError::identifier_not_found(&type_decl.identifier()));
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
        self.types
            .borrow_mut()
            .insert(identifier.to_string(), Arc::new(desc));

        Ok(())
    }

    #[inline]
    pub fn prefixed_identifier(
        prefix: Option<&str>,
        identifier: &str,
        type_params: &[&str],
    ) -> String {
        if type_params.is_empty() {
            if let Some(prefix) = prefix {
                format!("{}:{}", prefix, identifier)
            } else {
                identifier.to_owned()
            }
        } else {
            if let Some(prefix) = prefix {
                format!("{}:{}:<{}>", prefix, identifier, type_params.join(","))
            } else {
                format!("{}:<{}>", identifier, type_params.join(","))
            }
        }
    }

    #[inline]
    pub fn mangled(prefix: Option<&str>, identifier: &str, type_params: &[&str]) -> String {
        Self::mangled2(&Self::prefixed_identifier(prefix, identifier, type_params))
    }

    #[inline]
    pub fn mangled2(identifier: &str) -> String {
        format!("${}", identifier,)
    }

    pub fn type_cast_func_identifier(old_type: &str, new_type: &str) -> String {
        Self::prefixed_identifier(Some(new_type), class::CAST_NAME, &[old_type])
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
                            return Err(CompileError::literal_overflow(
                                inferred_to.identifier(),
                                position,
                            ))
                        }
                    }
                }
                Err(CompileError::type_mismatch2(
                    &inferred_to.identifier(),
                    value.primitive_type().as_str(),
                    position,
                ))
            }
            InferredType::Maybe(inferred_to) => {
                if let Some(primitive) = inferred_to.primitive_type() {
                    match value.try_convert_to(primitive) {
                        Ok(v) => return Ok((v, InferredType::Maybe(inferred_to.clone()))),
                        Err(_) => {
                            return Err(CompileError::literal_overflow(
                                inferred_to.identifier(),
                                position,
                            ))
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

    pub fn infer_float(
        &self,
        value: &Float,
        inferred_to: &InferredType,
        position: TokenPosition,
    ) -> Result<(Float, InferredType), CompileError> {
        match inferred_to {
            InferredType::Inferred(inferred_to) => {
                if let Some(primitive) = inferred_to.primitive_type() {
                    match value.try_convert_to(primitive) {
                        Ok(v) => return Ok((v, InferredType::Inferred(inferred_to.clone()))),
                        Err(_) => {
                            return Err(CompileError::literal_overflow(
                                inferred_to.identifier(),
                                position,
                            ))
                        }
                    }
                }
                Err(CompileError::type_mismatch2(
                    &inferred_to.identifier(),
                    value.primitive_type().as_str(),
                    position,
                ))
            }
            InferredType::Maybe(inferred_to) => {
                if let Some(primitive) = inferred_to.primitive_type() {
                    match value.try_convert_to(primitive) {
                        Ok(v) => return Ok((v, InferredType::Maybe(inferred_to.clone()))),
                        Err(_) => {
                            return Err(CompileError::literal_overflow(
                                inferred_to.identifier(),
                                position,
                            ))
                        }
                    }
                }
                Ok((value.clone(), InferredType::Maybe(inferred_to.clone())))
            }
            InferredType::Unknown => self.infer_float(
                &value,
                &InferredType::Maybe(self.builtin_number()),
                position,
            ),
        }
    }

    pub fn try_convert_type(
        &self,
        asm: Option<&mut FunctionAssembler>,
        from: &InferredType,
        target: &Arc<TypeDescriptor>,
        position: TokenPosition,
    ) -> Result<bool, CompileError> {
        let Some(src_type) = from.optimistic_type() else {
            return Err(CompileError::could_not_infer2(position));
        };

        if let Some(conv_func) = self.function(&Self::type_cast_func_identifier(
            src_type.identifier(),
            target.identifier(),
        )) {
            if let Some(asm) = asm {
                match conv_func.body() {
                    function::FunctionBody::Block(_) => {
                        asm.ir_call(conv_func.index().as_usize(), 1, 1)?;
                    }
                    function::FunctionBody::Inline(emitter) => {
                        emitter(asm)?;
                    }
                }
            }
            return Ok(true);
        }

        let Some(src_type) = src_type.primitive_type().filter(|v| *v != Primitive::Void) else {
            return Err(CompileError::cast_error(
                src_type.identifier(),
                target.identifier(),
                position,
            ));
        };
        let Some(dest_type) = target.primitive_type().filter(|v| *v != Primitive::Void) else {
            return Err(CompileError::cast_error(
                src_type.as_str(),
                target.identifier(),
                position,
            ));
        };

        if let Some(asm) = asm {
            if src_type != dest_type
                && (src_type.bits_of() != dest_type.bits_of()
                    || src_type.is_float()
                    || dest_type.is_float())
            {
                asm.ir_cast(dest_type.type_id(), src_type.type_id())?;
            }
        }

        Ok(true)
    }

    #[inline]
    pub fn canonical(&self, ty: Option<&Arc<TypeDescriptor>>) -> Arc<TypeDescriptor> {
        ty.map(|v| v.clone()).unwrap_or(self.builtin_void())
    }

    pub fn global_name(&self, identifier: &str) -> Option<GlobalNameIndex> {
        self.global_names.get(&identifier.to_string()).map(|v| *v)
    }

    #[inline]
    pub fn functions(&self) -> &[Arc<FunctionDescriptor>] {
        &self.functions
    }

    pub fn function(&self, identifier: &str) -> Option<&Arc<FunctionDescriptor>> {
        let index = match self.global_name(identifier)? {
            GlobalNameIndex::Function(v) => v,
            _ => return None,
        };
        for func in &self.functions {
            if func.index() == index {
                return Some(func);
            }
        }
        None
    }

    #[inline]
    pub fn classes(&self) -> &[Arc<ClassDescriptor>] {
        &self.classes
    }

    pub fn class(&self, identifier: &str) -> Option<&Arc<ClassDescriptor>> {
        let index = match self.global_name(identifier)? {
            GlobalNameIndex::Class(v) => v,
            _ => return None,
        };
        for class in &self.classes {
            if class.index() == index {
                return Some(class);
            }
        }
        None
    }

    #[inline]
    pub fn register_string(&mut self, s: &str) -> StringIndex {
        self.string_table.register(s)
    }

    #[inline]
    pub fn find_string(&self, s: &str) -> Option<StringIndex> {
        self.string_table.find(s)
    }

    #[inline]
    pub fn get_string(&self, index: StringIndex) -> &str {
        self.string_table.get_string(index)
    }
}

pub trait ResolvePrimitive<T: ?Sized> {
    fn resolve_primitive(&self, v: &T) -> Option<Primitive>;
}

impl ResolvePrimitive<TypeDeclaration> for TypeSystem {
    fn resolve_primitive(&self, desc: &TypeDeclaration) -> Option<Primitive> {
        self.get(&desc.to_string())
            .and_then(|ref v| self.resolve_primitive(v))
    }
}

impl ResolvePrimitive<str> for TypeSystem {
    fn resolve_primitive(&self, identifier: &str) -> Option<Primitive> {
        self.get(identifier)
            .and_then(|ref v| self.resolve_primitive(v))
    }
}

impl ResolvePrimitive<Arc<TypeDescriptor>> for TypeSystem {
    fn resolve_primitive(&self, desc: &Arc<TypeDescriptor>) -> Option<Primitive> {
        let mut desc = desc;
        loop {
            match desc.kind() {
                TypeKind::Primitive(primitive) => return Some(*primitive),
                TypeKind::Alias(alias) => desc = alias,
                TypeKind::Reference(_)
                | TypeKind::Class(_)
                | TypeKind::Function(_)
                | TypeKind::Optional(_) => return None,
            }
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
    fn make_optional(target: &Arc<TypeDescriptor>) -> Self {
        Self {
            identifier: format!("{}?", target.identifier()),
            kind: TypeKind::Optional(target.clone()),
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
    Class(Arc<ClassDescriptor>),
    Function(Arc<()>),
    Optional(Arc<TypeDescriptor>),
}

impl core::fmt::Debug for TypeKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Primitive(arg0) => write!(f, "Primitive({})", arg0),
            Self::Alias(arg0) => write!(f, "Alias({})", arg0.identifier()),
            Self::Reference(arg0) => write!(f, "Ref({})", arg0.identifier()),
            Self::Class(arg0) => write!(f, "Class({})", arg0.identifier().as_str()),
            // Self::Function(arg0) => write!(f, "Function({})", arg0.identifier().as_str()),
            Self::Function(arg0) => write!(f, "Function({:?})", arg0),
            Self::Optional(arg0) => write!(f, "Optional({})", arg0.identifier()),
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlobalNameIndex {
    Function(FuncIndex),
    Class(ClassIndex),
}

impl core::fmt::Debug for GlobalNameIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Function(arg0) => (arg0 as &dyn core::fmt::Debug).fmt(f),
            Self::Class(arg0) => (arg0 as &dyn core::fmt::Debug).fmt(f),
        }
    }
}
