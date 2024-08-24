use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::symbol:: SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

pub struct Detector9<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector9<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::MissingZeroCheck),
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
                    self.detect_missing_zero_check(function, &stbgr.symbol_pool, idx, stbgr)
                {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector9<'a> {
    pub fn detect_missing_zero_check(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let mut liquidity_op_found = false;
        let mut zero_check_found = false;

        for bytecode in function.code.iter() {
            match &bytecode {
                Bytecode::Call(_, _, Operation::Function(_, fun_id, _), _, _) => {
                    let fun_name = symbol_pool.string(fun_id.symbol()).to_string();
                    // 检查是否是涉及 liquidity 操作的函数
                    if fun_name.contains("coins") {
                        liquidity_op_found = true;
                    }
                }
                Bytecode::Call(_, _, Operation::Eq, _, _)
                | Bytecode::Call(_, _, Operation::Neq, _, _)
                | Bytecode::Call(_, _, Operation::Gt, _, _)
                | Bytecode::Call(_, _, Operation::Ge, _, _) => {
                    // 检查是否存在零值检查
                    zero_check_found = true;
                }
                _ => {}
            }
        }

        if liquidity_op_found && !zero_check_found {
            let curr_func_name = utils::get_function_name(idx, stbgr);
            Some(format!("{}: Missing zero check for liquidity", curr_func_name))
        } else {
            None
        }
    }
}
