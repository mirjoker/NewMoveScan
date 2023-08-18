use super::generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator};
use crate::utils::utils;
use move_binary_format::CompiledModule;
use move_model::model::FunId;
use std::{collections::BTreeMap, path::PathBuf};
use std::{
    fs,
    io::{BufReader, Read},
};

pub struct Packages<'a> {
    packages: BTreeMap<String, StacklessBytecodeGenerator<'a>>,
    // todo 新增 Status，其中维护构建失败和成功的数量
}

impl<'a> Packages<'a> {
    pub fn new(cms: &'a Vec<CompiledModule>) -> Self {
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
        let mut packages = BTreeMap::new();
        for stbgr in stbgrs {
            let mname = stbgr.module_data.name.clone();
            let mname = mname.display(&stbgr.symbol_pool).to_string();
            packages.insert(mname, stbgr);
        }
        Packages { packages: packages }
    }

    pub fn get_all_stbgr(&self) -> &BTreeMap<String, StacklessBytecodeGenerator<'a>> {
        &self.packages
    }

    pub fn get_stbgr_by_mname(&self, mname: String) -> Option<&StacklessBytecodeGenerator<'_>> {
        self.packages.get(&mname)
    }

    pub fn get_function(&self, mname: String, fid: FunId) -> &FunctionInfo {
        let stbgr = self.get_stbgr_by_mname(mname).unwrap();
        // module_data.function_idx_to_id.insert(def_idx, fun_id);
        let mut idx = 0;
        for (def_idx, fun_id) in stbgr.module_data.function_idx_to_id.iter() {
            if fid == *fun_id {
                idx = def_idx.0;
                break;
            }
        }
        &stbgr.functions[idx as usize]
    }
}
pub fn compile_module(filename: PathBuf) -> Option<CompiledModule> {
    let f = fs::File::open(filename).unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    let cm = CompiledModule::deserialize(&buffer);
    cm.ok()
}

pub fn build_compiled_modules(path: &String) -> Vec<CompiledModule> {
    let dir = PathBuf::from(&path);
    // 输入路径遍历
    let mut paths = Vec::new();
    utils::visit_dirs(&dir, &mut paths, false);
    // 输入文件解析(反序列化成CompiledModule)
    let mut cms = Vec::new();
    for filename in paths {
        // println!("Deserializing {:?}...", filename);
        if let Some(cm) = compile_module(filename.clone()) {
            cms.push(cm);
        } else {
            println!("\x1B[31mFail to deserialize {:?}, Skip.\x1B[0m", filename);
        }
    }
    cms
}
