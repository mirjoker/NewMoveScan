// Unnecessary Type Conversion

use crate::move_ir::generate_bytecode::FunctionInfo;
use move_model::ty::{PrimitiveType, Type};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

pub fn detect_unnecessary_type_conversion(
    function: &FunctionInfo,
    local_types: &Vec<Type>,
) -> bool {
    let mut ret_flag = false;
    for (_, bytecode) in function.code.iter().enumerate() {
        match &bytecode {
            Bytecode::Call(_, _, Operation::CastU8, srcs, _) => {
                if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U8) {
                    ret_flag = true;
                    break;
                }
            }
            Bytecode::Call(_, _, Operation::CastU16, srcs, _) => {
                if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U16) {
                    ret_flag = true;
                    break;
                }
            }
            Bytecode::Call(_, _, Operation::CastU32, srcs, _) => {
                if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U32) {
                    ret_flag = true;
                    break;
                }
            }
            Bytecode::Call(_, _, Operation::CastU64, srcs, _) => {
                if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U64) {
                    ret_flag = true;
                    break;
                }
            }
            Bytecode::Call(_, _, Operation::CastU128, srcs, _) => {
                if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U128) {
                    ret_flag = true;
                    break;
                }
            }
            Bytecode::Call(_, _, Operation::CastU256, srcs, _) => {
                if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U256) {
                    ret_flag = true;
                    break;
                }
            }
            _ => {
                continue;
            }
        }
    }
    ret_flag
}
