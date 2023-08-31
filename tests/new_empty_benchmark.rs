#![allow(unused_imports)]
#![allow(dead_code)]
use std::{
    fs::{File, self},
    io::{BufWriter, Write},
    os::fd,
    path::PathBuf,
};

use move_binary_format::access::ModuleAccess;

use MoveScanner::{utils::utils::visit_dirs, move_ir::packages::compile_module};
use num::complex::ComplexFloat;
use serde::{Serialize, Deserialize};

use std::io::{BufRead, BufReader};

use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Benchmark {
    package_name: String,
    package: Package,
}
impl Benchmark {
    fn new() -> Self {
        Benchmark { 
            package_name: String::new(), 
            package: Package::new() 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Package {
    module_name: String,
    chain_type: usize, // 0->aptos, 1->sui, 2->move
    function: Function,
    constant: Const,
}
impl Package {
    fn new() -> Self {
        Package { 
            module_name: String::new(), 
            chain_type: 2, 
            function: Function::new(), 
            constant: Const::new() 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Function{
    function_name: String,
    functiontag: FuctionTag,
}
impl Function {
    fn new() -> Self {
        Function { 
            function_name: String::new(), 
            functiontag: FuctionTag::new() 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Const{
    constant_name: String, // name = type + value
    tag: bool,
}
impl Const {
    fn new() -> Self {
        Const { 
            constant_name: String::new(), 
            tag: false 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FuctionTag {
    unused_private_functions: bool,
    recursive_function_call: bool,
    infinite_loop: bool,
    overflow: bool,
    unnecessary_bool_judgment: bool,
    unused_constant: bool,
    precision_loss: bool,
    unnecessary_type_conversion: bool,
    unchecked_return: bool
}
impl FuctionTag {
    fn new() -> Self {
        FuctionTag { 
            unused_private_functions: false, 
            recursive_function_call: false, 
            infinite_loop: false, 
            overflow: false, 
            unnecessary_bool_judgment: false, 
            unused_constant: false, 
            precision_loss: false, 
            unnecessary_type_conversion: false, 
            unchecked_return: false ,
        }
    }
}

fn get_root_dir(start_directory: &str) -> Vec<(bool, String, PathBuf, PathBuf)> {
    let mut result: Vec<(bool, String, PathBuf, PathBuf)> = Vec::new();

    for entry in WalkDir::new(start_directory).follow_links(true) {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name == "Move.toml" {
                    let mut name = "".to_string();
                    let mut is_aptos = true;
                    if let Ok(file) = File::open(entry.path()) {
                        let reader = BufReader::new(file);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                if line.contains("name = \"") {
                                    let _name = line
                                        .split("name = \"")
                                        .nth(1)
                                        .and_then(|s| s.split('"').next())
                                        .unwrap_or("")
                                        .to_string();
                                    name = _name;
                                } else if line.contains("name = \'") {
                                    let _name = line
                                        .split("name = \'")
                                        .nth(1)
                                        .and_then(|s| s.split('\'').next())
                                        .unwrap_or("")
                                        .to_string();
                                    name = _name;
                                }
                                if line.contains("MystenLabs") || line.contains("sui"){
                                    is_aptos = false;
                                } else if line.contains("aptos") {
                                    is_aptos = true;
                                }
                            }
                        }
                    }

                    let move_toml_path = entry.path();
                    if let Some(parent_dir) = move_toml_path.parent() {
                        let bytecode_dir = format!("build/{}/bytecode_modules/", name);
                        let bytecode_dir = parent_dir.join(bytecode_dir);
                        let source_dir = format!("build/{}/sources/", name);
                        let source_dir = parent_dir.join(source_dir);
                        if bytecode_dir.exists()
                            && bytecode_dir.is_dir()
                            && source_dir.exists()
                            && source_dir.is_dir()
                        {
                            // println!("{} -> {}",parent_dir.to_str().unwrap(), name);
                            result.push((is_aptos, name, bytecode_dir, source_dir));
                        } else {
                            // println!("Failed! {} -> {}", parent_dir.to_str().unwrap(), name);
                            println!("Not found bytecode_file directory: {:?}", name);
                        }
                    }
                }
            }
        }
    }
    result
}

#[test]
fn build_benchmark() {
    let root_dirs = ["../MoveScannerTest/OpenSource/res/repo/Aptos/", "../MoveScannerTest/OpenSource/res/repo/Sui/", "../MoveScannerTest/OpenSource/res/repo/Move/"];
    let mut benchmark = Benchmark::new();
    // for index in 0..3 {
    //     let root_dir = root_dirs[index];

    //     let root_dirs = get_root_dir(root_dir);
    //     let mut packages: Vec<DatasetFeature> = Vec::new();
    //     for (is_aptos, package_name, bytecode_dir, source_dir) in root_dirs {
    //         let mut paths = Vec::new();
    //         visit_dirs(&bytecode_dir, &mut paths, false);
    //         paths.sort();
    //         let mut cms = Vec::new();
    //         for filename in paths {
    //             if let Some(cm) = compile_module(filename.clone()) {
    //                 cms.push(cm);
    //             } else {
    //                 println!("Fail to deserialize {:?} !!!", filename);
    //             }
    //         }

    //         let mut paths = Vec::new();
    //         visit_dirs(&source_dir, &mut paths, false);
    //         paths.sort();

    //         let mut modules: Vec<Module> = Vec::new();
    //         for (i, cm) in cms.iter().enumerate() {
    //             let mut functions: Vec<Function> = Vec::new();
    //             for fdef in cm.function_defs.iter() {
    //                 let fname = cm.function_handle_at(fdef.function).name;
    //                 let function_name = cm.identifier_at(fname).to_string();
    //                 let cnt_opcodes = if fdef.code.as_ref().is_some() {
    //                     fdef.code.as_ref().unwrap().code.len()
    //                 } else {
    //                     0
    //                 };
    //                 functions.push(Function::new(function_name, cnt_opcodes))
    //             }
    //             let mname = cm.module_handles[0].name;
    //             let module_name = cm.identifier_at(mname).to_string();
    //             let cnt_constants = cm.constant_pool().len();
    //             let source_file = File::open(paths[i].clone()).expect("Failed to open file");
    //             let reader = BufReader::new(source_file);
    //             let cnt_codes = reader.lines().count();
    //             let module = Module::new(module_name, cnt_constants, cnt_codes, functions);
    //             modules.push(module);
    //         }

    //         let mut package_dir = bytecode_dir.to_str().unwrap().to_string();
    //         if let Some(start_index) = package_dir.find(root_dir) {
    //             let start_position = start_index + root_dir.len();
    //             if let Some(end_index) = package_dir[start_position..].find("/build") {
    //                 let end_position = start_position + end_index;
    //                 let result = &package_dir[start_position..end_position];
    //                 package_dir = result.to_string();
    //             }
    //         }

    //         let feature = DatasetFeature::new(is_aptos, package_dir, package_name, modules);
    //         packages.push(feature);
    //     }

    //     let file = File::create(result_path).expect("Failed to create file");
    //     let mut writer = BufWriter::new(file);
    //     let header = "package_dir,package_name,aptos_or_sui,cnt_modules,cnt_functions,cnt_constants,cnt_codes,cnt_opcodes";
    //     writeln!(writer, "{}", header).expect("Failed to write to file");

    //     for feature in &packages {
    //         let is_aptos = if feature.is_aptos { "aptos" } else { "sui" };
    //         let line = format!(
    //             "{},{},{},{},{},{},{},{}",
    //             feature.package_dir,
    //             feature.package_name,
    //             is_aptos,
    //             feature.cnt_modules,
    //             feature.cnt_functions,
    //             feature.cnt_constants,
    //             feature.cnt_codes,
    //             feature.cnt_opcodes
    //         );
    //         writeln!(writer, "{}", line).expect("Failed to write to file");
    //     }
    // }
    let mut file = fs::File::create("OpenSource_benchmark.json").expect("Failed to create json file");
    let json_result = serde_json::to_string(&benchmark).ok().unwrap();
    file.write(json_result.as_bytes()).expect("Failed to write to json file");
}
