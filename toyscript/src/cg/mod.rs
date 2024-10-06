//! ToyScript Code Generator

pub mod class;
pub mod function;
pub mod scope;

use crate::*;
use function::FunctionGenerator;
use keyword::ModifierFlag;
use toyir::{self, FuncTempIndex, Import};
use types::function::FunctionBody;

pub struct CodeGen;

impl CodeGen {
    pub fn generate(types: &TypeSystem) -> Result<toyir::Module, CompileError> {
        let mut module = toyir::Module::new(types.name());

        for func_desc in types.functions() {
            if let Some(import_from) = func_desc.import_from() {
                let mut params = Vec::new();
                for (_id, param) in func_desc.params() {
                    let primitive_type = types.storage_type(param);
                    params.push(primitive_type);
                }

                let mut results = Vec::new();
                let result_type = func_desc.result_type();
                if !result_type.is_special_type() {
                    let primitive_type = types.storage_type(result_type);
                    results.push(primitive_type);
                }

                module.add_import(Import::func(
                    FuncTempIndex::new(func_desc.index().as_u32()),
                    func_desc.signature(),
                    import_from.1,
                    import_from.0,
                    &params,
                    &results,
                    func_desc.modifiers().contains(ModifierFlag::EXPORT),
                ));
            } else {
                if let FunctionBody::Block(ref body) = func_desc.body() {
                    let function = FunctionGenerator::generate(func_desc, body, types)?;
                    module.add_function(function);
                }
            }
        }

        module.optimize().map_err(|_e| {
            CompileError::internal_inconsistency(
                &format!("internal error"),
                ErrorPosition::Unspecified,
            )
        })?;

        Ok(module)
    }
}
