use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

pub struct Detector10<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector10<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Major, DetectKind::OrdersIssue),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, &stbgr) {
                    continue;
                }
                if let Some(res) = self.detect_orders_error(function, &stbgr.symbol_pool, idx, &stbgr) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector10<'a> {
    pub fn detect_orders_error(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let mut orders_operations = Vec::new();
        for (code_offset, bytecode) in function.code.iter().enumerate() {
            if let Bytecode::Call(_, _, Operation::Function(_, fid, _), _, _) = bytecode {
                let function_name = symbol_pool.string(fid.symbol());
                if function_name.contains("orders") {
                    orders_operations.push((function_name.to_string(), code_offset));
                }
            }
        }

        let curr_func_name = utils::get_function_name(idx, stbgr);
        if !orders_operations.is_empty() {
            orders_operations.sort_by(|a, b| a.1.cmp(&b.1)); // 根据字节码偏移量排序
            let unique_operations: Vec<String> = orders_operations
                .into_iter()
                .map(|(name, _)| name)
                .collect::<Vec<String>>()
                .into_iter()
                .collect::<std::collections::HashSet<String>>()
                .into_iter()
                .collect();

            let res = format!(
                "{}({})",
                curr_func_name,
                unique_operations.join(", ")
            );
            Some(res)
        } else {
            None
        }
    }
}