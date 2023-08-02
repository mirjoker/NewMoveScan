#![allow(unused_imports)]
use std::path::PathBuf;
use std::str::FromStr;

use crate::move_ir::generate_bytecode::*;
use crate::utils::utils::compile_module;
use move_stackless_bytecode::stackless_bytecode::Bytecode;
use crate::move_ir::bytecode_display;

// #[test]
// fn test_generate_bytecode() {
//     let filename = PathBuf::from_str("testdata/examples_mv/aptos/infinite_loop.mv").unwrap();
//     let cm = compile_module(filename);
//     let mut bg = StacklessBytecodeGenerator::new(&cm);
//     bg.generate_function();
//     println!("{}", bg);
//     bg.get_control_flow_graph();
//     let cfg = bg.functions[0].cfg.as_ref().unwrap();
//     cfg.display();
// }
