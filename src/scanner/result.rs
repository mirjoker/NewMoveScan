use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ModuleName = String;
pub type FunctionName = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash,Clone)]
pub enum Status {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash,Clone)]
pub enum DetectorType {
    #[serde(rename = "unchecked_return")]
    UncheckedReturn,

    #[serde(rename = "overflow")]
    Overflow,

    #[serde(rename = "precision_loss")]
    PrecisionLoss,

    #[serde(rename = "infinite_loop")]
    InfiniteLoop,

    #[serde(rename = "unnecessary_type_conversion")]
    UnnecessaryTypeConversion,

    #[serde(rename = "unnecessary_bool_judgment")]
    UnnecessaryBoolJudgment,

    #[serde(rename = "unused_constant")]
    UnusedConstant,

    #[serde(rename = "unused_private_functions")]
    UnusedPrivateFunctions,
}
impl std::fmt::Display for DetectorType{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DetectorType::UncheckedReturn => write!(f, "unchecked_return"),
            DetectorType::Overflow => write!(f, "overflow"),
            DetectorType::PrecisionLoss => write!(f, "precision_loss"),
            DetectorType::InfiniteLoop => write!(f, "infinite_loop"),
            DetectorType::UnnecessaryTypeConversion => write!(f, "unnecessary_type_conversion"),
            DetectorType::UnnecessaryBoolJudgment => write!(f, "unnecessary_bool_judgment"),
            DetectorType::UnusedConstant => write!(f, "unused_constant"),
            DetectorType::UnusedPrivateFunctions => write!(f, "unused_private_functions"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash,Clone)]
pub enum FunctionType {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "native")]
    Native,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Result {
    pub modules_count: HashMap<Status, usize>,
    pub total_time: usize,
    pub modules: HashMap<ModuleName, ModuleInfo>,
}

impl Result {
    pub fn new(
        modules_count: HashMap<Status, usize>,
        total_time: usize,
        modules: HashMap<ModuleName, ModuleInfo>,
    ) -> Self {
        Self {
            modules_count,
            total_time,
            modules,
        }
    }

    pub fn empty() -> Self {
        return Self::new(
            HashMap::from([(Status::Success, 0), (Status::Failed, 0)]),
            0,
            HashMap::new(),
        );
    }

    pub fn add_module(&mut self, module_name: ModuleName, module_info: ModuleInfo) {
        self.modules.insert(module_name, module_info);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleInfo {
    pub time: usize,
    pub function_count: HashMap<FunctionType, usize>,
    pub constant_count: usize,
    pub detectors: HashMap<DetectorType, Vec<String>>,
    pub functions: HashMap<FunctionName, Vec<DetectorType>>,
}
impl ModuleInfo {
    pub fn new(
        time: usize,
        function_count: HashMap<FunctionType, usize>,
        constant_count: usize,
        detectors: HashMap<DetectorType, Vec<String>>,
        functions: HashMap<FunctionName, Vec<DetectorType>>,
    ) -> Self {
        Self {
            time,
            function_count,
            constant_count,
            detectors,
            functions,
        }
    }

    pub fn empty() -> Self {
        let function_count = HashMap::from([(FunctionType::All, 0), (FunctionType::Native, 0)]);
        let detectors = HashMap::from([
            (DetectorType::UncheckedReturn, Vec::<String>::new()),
            (DetectorType::Overflow, Vec::<String>::new()),
            (DetectorType::PrecisionLoss, Vec::<String>::new()),
            (DetectorType::InfiniteLoop, Vec::<String>::new()),
            (
                DetectorType::UnnecessaryTypeConversion,
                Vec::<String>::new(),
            ),
            (DetectorType::UnnecessaryBoolJudgment, Vec::<String>::new()),
            (DetectorType::UnusedConstant, Vec::<String>::new()),
            (DetectorType::UnusedPrivateFunctions, Vec::<String>::new()),
        ]);
        return Self::new(0, function_count, 0, detectors, HashMap::new());
    }

    // 更新
    pub fn update_detectors(&mut self, detector_type: DetectorType, value: String) {
        self.detectors.get_mut(&detector_type).unwrap().push(value);
    }

    // 批量更新
    pub fn updates_detectors(&mut self, detector_type: DetectorType, value: Vec<String>) {
        self.detectors
            .get_mut(&detector_type)
            .unwrap()
            .extend(value)
    }
    pub fn init_functions(&mut self, func_name: FunctionName){
        if !self.functions.contains_key(&func_name) {
            self.functions.insert(func_name, Vec::<DetectorType>::new());
        }
    }

    pub fn update_functions(&mut self, func_name: FunctionName,detector_type: DetectorType) {
        self.functions.get_mut(&func_name).unwrap().push(detector_type);
    }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrettyResult{
    pub modules_count: HashMap<Status, usize>,
    pub total_time: usize,
    pub modules: HashMap<ModuleName, HashMap<DetectorType,Vec<String>>>
}
impl PrettyResult {
    pub fn from(result:Result) -> Self {
        let mut modules = HashMap::new();
        for (module_name,module_info) in result.modules{
            for (detector_type, values) in module_info.detectors{
                if values.is_empty(){
                    continue;
                }
                if !modules.contains_key(&module_name){
                    modules.insert(module_name.clone(), HashMap::new());
                }
                modules.get_mut(&module_name).unwrap().insert(detector_type.clone(), values.clone());
            }
        }
        Self { modules_count: result.modules_count, total_time: result.total_time, modules: modules }
    }
}
impl std::fmt::Display for PrettyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\nsuccess: \x1B[32m{:<6}\x1B[0m failed: \x1B[31m{:<6}\x1B[0m pass: \x1B[32m{:<6}\x1B[0m wrong: \x1B[31m{:<6}\x1B[0m time: \x1B[34m{}\x1B[0m us\n",
            self.modules_count.get(&Status::Success).unwrap(),
            self.modules_count.get(&Status::Failed).unwrap(),
            self.modules_count.get(&Status::Success).unwrap()+self.modules_count.get(&Status::Failed).unwrap()-self.modules.len(),
            self.modules.len(),
            self.total_time
        )?;

        for (module_index,(module_name,detectors_result))in self.modules.clone().iter().enumerate(){
            writeln!(f,"no: {}",module_index)?;
            writeln!(f,"module_name: {}",module_name)?;
            for (detector_type,values) in detectors_result{
                write!(f,"\x1B[31m{}\x1B[0m: ",detector_type)?;
                let values_str = values.iter().join(",");
                match detector_type {
                    DetectorType::UncheckedReturn => {
                        let color_value_str  = &values_str.replace("(", "\x1B[33m(").replace(")", ")\x1B[0m");
                        writeln!(f,"[ {} ]",color_value_str)?;
                    }
                    _ => {
                        writeln!(f,"[ {} ] ",values_str)?;
                    }
                }
            }
            writeln!(f,"\n")?;
        }
        Ok(())
    }
}

