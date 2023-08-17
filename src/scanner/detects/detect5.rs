// unused_constant

use move_binary_format::{file_format::Bytecode as MoveBytecode, internals::ModuleIndex};
use crate::{
    move_ir::{
        generate_bytecode::StacklessBytecodeGenerator,
        packages::Packages,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
pub struct Detector5<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector5<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UnusedConstant),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            let mut unused_private_functions_name = self.detect_unused_constants(stbgr);
            self.content
                .result
                .get_mut(mname)
                .unwrap()
                .append(&mut unused_private_functions_name);
        }
        &self.content
    }
}

impl<'a> Detector5<'a> {
    pub fn detect_unused_constants(&self,stbgr: &StacklessBytecodeGenerator) -> Vec<String> {
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
        .iter()
        .map(|x| format!("{:?}", x))
        .collect()
    }
}