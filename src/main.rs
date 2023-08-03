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
};

fn main() {
    let mut detection_results = Detection_Results::new();
    let cli = Cli::parse();
    let dir = PathBuf::from(&cli.filedir);
    let mut paths = Vec::new();
    utils::visit_dirs(&dir, &mut paths, false);

    let mut cms = Vec::new();
    // let mut failed_modules_count: usize = 0;
    for filename in paths {
        println!("Deserializing {:?}...", filename);
        if let Some(cm) = compile_module(filename) {
            detection_results.modules_count += 1;
            cms.push(cm);
        } else {
            detection_results.failed_modules_count += 1;
            // failed_modules_count = failed_modules_count + 1;
        }
    }

    // let all_modules = Modules::new(&cms);
    // let dep_graph = all_modules.compute_dependency_graph();
    // let modules = dep_graph.compute_topological_order().unwrap();

    let mut stbgrs = Vec::new();
    for cm in cms.iter() {
        let mut stbgr = StacklessBytecodeGenerator::new(&cm);
        stbgr.generate_function();
        stbgr.get_control_flow_graph();
        stbgr.build_call_graph();
        stbgr.get_data_dependency(&mut stbgrs);
        stbgrs.push(stbgr);
    }

    let mut packages = Packages::new();
    for stbgr in stbgrs.iter() {
        packages.insert_stbgr(stbgr);
    }

    // let mut result = Map::new();
    let start = Instant::now();
    // let mut result_modules = Map::new();
    for (mname, &stbgr) in packages.get_all_stbgr().iter() {
        // 记录每个module的分析市场，函数对应的威胁
        // let mut result_mname = Map::new();
        let start = Instant::now();
        detection_results.modules.insert(mname.to_string(), Module_Details::new());
        match &cli.command {
            Some(Commands::Printer { printer }) => {
                match &printer {
                    Some(Infos::CFG) => {
                        let dot_dir = "./dots";
                        if !fs::metadata(dot_dir).is_ok() {
                            match fs::create_dir(dot_dir) {
                                Ok(_) => {}
                                Err(err) => println!("Failed to create folder: {}", err),
                            };
                        }
                        for (idx, function) in stbgr.functions.iter().enumerate() {
                            let name = stbgr.module.identifier_at(
                                stbgr
                                    .module
                                    .function_handle_at(stbgr.module.function_defs[idx].function)
                                    .name,
                            );
                            let filename = PathBuf::from(format!("{}/{}.dot", dot_dir, name));
                            generate_cfg_in_dot_format(&stbgr.functions[idx], filename, &stbgr);
                            function.cfg.as_ref().unwrap().display();
                        }
                    }
                    Some(Infos::IR) => {
                        println!("{}", stbgr.display(true, None));
                    }
                    Some(Infos::CM) => {
                        println!("{:#?}", stbgr.module);
                    }
                    Some(Infos::FNs) => {
                        println!("{}", stbgr.display(false, None));
                    }
                    Some(Infos::DU) => {
                        for (idx, function) in stbgr.functions.iter().enumerate() {
                            println!("{:?}", &function.def_attrid);
                            println!("{:?}", &function.use_attrid);
                        }
                    }
                    Some(Infos::CG) => {
                        // let dot_dir = "./dots";
                        let graph = stbgr.call_graph2str();
                        let dot_graph = format!(
                            "{}",
                            Dot::with_attr_getters(&graph, &[], &|_, _| "".to_string(), &|_, _| {
                                "shape=box".to_string()
                            })
                        );
                        let dotfile = PathBuf::from(format!("{}.dot", mname));
                        fs::write(&dotfile, &dot_graph).expect("generating dot file for CFG");
                    }
                    _ => {}
                }
            }
            Some(Commands::Detection { detection }) => {
                println!("============== Handling for {} ==============", mname);
                let mut detects: Vec<Vec<usize>> = vec![Vec::new(); 6];
                match *detection {
                    Some(Defects::UncheckedReturn) => {
                        stbgr
                            .functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                if detect_unchecked_return(
                                    function,
                                    &stbgr.symbol_pool,
                                    idx,
                                    stbgr.module,
                                ) {
                                    detects[0].push(idx);
                                }
                            })
                            .for_each(drop);
                    }
                    Some(Defects::Overflow) => {
                        stbgr
                            .functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                if detect_overflow(&packages, &stbgr, idx) {
                                    detects[1].push(idx);
                                }
                            })
                            .for_each(drop);
                    }
                    Some(Defects::PrecisionLoss) => {
                        stbgr
                            .functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                if detect_precision_loss(function, &stbgr.symbol_pool) {
                                    detects[2].push(idx);
                                }
                            })
                            .for_each(drop);
                    }
                    Some(Defects::InfiniteLoop) => {
                        stbgr
                            .functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                if detect_infinite_loop(&packages, &stbgr, idx) {
                                    detects[3].push(idx);
                                }
                            })
                            .for_each(drop);
                    }
                    Some(Defects::UnnecessaryTypeConversion) => {
                        let _ = stbgr
                            .functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                if detect_unnecessary_type_conversion(
                                    function,
                                    &function.local_types,
                                ) {
                                    detects[4].push(idx);
                                }
                            })
                            .for_each(drop);
                    }
                    Some(Defects::UnnecessaryBoolJudgment) => {
                        stbgr
                            .functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                if detect_unnecessary_bool_judgment(function, &function.local_types)
                                {
                                    detects[5].push(idx);
                                }
                            })
                            .for_each(drop);
                    }
                    Some(Defects::UnusedConstant) => {
                        let unused_constants = detect_unused_constants(&stbgr);
                        println!("Unused constants: {:?}", unused_constants);
                    }
                    Some(Defects::UnusedPrivateFunctions) => {
                        let unused_private_functions = detect_unused_private_functions(&stbgr);
                        let unused_private_function_names = unused_private_functions
                            .iter()
                            .map(|func| func.symbol().display(&stbgr.symbol_pool).to_string())
                            .collect_vec();
                        println!(
                            "Unused private functions: {:?}",
                            unused_private_function_names
                        );
                    }
                    None => {}
                }
            }
            None => {
                println!(
                    "============== Handling for {} ==============",
                    mname.clone()
                );
                let mut detects: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); 6];
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
                        detect_result.get_mut("Unchecked_return").unwrap().
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
                if !unused_constants.is_empty() {
                    println!("Unused constants: {:?}", unused_constants);
                }
                let unused_private_functions = detect_unused_private_functions(&stbgr);
                let unused_private_function_names = unused_private_functions
                    .iter()
                    .map(|func| func.symbol().display(&stbgr.symbol_pool).to_string())
                    .collect_vec();
                if !unused_private_function_names.is_empty() {
                    println!(
                        "Unused private functions: {:?}",
                        unused_private_function_names
                    );
                }
                format_result(&detects, stbgr.module);
                println!("==============================================\n");

                // json文件
                for (_, c) in unused_constants.iter().enumerate() {
                    detection_results.modules.get_mut(mname).unwrap().
                    detect_result.get_mut("Unused_Constant").unwrap().
                    push(format!("{:?}", c));
                }
                // detection_results.modules.get_mut(mname).unwrap().detect_result.get_mut("Unused_Constant").unwrap().push(format!("{}", unused_constants.len()));
                // detection_results.modules.get_mut(mname).unwrap().detect_result.get_mut("Unused_Constant").unwrap().push(format!("{:?}", unused_constants));
                detection_results.modules.get_mut(mname).unwrap().
                detect_result.get_mut("Unused_Private_Functions").unwrap().append(&mut unused_private_function_names.clone());
                // let mut result_detects = Map::new();
                // result_detects.insert(
                //     "Unused constants num".to_string(),
                //     Value::Number(unused_constants.len().into()),
                // );
                // result_detects.insert(
                //     "Unused private functions".to_string(),
                //     Value::Array(
                //         unused_private_function_names
                //             .iter()
                //             .map(|x| Value::String(x.clone()))
                //             .collect(),
                //     ),
                // );
                // for (i, detect) in detects.iter().enumerate() {
                //     result_detects.insert(
                //         DETECT_TYPES[i].to_string(),
                //         Value::Array(
                //             detect
                //                 .iter()
                //                 .map(|&x| Value::String(stbgr.functions[x].name.clone()))
                //                 .collect(),
                //         ),
                //     );
                // }
                // result_mname.insert("detects".to_string(), Value::Object(result_detects));

                // let mut result_functions = Map::new();
                for (idx, function) in stbgr.functions.iter().enumerate() {
                    let fname = &function.name;
                    let mut result4 = vec![];
                    for (i, detect) in detects.iter().enumerate() {
                        if detect.contains(&idx) {
                            result4.push(DETECT_TYPES[i].to_string());
                        }
                    }
                    detection_results.modules.get_mut(mname).unwrap().functions.insert(fname.clone(), result4);
                    // result_functions.insert(
                    //     fname.clone(),
                    //     Value::Array(result4.iter().map(|&x| Value::String(x.into())).collect()),
                    // );
                }
                detection_results.modules.get_mut(mname).unwrap().function_counts = stbgr.functions.len();
                // result_mname.insert("function_counts".to_string(), Value::Number(stbgr.functions.len().into()));
                // result_mname.insert("functions".to_string(), Value::Object(result_functions));
                // let duration = start.elapsed();
                let duration = start.elapsed().as_micros().to_usize().unwrap();
                detection_results.modules.get_mut(mname).unwrap().time = duration;
                // result_mname.insert("time(ms)".to_string(), Value::Number(duration.into()));
            }
        }
        // result_modules.insert(mname.clone(), Value::Object(result_mname));
    }
    // let duration = start.elapsed();
    let duration = start.elapsed().as_micros().to_usize().unwrap();
    detection_results.total_time = duration;
    // result.insert("total_time(ms)".to_string(), Value::Number(duration.into()));
    // result.insert("failed_module_counts".to_string(), Value::Number(failed_modules_count.into()));
    // result.insert(
    //     "module_counts".to_string(),
    //     Value::Number(packages.get_all_stbgr().len().into()),
    // );
    // result.insert("modules".to_string(), Value::Object(result_modules));
    if let Some(json_file) = &cli.json_file {
        let json_output = serde_json::to_string(&detection_results).ok().unwrap();
        let mut file = fs::File::create(json_file).expect("Failed to create json file");
        file.write(json_output.as_bytes())
            .expect("Failed to write to json file");
    }
}

static DETECT_TYPES: [&'static str; 6] = [
    "Unchecked return",
    "Overflow",
    "Precision loss",
    "Infinite loop",
    "Unnecessary type conversion",
    "Unnecessary bool judgment",
];

fn format_result(detects: &Vec<BTreeSet<usize>>, cm: &CompiledModule) {
    for (i, d_type) in DETECT_TYPES.iter().enumerate() {
        if detects[i].len() == 0 {
            continue;
        }
        let detect_fname = detects[i]
            .iter()
            .map(|idx| {
                let handle = cm.function_handle_at(cm.function_defs[*idx].function);
                cm.identifier_at(handle.name).as_str()
            })
            .collect_vec();
        println!("{}: {:?}", *d_type, detect_fname);
    }
}
