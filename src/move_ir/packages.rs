use std::collections::BTreeMap;

use move_binary_format::CompiledModule;
use move_model::{ast::ModuleName, model::FunId};

use super::generate_bytecode::{StacklessBytecodeGenerator, FunctionInfo};


pub struct Packages<'a, 'b>{
    packages: BTreeMap<String, &'b StacklessBytecodeGenerator<'a>>,
}

impl<'a, 'b> Packages<'a, 'b> {
    pub fn new() -> Self {
        Packages { 
            packages: BTreeMap::new(),
        }
    }

    pub fn insert_stbgr(&mut self, stbgr: &'b StacklessBytecodeGenerator<'a>) {
        let mname = stbgr.module_data.name.clone();
        let mname = mname.display(&stbgr.symbol_pool).to_string();
        self.packages.insert(mname, stbgr);
    }

    pub fn get_all_stbgr(&self) -> &BTreeMap<String, &'b StacklessBytecodeGenerator<'a>> {
        &self.packages
    }

    pub fn get_stbgr_by_mname(&self, mname: String) -> Option<&&StacklessBytecodeGenerator<'_>> {
        self.packages.get(&mname)
    }

    pub fn get_function(&self, mname: String, fid: FunId) -> &FunctionInfo {
        let stbgr = self.get_stbgr_by_mname(mname).unwrap();
        // module_data.function_idx_to_id.insert(def_idx, fun_id);
        let mut idx = 0;
        for (def_idx, fun_id) in stbgr.module_data.function_idx_to_id.iter() {
            if fid == *fun_id {
                idx = def_idx.0;
                break;
            }
        }
        &stbgr.functions[idx as usize]
    }
}
