use super::Module;
use crate::*;
use ir::{
    code::FuncCode,
    export::ExportDesc,
    index::{FuncIndex, LocalIndex, TypeIndex},
    types::{IdType, Type},
    WasmSectionId,
};
use leb128::{Leb128Writer, WriteError, WriteLeb128};

#[derive(Default)]
pub struct Functions(pub(super) Vec<Func>);

impl Functions {
    pub(super) fn process_tir<'a>(
        module: &mut Module,
        functions: impl Iterator<Item = &'a toyir::Function>,
    ) -> Result<(), AssembleError> {
        let n_imports = module.imports.num_import_funcs();
        for tir_func in functions {
            let funcidx = FuncIndex((n_imports + module.funcs.0.len()) as u32);

            let mut local_ids = BTreeMap::default();

            module.register_name(
                tir_func.signature(),
                IdType::Func(funcidx),
                ErrorPosition::Unspecified,
            )?;

            let results = tir_func
                .results()
                .iter()
                .map(|v| v.primitive_type().wasm_binding().unwrap())
                .collect::<Vec<_>>();

            let mut local_and_params = Vec::new();
            for param in tir_func.params() {
                let valtype = param.primitive_type().wasm_binding().unwrap();
                local_and_params.push(valtype);
            }

            let type_use = Type::from_iter(
                local_and_params.iter().map(|v| *v),
                results.iter().map(|v| *v),
            );
            let typeidx = module.types.define(type_use)?;

            module.funcs.0.push(Func(typeidx));

            let mut locals = Vec::new();
            for local in tir_func.locals() {
                if let Some(identifier) = local.identifier() {
                    local_ids.insert(
                        identifier.to_owned(),
                        LocalIndex((local_and_params.len()) as u32),
                    );
                }
                let valtype = local.primitive_type().wasm_binding().unwrap();
                local_and_params.push(valtype);
                locals.push(valtype);
            }

            if let Some(export) = tir_func.exports() {
                module.exports.export(export, ExportDesc::Func(funcidx))?;
            }

            let code = FuncCode::new(
                results,
                locals,
                local_ids,
                local_and_params,
                wasm::code::Code::ToyIr(wasm::code::ToyIr(tir_func.codes().clone())),
            );
            module.codes.0.push(code);
        }

        Ok(())
    }

    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                writer.write(item.0.as_usize())?;
            }
        }
        Ok(WasmSectionId::Function)
    }
}

impl core::fmt::Debug for Functions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Func(pub TypeIndex);
