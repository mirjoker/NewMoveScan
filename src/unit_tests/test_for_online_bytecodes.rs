use std::path::PathBuf;
use itertools::Itertools;
use move_binary_format::{file_format::FunctionDefinitionIndex, access::ModuleAccess, CompiledModule};
use crate::{utils::utils::{self, compile_module}, move_ir::{generate_bytecode::StacklessBytecodeGenerator, packages::Packages}, 
detect::{detect1::detect_unchecked_return, detect2::detect_overflow, detect3::detect_precision_loss, detect4::detect_infinite_loop, 
    detect7::detect_unnecessary_type_conversion, detect8::detect_unnecessary_bool_judgment, detect5::detect_unused_constants, detect6::detect_unused_private_functions}};
use std::time::{Duration, Instant};

#[test]
fn test_for_online_bytecodes() {
    let start = Instant::now();
    let dir = PathBuf::from("/home/yule/Movebit/aptos_onchain_bytecode");
    let mut paths = Vec::new();
    utils::visit_dirs(&dir, &mut paths, false);
    let mut bytecode_cnt = 0;
    let mut bytecode_fail_to_deserialize_cnt = 0;
    let mut func_cnt = 0;
    let mut native_func_cnt = 0;
    let mut defects_cnt = vec![0,0,0,0,0,0,0,0];
    let defects_name = vec!["unckecked return","overflow","precision loss","infinite loop",
    "unnecessary type conversion","unnecessary bool judgment","unused constants","unused private functions"];
    for filename in paths.iter() {
        bytecode_cnt += 1;
        let cm = compile_module(filename.to_path_buf());
        if(cm.is_none()) {
            bytecode_fail_to_deserialize_cnt += 1;
            continue;
        }
        let cm = cm.unwrap();
        let mut stbgr = StacklessBytecodeGenerator::new(&cm);
        stbgr.generate_function();
        stbgr.get_control_flow_graph();
        stbgr.build_call_graph();
        let mut packages = Packages::new();
        packages.insert_stbgr(&stbgr);
        for (_, &stbgr) in packages.get_all_stbgr().iter(){
            for (idx, function) in stbgr.functions.iter().enumerate() {
                func_cnt += 1;
                let func_define = stbgr
                    .module
                    .function_def_at(FunctionDefinitionIndex::new(idx as u16));
                if func_define.is_native() {
                    native_func_cnt += 1;
                    continue;

                };

                if detect_unchecked_return(function, &stbgr.symbol_pool, idx, stbgr.module) {
                    defects_cnt[0] += 1;
                }
                if detect_overflow(&packages, &stbgr, idx) {
                    defects_cnt[1] += 1;
                }
                if detect_precision_loss(function, &stbgr.symbol_pool) {
                    defects_cnt[2] += 1;
                }
                if detect_infinite_loop(&packages, &stbgr, idx) {
                    defects_cnt[3] += 1;
                }
                if detect_unnecessary_type_conversion(function, &function.local_types) {
                    defects_cnt[4] += 1;
                }
                if detect_unnecessary_bool_judgment(function, &function.local_types) {
                    defects_cnt[5] += 1;
                }
            }
            let unused_constants = detect_unused_constants(&stbgr);
            defects_cnt[6] += unused_constants.len();

            let unused_private_functions = detect_unused_private_functions(&stbgr);
            defects_cnt[7] += unused_private_functions.len();
        }
    }
    println!("bytecode_cnt:{}", bytecode_cnt);
    println!("bytecode_fail_to_deserialize_cnt:{}", bytecode_fail_to_deserialize_cnt);
    println!("func_cnt:{}", func_cnt);
    println!("native_func_cnt:{}", native_func_cnt);
    let mut i = 0;
    let mut sum = 0;
    while i < 8 {
        println!("{}:{}",defects_name[i],defects_cnt[i]);
        sum += defects_cnt[i];
        i += 1;
    }
    println!("total:{}", sum);
    let duration = start.elapsed();
    println!("Time elapsed in test_for_online_bytecodes() is: {:?}", duration);
}