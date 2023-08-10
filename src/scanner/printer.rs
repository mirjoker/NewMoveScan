use crate::cli::parser::Args;
use crate::move_ir::packages::Packages;
use crate::{
    cli::parser::*,
    move_ir::{
        control_flow_graph::generate_cfg_in_dot_format,
        generate_bytecode::StacklessBytecodeGenerator,
    },
    utils::utils::{self, compile_module},
};
use move_binary_format::access::ModuleAccess;
use petgraph::dot::Dot;
use std::{fs, path::PathBuf};

pub struct Printer {
    pub args: Args,
    // pub result: Result,
}

impl Printer {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            // result: Result::empty(),
        }
    }
    pub fn run(&mut self) {
        // 开始检测
        let dir = PathBuf::from(&self.args.path);
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
                println!("Fail to deserialize {:?} !!!", filename);
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

        // 遍历packages中的stbgr
        for (mname, &stbgr) in packages.get_all_stbgr().iter() {
            match self.args.ir_type {
                Some(IR::CFG) => {
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
                Some(IR::SB) => {
                    println!("{}", stbgr.display(true, None));
                }
                Some(IR::CM) => {
                    println!("{:#?}", stbgr.module);
                }
                Some(IR::FS) => {
                    println!("{}", stbgr.display(false, None));
                }
                Some(IR::DU) => {
                    for (_idx, function) in stbgr.functions.iter().enumerate() {
                        println!("{:?}", &function.def_attrid);
                        println!("{:?}", &function.use_attrid);
                    }
                }
                Some(IR::CG) => {
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
    }
}
