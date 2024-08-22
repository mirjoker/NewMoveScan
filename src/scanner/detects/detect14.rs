use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_binary_format::file_format::StructHandleIndex;
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
use move_model::model::StructId;

pub struct Detector14<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector14<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Critical, DetectKind::WitnessCopy),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) = self.detect_witness_copy(function, &stbgr.symbol_pool, idx, stbgr) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector14<'a> {
    pub fn detect_witness_copy(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        for bytecode in function.code.iter() {
            if let Bytecode::Call(_, _, Operation::Pack(_, struct_id, _), _, _) = bytecode {
                if let Some(struct_handle_index) = self.get_struct_handle_index(struct_id, stbgr) {
                    if self.is_witness_struct_with_copy_ability(&struct_handle_index, stbgr, symbol_pool) {
                        let curr_func_name = utils::get_function_name(idx, stbgr);
                        return Some(format!(
                            "{}: Witness struct with copy ability used in the function",
                            curr_func_name
                        ));
                    }
                }
            }
        }
        None
    }

    fn is_witness_struct_with_copy_ability(
        &self,
        struct_handle_index: &StructHandleIndex,
        stbgr: &StacklessBytecodeGenerator,
        symbol_pool: &SymbolPool,
    ) -> bool {
        let struct_id = stbgr.get_struct_id_by_idx(struct_handle_index);
        let module_id = self.get_module_id(stbgr);
        if let Some(qsymbol) = stbgr.reverse_struct_table.get(&(module_id, struct_id)) {
            let struct_name = symbol_pool.string(qsymbol.symbol).to_string();
            if struct_name == "Witness" {
                // 检查该结构体是否具有 copy 能力
                let abilities = &stbgr.module.struct_handles[struct_handle_index.0 as usize].abilities;
                return abilities.has_copy();
            }
        }
        false
    }
    

    fn get_struct_handle_index(
        &self,
        struct_id: &StructId,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<StructHandleIndex> {
        stbgr
            .module
            .struct_handles
            .iter()
            .enumerate()
            .find(|(idx, _)| {
                stbgr.get_struct_id_by_idx(&StructHandleIndex::new(*idx as u16)) == *struct_id
            })
            .map(|(idx, _)| StructHandleIndex::new(idx as u16))
    }
    

    fn get_module_id(&self, _stbgr: &StacklessBytecodeGenerator) -> move_model::model::ModuleId {
        let index = 0; 
        move_model::model::ModuleId::new(index)
    }
}
