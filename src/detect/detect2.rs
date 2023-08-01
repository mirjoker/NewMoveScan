// overflow

use crate::move_ir::{
    generate_bytecode::StacklessBytecodeGenerator,
    packages::Packages,
};
use move_model::ty::{PrimitiveType, Type};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

pub fn detect_overflow(
    packages: &Packages,
    stbgr: &StacklessBytecodeGenerator,
    idx: usize,
) -> bool {
    let function = &stbgr.functions[idx];
    let mut ret_flag = false;
    let local_types = &function.local_types;
    let dd = &stbgr.data_dependency[idx];
    for (_, bytecode) in function.code.iter().enumerate() {
        match &bytecode {
            Bytecode::Call(_, dsts, Operation::Shl, srcs, _) => {
                let num_max = dd.get(srcs[0]).max.unwrap();
                let shl_bit_max = dd.get(srcs[1]).max.unwrap();
                let ty = &function.local_types[dsts[0]];
                if 256 - num_max.leading_zeros() + shl_bit_max.as_u32() > get_ubits(ty) {
                    ret_flag = true;
                }
            }
            _ => {}
        }
    }
    ret_flag
}

fn get_ubits(ty: &Type) -> u32 {
    match ty {
        Type::Primitive(PrimitiveType::U8) => 8,
        Type::Primitive(PrimitiveType::U16) => 16,
        Type::Primitive(PrimitiveType::U32) => 32,
        Type::Primitive(PrimitiveType::U64) => 64,
        Type::Primitive(PrimitiveType::U128) => 128,
        Type::Primitive(PrimitiveType::U256) => 256,
        _ => 0,
    }
}
