// precision_loss
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
use std::rc::Rc;
pub struct Detector3<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector3<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::PrecisionLoss),
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
                if let Some(res) = self.detect_precision_loss(function, stbgr, idx) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector3<'a> {
    pub fn detect_precision_loss(
        &self,
        function: &FunctionInfo,
        stbgr: &StacklessBytecodeGenerator,
        idx: usize,
    ) -> Option<String> {
        let symbol_pool = &stbgr.symbol_pool;
        let mut ret_flag = false;
        for (code_offset, bytecode) in function.code.iter().enumerate() {
            match &bytecode {
                Bytecode::Call(_, _, Operation::Mul, srcs, _) => {
                    let oprand1 = utils::get_def_bytecode(&function, srcs[0], code_offset);
                    let oprand2 = utils::get_def_bytecode(&function, srcs[1], code_offset);
                    // println!("{:?}", oprand1);
                    // println!("{:?}", oprand2);
                    if self.is_div(oprand1)
                        || self.is_div(oprand2)
                        || self.is_sqrt(oprand1, symbol_pool)
                        || self.is_sqrt(oprand2, symbol_pool)
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

    fn is_div(&self, bytecode: &Bytecode) -> bool {
        let mut ret_flag = false;
        match bytecode {
            Bytecode::Call(_, _, Operation::Div, _, _) => {
                ret_flag = true;
            }
            _ => {}
        }
        return ret_flag;
    }

    fn is_sqrt(&self, bytecode: &Bytecode, symbol_pool: &SymbolPool) -> bool {
        let mut ret_flag = false;
        match bytecode {
            Bytecode::Call(_, _, Operation::Function(_, funid, _), _, _) => {
                if symbol_pool.string(funid.symbol()) == Rc::from("sqrt".to_string()) {
                    ret_flag = true;
                }
            }
            _ => {}
        }
        return ret_flag;
    }
}
