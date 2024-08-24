use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_binary_format::file_format::Visibility;
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

pub struct Detector11<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector11<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::EmitWithoutFriend),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());

            for (idx, function) in stbgr.functions.iter().enumerate() {
                // 跳过 native 函数
                if utils::is_native(idx, stbgr) {
                    continue;
                }

                let visibility = &function.visibility;

                if let Some(res) = self.detect_event_emit_without_friend(function, visibility, &stbgr.symbol_pool, idx, stbgr)
                {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector11<'a> {
    pub fn detect_event_emit_without_friend(
        &self,
        function_info: &FunctionInfo,
        visibility: &Visibility,  // 直接使用 Visibility 枚举
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let curr_func_name = utils::get_function_name(idx, stbgr);
        let is_public = matches!(visibility, Visibility::Public);

        let mut contains_emit_event = false;

        for bytecode in function_info.code.iter() {
            if let Bytecode::Call(_, _, Operation::Function(_, fun_id, _), _, _) = bytecode {
                let fun_name = symbol_pool.string(fun_id.symbol()).to_string();
                if fun_name.contains("emit_event") {
                    contains_emit_event = true;
                    break;
                }
            }
        }

        if is_public && contains_emit_event {
            return Some(curr_func_name);
        }

        None
    }
}
