// unused_private_functions

use crate::{
    move_ir::{
        generate_bytecode:: StacklessBytecodeGenerator,
        packages::Packages,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_binary_format::{
    access::ModuleAccess, file_format::Visibility, views::FunctionDefinitionView,
};
use move_model::model::{FunId, QualifiedId};
use petgraph::Direction;
pub struct Detector6<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector6<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UnusedPrivateFunctions),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            let mut unused_private_functions_name = self.detect_unused_private_functions(stbgr);
            self.content
                .result
                .get_mut(mname)
                .unwrap()
                .append(&mut unused_private_functions_name);
        }
        &self.content
    }
}

impl<'a> Detector6<'a> {
    pub fn detect_unused_private_functions(
        &self,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Vec<String> {
        let mut unused_private_functions_name = Vec::new();
        let unused_functions = self.get_unused_functions(stbgr);
        for func in unused_functions {
            let function_data = stbgr.module_data.function_data.get(&func.id).unwrap();
            let view = FunctionDefinitionView::new(
                stbgr.module,
                stbgr.module.function_def_at(function_data.def_idx),
            );
            if view.visibility() == Visibility::Private
                && !view.is_entry()
                && !view.name().as_str().starts_with("init")
            {
                unused_private_functions_name
                    .push(func.id.symbol().display(&stbgr.symbol_pool).to_string());
            }
        }
        unused_private_functions_name
    }
    fn get_unused_functions(
        &self,
        stbgr: &'a StacklessBytecodeGenerator,
    ) -> Vec<&'a QualifiedId<FunId>> {
        let mut unused_functions: Vec<&QualifiedId<FunId>> = vec![];
        for (fid, nid) in stbgr.func_to_node.iter() {
            // 调用边，即入边
            // if stbgr.module_data.function_data.get(&fid.id).is_none() {
            //     // 理论上没有必要的操作，但是有脏东西，如aborted
            //     continue;
            // }
            let neighbors = stbgr
                .call_graph
                .neighbors_directed(*nid, Direction::Incoming);
            if neighbors.into_iter().next().is_none() {
                unused_functions.push(fid);
            }
        }
        unused_functions
    }
}
