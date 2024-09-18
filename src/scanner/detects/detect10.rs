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

pub struct Detector10<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector10<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UnnecessaryEmit),
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

                if let Some(res) = self.detect_event_emit_without_friend(function, visibility, &stbgr.symbol_pool, idx, stbgr) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector10<'a> {
    pub fn detect_event_emit_without_friend(
        &self,
        function_info: &FunctionInfo,
        visibility: &Visibility,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let curr_func_name = utils::get_function_name(idx, stbgr);
        let is_public = matches!(visibility, Visibility::Public);

        let mut contains_emit_event = false;
        let mut contains_state_update = false;

        for bytecode in function_info.code.iter() {
            match bytecode {
                // 检查是否包含事件触发操作
                Bytecode::Call(_, _, Operation::Function(_, fun_id, _), _, _) => {
                    let fun_name = symbol_pool.string(fun_id.symbol()).to_string();
                    if fun_name.contains("emit_event") {
                        contains_emit_event = true;
                    }
                    // 检查是否涉及资金操作或状态更新
                    if fun_name.contains("withdraw") || fun_name.contains("deposit") || fun_name.contains("transfer") ||
                       fun_name.contains("mint") || fun_name.contains("burn") || fun_name.contains("merge") || fun_name.contains("extract") ||
                       fun_name.contains("add") || fun_name.contains("borrow_mut") || fun_name.contains("push_back") {
                        contains_state_update = true;
                    }
                },
                Bytecode::Call(_, _, Operation::MoveTo(..), _, _) |
                Bytecode::Call(_, _, Operation::MoveFrom(..), _, _) => {
                    contains_state_update = true;
                },
                _ => {}
            }
        }

        // 如果是公共函数，且包含事件触发，但不涉及状态更新
        if is_public && contains_emit_event && !contains_state_update {
            if self.is_called_by_other_functions(&curr_func_name, symbol_pool, stbgr) {
                return Some(curr_func_name);
            }
        }

        None
    }
    fn is_called_by_other_functions(
        &self,
        target_func_name: &str,
        symbol_pool: &SymbolPool,
        stbgr: &StacklessBytecodeGenerator,
    ) -> bool {
        for (idx, function) in stbgr.functions.iter().enumerate() {
            if utils::get_function_name(idx, stbgr) == target_func_name {
                continue;
            }
    
            for bytecode in function.code.iter() {
                match bytecode {
                    Bytecode::Call(_, _, Operation::Function(_, fun_id, _), _, _) => {
                        let callee_name = symbol_pool.string(fun_id.symbol()).to_string();
                        if callee_name == target_func_name {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
    
        false
    }
}

