use super::keyword::ModifierFlag;
use crate::*;
use ast::{class::ClassDeclaration, expression::Expression};
use cg::scope::VariableDescriptor;
use toyir::LocalIndex;
use types::{
    function::FunctionDescriptor,
    index::{ClassIndex, FuncIndex},
    TypeDescriptor,
};

pub const CTOR_NAME: &str = ".ctor";

#[derive(Debug)]
pub struct ClassDescriptor {
    index: ClassIndex,
    modifiers: ModifierFlag,
    identifier: Identifier,
    super_class: Option<Arc<TypeDescriptor>>,
    members: BTreeMap<String, ClassMember>,
}

impl ClassDescriptor {
    pub fn parse(
        decl: &ClassDeclaration,
        types: &TypeSystem,
        class_idx: ClassIndex,
    ) -> Result<Self, CompileError> {
        let modifiers = decl.modifiers();

        let identifier = decl.identifier().clone();

        let super_class = if let Some(type_desc) = decl.super_class() {
            if let Some(v) = types.get(type_desc.as_str()) {
                Some(v.clone())
            } else {
                return Err(CompileError::identifier_not_found(&type_desc));
            }
        } else {
            None
        };

        let mut members = BTreeMap::<String, ClassMember>::new();
        for vars in decl.var_decls() {
            for var in vars.varibales() {
                let key = var.identifier().to_string();
                if members.get(&key).is_some() {
                    return Err(CompileError::duplicate_identifier(&var.identifier()));
                }
                let type_desc = if let Some(type_decl) = var.type_decl() {
                    if let Some(v) = types.from_ast(type_decl) {
                        if v.is_special_type() {
                            return Err(CompileError::invalid_type(type_decl.identifier()));
                        }
                        Some(v)
                    } else {
                        return Err(CompileError::identifier_not_found(type_decl.identifier()));
                    }
                } else {
                    None
                };
                let mut var_desc = VariableDescriptor::from_var_decl(var, type_desc.as_ref());
                var_desc.set_index(unsafe { LocalIndex::new(members.len() as u32) });
                members.insert(
                    key,
                    ClassMember::Field(var_desc, var.assignment().map(|v| v.clone())),
                );
            }
        }

        let prefix = identifier.to_string();
        let prefix = Some(prefix.as_str());
        for func in decl.functions() {
            let key = func.identifier().as_str();
            let key = match Keyword::from_str(key) {
                Some(Keyword::Constructor) => CTOR_NAME.to_owned(),
                _ => key.to_string(),
            };
            if members.get(&key).is_some() {
                return Err(CompileError::duplicate_identifier(&func.identifier()));
            }
            let func_desc =
                FunctionDescriptor::parse(prefix, Some(key.as_str()), func, types, unsafe {
                    FuncIndex::new(usize::MAX)
                })?;
            members.insert(key, ClassMember::Method(func_desc));
        }

        Ok(ClassDescriptor {
            index: class_idx,
            modifiers,
            identifier,
            super_class,
            members,
        })
    }

    #[inline]
    pub fn index(&self) -> ClassIndex {
        self.index
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn modifiers(&self) -> ModifierFlag {
        self.modifiers
    }

    #[inline]
    pub fn super_class(&self) -> Option<&Arc<TypeDescriptor>> {
        self.super_class.as_ref()
    }

    #[inline]
    pub fn members(&self) -> &BTreeMap<String, ClassMember> {
        &self.members
    }
}

#[derive(Debug)]
pub enum ClassMember {
    Field(VariableDescriptor, Option<Expression>),
    Method(Arc<FunctionDescriptor>),
}

impl ClassMember {
    #[inline]
    pub fn identifier(&self) -> &str {
        match self {
            ClassMember::Field(var_desc, _) => var_desc.identifier().as_str(),
            ClassMember::Method(func_desc) => func_desc.identifier(),
        }
    }
}
