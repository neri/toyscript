use crate::*;
use ir::{
    global::GlobalType,
    index::{MemoryIndex, TableIndex, TypeIndex},
    Module, WasmSectionId,
};
use leb128::{Leb128Writer, WriteError, WriteLeb128};

#[derive(Default)]
pub struct Imports(pub(super) Vec<Import>);

impl Imports {
    pub(super) fn process_tir(
        module: &mut Module,
        imports: &[toyir::Import],
    ) -> Result<(), AssembleError> {
        for tir_import in imports {
            match tir_import.import_desc() {
                toyir::ImportDescriptor::Function(import_func) => {
                    let type_use = ir::Type::from_iter(
                        import_func
                            .params()
                            .iter()
                            .map(|v| v.wasm_binding().unwrap()),
                        import_func
                            .results()
                            .iter()
                            .map(|v| v.wasm_binding().unwrap()),
                    );
                    let typeidx = module.types.define(type_use)?;

                    module.import_funcs.push(typeidx);
                    module.imports.0.push(Import {
                        mod_name: tir_import.from().to_owned(),
                        name: tir_import.name().to_owned(),
                        desc: ImportDesc::Func(typeidx),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn write_to_wasm(&self, writer: &mut Leb128Writer) -> Result<WasmSectionId, WriteError> {
        if self.0.len() > 0 {
            writer.write(self.0.len())?;
            for item in self.0.iter() {
                writer.write(&item.mod_name)?;
                writer.write(&item.name)?;
                match item.desc {
                    ImportDesc::Func(v) => {
                        writer.write(0)?;
                        writer.write(v.as_usize())?;
                    }
                    ImportDesc::Table(v) => {
                        writer.write(1)?;
                        writer.write(v.as_usize())?;
                    }
                    ImportDesc::Memory(v) => {
                        writer.write(2)?;
                        writer.write(v.as_usize())?;
                    }
                    ImportDesc::Global(v) => {
                        writer.write(3)?;
                        v.write_to_wasm(writer)?;
                    }
                }
            }
        }
        Ok(WasmSectionId::Import)
    }

    pub fn num_import_funcs(&self) -> usize {
        self.0
            .iter()
            .filter(|v| matches!(v.desc, ImportDesc::Func(_)))
            .count()
    }

    pub fn num_import_globals(&self) -> usize {
        self.0
            .iter()
            .filter(|v| matches!(v.desc, ImportDesc::Global(_)))
            .count()
    }
}

impl core::fmt::Debug for Imports {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

#[derive(Debug)]
pub struct Import {
    pub mod_name: String,
    pub name: String,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone, Copy)]
pub enum ImportDesc {
    Func(TypeIndex),
    Table(TableIndex),
    Memory(MemoryIndex),
    Global(GlobalType),
}
