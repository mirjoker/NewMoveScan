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
    /// 检测 `assert` 是否在参数验证的第一行
    pub fn detect_inefficient_assert(
        &self,
        function: &FunctionInfo,
        _symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let args_count = function.args_count;
        let mut found_assertion = false;

        for (code_offset, bytecode) in function.code.iter().enumerate() {
            match bytecode {
                // 检查 Branch 和 Abort 指令，是否与参数验证相关
                Bytecode::Branch(_, _, _, cond) | Bytecode::Abort(_, cond) => {
                    if self.check_arguments(&[*cond], args_count) {
                        // 如果 `assert` 不在第一行并且已经有其他操作了
                        if code_offset > 0 {
                            let curr_func_name = utils::get_function_name(idx, stbgr);
                            return Some(curr_func_name); // 返回检测到的函数名，报告问题
                        }
                        // 记录已经找到`assert`
                        found_assertion = true;
                    }
                }
                _ => {
                    // 如果当前指令不是参数相关的操作并且assert不在第一行，报告问题
                    if !self.is_parameter_related(bytecode) && !found_assertion {
                        let curr_func_name = utils::get_function_name(idx, stbgr);
                        return Some(curr_func_name);
                    }
                }
            }
        }

        None
    }

    /// 检查字节码是否与参数相关
    fn check_arguments(&self, args: &[usize], args_count: usize) -> bool {
        args.iter().any(|&arg| arg < args_count)
    }

    /// 判断字节码是否是参数相关的操作
    fn is_parameter_related(&self, bytecode: &Bytecode) -> bool {
        match bytecode {
            Bytecode::Assign(_, _, _, _) | Bytecode::Load(..) => true, // 根据实际情况扩展可识别的参数相关操作
            _ => false,
        }
    }
}
