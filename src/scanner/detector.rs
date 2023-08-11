use crate::move_ir::packages::Packages;
use crate::{
    cli::parser::*,
    move_ir::generate_bytecode::StacklessBytecodeGenerator,
    scanner::{
        detects::{
            detect1::detect_unchecked_return, detect2::detect_overflow,
            detect3::detect_precision_loss, detect4::detect_infinite_loop,
            detect5::detect_unused_constants, detect6::detect_unused_private_functions,
            detect7::detect_unnecessary_type_conversion, detect8::detect_unnecessary_bool_judgment,
        },
        result::{DetectorType, FunctionType, ModuleInfo, PrettyResult, Result, Status},
    },
    utils::utils::{self, compile_module},
};
use itertools::Itertools;
use move_binary_format::{access::ModuleAccess, file_format::FunctionDefinitionIndex};
use num::ToPrimitive;
use std::{fs, io::Write, path::PathBuf, time::Instant};

pub struct Detector {
    pub args: Args,
    pub result: Result,
}

impl Detector {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            result: Result::empty(),
        }
    }
    pub fn get_function_name(&self, idx: usize, stbgr: &StacklessBytecodeGenerator) -> String {
        let func_name = stbgr
            .module
            .identifier_at(
                stbgr
                    .module
                    .function_handle_at(stbgr.module.function_defs[idx].function)
                    .name,
            )
            .to_string();
        return func_name;
    }

    pub fn output_result(&self) {
        let json_result = serde_json::to_string(&self.result).ok().unwrap();
        let pretty_result = PrettyResult::from(self.result.clone());
        if let Some(output) = &self.args.output {
            // 输出到指定目录
            let mut file = fs::File::create(output).expect("Failed to create json file");
            file.write(json_result.as_bytes())
                .expect("Failed to write to json file");
        }
        if self.args.json {
            let pretty_json_result = serde_json::to_string(&pretty_result).ok().unwrap();
            println!("{pretty_json_result}");
        // 命令行以 json 格式输出
        } else {
            println!("{pretty_result}");
            // 以非命令行格式输出
        }
    }

    pub fn run(&mut self) {
        // 开始检测
        let time_start = Instant::now();
        let dir = PathBuf::from(&self.args.path);
        // 输入路径遍历
        let mut paths = Vec::new();
        utils::visit_dirs(&dir, &mut paths, false);
        // 输入文件解析(反序列化成CompiledModule)
        let mut cms = Vec::new();
        for filename in paths {
            // println!("Deserializing {:?}...", filename);
            if let Some(cm) = compile_module(filename.clone()) {
                *self.result.modules_count.get_mut(&Status::Success).unwrap() += 1;
                cms.push(cm);
            } else {
                println!("Fail to deserialize {:?} !!!", filename);
                *self.result.modules_count.get_mut(&Status::Failed).unwrap() += 1;
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
            let module_time_start = Instant::now();
            let mut module_info = ModuleInfo::empty();
            module_info.constant_count = stbgr.module.constant_pool.len();
            *module_info
                .function_count
                .get_mut(&FunctionType::All)
                .unwrap() = stbgr.functions.len();
            // 遍历stbgr中的functions
            for (idx, function) in stbgr.functions.iter().enumerate() {
                let func_define = stbgr
                    .module
                    .function_def_at(FunctionDefinitionIndex::new(idx as u16));
                if func_define.is_native() {
                    *module_info
                        .function_count
                        .get_mut(&FunctionType::Native)
                        .unwrap() += 1;
                    continue;
                };
                let func_name = self.get_function_name(idx, stbgr);

                // 初始化 functions
                module_info.init_functions(func_name.clone());

                // 更新 detectors 和 functions
                let mut unchecked_return_func_list = detect_unchecked_return(function, &stbgr.symbol_pool, idx, stbgr.module);
                if !unchecked_return_func_list.is_empty() {
                    // 先排序，再去重。Tips：dedup 用于去除连续的重复元素
                    unchecked_return_func_list.sort();
                    unchecked_return_func_list.dedup();
                    let func_str = format!("{}({})", func_name.clone(), unchecked_return_func_list.into_iter().join(","));
                    module_info.update_detectors(DetectorType::UncheckedReturn, func_str);
                    module_info.update_functions(func_name.clone(), DetectorType::UncheckedReturn);
                }
                if detect_overflow(&packages, &stbgr, idx) {
                    module_info.update_detectors(DetectorType::Overflow, func_name.clone());
                    module_info.update_functions(func_name.clone(), DetectorType::Overflow);
                }
                if detect_precision_loss(function, &stbgr.symbol_pool) {
                    module_info.update_detectors(DetectorType::PrecisionLoss, func_name.clone());
                    module_info.update_functions(func_name.clone(), DetectorType::PrecisionLoss);
                }
                if detect_infinite_loop(&packages, &stbgr, idx) {
                    module_info.update_detectors(DetectorType::InfiniteLoop, func_name.clone());
                    module_info.update_functions(func_name.clone(), DetectorType::InfiniteLoop);
                }
                if detect_unnecessary_type_conversion(function, &function.local_types) {
                    module_info.update_detectors(
                        DetectorType::UnnecessaryTypeConversion,
                        func_name.clone(),
                    );
                    module_info.update_functions(
                        func_name.clone(),
                        DetectorType::UnnecessaryTypeConversion,
                    );
                }
                if detect_unnecessary_bool_judgment(function, &function.local_types) {
                    module_info
                        .update_detectors(DetectorType::UnnecessaryBoolJudgment, func_name.clone());
                    module_info
                        .update_functions(func_name.clone(), DetectorType::UnnecessaryBoolJudgment);
                }
            }
            let unused_private_functions = detect_unused_private_functions(&stbgr);
            let unused_private_function_names = unused_private_functions
                .iter()
                .map(|func| {
                    let func_name = func.symbol().display(&stbgr.symbol_pool).to_string();
                    module_info
                        .update_functions(func_name.clone(), DetectorType::UnusedPrivateFunctions);
                    return func_name;
                })
                .collect_vec();

            module_info.updates_detectors(
                DetectorType::UnusedConstant,
                detect_unused_constants(&stbgr)
                    .iter()
                    .map(|x| format!("{:?}", x))
                    .collect_vec(),
            );
            module_info.updates_detectors(
                DetectorType::UnusedPrivateFunctions,
                unused_private_function_names,
            );
            module_info.time = module_time_start.elapsed().as_micros().to_usize().unwrap();
            self.result.add_module(mname.to_string(), module_info);
        }
        self.result.total_time = time_start.elapsed().as_micros().to_usize().unwrap();
    }
}
