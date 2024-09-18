use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_binary_format::file_format::FunctionDefinitionIndex;
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
            content: DetectContent::new(Severity::Minor, DetectKind::UncheckedZero),
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
        let mut liquidity_var = None;  // 跟踪 liquidity 变量
        let mut liquidity_check_found = false;  // 是否找到 liquidity > 0 检查

        for bytecode in function.code.iter() {
            match &bytecode {
                // 捕捉涉及流动性操作的函数调用
                Bytecode::Call(_, _, Operation::Function(_, fun_id, _), _, _) => {
                    let fun_name = symbol_pool.string(fun_id.symbol()).to_string();
                    if fun_name.contains("add_liquidity") {
                        liquidity_op_found = true;
                    }
                }

                // 捕捉流动性变量的赋值操作
                Bytecode::Assign(_, dest, _src, _) => {
                    let var_symbol = stbgr.get_local_name(FunctionDefinitionIndex::new(idx as u16), *dest);
                    let var_name = symbol_pool.string(var_symbol);
                    if var_name.contains("liquidity") || var_name.contains("lp_amount") {
                        liquidity_var = Some(*dest);  // 记录 liquidity 变量
                    }
                }

                // 检查条件分支和中止操作
                Bytecode::Branch(_, _, _, cond) | Bytecode::Abort(_, cond) => {
                    if let Some(liquidity) = liquidity_var {
                        // 检查是否 cond 中是 liquidity 变量
                        if *cond == liquidity {
                            // 检查条件是否是与 0 进行比较
                            liquidity_check_found = true;
                        }
                    }
                }

                _ => {}
            }
        }

        // 如果找到流动性操作，但没有找到 liquidity 相关的断言，则返回结果
        if liquidity_op_found && !liquidity_check_found {
            let curr_func_name = utils::get_function_name(idx, stbgr);
            Some(format!("{}: Missing 'liquidity > 0' check for liquidity", curr_func_name))
        } else {
            None
        }
    }
}




