// unused_constant

use crate::move_ir::generate_bytecode::StacklessBytecodeGenerator;
use move_binary_format::{file_format::Bytecode as MoveBytecode, internals::ModuleIndex};
use move_core_types::value::MoveValue;

pub fn detect_unused_constants(stbgr: &StacklessBytecodeGenerator) -> Vec<MoveValue> {
    let cm = stbgr.module;
    let const_pool = &cm.constant_pool;
    let len = const_pool.len();
    let mut is_visited = vec![false; len];
    for fun in cm.function_defs.iter() {
        if let Some(codes) = &fun.code {
            for code in codes.code.iter() {
                match code {
                    MoveBytecode::LdConst(idx) => {
                        is_visited[idx.into_index()] = true;
                    }
                    _ => {}
                }
            }
        } else {
            continue;
        }
    }
    let mut unused_value: Vec<move_core_types::value::MoveValue> = vec![];
    for (id, visited) in is_visited.into_iter().enumerate() {
        if !visited {
            let constant = &stbgr.module.constant_pool[id];
            let value = constant.deserialize_constant().unwrap();
            unused_value.push(value);
        }
    }
    unused_value
}
