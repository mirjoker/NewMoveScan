use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::Bytecode;

pub struct Detector13<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector13<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::ParameterValidationIssue),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) =
                    self.detect_inefficient_assert(function, &stbgr.symbol_pool, idx, stbgr)
                {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector13<'a> {
    pub fn detect_inefficient_assert(
        &self,
        function: &FunctionInfo,
        _symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let args_count = function.args_count as usize;
        let mut found_assertion = false;

        for (code_offset, bytecode) in function.code.iter().enumerate() {
            match bytecode {
                // 检查 Branch 和 Abort 指令，是否与参数验证相关
                Bytecode::Branch(_, _, _, cond) | Bytecode::Abort(_, cond) => {
                    if self.check_arguments(&[*cond], args_count) {
                        // 如果在之前已经发现了其他操作，说明assert不在第一行
                        if code_offset > 0 {
                            let curr_func_name = utils::get_function_name(idx, stbgr);
                            return Some(curr_func_name);
                        }
                        found_assertion = true;
                    }
                }
                _ => {
                    // 如果还没发现 `assert`，但已经出现了其他操作，意味着assert不在第一行
                    if !found_assertion {
                        let curr_func_name = utils::get_function_name(idx, stbgr);
                        return Some(curr_func_name);
                    }
                }
            }
        }

        None
    }

    fn check_arguments(&self, args: &[usize], args_count: usize) -> bool {
        args.iter().any(|&arg| arg < args_count)
    }
}
