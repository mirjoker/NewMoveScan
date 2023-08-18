use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

pub type ModuleName = String;
pub type FunctionName = String;

#[derive(Debug, Display,Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Pass,
    Wrong,
}

#[derive(Debug, EnumIter, Display, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DetectKind {
    UncheckedReturn,
    Overflow,
    PrecisionLoss,
    InfiniteLoop,
    UnnecessaryTypeConversion,
    UnnecessaryBoolJudgment,
    UnusedConstant,
    UnusedPrivateFunctions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    All,
    Native,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Minor,
    Medium,
    Major,
    Critical,
}

pub struct DetectContent {
    pub severity: Severity,
    pub kind: DetectKind,
    pub result: HashMap<ModuleName, Vec<String>>,
}

impl DetectContent {
    pub fn new(severity: Severity, kind: DetectKind) -> Self {
        Self {
            severity,
            kind,
            result: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Result {
    pub modules_status: HashMap<Status, usize>,
    pub total_time: usize,
    pub modules: HashMap<ModuleName, ModuleInfo>,
}

impl Result {
    pub fn new(
        modules_status: HashMap<Status, usize>,
        total_time: usize,
        modules: HashMap<ModuleName, ModuleInfo>,
    ) -> Self {
        Self {
            modules_status,
            total_time,
            modules,
        }
    }

    pub fn empty() -> Self {
        return Self::new(
            HashMap::from([(Status::Pass, 0), (Status::Wrong, 0)]),
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
    pub function_count: HashMap<FunctionType, usize>,
    pub constant_count: usize,
    pub detectors: HashMap<DetectKind, Vec<String>>,
}
impl ModuleInfo {
    pub fn new(
        function_count: HashMap<FunctionType, usize>,
        constant_count: usize,
        detectors: HashMap<DetectKind, Vec<String>>,
    ) -> Self {
        Self {
            function_count,
            constant_count,
            detectors,
        }
    }

    pub fn empty() -> Self {
        let function_count = HashMap::from([(FunctionType::All, 0), (FunctionType::Native, 0)]);
        let mut detectors = HashMap::new();
        for detect_kind in DetectKind::iter() {
            detectors.insert(detect_kind, Vec::<String>::new());
        }
        return Self::new(function_count, 0, detectors);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrettyResult {
    pub modules_status: HashMap<Status, usize>,
    pub total_time: usize,
    pub modules: HashMap<ModuleName, HashMap<DetectKind, Vec<String>>>,
}
impl PrettyResult {
    pub fn from(result: Result) -> Self {
        let mut modules = HashMap::new();
        for (module_name, module_info) in result.modules {
            for (detector_type, values) in module_info.detectors {
                if values.is_empty() {
                    continue;
                }
                if !modules.contains_key(&module_name) {
                    modules.insert(module_name.clone(), HashMap::new());
                }
                modules
                    .get_mut(&module_name)
                    .unwrap()
                    .insert(detector_type.clone(), values.clone());
            }
        }
        Self {
            modules_status: result.modules_status,
            total_time: result.total_time,
            modules: modules,
        }
    }
}
impl std::fmt::Display for PrettyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\n{}: \x1B[32m{:<6}\x1B[0m {}: \x1B[31m{:<6}\x1B[0m time: \x1B[34m{}\x1B[0m us\n",
            Status::Pass,
            self.modules_status.get(&Status::Pass).unwrap(),
            Status::Wrong,
            self.modules_status.get(&Status::Wrong).unwrap(),
            self.total_time
        )?;

        for (module_index, (module_name, detectors_result)) in
            self.modules.clone().iter().enumerate()
        {
            writeln!(f, "no: {}", module_index)?;
            writeln!(f, "module_name: {}", module_name)?;
            for (detector_type, values) in detectors_result {
                write!(f, "\x1B[31m{}\x1B[0m: ", detector_type)?;
                let values_str = values.iter().join(",");
                match detector_type {
                    DetectKind::UncheckedReturn => {
                        let color_value_str = &values_str
                            .replace("(", "\x1B[33m(")
                            .replace(")", ")\x1B[0m");
                        writeln!(f, "[ {} ]", color_value_str)?;
                    }
                    _ => {
                        writeln!(f, "[ {} ] ", values_str)?;
                    }
                }
            }
            writeln!(f, "\n")?;
        }
        Ok(())
    }
}
