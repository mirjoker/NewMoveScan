// #[test]
// fn test_for_online_bytecodes() {
//     let start = Instant::now();
//     let mut detection_results = DetectionResults::new();
//     let dir = PathBuf::from("/home/yww/MoveScannerTest/src/sui_onchain_bytecode");
//     let mut paths = Vec::new();
//     utils::visit_dirs(&dir, &mut paths, false);

//     let mut bytecode_cnt = 0;
//     let mut bytecode_fail_to_deserialize_cnt = 0;
//     let mut func_cnt = 0;
//     let mut native_func_cnt = 0;
//     let mut constant_cnt = 0;
//     let mut defects_cnt = vec![0,0,0,0,0,0,0,0];
//     // let defects_name = vec!["Unchecked_return","Overflow","PrecisionLoss","InfiniteLoop",
//     // "UnnecessaryTypeConversion","UnnecessaryBoolJudgment","UnusedConstant","UnusedPrivateFunctions"];
//     for filename in paths.iter() {
//         bytecode_cnt += 1;
//         let cm = compile_module(filename.to_path_buf());
//         if(cm.is_none()) {
//             bytecode_fail_to_deserialize_cnt += 1;
//             detection_results.failed_modules_count += 1;
//             println!("Fail to deserialize {:?} !!!", filename);
//             continue;
//         } else {
//             detection_results.modules_count += 1;
//         }
//         let cm = cm.unwrap();
//         let mut stbgrs = Vec::new();
//         let mut stbgr = StacklessBytecodeGenerator::new(&cm);
//         stbgr.generate_function();
//         stbgr.get_control_flow_graph();
//         stbgr.build_call_graph();
//         stbgr.get_data_dependency(&mut stbgrs);

//         let mut packages = Packages::new();
//         packages.insert_stbgr(&stbgr);

//         for (_, &stbgr) in packages.get_all_stbgr().iter(){
//             let mname = filename.file_name().unwrap().to_str().unwrap();
//             let start = Instant::now();
//             detection_results.modules.insert(mname.to_string(), ModuleDetails::new());
//             detection_results.modules.get_mut(mname).unwrap().constant_counts = stbgr.module.constant_pool.len();
//             constant_cnt += stbgr.module.constant_pool.len();
//             let mut detects: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); 6];
//             for (idx, function) in stbgr.functions.iter().enumerate() {
//                 func_cnt += 1;
//                 let func_define = stbgr
//                     .module
//                     .function_def_at(FunctionDefinitionIndex::new(idx as u16));
//                 if func_define.is_native() {
//                     native_func_cnt += 1;
//                     detection_results.modules.get_mut(mname).unwrap().native_function_counts += 1;
//                     continue;

//                 };

//                 if detect_unchecked_return(function, &stbgr.symbol_pool, idx, stbgr.module) {
//                     detection_results.modules.get_mut(mname).unwrap().
//                     detect_result.get_mut("UncheckedReturn").unwrap().
//                     push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
//                     defects_cnt[0] += 1;
//                     detects[0].insert(idx);
//                 }
//                 if detect_overflow(&packages, &stbgr, idx) {
//                     detection_results.modules.get_mut(mname).unwrap().
//                     detect_result.get_mut("Overflow").unwrap().
//                     push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
//                     defects_cnt[1] += 1;
//                     detects[1].insert(idx);
//                 }
//                 if detect_precision_loss(function, &stbgr.symbol_pool) {
//                     detection_results.modules.get_mut(mname).unwrap().
//                     detect_result.get_mut("PrecisionLoss").unwrap().
//                     push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
//                     defects_cnt[2] += 1;
//                     detects[2].insert(idx);
//                 }
//                 if detect_infinite_loop(&packages, &stbgr, idx) {
//                     detection_results.modules.get_mut(mname).unwrap().
//                     detect_result.get_mut("InfiniteLoop").unwrap().
//                     push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
//                     defects_cnt[3] += 1;
//                     detects[3].insert(idx);
//                 }
//                 if detect_unnecessary_type_conversion(function, &function.local_types) {
//                     detection_results.modules.get_mut(mname).unwrap().
//                     detect_result.get_mut("UnnecessaryTypeConversion").unwrap().
//                     push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
//                     defects_cnt[4] += 1;
//                     detects[4].insert(idx);
//                 }
//                 if detect_unnecessary_bool_judgment(function, &function.local_types) {
//                     detection_results.modules.get_mut(mname).unwrap().
//                     detect_result.get_mut("UnnecessaryBoolJudgment").unwrap().
//                     push(stbgr.module.identifier_at(stbgr.module.function_handle_at(stbgr.module.function_defs[idx].function).name).to_string());
//                     defects_cnt[5] += 1;
//                     detects[5].insert(idx);
//                 }
//             }
//             let unused_constants = detect_unused_constants(&stbgr);
//             defects_cnt[6] += unused_constants.len();
//             for (_, c) in unused_constants.iter().enumerate() {
//                 detection_results.modules.get_mut(mname).unwrap().
//                 detect_result.get_mut("UnusedConstant").unwrap().
//                 push(format!("{:?}", c));
//             }
//             let unused_private_functions = detect_unused_private_functions(&stbgr);
//             let unused_private_function_names = unused_private_functions
//                 .iter()
//                 .map(|func| func.symbol().display(&stbgr.symbol_pool).to_string())
//                 .collect_vec();
//             defects_cnt[7] += unused_private_functions.len();
//             detection_results.modules.get_mut(mname).unwrap().
//             detect_result.get_mut("UnusedPrivateFunctions").unwrap().append(&mut unused_private_function_names.clone());


//             for (idx, function) in stbgr.functions.iter().enumerate() {
//                 let fname = &function.name;
//                 let mut tmp = vec![];
//                 for (i, detect) in detects.iter().enumerate() {
//                     if detect.contains(&idx) {
//                         tmp.push(Defects::get_defect_neme(i));
//                     }
//                 }
//                 if unused_private_function_names.contains(fname) {
//                     tmp.push(Defects::get_defect_neme(7))
//                 }
//                 detection_results.modules.get_mut(mname).unwrap().functions.insert(fname.clone(), tmp);
//             }
//             detection_results.modules.get_mut(mname).unwrap().function_counts = stbgr.functions.len();
//             let duration = start.elapsed().as_micros().to_usize().unwrap();
//             detection_results.modules.get_mut(mname).unwrap().time = duration;
//         }
//     }
//     println!("bytecode_cnt:{}", bytecode_cnt);
//     println!("bytecode_fail_to_deserialize_cnt:{}", bytecode_fail_to_deserialize_cnt);
//     println!("func_cnt:{}", func_cnt);
//     println!("native_func_cnt:{}", native_func_cnt);
//     println!("constant_cnt:{}", constant_cnt);
//     let mut i = 0;
//     let mut sum = 0;
//     while i < 8 {
//         println!("{}:{}",Defects::get_defect_neme(i),defects_cnt[i]);
//         sum += defects_cnt[i];
//         i += 1;
//     }
//     println!("total:{}", sum);
//     let duration = start.elapsed();
//     println!("Time elapsed in test_for_online_bytecodes() is: {:?}", duration);
//     let duration = duration.as_micros().to_usize().unwrap();
//     detection_results.total_time = duration;

//     let json_output = serde_json::to_string(&detection_results).ok().unwrap();
//     let mut file = fs::File::create("sui_chaincode_result.json").expect("Failed to create json file");
//     file.write(json_output.as_bytes())
//     .expect("Failed to write to json file");

// }