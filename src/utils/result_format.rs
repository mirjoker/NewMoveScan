use std::collections::BTreeMap;


use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionResults {
    pub modules_count : usize,
    pub failed_modules_count : usize,
    pub total_time : usize,
    pub modules : BTreeMap<String, ModuleDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleDetails {
    pub time : usize,
    pub function_counts : usize,
    pub native_function_counts : usize,
    pub constant_counts : usize,
    pub detect_result : BTreeMap<String, Vec<String>>,
    pub functions : BTreeMap<String, Vec<String>>
}

// pub struct Defect_Result {
//     pub Unchecked_return : Vec<String>,
//     pub Overflow : Vec<String>,
//     pub PrecisionLoss : Vec<String>,
//     pub InfiniteLoop : Vec<String>,
//     pub UnusedPrivateFunctions : Vec<String>,
//     pub UnnecessaryTypeConversion : Vec<String>,
//     pub UnnecessaryBoolJudgment : Vec<String>,
//     pub UnusedConstant : Vec<String>,
// }

// impl Defect_Result {
//     pub fn new() -> Self {
//         Defect_Result {
//             Unchecked_return : vec![],
//             Overflow : vec![],
//             PrecisionLoss : vec![],
//             InfiniteLoop : vec![],
//             UnusedPrivateFunctions : vec![],
//             UnnecessaryTypeConversion : vec![],
//             UnnecessaryBoolJudgment : vec![],
//             UnusedConstant : vec![],
//         }
//     }
//     // pub fn update(&mut self, input: Vec<Vec<String>>) {
//     //     self.Unchecked_return = input[0].clone();
//     //     self.Overflow = input[1].clone();
//     //     self.PrecisionLoss = input[2].clone();
//     //     self.InfiniteLoop = input[3].clone();
//     //     self.UnusedPrivateFunctions = input[4].clone();
//     //     self.UnnecessaryTypeConversion = input[5].clone();
//     //     self.UnnecessaryBoolJudgment = input[6].clone();
//     //     self.UnusedConstant = input[7].clone();
//     // }
// }

impl ModuleDetails {
    pub fn new() -> Self {
        let mut detect_result = BTreeMap::new();
        detect_result.insert("UncheckedReturn".to_string(), vec![]);
        detect_result.insert("Overflow".to_string(), vec![]);
        detect_result.insert("PrecisionLoss".to_string(), vec![]);
        detect_result.insert("InfiniteLoop".to_string(), vec![]);
        detect_result.insert("UnusedPrivateFunctions".to_string(), vec![]);
        detect_result.insert("UnnecessaryTypeConversion".to_string(), vec![]);
        detect_result.insert("UnnecessaryBoolJudgment".to_string(), vec![]);
        detect_result.insert("UnusedConstant".to_string(), vec![]);
        let functions = BTreeMap::new();
        ModuleDetails { 
            time: 0, 
            function_counts: 0, 
            native_function_counts: 0,
            constant_counts : 0,
            detect_result: detect_result, 
            functions: functions
        }
    }
}

impl DetectionResults {
    pub fn new() -> Self {
        let module_details = BTreeMap::new();
        DetectionResults {
            modules_count : 0,
            failed_modules_count : 0,
            total_time : 0,
            modules : module_details,
        }
    }
}