//! ToyAssembly Intermediate Representation

use crate::*;
use code::Codes;
use data::DataSegments;
use elem::Elems;
use export::Exports;
use func::Functions;
use global::Globals;
use import::{Import, Imports};
use index::{FuncIndex, GlobalIndex, TypeIndex};
use leb128::{Leb128Writer, WriteError, WriteLeb128};
use memory::Memories;
use table::Tables;
use types::{IdType, Type, Types};

pub mod code;
pub mod data;
pub mod elem;
pub mod export;
pub mod func;
pub mod global;
pub mod import;
pub mod index;
pub mod memory;
pub mod table;
pub mod types;

#[derive(Debug, Default)]
pub struct Module {
    pub(self) types: Types,
    pub(self) imports: Imports,
    pub(self) funcs: Functions,
    pub(self) tables: Tables,
    pub(self) memories: Memories,
    pub(self) globals: Globals,
    pub(self) exports: Exports,
    pub(self) start: Option<FuncIndex>,
    pub(self) elems: Elems,
    pub(self) codes: Codes,
    pub(self) data_segs: DataSegments,
    names: BTreeMap<String, IdType>,
}

impl Module {
    /// Magic number of WebAssembly Binary Format
    pub const WASM_BINARY_MAGIC: [u8; 4] = *b"\0asm";
    /// Current version number is 1
    pub const WASM_BINARY_VERSION: [u8; 4] = *b"\x01\0\0\0";

    pub fn from_ast(ast_module: ast::AstModule) -> Result<Self, AssembleError> {
        let mut module = Module {
            ..Default::default()
        };

        for item in ast_module.types() {
            let new_item = Type::from_ast(item.params(), item.results());
            if let Some(id) = item.identifier() {
                module
                    .register_ast_name(id, IdType::Type(TypeIndex(module.types.0.len() as u32)))?;
            }
            module.types.0.push(new_item);
        }

        Imports::convert(&mut module, &ast_module)?;

        let ast::AstModule {
            types: _,
            imports: _,
            functions,
            tables,
            memories,
            globals,
            exports,
            start,
            elems,
            data_segments,
        } = ast_module;

        Functions::convert(&mut module, functions)?;
        Tables::convert(&mut module, tables)?;
        Memories::convert(&mut module, memories)?;
        Globals::convert(&mut module, globals)?;
        Exports::convert(&mut module, exports)?;

        if let Some(start) = start {
            module.start = Some(module.get_funcidx(&start.0)?);
        }

        Elems::convert(&mut module, elems)?;
        DataSegments::convert(&mut module, data_segments)?;

        module.assemble()?;

        Ok(module)
    }

    pub fn from_toyir(tir_module: toyir::Module) -> Result<Self, AssembleError> {
        let mut module = Module {
            ..Default::default()
        };

        Functions::process_tir(&mut module, tir_module.functions())?;

        module.assemble()?;

        Ok(module)
    }

    pub fn assemble(&mut self) -> Result<(), AssembleError> {
        let mut codes = self.codes.drain();
        for code in codes.iter_mut() {
            code.assemble(&self)?;
        }
        self.codes.0.extend(codes);

        Ok(())
    }

    pub fn write_to_wasm(&self) -> Result<Vec<u8>, WriteError> {
        let mut writer = Leb128Writer::new();
        writer.write_bytes(&Self::WASM_BINARY_MAGIC)?;
        writer.write_bytes(&Self::WASM_BINARY_VERSION)?;

        fn write<F>(writer: &mut Leb128Writer, kernel: F) -> Result<(), WriteError>
        where
            F: FnOnce(&mut Leb128Writer) -> Result<WasmSectionId, WriteError>,
        {
            let mut payload = Leb128Writer::new();
            let section_id = kernel(&mut payload)?;
            if payload.len() > 0 {
                writer.write_byte(section_id as u8)?;
                writer.write_blob(payload.as_slice())?;
            }
            Ok(())
        }

        write(&mut writer, |writer| self.types.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.imports.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.funcs.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.tables.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.memories.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.globals.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.exports.write_to_wasm(writer))?;

        write(&mut writer, |writer| {
            if let Some(start) = self.start {
                writer.write(start.as_usize())?;
            }
            Ok(WasmSectionId::Start)
        })?;

        write(&mut writer, |writer| self.elems.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.codes.write_to_wasm(writer))?;
        write(&mut writer, |writer| self.data_segs.write_to_wasm(writer))?;
        write(&mut writer, |writer| {
            self.write_name_section_to_wasm(writer)
        })?;

        Ok(writer.into_vec())
    }

