use crate::{
    cli::parser::*,
    move_ir::{
        packages::{build_compiled_modules, Packages},
        utils,
    },
    scanner::{
        detects::{
            detect9::Detector9,detect10::Detector10,detect11::Detector11,detect12::Detector12,
        },
        result::*,
    },
};
use num::ToPrimitive;
use std::{fs, io::Write, path::Path, time::Instant};

pub trait AbstractDetector<'a> {
    fn new(packages: &'a Packages<'a>) -> Self
    where
        Self: Sized;
    fn run(&mut self) -> &DetectContent;
}

pub struct Detectors {
    pub args: Args,
    pub result: Result,
}

impl Detectors {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            result: Result::empty(),
        }
    }

    pub fn run(&mut self) {
        let clock = Instant::now();
        // package构建
        let cms = build_compiled_modules(&self.args.path);
        let packages = Packages::new(&cms);
        self.init_result(&packages);
        let mut detectors: Vec<Box<dyn AbstractDetector>> = vec![
            Box::new(Detector9::new(&packages)),
            Box::new(Detector10::new(&packages)),
            Box::new(Detector11::new(&packages)),
            Box::new(Detector12::new(&packages)),
        ];

        for detector in detectors.iter_mut() {
            let detect_content = detector.run();
            self.merge_result(detect_content);
        }

        self.complete_result(clock);
    }

    pub fn output_result(&self) {
        let json_result = serde_json::to_string(&self.result).ok().unwrap();
        let pretty_result = PrettyResult::from(self.result.clone());
        if let Some(output) = &self.args.output {
            // 输出到指定目录
            let output_path = Path::new(output);
            if let Some(dir_path) = output_path.parent() {
                // 不存在则递归创建路径目录
                if !dir_path.exists() {
                    if let Err(error) = fs::create_dir_all(dir_path) {
                        println!("Error creating directories: {:?}", error);
                    }
                }
            }
            let mut file = fs::File::create(output).expect("Failed to create json file");
            file.write(json_result.as_bytes())
                .expect("Failed to write to json file");
        }
        if self.args.none {
            return;
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

    // 为每个 module 生成 ModuleInfo，计算其中的 constant_count 和 function_count
    fn init_result(&mut self, packages: &Packages) {
        for (mname, &ref stbgr) in packages.get_all_stbgr().iter() {
            let mut module_info = ModuleInfo::empty();
            module_info.constant_count = stbgr.module.constant_pool.len();
            *module_info
                .function_count
                .get_mut(&FunctionType::All)
                .unwrap() = stbgr.functions.len();
            // 遍历stbgr中的functions
            for (idx, _function) in stbgr.functions.iter().enumerate() {
                if utils::is_native(idx, stbgr) {
                    *module_info
                        .function_count
                        .get_mut(&FunctionType::Native)
                        .unwrap() += 1;
                }
            }
            self.result.add_module(mname.to_string(), module_info);
        }
    }

    // 将每个 detector 检测结果同步到 result 中
    fn merge_result(&mut self, detect_content: &DetectContent) {
        let kind = detect_content.kind.clone();
        for (module_name, detect_res) in detect_content.result.iter() {
            detect_res.iter().for_each(|r| {
                self.result
                    .modules
                    .get_mut(module_name)
                    .unwrap()
                    .detectors
                    .get_mut(&kind)
                    .unwrap()
                    .push(r.to_string());
            })
        }
    }

    // 收尾工作，生成执行总耗时、pass 和 wrong 的 module 数量
    fn complete_result(&mut self, clock: Instant) {
        self.result.total_time = clock.elapsed().as_micros().to_usize().unwrap();
        let module_count = self.result.modules.len();
        let mut wrong_module_count = 0;
        for (_module_name, module_info) in self.result.modules.iter() {
            let mut pass = true;
            for (_detector_type, values) in module_info.detectors.iter() {
                if !values.is_empty() {
                    pass = false;
                }
            }
            if !pass {
                wrong_module_count += 1;
            }
        }
        self.result
            .modules_status
            .insert(Status::Pass, module_count - wrong_module_count);
        self.result
            .modules_status
            .insert(Status::Wrong, wrong_module_count);
    }
}
