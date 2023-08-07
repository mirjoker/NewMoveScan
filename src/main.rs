use std::borrow::BorrowMut;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::time::Instant;
use std::{fmt::format, fs, path::PathBuf, str::FromStr, vec};

use clap::Parser;
use itertools::Itertools;
use move_binary_format::{
    access::ModuleAccess, file_format::FunctionDefinitionIndex, CompiledModule,
};
use move_bytecode_utils::Modules;
use num::ToPrimitive;
use petgraph::dot::{Config, Dot};
use petgraph::graph::Graph;
use serde_json::{json, Map, Value};
use MoveScanner::move_ir::packages::Packages;
use MoveScanner::{
    cli::parser::*,
    detect::{
        detect1::detect_unchecked_return, detect2::detect_overflow, detect3::detect_precision_loss,
        detect4::detect_infinite_loop, detect5::detect_unused_constants,
        detect6::detect_unused_private_functions, detect7::detect_unnecessary_type_conversion,
        detect8::detect_unnecessary_bool_judgment,
    },
    move_ir::{
        bytecode_display::display,
        control_flow_graph::generate_cfg_in_dot_format,
        generate_bytecode::StacklessBytecodeGenerator,
        sbir_generator::{Blockchain, MoveScanner as Mc},
    },
    utils::utils::{self, compile_module},
    utils::result_format::{self, Detection_Results, Module_Details},
    utils::defect_enum::Defects,
};

fn main() {

    let mut detection_results = Detection_Results::new();

    // 命令行参数解析
    let cli = Cli::parse();
    let dir = PathBuf::from(&cli.filedir);
    // 输入路径遍历
    let mut paths = Vec::new();
    utils::visit_dirs(&dir, &mut paths, false);
    // 输入文件解析(反序列化成CompiledModule)
    let mut cms = Vec::new();
    for filename in paths {
        // println!("Deserializing {:?}...", filename);
        if let Some(cm) = compile_module(filename.clone()) {
            detection_results.modules_count += 1;
            cms.push(cm);
        } else {
            println!("Fail to deserialize {:?} !!!", filename);
            detection_results.failed_modules_count += 1;
        }
    }
    // 根据cms构建StacklessBytecodeGenerator，并进行IR转换、cfg构建、call_gragh构建、data_dependency分析
    let mut stbgrs = Vec::new();
    for cm in cms.iter() {
        let mut stbgr = StacklessBytecodeGenerator::new(&cm);
        stbgr.generate_function();
        stbgr.get_control_flow_graph();
        stbgr.build_call_graph();
        stbgr.get_data_dependency(&mut stbgrs);
        stbgrs.push(stbgr);
    }
    // package构建
    let mut packages = Packages::new();
    for stbgr in stbgrs.iter() {
        packages.insert_stbgr(stbgr);
    }

    // 开始检测
    let start = Instant::now();
    // 遍历packages中的stbgr
    for (mname, &stbgr) in packages.get_all_stbgr().iter() {
        let start = Instant::now();
        // 向detection_results.modules插入该module的检测信息
        detection_results.modules.insert(mname.to_string(), Module_Details::new());
        detection_results.modules.get_mut(mname).unwrap().constant_counts = stbgr.module.constant_pool.len();
        
        // println!(
        //     "============== Handling for {} ==============",
        //     mname.clone()
        // );
        let mut detects: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); 6];
        // 遍历stbgr中的functions
        for (idx, function) in stbgr.functions.iter().enumerate() {
            let func_define = stbgr
                .module
                .function_def_at(FunctionDefinitionIndex::new(idx as u16));
            detection_results.modules.get_mut(mname).unwrap().function_counts += 1;
            if func_define.is_native() {
                detection_results.modules.get_mut(mname).unwrap().native_function_counts += 1;
                continue;
            };

            if detect_unchecked_return(function, &stbgr.symbol_pool, idx, stbgr.module) {
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Unchecked_Return").unwrap().
                push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
                detects[0].insert(idx);
            }
            if detect_overflow(&packages, &stbgr, idx) {
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Overflow").unwrap().
                push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
                detects[1].insert(idx);
            }
            if detect_precision_loss(function, &stbgr.symbol_pool) {
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Precision_Loss").unwrap().
                push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
                detects[2].insert(idx);
            }
            if detect_infinite_loop(&packages, &stbgr, idx) {
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Infinite_Loop").unwrap().
                push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
                detects[3].insert(idx);
            }
            if detect_unnecessary_type_conversion(function, &function.local_types) {
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Unnecessary_Type_Conversion").unwrap().
                push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
                detects[4].insert(idx);
            }
            if detect_unnecessary_bool_judgment(function, &function.local_types) {
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Unnecessary_Bool_Judgment").unwrap().
                push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
                detects[5].insert(idx);
            }
        }
        let unused_constants = detect_unused_constants(&stbgr);
        let unused_private_functions = detect_unused_private_functions(&stbgr);
        let unused_private_function_names = unused_private_functions
            .iter()
            .map(|func| func.symbol().display(&stbgr.symbol_pool).to_string())
            .collect_vec();

        // json文件
        for (_, c) in unused_constants.iter().enumerate() {
            detection_results.modules.get_mut(mname).unwrap().
            detect_result.get_mut("Unused_Constant").unwrap().
            push(format!("{:?}", c));
        }
        detection_results.modules.get_mut(mname).unwrap().
        detect_result.get_mut("Unused_Private_Functions").unwrap().append(&mut unused_private_function_names.clone());
        
        for (idx, function) in stbgr.functions.iter().enumerate() {
            let fname = &function.name;
            let mut tmp = vec![];
            for (i, detect) in detects.iter().enumerate() {
                if detect.contains(&idx) {
                    tmp.push(Defects::get_defect_neme(i));
                }
            }
            if unused_private_function_names.contains(fname) {
                tmp.push(Defects::get_defect_neme(7));
            }
            detection_results.modules.get_mut(mname).unwrap().functions.insert(fname.clone(), tmp);
        }
        detection_results.modules.get_mut(mname).unwrap().function_counts = stbgr.functions.len();
        let duration = start.elapsed().as_micros().to_usize().unwrap();
        detection_results.modules.get_mut(mname).unwrap().time = duration;

    }
    let duration = start.elapsed().as_micros().to_usize().unwrap();
    detection_results.total_time = duration;
    if let Some(json_file) = &cli.json_file {
        let json_output = serde_json::to_string(&detection_results).ok().unwrap();
        let mut file = fs::File::create(json_file).expect("Failed to create json file");
        file.write(json_output.as_bytes())
            .expect("Failed to write to json file");
    }
}
