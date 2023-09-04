#![allow(unused_imports)]
#![allow(dead_code)]
use std::{
    fs::File,
    io::{BufWriter, Write},
    os::fd,
    path::PathBuf,
};

use move_binary_format::access::ModuleAccess;

use MoveScanner::{utils::utils::visit_dirs, move_ir::packages::compile_module};
use num::complex::ComplexFloat;

use std::io::{BufRead, BufReader};

use walkdir::WalkDir;

struct Function {
    _function_name: String,
    cnt_opcodes: usize,
}

impl Function {
    fn new(_function_name: String, cnt_opcodes: usize) -> Self {
        Function {
            _function_name,
            cnt_opcodes,
        }
    }
}

struct Module {
    _module_name: String,
    cnt_functions: usize,
    cnt_constants: usize,
    cnt_codes: usize,
    cnt_opcodes: usize,
    _functions: Vec<Function>,
}

impl Module {
    fn new(
        _module_name: String,
        cnt_constants: usize,
        cnt_codes: usize,
        _functions: Vec<Function>,
    ) -> Self {
        let cnt_functions = _functions.len();
        let cnt_opcodes = _functions.iter().map(|f| f.cnt_opcodes).sum();
        Module {
            _module_name,
            cnt_functions,
            cnt_constants,
            cnt_codes,
            cnt_opcodes,
            _functions,
        }
    }
}

struct DatasetFeature {
    chain_type: usize,
    package_dir: String,
    package_name: String,
    cnt_modules: usize,
    cnt_functions: usize,
    cnt_constants: usize,
    cnt_codes: usize,
    cnt_opcodes: usize,
    _modules: Vec<Module>,
}

impl DatasetFeature {
    fn new(chain_type: usize, package_dir: String, package_name: String, _modules: Vec<Module>) -> Self {
        let cnt_modules = _modules.len();
        let cnt_functions = _modules.iter().map(|m| m.cnt_functions).sum();
        let cnt_constants = _modules.iter().map(|m| m.cnt_constants).sum();
        let cnt_codes = _modules.iter().map(|m| m.cnt_codes).sum();
        let cnt_opcodes = _modules.iter().map(|m| m.cnt_opcodes).sum();
        DatasetFeature {
            chain_type,
            package_dir,
            package_name,
            cnt_modules,
            cnt_functions,
            cnt_constants,
            cnt_codes,
            cnt_opcodes,
            _modules,
        }
    }

    fn to_json(&self, _json_file: PathBuf) {
        println!(
            "{}: {} {} {} {} {}",
            self.package_name,
            self.cnt_modules,
            self.cnt_functions,
            self.cnt_constants,
            self.cnt_codes,
            self.cnt_opcodes
        );
    }
}

fn get_root_dir(start_directory: &str) -> Vec<(String, PathBuf, PathBuf)> {
    let mut result: Vec<(String, PathBuf, PathBuf)> = Vec::new();

    for entry in WalkDir::new(start_directory).follow_links(true) {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name == "Move.toml" {
                    let mut name = "".to_string();
                    // let mut is_aptos = true;
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
                                // if line.contains("MystenLabs") || line.contains("sui"){
                                //     is_aptos = false;
                                // } else if line.contains("aptos") {
                                //     is_aptos = true;
                                // }
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
                            result.push((name, bytecode_dir, source_dir));
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
fn test_run() {
    let root_dirs = ["../MoveScannerTest/OpenSource/res/repo/Aptos/", "../MoveScannerTest/OpenSource/res/repo/Sui/", "../MoveScannerTest/OpenSource/res/repo/Move/"];
    let result_paths = ["../MoveScannerTest/OpenSource/res/features/OpenSourceAptos.csv", "../MoveScannerTest/OpenSource/res/features/OpenSourceSui.csv", "../MoveScannerTest/OpenSource/res/features/OpensourceMove.csv"];
    count_feature(root_dirs.to_vec(), result_paths.to_vec());
    let root_dirs = ["../MoveScannerTest/Audit/res/repo/audit_project_set/aptos", "../MoveScannerTest/Audit/res/repo/audit_project_set/sui"];
    let result_paths = ["../MoveScannerTest/Audit/res/features/AuditAptos.csv", "../MoveScannerTest/Audit/res/features/AuditSui.csv"];
    count_feature(root_dirs.to_vec(), result_paths.to_vec());
}

fn count_feature(root_dirs: Vec<&str>, result_paths: Vec<&str>) {
    for index in 0..root_dirs.len() {
        let root_dir = root_dirs[index];
        let result_path = result_paths[index];

        let root_dirs = get_root_dir(root_dir);
        let mut packages: Vec<DatasetFeature> = Vec::new();
        for (package_name, bytecode_dir, source_dir) in root_dirs {
            let mut paths = Vec::new();
            visit_dirs(&bytecode_dir, &mut paths, false);
            paths.sort();
            let mut cms = Vec::new();
            for filename in paths {
                if let Some(cm) = compile_module(filename.clone()) {
                    cms.push(cm);
                } else {
                    println!("Fail to deserialize {:?} !!!", filename);
                }
            }

            let mut paths = Vec::new();
            visit_dirs(&source_dir, &mut paths, false);
            paths.sort();

            let mut modules: Vec<Module> = Vec::new();
            for (i, cm) in cms.iter().enumerate() {
                let mut functions: Vec<Function> = Vec::new();
                for fdef in cm.function_defs.iter() {
                    let fname = cm.function_handle_at(fdef.function).name;
                    let function_name = cm.identifier_at(fname).to_string();
                    let cnt_opcodes = if fdef.code.as_ref().is_some() {
                        fdef.code.as_ref().unwrap().code.len()
                    } else {
                        0
                    };
                    functions.push(Function::new(function_name, cnt_opcodes))
                }
                let mname = cm.module_handles[0].name;
                let module_name = cm.identifier_at(mname).to_string();
                let cnt_constants = cm.constant_pool().len();
                let source_file = File::open(paths[i].clone()).expect("Failed to open file");
                let reader = BufReader::new(source_file);
                let cnt_codes = reader.lines().count();
                let module = Module::new(module_name, cnt_constants, cnt_codes, functions);
                modules.push(module);
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

            let feature = DatasetFeature::new(index, package_dir, package_name, modules);
            packages.push(feature);
        }

        let file = File::create(result_path).expect("Failed to create file");
        let mut writer = BufWriter::new(file);
        let header = "package_dir,package_name,aptos_or_sui,cnt_modules,cnt_functions,cnt_constants,cnt_codes,cnt_opcodes";
        writeln!(writer, "{}", header).expect("Failed to write to file");

        for feature in &packages {
            let chain_type;
            if feature.chain_type == 0 { 
                chain_type = "aptos" ;
            } else if feature.chain_type == 1 {
                chain_type = "sui" ;
            } else {
                chain_type = "move" ;
            }
            let line = format!(
                "{},{},{},{},{},{},{},{}",
                feature.package_dir,
                feature.package_name,
                chain_type,
                feature.cnt_modules,
                feature.cnt_functions,
                feature.cnt_constants,
                feature.cnt_codes,
                feature.cnt_opcodes
            );
            writeln!(writer, "{}", line).expect("Failed to write to file");
        }
    }
}
