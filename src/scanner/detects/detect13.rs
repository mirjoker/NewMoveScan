use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

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
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let args_count = function.args_count as usize;
        let mut first_assert_position = None;

        for (code_offset, bytecode) in function.code.iter().enumerate() {
            match bytecode {
                Bytecode::Call(_, args, Operation::Function(_, fun_id, _), _, _) => {
                    let fun_name = symbol_pool.string(fun_id.symbol()).to_string();
                    if fun_name.contains("assert") {
                        // Check if `assert` is validating function parameters
                        if self.check_arguments(args, args_count) {
                            // Record the position of the first `assert` that validates parameters
                            if first_assert_position.is_none() {
                                first_assert_position = Some(code_offset);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // If we found an `assert` that validates parameters, but it's not at the start of the function
        if let Some(first_assert_pos) = first_assert_position {
            if first_assert_pos > 0 {
                let curr_func_name = utils::get_function_name(idx, stbgr);
                return Some(curr_func_name);
            }
        }

        None
    }

    fn check_arguments(&self, args: &[usize], args_count: usize) -> bool {
        args.iter().any(|&arg| arg < args_count)
    }
}


