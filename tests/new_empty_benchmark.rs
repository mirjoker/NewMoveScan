#![allow(unused_imports)]
#![allow(dead_code)]
use std::{
    fs::{File, self},
    io::{BufWriter, Write},
    os::fd,
    path::PathBuf, collections::BTreeMap,
};

use move_binary_format::access::ModuleAccess;

use MoveScanner::{utils::utils::visit_dirs, move_ir::packages::compile_module};
use num::complex::ComplexFloat;
use serde::{Serialize, Deserialize};

use std::io::{BufRead, BufReader};

use walkdir::WalkDir;


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Package {
    chain_type: usize, // 0->aptos, 1->sui, 2->move
    modules: BTreeMap<String, Module>,
}
impl Package {
    fn new() -> Self {
        Package { 
            chain_type: 2, 
            modules: BTreeMap::new(), 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Module {
    function: BTreeMap<String, FuctionTag>,
    constant: BTreeMap<String, bool>,
}
impl Module {
    fn new() -> Self {
        Module { 
            function: BTreeMap::new(), 
            constant: BTreeMap::new(), 
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

fn get_root_dir(start_directory: &str) -> Vec<(usize, String, PathBuf)> {
    let mut result: Vec<(usize, String, PathBuf)> = Vec::new();

    for entry in WalkDir::new(start_directory).follow_links(true) {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name == "Move.toml" {
                    let mut name = "".to_string();
                    let mut chain_type: usize = 2;
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
                                if line.contains("MystenLabs") || line.contains("sui") || line.contains("Sui") {
                                    chain_type = 1;
                                } else if line.contains("aptos") || line.contains("Aptos") {
                                    chain_type = 0;
                                }
                            }
                        }
                    }

                    let move_toml_path = entry.path();
                    if let Some(parent_dir) = move_toml_path.parent() {
                        let bytecode_dir = format!("build/{}/bytecode_modules/", name);
                        let bytecode_dir = parent_dir.join(bytecode_dir);
                        if bytecode_dir.exists()
                            && bytecode_dir.is_dir()
                        {
                            // println!("{} -> {}",parent_dir.to_str().unwrap(), name);
                            result.push((chain_type, name, bytecode_dir));
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
    let mut benchmark: BTreeMap<String, Package> = BTreeMap::new();
    for index in 0..3 {
        let root_dir = root_dirs[index];
        let root_dirs = get_root_dir(root_dir);

        for (chain_type, package_name, bytecode_dir) in root_dirs {
            let mut package = Package::new();
            package.chain_type = chain_type;

            let mut paths = Vec::new();
            visit_dirs(&bytecode_dir, &mut paths, false);
            paths.sort();
            for filename in paths {
                if let Some(cm) = compile_module(filename.clone()) {
                    let mut module = Module::new();

                    for fdef in cm.function_defs.iter() {
                        let fname = cm.function_handle_at(fdef.function).name;
                        let function_name = cm.identifier_at(fname).to_string();
                        module.function.insert(function_name, FuctionTag::new());
                    }

                    let constants = &cm.constant_pool;
                    for cst in constants.iter() {
                        module.constant.insert(format!("{:?}",cst), false);
                    }
                    
                    let mname = cm.module_handles[0].name;
                    let module_name = cm.identifier_at(mname).to_string();
                    package.modules.insert(module_name, module);

                } else {
                    println!("Fail to deserialize {:?} !!!", filename);
                }
            }

            let mut package_dir = bytecode_dir.to_str().unwrap().to_string();
            if let Some(start_index) = package_dir.find(root_dir) {
                let start_position = start_index + root_dir.len();
                if let Some(end_index) = package_dir[start_position..].find("/build") {
                    let end_position = start_position + end_index;
                    let result = &package_dir[start_position..end_position];
                    package_dir = result.to_string();
                }
            }
            benchmark.insert(package_dir, package);
        }
    }
    let mut file = fs::File::create("OpenSource_benchmark.json").expect("Failed to create json file");
    let json_result = serde_json::to_string(&benchmark).ok().unwrap();
    file.write(json_result.as_bytes()).expect("Failed to write to json file");
}
