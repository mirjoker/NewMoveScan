// unckecked_return

use std::cmp;

use crate::move_ir::{
    generate_bytecode::FunctionInfo,
    packages::Packages
};
use move_binary_format::CompiledModule;
use move_model::symbol::{Symbol, SymbolPool};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
// use crate::scanner::detectors::AbstractDetector;
use itertools::Itertools;
use move_binary_format::{access::ModuleAccess, file_format::FunctionDefinitionIndex};
use num::ToPrimitive;
use std::{fs, io::Write, path::PathBuf, time::Instant};
use crate::{
    cli::parser::*,
    move_ir::generate_bytecode::StacklessBytecodeGenerator,
    scanner::{
        detector::{
            detector2::detect_overflow,
            detector3::detect_precision_loss, detector4::detect_infinite_loop,
            detector5::detect_unused_constants, detector6::detect_unused_private_functions,
            detector7::detect_unnecessary_type_conversion, detector8::detect_unnecessary_bool_judgment,
        },
        result::{DetectorType, FunctionType, ModuleInfo, PrettyResult, Result, Status},
    },
    utils::utils::{self, compile_module},
};
pub struct Detector1<'a, 'b> {
    packages:  &'a Packages<'a,'b>
    // detect_result: DetectResult
}
impl<'a, 'b> Detector1<'a, 'b> {
    pub fn detect_unchecked_return(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        _idx: usize,
        _cm: &CompiledModule,
    ) -> Vec<String> {
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
                        for pop in function.code
                            [code_offset + 1..cmp::min(function.code.len(), code_offset + ret_cnt + 1)]
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
        let mut ret: Vec<String> = Vec::new();
        if !be_call_funcs.is_empty() {
            for fun in be_call_funcs.iter() {
                ret.push(symbol_pool.string(*fun).to_string());
                // println!("function **:{} has return values but do not be used in {}", symbol_pool.string(*fun), cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name));
            }
        }
        return ret;
    }
}
impl<'a, 'b> Detector1<'a, 'b> {
// impl<'a, 'b> AbstractDetector<'a, 'b> for Detector1<'a, 'b> {
    fn new(packages: &Packages<'a, 'b>) -> Self {
        Self {
            packages,
        }
    }
    fn run(&self){
        for (mname, &stbgr) in self.packages.get_all_stbgr().iter() {
            let module_time_start = Instant::now();
            let mut module_info = ModuleInfo::empty();
            module_info.constant_count = stbgr.module.constant_pool.len();
            *module_info
                .function_count
                .get_mut(&FunctionType::All)
                .unwrap() = stbgr.functions.len();
            // 遍历stbgr中的functions
            for (idx, function) in stbgr.functions.iter().enumerate() {
                let func_define = stbgr
                    .module
                    .function_def_at(FunctionDefinitionIndex::new(idx as u16));
                if func_define.is_native() {
                    *module_info
                        .function_count
                        .get_mut(&FunctionType::Native)
                        .unwrap() += 1;
                    continue;
                };
                // let func_name = self.get_function_name(idx, stbgr);
                let mut unchecked_return_func_list = self.detect_unchecked_return(function, &stbgr.symbol_pool, idx, stbgr.module);
                // if !unchecked_return_func_list.is_empty() {
                //     // 先排序，再去重。Tips：dedup 用于去除连续的重复元素
                //     unchecked_return_func_list.sort();
                //     unchecked_return_func_list.dedup();
                //     let func_str = format!("{}({})", func_name.clone(), unchecked_return_func_list.into_iter().join(","));
                //     module_info.update_detectors(DetectorType::UncheckedReturn, func_str);
                // }
            }
        }
    }
}


