use std::{fs, path::PathBuf, io::{BufReader, Read}};

use move_binary_format::{CompiledModule, file_format::Visibility};
use move_model::{model::ModuleEnv, ty::{self, *}};


// 依赖的 module 的 address
const DEPADDRESSES: [&str; 2] = ["0x1::", "0x3::"];

pub struct DotWeight {}

impl std::fmt::Display for DotWeight {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}

// get all .mv files in dir and subdir
pub fn visit_dirs(dir: &PathBuf, paths: &mut Vec<PathBuf>, subdir: bool) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if subdir {
                    visit_dirs(&path, paths, subdir);
                }
            } else {
                paths.push(path);
            }
        }
    } else if dir.is_file() {
        paths.push(dir.to_path_buf());
    }
}

pub fn is_dep_module(module_env: &ModuleEnv) -> bool {
    // if the module is dependent module
    let mut is_dep = false;
    let module_addr = module_env.get_full_name_str();
    for addr in DEPADDRESSES {
        if module_addr.starts_with(addr) {
            is_dep = true;
            break;
        }
    }
    return is_dep
}

pub fn compile_module(filename: PathBuf) -> Option<CompiledModule> {
    let f = fs::File::open(filename).unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    let cm = CompiledModule::deserialize(&buffer);
    cm.ok()
}

pub fn visibility_str(visibility: &Visibility) -> &str {
    match visibility {
        Visibility::Public => "public ",
        Visibility::Friend => "public(friend) ",
        Visibility::Private => "",
    }
}

pub fn display_type(ty: &Type) {
    match ty {
        ty::Type::Primitive(base_ty) => {
            match base_ty {
                PrimitiveType::Bool => {
                    println!("{}", "Bool"); 
                }
                PrimitiveType::U8 => {
                    println!("{}", "U8"); 
                }
                PrimitiveType::U16 => {
                    println!("{}", "U16"); 
                }
                PrimitiveType::U32 => {
                    println!("{}", "Bool"); 
                }
                PrimitiveType::U64 => {
                    println!("{}", "U64"); 
                }
                PrimitiveType::U128 => {
                    println!("{}", "U128"); 
                }
                PrimitiveType::U256 => {
                    println!("{}", "U256"); 
                }
                PrimitiveType::Address => {
                    println!("{}", "Address"); 
                }
                PrimitiveType::Signer => {
                    println!("{}", "Signer"); 
                }
                _ => {
                    println!("{}", "Else"); 
                }
            }
        },
        ty::Type::Tuple(_) => {
            println!("{}", "Tuple");
        },
        ty::Type::Vector(_) => {
            println!("{}", "Vector");
        },
        ty::Type::Struct(_, _, _) => {
            println!("{}", "Struct");
        },
        ty::Type::TypeParameter(_) => {
            println!("{}", "TypeParameter");
        },
        ty::Type::Reference(flag, _) => {
            println!("{}, {}", flag, "Reference");
        },
        _ => {
            println!("{}", "Else");
        }
    }
}

use anyhow::anyhow;
use move_stackless_bytecode::{
    function_target_pipeline::FunctionTargetPipeline,
    usage_analysis::UsageProcessor,
};
// IR 优化
pub fn get_tested_transformation_pipeline(
    dir_name: &str,
) -> anyhow::Result<Option<FunctionTargetPipeline>> {
    match dir_name {
        "from_move" => Ok(None),
        "usage_analysis" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(UsageProcessor::new());
            Ok(Some(pipeline))
        }
        _ => Err(anyhow!(
            "the sub-directory `{}` has no associated pipeline to test",
            dir_name
        )),
    }
}

pub fn format_vec_u8(vec: &[u8]) -> String {
    let mut res = "".to_string();
    let n = vec.len();
    match n {
        8 => { // U64
            let mut num: u64 = 0;
            let mut base: u64 = 1;
            for (i, v) in vec.iter().enumerate() {
                if i != 0 {
                    base = base << 8;
                }
                num = num + u64::from(*v) * base;

            }
            res = num.to_string();
        },
        16 => { // U128 or Address
            let mut num: u128 = 0;
            let mut base: u128 = 1;
            for (i, v) in vec.iter().enumerate() {
                if i != 0 {
                    base = base << 8;
                }
                num = num + u128::from(*v) * base;
            }
            let hex_string = hex::encode(vec);
            res = hex_string + "/";
            res = res + num.to_string().as_str();
        },
        _ => {
            for v in vec.iter() {
                let ch = std::char::from_u32(*v as u32).expect("Invalid ASCII value");
                res.push(ch);
            }
        }
    }
    res
}