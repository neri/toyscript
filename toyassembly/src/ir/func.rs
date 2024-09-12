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
    pub(super) fn convert(
        module: &mut Module,
        functions: Vec<ast::function::Function>,
    ) -> Result<(), AssembleError> {
        let n_imports = module.imports.num_import_funcs();
        for ast_func in functions {
            if matches!(ast_func.vis(), Some(ast::types::ExtVis::Import(_))) {
                continue;
            }
            let ast::function::Function {
                id,
                vis,
                typeuse,
                locals: ast_locals,
                instr,
            } = ast_func;

            let funcidx = FuncIndex((n_imports + module.funcs.0.len()) as u32);

            let mut local_ids = BTreeMap::default();

            if let Some(ref id) = id {
                module.register_ast_name(id, IdType::Func(funcidx))?;
            }

            let typeidx = module.define_typeuse(&typeuse)?;
            match typeuse.kind() {
                ast::types::TypeUseKind::Index(_) => {}
                ast::types::TypeUseKind::FuncType(functype) => {
                    for (index, param) in functype.params().iter().enumerate() {
                        if let Some(id) = param.identifier() {
                            Self::define_local(&mut local_ids, id, LocalIndex(index as u32))?;
                        }
                    }
                }
                ast::types::TypeUseKind::Both(_idx, functype) => {
                    for (index, param) in functype.params().iter().enumerate() {
                        if let Some(id) = param.identifier() {
                            Self::define_local(&mut local_ids, id, LocalIndex(index as u32))?;
                        }
                    }
                }
            }

            let func_type = module.get_type(typeidx);
            let results = func_type.results().to_vec();
            let mut local_and_params = func_type.params().iter().map(|v| *v).collect::<Vec<_>>();
            let mut locals = Vec::new();
            for local in ast_locals.iter() {
                if let Some(id) = local.identifier() {
                    Self::define_local(
                        &mut local_ids,
                        id,
                        LocalIndex((local_and_params.len()) as u32),
                    )?;
                }
                local_and_params.push(local.valtype());
                locals.push(local.valtype());
            }

            if let Some(vis) = vis {
                match vis {
                    ast::types::ExtVis::Import(_) => {}
                    ast::types::ExtVis::Export(export) => {
                        module
                            .exports
                            .export(export.name(), ExportDesc::Func(funcidx))?;
                    }
                }
            }

            module.funcs.0.push(Func(typeidx));

            let code = FuncCode::new(results, locals, local_ids, local_and_params, instr);
            module.codes.0.push(code);
        }

        Ok(())
    }

    pub(super) fn process_tir(
        module: &mut Module,
        functions: &[toyir::Function],
    ) -> Result<(), AssembleError> {
        let n_imports = module.imports.num_import_funcs();
        for tir_func in functions {
            let funcidx = FuncIndex((n_imports + module.funcs.0.len()) as u32);

            let mut local_ids = BTreeMap::default();

            let id =
                ast::identifier::Identifier::from_str(tir_func.signature(), TokenPosition::empty())
                    .unwrap();

            module.register_ast_name(&id, IdType::Func(funcidx))?;

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

    fn define_local(
        local_ids: &mut BTreeMap<String, LocalIndex>,
        id: &ast::identifier::Identifier,
        value: LocalIndex,
    ) -> Result<(), AssembleError> {
        let key = id.name().to_owned();
        if local_ids.get(&key).is_some() {
            return Err(AssembleError::duplicated_identifier(id));
        }
        local_ids.insert(key, value);
        Ok(())
    }
}

impl core::fmt::Debug for Functions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Func(pub TypeIndex);
