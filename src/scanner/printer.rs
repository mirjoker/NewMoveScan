use crate::cli::parser::Args;
use crate::{
    cli::parser::*,
    move_ir::{
        control_flow_graph::generate_cfg_in_dot_format,
        packages::{build_compiled_modules, Packages},
    },
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
        let cms = build_compiled_modules(&self.args.path);
        let packages = Packages::new(&cms);
        // 遍历packages中的stbgr
        for (mname, &ref stbgr) in packages.get_all_stbgr().iter() {
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
