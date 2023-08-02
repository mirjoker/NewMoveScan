#![allow(unused_imports)]
use std::path::PathBuf;
use std::str::FromStr;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::FunctionDefinitionIndex;
use move_binary_format::views::FunctionDefinitionView;

use crate::detect::detect2::detect_overflow;
use crate::detect::detect3::detect_precision_loss;
use crate::detect::detect4::detect_infinite_loop;
use crate::detect::detect7::detect_unnecessary_type_conversion;
use crate::detect::detect8::detect_unnecessary_bool_judgment;
use crate::move_ir::control_flow_graph::generate_cfg_in_dot_format;
use crate::move_ir::data_dependency;
use crate::move_ir::{
    bytecode_display, 
    generate_bytecode::StacklessBytecodeGenerator
};
use crate::utils::utils::compile_module;
use crate::detect::detect1::detect_unchecked_return;


// #[test]
// fn test_detect_unchecked_return() {
//     let filename = PathBuf::from_str("/home/yule/Movebit/detect/build/movebit/bytecode_modules/unchecked_return.mv").unwrap();
//     let cm = compile_module(filename);
//     let mut stbgr = StacklessBytecodeGenerator::new(&cm);
//     stbgr.generate_function();
//     for (idx, function) in stbgr.functions.iter().enumerate() {
//         if detect_unchecked_return(function, &stbgr.symbol_pool, idx, &cm) {
//             println!("{} : {}", cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name), "unchecked return");
//         }
//     }
// }

// #[test]
// fn test_loop() {
//     let filename = PathBuf::from_str("/Users/lteng/Movebit/detect/build/movebit/bytecode_modules/infinite_loop.mv").unwrap();
//     // let filename = PathBuf::from_str("/Users/lteng/Movebit/AptosProjects/lend-config/build/LendConfig/bytecode_modules/borrow_interest_rate.mv").unwrap();
//     let cm = compile_module(filename);
//     let mut stbgr = StacklessBytecodeGenerator::new(&cm);
//     stbgr.generate_function();
//     stbgr.get_control_flow_graph();
//     let filename = PathBuf::from("cfg.dot");
//     generate_cfg_in_dot_format(&stbgr.functions[0], filename, &stbgr);
//     let data_depent = data_dependency(&stbgr, 0);
//     detect_infinite_loop(&stbgr, 0);
//     stbgr.functions.iter().enumerate().map(|(idx, function)| {
//         detect_unchecked_return(function, &stbgr.symbol_pool, idx, &cm);
//     });
// }
