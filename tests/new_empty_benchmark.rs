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
    constant: usize,
}
impl Module {
    fn new() -> Self {
        Module { 
            function: BTreeMap::new(), 
            constant: 0, 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FuctionTag {
    unused_private_functions: usize,
    infinite_loop: usize,
    overflow: usize,
    unnecessary_bool_judgment: usize,
    precision_loss: usize,
    unnecessary_type_conversion: usize,
    unchecked_return: usize
}
impl FuctionTag {
    fn new() -> Self {
        FuctionTag { 
            unused_private_functions: 0, 
            infinite_loop: 0, 
            overflow: 0, 
            unnecessary_bool_judgment: 0, 
            precision_loss: 0, 
            unnecessary_type_conversion: 0, 
            unchecked_return: 0 ,
        }
    }
}

fn get_root_dir(start_directory: &str) -> Vec<(String, PathBuf)> {
    let mut result: Vec<(String, PathBuf)> = Vec::new();

    for entry in WalkDir::new(start_directory).follow_links(true) {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name == "Move.toml" {
                    let mut name = "".to_string();
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
                            result.push((name, bytecode_dir));
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
    new_benchmark(root_dirs.to_vec(), "OpenSource_benchmark.json".to_owned());
    let root_dirs = ["../MoveScannerTest/Audit/res/repo/audit_project_set/aptos", "../MoveScannerTest/Audit/res/repo/audit_project_set/sui"];
    new_benchmark(root_dirs.to_vec(), "Audit_benchmark.json".to_owned());
}

fn new_benchmark(root_dirs: Vec<&str>, output: String) {
    let mut benchmark: BTreeMap<String, Package> = BTreeMap::new();
    for index in 0..root_dirs.len() {
        let root_dir = root_dirs[index];
        let root_dirs = get_root_dir(root_dir);

        for (package_name, bytecode_dir) in root_dirs {
            let mut package = Package::new();
            package.chain_type = index;

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
    let mut file = fs::File::create(output).expect("Failed to create json file");
    let json_result = serde_json::to_string(&benchmark).ok().unwrap();
    file.write(json_result.as_bytes()).expect("Failed to write to json file");
}