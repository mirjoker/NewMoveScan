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

pub struct Detector11<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector11<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UnnecessaryAccessControl),
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

impl<'a> Detector11<'a> {
    pub fn detect_missing_access_control_assertion(
        &self,
        function: &FunctionInfo,
        symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let mut signer_params = HashSet::new();
        let mut found_assertion = false;
        let mut found_access_control = false;
        let mut fund_transfer_found = false;
        let data_depent = &stbgr.data_dependency[idx];

        // 1. 遍历函数参数的实际类型，找出 &signer 类型的参数
        for (i, param_type) in function.local_types.iter().enumerate() {
            if matches_signer_type(param_type) {
                signer_params.insert(i);
            }
        }

        if signer_params.is_empty() {
            return None; // 没有 &signer 参数，无需检查
        }

        // 2. 遍历字节码，寻找资金转移操作和权限验证函数
        for bytecode in &function.code {
            match bytecode {
                // 资金转移操作，如 MoveTo、MoveFrom
                Bytecode::Call(_, _, Operation::MoveTo(_, _, _), _, _) | 
                Bytecode::Call(_, _, Operation::MoveFrom(_, _, _), _, _) => {
                    fund_transfer_found = true;
                }

                // 权限验证函数调用，使用 symbol_pool 获取函数名
                Bytecode::Call(_, _, Operation::Function(_, fun_id, _), _, _) => {
                    let fun_name = symbol_pool.string(fun_id.symbol()).to_string();
                    if fun_name.contains("assert_admin") || fun_name.contains("assert_owner") || fun_name.contains("require_admin") {
                        found_access_control = true;
                    }
                }

                // 条件跳转和中止指令，涉及 signer 参数
                Bytecode::Branch(_, _, _, cond) | Bytecode::Abort(_, cond) => {
                    if signer_params.contains(cond) || data_depent.data.contains_key(cond) {
                        found_assertion = true;
                    }
                }
                _ => continue,
            }
        }

        // 3. 判断是否同时存在 &signer 参数、资金转移操作、并且没有适当的访问控制
        let curr_func_name = utils::get_function_name(idx, stbgr);
        if fund_transfer_found && !found_assertion && !found_access_control {
            Some(format!("{}", curr_func_name))
        } else {
            None
        }
    }
}

fn matches_signer_type(param_type: &Type) -> bool {
    match param_type {
        Type::Reference(true, inner_type) => {
            if let Type::Primitive(PrimitiveType::Signer) = *inner_type.as_ref() {
                return true;
            }
        }
        Type::Reference(false, inner_type) => {
            if let Type::Primitive(PrimitiveType::Signer) = *inner_type.as_ref() {
                return true;
            }
        }
        _ => {}
    }
    false
}
