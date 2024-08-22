use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use std::collections::HashSet;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
pub struct Detector12<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector12<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::RedundantCheck),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) = self.detect_redundant_checks(function, stbgr) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector12<'a> {
    pub fn detect_redundant_checks(
        &self,
        function: &FunctionInfo,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let mut seen_conditions: HashSet<String> = HashSet::new();
        let mut redundant_checks: Vec<String> = Vec::new();

        for bytecode in &function.code {
            match bytecode {
                Bytecode::Branch(_, cond, _, _) => {
                    let cond_str = format!("{:?}", cond);
                    if seen_conditions.contains(&cond_str) {
                        redundant_checks.push(cond_str);
                    } else {
                        seen_conditions.insert(cond_str);
                    }
                }
                Bytecode::Call(_, _, Operation::Function(_, funid, _), _, _) => {
                    let funid_str = stbgr.symbol_pool.string(funid.symbol()).to_string();
                    if seen_conditions.contains(&funid_str) {
                        redundant_checks.push(funid_str);
                    } else {
                        seen_conditions.insert(funid_str);
                    }
                }
                _ => continue,
            }
        }

        if !redundant_checks.is_empty() {
            let func_name = utils::get_function_name(function.idx, stbgr);
            Some(func_name)
        } else {
            None
        }
    }
}
