// Unnecessary Type Conversion

use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::ty::PrimitiveType;
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
pub struct Detector7<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector7<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UnnecessaryTypeConversion),
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
                if let Some(res) = self.detect_unnecessary_type_conversion(function, stbgr, idx) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector7<'a> {
    pub fn detect_unnecessary_type_conversion(
        &self,
        function: &FunctionInfo,
        stbgr: &StacklessBytecodeGenerator,
        idx: usize,
    ) -> Option<String> {
        let local_types = &function.local_types;
        let mut ret_flag = false;
        for (_, bytecode) in function.code.iter().enumerate() {
            match &bytecode {
                Bytecode::Call(_, _, Operation::CastU8, srcs, _) => {
                    if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U8) {
                        ret_flag = true;
                        break;
                    }
                }
                Bytecode::Call(_, _, Operation::CastU16, srcs, _) => {
                    if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U16) {
                        ret_flag = true;
                        break;
                    }
                }
                Bytecode::Call(_, _, Operation::CastU32, srcs, _) => {
                    if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U32) {
                        ret_flag = true;
                        break;
                    }
                }
                Bytecode::Call(_, _, Operation::CastU64, srcs, _) => {
                    if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U64) {
                        ret_flag = true;
                        break;
                    }
                }
                Bytecode::Call(_, _, Operation::CastU128, srcs, _) => {
                    if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U128)
                    {
                        ret_flag = true;
                        break;
                    }
                }
                Bytecode::Call(_, _, Operation::CastU256, srcs, _) => {
                    if local_types[srcs[0]] == move_model::ty::Type::Primitive(PrimitiveType::U256)
                    {
                        ret_flag = true;
                        break;
                    }
                }
                _ => {
                    continue;
                }
            }
        }
        if ret_flag {
            let curr_func_name = utils::get_function_name(idx, stbgr);
            Some(curr_func_name)
        } else {
            None
        }
    }
}
