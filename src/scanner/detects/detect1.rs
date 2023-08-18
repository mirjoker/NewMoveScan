// unckecked_return
use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::symbol::{Symbol, SymbolPool};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
use std::cmp;

pub struct Detector1<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector1<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UncheckedReturn),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            // result: HashMap<ModuleName, Vec<String>>
            // Vec<String> 中存储了当前 ModuleName 下的漏洞的函数名（通常来说，如果是unused_constant则会存储对应的常量类型和值）
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                // 跳过 native 函数
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) =
                    self.detect_unchecked_return(function, &stbgr.symbol_pool, idx, stbgr)
                {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

// 若在给定的方法中检测到了漏洞，则返回Some，否则None
impl<'a> Detector1<'a> {
    pub fn detect_unchecked_return(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        // let mut ret_flag = false;
        let mut be_call_funcs: Vec<Symbol> = Vec::new();
        for (code_offset, bytecode) in function.code.iter().enumerate() {
            match &bytecode {
                Bytecode::Call(_, dsts, Operation::Function(_, fid, _), _, _) => {
                    let ret_cnt = dsts.len();
                    // 函数没有返回值 false
                    if ret_cnt == 0 {
                        continue;
                    } else {
                        for pop in function.code[code_offset + 1
                            ..cmp::min(function.code.len(), code_offset + ret_cnt + 1)]
                            .iter()
                        {
                            match pop {
                                Bytecode::Call(_, _, Operation::Destroy, _, _) => {
                                    // ret_flag = true;
                                    be_call_funcs.push(fid.symbol());
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }
                }
                _ => {
                    continue;
                }
            }
        }
        let mut uncheck_return_func_name: Vec<String> = Vec::new();
        if !be_call_funcs.is_empty() {
            for fun in be_call_funcs.iter() {
                uncheck_return_func_name.push(symbol_pool.string(*fun).to_string());
                // println!("function **:{} has return values but do not be used in {}", symbol_pool.string(*fun), cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name));
            }
        }
        let curr_func_name = utils::get_function_name(idx, stbgr);
        if !uncheck_return_func_name.is_empty() {
            // 先排序，再去重。Tips：dedup 用于去除连续的重复元素
            uncheck_return_func_name.sort();
            uncheck_return_func_name.dedup();
            let res = format!(
                "{}({})",
                curr_func_name.clone(),
                uncheck_return_func_name
                    .into_iter()
                    .collect::<Vec<String>>()
                    .join(",")
            );
            Some(res)
        } else {
            None
        }
        // res Expmple：f1(f2,f3)
        // f1 函数中调用了 f2、f3,但没有检查 f2、f3的返回值
    }
}