    fn write_name_section_to_wasm(
        &self,
        writer: &mut Leb128Writer,
    ) -> Result<WasmSectionId, WriteError> {
        let mut func_names = self
            .names
            .iter()
            .filter_map(|(name, item)| match item {
                IdType::Func(v) => Some((*v, name.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        func_names.sort();

        if func_names.len() > 0 {
            writer.write("name")?;
            let mut subsec = Leb128Writer::new();
            subsec.write(func_names.len())?;
            for item in func_names {
                subsec.write(item.0.as_usize())?;
                subsec.write(&item.1)?;
            }
            writer.write_byte(1)?;
            writer.write_blob(subsec.as_slice())?;
        }
        Ok(WasmSectionId::Custom)
    }

    #[inline]
    pub fn types(&self) -> &[Type] {
        &self.types.0
    }

    #[inline]
    pub fn get_type(&self, index: TypeIndex) -> &Type {
        &self.types.0[index.as_usize()]
    }

    #[inline]
    pub fn imports(&self) -> &[Import] {
        &self.imports.0
    }

    pub fn define_typeuse(
        &mut self,
        ast_typeuse: &ast::types::TypeUse,
    ) -> Result<TypeIndex, AssembleError> {
        match ast_typeuse.kind() {
            ast::types::TypeUseKind::Index(typeidx) => self.get_typeidx(&typeidx.0),
            ast::types::TypeUseKind::FuncType(func_type) => {
                let new_item = Type::from_ast(func_type.params(), func_type.results());
                self.types.define(new_item)
            }
            ast::types::TypeUseKind::Both(_typeidx, func_type) => {
                let new_item = Type::from_ast(func_type.params(), func_type.results());
                self.types.define(new_item)
            }
        }
    }

    pub fn find_typeuse(
        &self,
        ast_typeuse: &ast::types::TypeUse,
    ) -> Result<TypeIndex, AssembleError> {
        match ast_typeuse.kind() {
            ast::types::TypeUseKind::Index(typeidx) => self.get_typeidx(&typeidx.0),
            ast::types::TypeUseKind::FuncType(func_type) => {
                let item = Type::from_ast(func_type.params(), func_type.results());
                self.types
                    .find(item)
                    .ok_or(AssembleError::undefined_identifier(
                        ast_typeuse.token().source::<()>(),
                        ast_typeuse.token().position().into(),
                    ))
            }
            ast::types::TypeUseKind::Both(_typeidx, func_type) => {
                //
                let item = Type::from_ast(func_type.params(), func_type.results());
                self.types
                    .find(item)
                    .ok_or(AssembleError::undefined_identifier(
                        ast_typeuse.token().source::<()>(),
                        ast_typeuse.token().position().into(),
                    ))
            }
        }
    }

    pub fn register_ast_name(
        &mut self,
        id: &ast::identifier::Identifier,
        id_type: IdType,
    ) -> Result<(), AssembleError> {
        let key = id.name();

        if self.names.get(key).is_some() {
            return Err(AssembleError::duplicated_identifier(id));
        }

        self.names.insert(key.to_owned(), id_type);

        Ok(())
    }

    pub fn get_typeidx(
        &self,
        index: &ast::identifier::IndexToken,
    ) -> Result<TypeIndex, AssembleError> {
        match index {
            ast::identifier::IndexToken::Num(num) => {
                let typeidx = num.get();
                AssembleError::check_index(
                    typeidx,
                    0..(self.types.0.len() as u32),
                    num.position().into(),
                )?;
                Ok(TypeIndex(typeidx))
            }
            ast::identifier::IndexToken::Id(id) => match self.get(id.name()) {
                Some(IdType::Type(v)) => Ok(v),
                _ => Err(AssembleError::undefined_identifier(
                    id.name(),
                    id.position().into(),
                )),
            },
        }
    }

    pub fn get_funcidx(
        &self,
        id: &ast::identifier::IndexToken,
    ) -> Result<FuncIndex, AssembleError> {
        match id {
            ast::identifier::IndexToken::Num(num) => {
                let funcidx = num.get();
                AssembleError::check_index(
                    funcidx,
                    0..(self.max_func_len() as u32),
                    num.position().into(),
                )?;
                Ok(FuncIndex(funcidx))
            }
            ast::identifier::IndexToken::Id(id) => match self.get(id.name()) {
                Some(IdType::Func(v)) => Ok(v),
                _ => Err(AssembleError::undefined_identifier(
                    id.name(),
                    id.position().into(),
                )),
            },
        }
    }

    pub fn get_globalidx(
        &self,
        id: &ast::identifier::IndexToken,
    ) -> Result<GlobalIndex, AssembleError> {
        match id {
            ast::identifier::IndexToken::Num(num) => {
                let globalidx = num.get();
                AssembleError::check_index(
                    globalidx,
                    0..(self.max_global_len() as u32),
                    num.position().into(),
                )?;
                Ok(GlobalIndex(globalidx))
            }
            ast::identifier::IndexToken::Id(id) => match self.get(id.name()) {
                Some(IdType::Global(v)) => Ok(v),
                _ => Err(AssembleError::undefined_identifier(
                    id.name(),
                    id.position().into(),
                )),
            },
        }
    }

    #[inline]
    pub fn get(&self, id: &str) -> Option<IdType> {
        self.names.get(id).map(|v| *v)
    }

    #[inline]
    pub fn max_func_len(&self) -> usize {
        self.imports.num_import_funcs() + self.funcs.0.len()
    }

    #[inline]
    pub fn max_global_len(&self) -> usize {
        self.imports.num_import_globals() + self.globals.0.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmSectionId {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
}
