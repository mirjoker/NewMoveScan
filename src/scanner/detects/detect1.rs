// unckecked_return

use std::cmp;

use crate::move_ir::generate_bytecode::FunctionInfo;
use move_binary_format::CompiledModule;
use move_model::symbol::{Symbol, SymbolPool};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

pub fn detect_unchecked_return(
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
