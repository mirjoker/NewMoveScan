use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::symbol:: SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
use move_model::ty::Type;

pub struct Detector10<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector10<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::GlobalStorageWithVector),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) = self.detect_global_storage_with_vector(function, &stbgr.symbol_pool, idx, stbgr) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector10<'a> {
    pub fn detect_global_storage_with_vector(
        &self,
        function: &FunctionInfo,
        _symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let mut detected_vector_usage: Vec<String> = Vec::new();
        for bytecode in function.code.iter() {
            match &bytecode {
                Bytecode::Call(_, _, oper, _, _) => {
                    if self.is_global_storage_operation(oper) && self.contains_vector_type(oper) {
                        detected_vector_usage.push(utils::get_function_name(idx, stbgr));
                    }
                }
                _ => continue,
            }
        }
        
        if !detected_vector_usage.is_empty() {
            detected_vector_usage.sort();
            detected_vector_usage.dedup();
            let curr_func_name = utils::get_function_name(idx, stbgr);
            let res = format!(
                "{}({})",
                curr_func_name,
                detected_vector_usage.join(","),
            );
            Some(res)
        } else {
            None
        }
    }

    fn is_global_storage_operation(&self, oper: &Operation) -> bool {
        matches!(
            oper,
            Operation::MoveTo(..) | Operation::MoveFrom(..) | Operation::Exists(..)
        )
    }

    fn contains_vector_type(&self, oper: &Operation) -> bool {
        let types = match oper {
            Operation::MoveTo(_, _, types)
            | Operation::MoveFrom(_, _, types)
            | Operation::Exists(_, _, types) => types,
            _ => return false,
        };
        types.iter().any(|ty| matches!(ty, Type::Vector(..)))
    }
}
