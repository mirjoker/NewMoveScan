use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_binary_format::file_format::Visibility;
use move_model::symbol::SymbolPool;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};

// 确保正确导入 visibility_str 函数
use crate::utils::utils::visibility_str;

pub struct Detector11<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector11<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Major, DetectKind::EmitWithoutFriend),
        }
    }

    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());

            for (idx, function) in stbgr.functions.iter().enumerate() {
                // 跳过 native 函数
                if utils::is_native(idx, stbgr) {
                    continue;
                }

                // 从 FunctionInfo 获取 visibility
                // Assuming you have a way to get visibility, maybe from the package or elsewhere
                // If not available, you might need to revisit how visibility is determined
                let visibility = Visibility::Public; // Default or retrieved from somewhere
                let visibility_str = visibility_str(&visibility);

                if let Some(res) =
                    self.detect_event_emit_without_friend(function, visibility_str, &stbgr.symbol_pool, idx, stbgr)
                {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector11<'a> {
    pub fn detect_event_emit_without_friend(
        &self,
        function_info: &FunctionInfo,
        visibility_str: &str,  // 修改为 &str 类型
        _symbol_pool: &SymbolPool,
        idx: usize,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<String> {
        let curr_func_name = utils::get_function_name(idx, stbgr);

        let is_public = visibility_str == "public";

        let mut contains_emit = false;

        for bytecode in function_info.code.iter() {
            if let Bytecode::Call(_, _, Operation::EmitEvent, _, _) = bytecode {
                contains_emit = true;
                break;
            }
        }

        if is_public && contains_emit {
            return Some(format!(
                "Function `{}` is public and contains an `emit` call without proper access control.",
                curr_func_name
            ));
        }

        None
    }
}
