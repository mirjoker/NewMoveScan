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
use std::collections::HashSet;
use move_model::ty::{PrimitiveType, Type};

pub struct Detector12<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector12<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::MissingAccessControlAssertion),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) =
                    self.detect_missing_access_control_assertion(function, &stbgr.symbol_pool, idx, stbgr)
                {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector12<'a> {
    pub fn detect_missing_access_control_assertion(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let mut signer_params = HashSet::new();
        let mut found_assertion = false;

        // 根据函数的 args_count 假设参数数量
        let args_count = function.args_count;

        // 假设第一个参数是 &signer 类型
        for i in 0..args_count {
            if is_signer_param(i, function, symbol_pool) {
                signer_params.insert(i);
            }
        }

        if signer_params.is_empty() {
            return None; // 没有 &signer 参数，无需检查
        }

        // 检查是否存在针对 &signer 参数的断言
        for bytecode in &function.code {
            match bytecode {
                Bytecode::Call(_, _, Operation::Function(_, _fid, _), _, _) => {
                    // 示例：检查是否调用了特定的断言函数（这里你可能需要根据实际逻辑调整）
                    if let Some(operation) = function.code.iter().find_map(|b| {
                        if let Bytecode::Call(_, _, op, _, _) = b {
                            Some(op)
                        } else {
                            None
                        }
                    }) {
                        match operation {
                            Operation::OpaqueCallBegin(_, _, _) => {
                                // 示例：假设有 OpaqueCallBegin 表示可能存在断言
                                found_assertion = true; // 根据实际逻辑进行判断
                                break;
                            }
                            _ => continue,
                        }
                    }
                }
                _ => continue,
            }
        }

        let curr_func_name = utils::get_function_name(idx, stbgr);
        if !found_assertion {
            Some(format!("{}: 缺少对 &signer 参数的访问控制断言", curr_func_name))
        } else {
            None
        }
    }
}

fn is_signer_param(index: usize, function: &FunctionInfo, _symbol_pool: &SymbolPool) -> bool {
    if index >= function.args_count {
        return false; // 索引超出参数范围
    }

    // 获取函数参数的类型
    if let Some(param_type) = function.local_types.get(index) {
        return matches_signer_type(param_type);
    }

    false
}

fn matches_signer_type(param_type: &Type) -> bool {
    // 检查类型是否为 &signer，即 Reference(true, Box::new(Primitive(PrimitiveType::Signer)))
    if let Type::Reference(true, ref inner_type) = param_type {
        if let Type::Primitive(PrimitiveType::Signer) = *inner_type.as_ref() {
            return true;
        }
    }
    false
}
