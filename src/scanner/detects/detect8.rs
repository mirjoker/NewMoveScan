// Unnecessary Bool Judgment

use crate::{
    move_ir::{
        generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator},
        packages::Packages,
        utils,
    },
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::ty::{PrimitiveType, Type};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Constant, Operation};
pub struct Detector8<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector8<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::UnnecessaryBoolJudgment),
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
                if let Some(res) = self.detect_unnecessary_bool_judgment(function, stbgr, idx) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector8<'a> {
    pub fn detect_unnecessary_bool_judgment(
        &self,
        function: &FunctionInfo,
        stbgr: &StacklessBytecodeGenerator,
        idx: usize,
    ) -> Option<String> {
        let local_types = &function.local_types;
        let mut ret_flag = false;
        for (code_offset, bytecode) in function.code.iter().enumerate() {
            match &bytecode {
                Bytecode::Call(_, _, Operation::Eq, srcs, _)
                | Bytecode::Call(_, _, Operation::Neq, srcs, _) => {
                    let oprand1 = utils::get_def_bytecode(&function, srcs[0], code_offset);
                    let oprand2 = utils::get_def_bytecode(&function, srcs[1], code_offset);
                    if (self.is_ldbool(oprand1) && self.ret_is_bool(oprand2, local_types))
                        || (self.is_ldbool(oprand2) && self.ret_is_bool(oprand1, local_types))
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

    fn is_ldbool(&self, bytecode: &Bytecode) -> bool {
        let mut ret_flag = false;
        match bytecode {
            Bytecode::Load(_, _, c) => {
                if *c == Constant::Bool(true) || *c == Constant::Bool(false) {
                    ret_flag = true;
                }
            }
            _ => {}
        }
        return ret_flag;
    }

    fn ret_is_bool(&self, bytecode: &Bytecode, local_types: &Vec<Type>) -> bool {
        let mut ret_flag = false;
        match bytecode {
            Bytecode::Call(_, dst, _, _, _) => {
                if local_types[dst[0]] == move_model::ty::Type::Primitive(PrimitiveType::Bool) {
                    ret_flag = true;
                }
            }
            Bytecode::Assign(_, dst, _, _) => {
                if local_types[*dst] == move_model::ty::Type::Primitive(PrimitiveType::Bool) {
                    ret_flag = true;
                }
            }
            Bytecode::Load(_, dst, _) => {
                if local_types[*dst] == move_model::ty::Type::Primitive(PrimitiveType::Bool) {
                    ret_flag = true;
                }
            }
            _ => {}
        }
        return ret_flag;
    }
}
