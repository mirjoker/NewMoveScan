// precision_loss

use std::rc::Rc;

use crate::move_ir::{generate_bytecode::FunctionInfo, utils::get_def_bytecode};
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{
    Bytecode, Operation
};


pub fn detect_precision_loss(function: &FunctionInfo, symbol_pool: &SymbolPool) -> bool {
    let mut ret_flag = false;
    for (code_offset, bytecode) in function.code.iter().enumerate() {
        match &bytecode {
            Bytecode::Call(_, _, Operation::Mul, srcs, _) => {
                let oprand1 = get_def_bytecode(&function, srcs[0], code_offset);
                let oprand2 = get_def_bytecode(&function, srcs[1], code_offset);
                // println!("{:?}", oprand1);
                // println!("{:?}", oprand2);
                if is_div(oprand1) || is_div(oprand2) || is_sqrt(oprand1, symbol_pool) || is_sqrt(oprand2, symbol_pool) {
                    ret_flag = true;
                    break;
                }
            },
            _ => {
                continue;
            }
        }
    }
    ret_flag 
}

fn is_div(bytecode: &Bytecode) -> bool {
    let mut ret_flag = false;
    match bytecode {
        Bytecode::Call(_, _, Operation::Div, _, _) => {
            ret_flag = true;
        },
        _ => {
        }
    }
    return ret_flag;
}

fn is_sqrt(bytecode: &Bytecode, symbol_pool: &SymbolPool) -> bool {
    let mut ret_flag = false;
    match bytecode {
        Bytecode::Call(_, _, Operation::Function(_, funid, _), _, _) => {
            if symbol_pool.string(funid.symbol()) == Rc::from("sqrt".to_string()) {
                ret_flag = true;
            }
        },
        _ => {
        }
    }
    return ret_flag;
}