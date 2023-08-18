// overflow
use crate::{
    move_ir::{generate_bytecode::StacklessBytecodeGenerator, packages::Packages, utils},
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::ty::{PrimitiveType, Type};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
pub struct Detector2<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector2<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::Overflow),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, _function) in stbgr.functions.iter().enumerate() {
                // 跳过 native 函数
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) = self.detect_overflow(stbgr, idx) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector2<'a> {
    pub fn detect_overflow(
        &self,
        stbgr: &StacklessBytecodeGenerator,
        idx: usize,
    ) -> Option<String> {
        let function = &stbgr.functions[idx];
        let mut ret_flag = false;
        let _local_types = &function.local_types;
        let dd = &stbgr.data_dependency[idx];
        for (_, bytecode) in function.code.iter().enumerate() {
            match &bytecode {
                Bytecode::Call(_, dsts, Operation::Shl, srcs, _) => {
                    let num_max = dd.get(srcs[0]).max.unwrap();
                    let shl_bit_max = dd.get(srcs[1]).max.unwrap();
                    let ty = &function.local_types[dsts[0]];
                    if 256 - num_max.leading_zeros() + shl_bit_max.as_u32() > self.get_ubits(ty) {
                        ret_flag = true;
                    }
                }
                _ => {}
            }
        }
        if ret_flag {
            let curr_func_name = utils::get_function_name(idx, stbgr);
            Some(curr_func_name)
        } else {
            None
        }
    }

    fn get_ubits(&self, ty: &Type) -> u32 {
        match ty {
            Type::Primitive(PrimitiveType::U8) => 8,
            Type::Primitive(PrimitiveType::U16) => 16,
            Type::Primitive(PrimitiveType::U32) => 32,
            Type::Primitive(PrimitiveType::U64) => 64,
            Type::Primitive(PrimitiveType::U128) => 128,
            Type::Primitive(PrimitiveType::U256) => 256,
            _ => 0,
        }
    }
}
