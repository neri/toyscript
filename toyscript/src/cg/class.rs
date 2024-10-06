use super::{types::class::ClassDescriptor, CompileError, TypeSystem};
use crate::*;
use cg::{function::FunctionGenerator, scope::VariableStorage};

pub struct ClassGenerator {
    //
}

impl ClassGenerator {
    pub fn generate(types: &TypeSystem, class_desc: &ClassDescriptor) -> Result<(), CompileError> {
        let var_storage = VariableStorage::new(types);
        let scope = var_storage.root_scope();

        let mut fields = Vec::new();
        for member in class_desc.members() {
            match member.1 {
                crate::types::class::ClassMember::Method(_) => continue,
                crate::types::class::ClassMember::Field(var_desc, expr) => {
                    let mut var_desc = var_desc.clone();
                    let expected_type = var_desc.inferred_type();
                    let result_type = if let Some(expr) = expr {
                        let (_item, result_type) = FunctionGenerator::infer_expression(
                            expr,
                            expected_type.optimistic_type(),
                            &scope,
                        )?;
                        result_type
                            .optimistic_type()
                            .map(|v| var_desc.infer(v, expr.position()));
                        result_type
                    } else {
                        expected_type.clone()
                    };
                    if result_type.optimistic_type().is_none() {
                        return Err(CompileError::could_not_infer(var_desc.identifier()));
                    }
                    fields.push(var_desc);
                }
            }
        }

        todo!()
    }
}
