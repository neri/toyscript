use super::*;

#[derive(Debug)]
pub struct Module {
    name: String,
    imports: Vec<Import>,
    functions: BTreeMap<FuncTempIndex, Function>,
    func_indexes: BTreeMap<FuncTempIndex, u32>,
}

impl Module {
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            imports: Vec::new(),
            functions: BTreeMap::new(),
            func_indexes: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn add_function(&mut self, function: Function) {
        self.functions.insert(function.function_index(), function);
    }

    #[inline]
    pub fn add_import(&mut self, import: Import) {
        self.imports.push(import);
    }

    #[inline]
    pub fn functions(&self) -> alloc::collections::btree_map::Values<FuncTempIndex, Function> {
        self.functions.values()
    }

    #[inline]
    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    #[inline]
    pub fn func_indexes(&self) -> impl Iterator<Item = (&FuncTempIndex, &u32)> {
        self.func_indexes.iter()
    }

    pub fn rebuild_function_indexes(&mut self) {
        self.func_indexes.clear();

        let mut new_index = 0;

        for item in &self.imports {
            match item.import_desc() {
                ImportDescriptor::Function(v) => {
                    self.func_indexes.insert(v.function_index(), new_index);
                    new_index += 1;
                }
            }
        }
        for item in self.functions.values() {
            self.func_indexes.insert(item.function_index(), new_index);
            new_index += 1;
        }
    }

    pub fn optimize(&mut self) -> Result<(), ()> {
        // Delete unreferenced functions
        let mut retain_list = Vec::with_capacity(self.functions.len() + self.imports.len());
        let mut unprocessed_list = Vec::with_capacity(self.functions.len());
        fn add_retain(vec: &mut Vec<FuncTempIndex>, target: FuncTempIndex) {
            if !vec.contains(&target) {
                vec.push(target);
            }
        }
        for item in &self.imports {
            match item.import_desc() {
                ImportDescriptor::Function(func) => {
                    if item.is_external() {
                        add_retain(&mut retain_list, func.function_index());
                    }
                }
            }
        }
        for item in self.functions.values() {
            if item.exports().is_some() {
                add_retain(&mut retain_list, item.function_index());
                for item in item.dependencies() {
                    add_retain(&mut retain_list, *item);
                }
            } else {
                unprocessed_list.push(item.function_index());
            }
        }
        loop {
            let mut processed_list = Vec::with_capacity(unprocessed_list.len());
            for item in &unprocessed_list {
                if retain_list.contains(item) {
                    let func = self.functions.get(item).unwrap();
                    add_retain(&mut retain_list, func.function_index());
                    for func in func.dependencies() {
                        add_retain(&mut retain_list, *func);
                    }
                    processed_list.push(*item);
                }
            }
            unprocessed_list.retain(|v| !processed_list.contains(v));

            let mut exit_flag = true;
            for item in &unprocessed_list {
                if retain_list.contains(item) {
                    exit_flag = false;
                    break;
                }
            }
            if exit_flag {
                break;
            }
        }
        self.functions.retain(|k, _v| retain_list.contains(&k));
        self.imports.retain(|v| match v.import_desc() {
            ImportDescriptor::Function(v) => retain_list.contains(&v.function_index()),
        });

        self.rebuild_function_indexes();

        Ok(())
    }
}
