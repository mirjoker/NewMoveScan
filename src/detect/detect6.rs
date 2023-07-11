// unused_private_functions

use move_binary_format::{file_format::Visibility, views::FunctionDefinitionView, access::ModuleAccess};
use move_model::model::{QualifiedId, FunId};
use crate::move_ir::generate_bytecode::StacklessBytecodeGenerator;
use petgraph::Direction;

fn get_unused_functions<'a>(stbgr: &'a StacklessBytecodeGenerator) -> Vec<&'a QualifiedId<FunId>> {
    let mut unused_functions: Vec<&QualifiedId<FunId>> = vec![];
    for (fid, nid) in stbgr.func_to_node.iter() {
        // 调用边，即入边
        // if stbgr.module_data.function_data.get(&fid.id).is_none() {
        //     // 理论上没有必要的操作，但是有脏东西，如aborted
        //     continue;
        // }
        let neighbors = stbgr.call_graph.neighbors_directed(*nid, Direction::Incoming);
        if neighbors.into_iter().next().is_none() {
            unused_functions.push(fid);
        }
    }
    unused_functions
}

pub fn detect_unused_private_functions(stbgr: &StacklessBytecodeGenerator) -> Vec<FunId> {
    let mut unused_private_functions: Vec<FunId> = vec![];
    let unused_functions = get_unused_functions(stbgr);
    for func in unused_functions {
        let function_data = stbgr.module_data.function_data.get(&func.id).unwrap();
        let view = FunctionDefinitionView::new(
            stbgr.module, 
            stbgr.module.function_def_at(function_data.def_idx)
        );
        if view.visibility() == Visibility::Private && !view.is_entry() 
            && !view.name().as_str().starts_with("init"){
                unused_private_functions.push(func.id);
        }
    }
    unused_private_functions
}