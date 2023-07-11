// Unnecessary Bool Judgment

use crate::move_ir::{generate_bytecode::FunctionInfo, utils::get_def_bytecode};
use move_stackless_bytecode::stackless_bytecode::{
    Bytecode, Operation, Constant
};
use move_model::{ty::{PrimitiveType, Type}};

pub fn detect_unnecessary_bool_judgment(function: &FunctionInfo, local_types: &Vec<Type>) -> bool {
    let mut ret_flag = false;
    for (code_offset, bytecode) in function.code.iter().enumerate() {
        match &bytecode {
            Bytecode::Call(_, _, Operation::Eq, srcs, _) 
            | Bytecode::Call(_, _, Operation::Neq, srcs, _) => {
                let oprand1 = get_def_bytecode(&function, srcs[0], code_offset);
                let oprand2 = get_def_bytecode(&function, srcs[1], code_offset);
                if (is_ldbool(oprand1) && ret_is_bool(oprand2, local_types)) || (is_ldbool(oprand2) && ret_is_bool(oprand1, local_types)) {
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

// fn get_oprand_bytecode(bytecodes: &Vec<Bytecode>, code_offset: usize, src_idx: usize) -> &Bytecode {
//     let mut tmp_index = code_offset - 1;
//     while tmp_index!=0 {
//         match &bytecodes[tmp_index] {
//             Bytecode::Call(_, dst, _, _, _) => {
//                 if dst.is_empty() {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//                 if dst[0] == src_idx  {
//                     return &bytecodes[tmp_index];
//                 } else {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//             },
//             Bytecode::Assign(_, dst, _, _) => {
//                 if *dst == src_idx {
//                     return &bytecodes[tmp_index];
//                 } else {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//             },
//             Bytecode::Load(_, dst, _) => {
//                 if *dst == src_idx {
//                     return &bytecodes[tmp_index];
//                 } else {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//             },
//             _ => {
//                 tmp_index = tmp_index - 1;
//                 continue;
//             }
//         }
//     }
//     return &bytecodes[tmp_index];
// }

fn is_ldbool(bytecode: &Bytecode) -> bool {
    let mut ret_flag = false;
    match bytecode {
        Bytecode::Load(_, _, c) => {
            if *c == Constant::Bool(true) || *c == Constant::Bool(false){
                ret_flag = true;
            }
        },
        _ => {
        }
    }
    return ret_flag;
}

fn ret_is_bool(bytecode: &Bytecode, local_types: &Vec<Type>) -> bool {
    let mut ret_flag = false;
    match bytecode {
        Bytecode::Call(_, dst, _, _, _) => {
            if local_types[dst[0]] == move_model::ty::Type::Primitive(PrimitiveType::Bool) {
                ret_flag = true;
            }
        },
        Bytecode::Assign(_, dst, _, _) => {
            if local_types[*dst] == move_model::ty::Type::Primitive(PrimitiveType::Bool) {
                ret_flag = true;
            }
        },
        Bytecode::Load(_, dst, _) => {
            if local_types[*dst] == move_model::ty::Type::Primitive(PrimitiveType::Bool) {
                ret_flag = true;
            }
        },
        _ => {
        }
    }
    return ret_flag;
}