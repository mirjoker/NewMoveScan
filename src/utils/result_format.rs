use std::{fs, collections::BTreeMap};
use move_cli::base::new;
use serde_json::{self, Map, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Detection_Results {
    pub modules_count : usize,
    pub failed_modules_count : usize,
    pub total_time : usize,
    pub modules : BTreeMap<String, Module_Details>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module_Details {
    pub time : usize,
    pub function_counts : usize,
    pub native_function_counts : usize,
    pub detect_result : BTreeMap<String, Vec<String>>,
    pub functions : BTreeMap<String, Vec<String>>
}

// pub struct Defect_Result {
//     pub Unchecked_return : Vec<String>,
//     pub Overflow : Vec<String>,
//     pub Precision_Loss : Vec<String>,
//     pub Infinite_Loop : Vec<String>,
//     pub Unused_Private_Functions : Vec<String>,
//     pub Unnecessary_Type_Conversion : Vec<String>,
//     pub Unnecessary_Bool_Judgment : Vec<String>,
//     pub Unused_Constant : Vec<String>,
// }

// impl Defect_Result {
//     pub fn new() -> Self {
//         Defect_Result {
//             Unchecked_return : vec![],
//             Overflow : vec![],
//             Precision_Loss : vec![],
//             Infinite_Loop : vec![],
//             Unused_Private_Functions : vec![],
//             Unnecessary_Type_Conversion : vec![],
//             Unnecessary_Bool_Judgment : vec![],
//             Unused_Constant : vec![],
//         }
//     }
//     // pub fn update(&mut self, input: Vec<Vec<String>>) {
//     //     self.Unchecked_return = input[0].clone();
//     //     self.Overflow = input[1].clone();
//     //     self.Precision_Loss = input[2].clone();
//     //     self.Infinite_Loop = input[3].clone();
//     //     self.Unused_Private_Functions = input[4].clone();
//     //     self.Unnecessary_Type_Conversion = input[5].clone();
//     //     self.Unnecessary_Bool_Judgment = input[6].clone();
//     //     self.Unused_Constant = input[7].clone();
//     // }
// }

impl Module_Details {
    pub fn new() -> Self {
        let mut detect_result = BTreeMap::new();
        detect_result.insert("Unchecked_return".to_string(), vec![]);
        detect_result.insert("Overflow".to_string(), vec![]);
        detect_result.insert("Precision_Loss".to_string(), vec![]);
        detect_result.insert("Infinite_Loop".to_string(), vec![]);
        detect_result.insert("Unused_Private_Functions".to_string(), vec![]);
        detect_result.insert("Unnecessary_Type_Conversion".to_string(), vec![]);
        detect_result.insert("Unnecessary_Bool_Judgment".to_string(), vec![]);
        detect_result.insert("Unused_Constant".to_string(), vec![]);
        let functions = BTreeMap::new();
        Module_Details { 
            time: 0, 
            function_counts: 0, 
            native_function_counts: 0,
            detect_result: detect_result, 
            functions: functions
        }
    }
}

impl Detection_Results {
    pub fn new() -> Self {
        let module_details = BTreeMap::new();
        Detection_Results {
            modules_count : 0,
            failed_modules_count : 0,
            total_time : 0,
            modules : module_details,
        }
    }
}