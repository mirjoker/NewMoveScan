#[derive(Debug)]
pub enum Defects {
    Unchecked_Return, 
    Overflow, 
    Precision_Loss, 
    Infinite_Loop, 
    Unnecessary_Type_Conversion, 
    Unnecessary_Bool_Judgment, 
    Unused_Constant, 
    Unused_Private_Functions, 
}

impl Defects {
    pub fn get_defect_neme(index : usize) -> String {
        match index {
            0 => {
                return format!("{:?}",Defects::Unchecked_Return).to_string();
            },
            1 => {
                return format!("{:?}",Defects::Overflow).to_string();
            },
            2 => {
                return format!("{:?}",Defects::Precision_Loss).to_string();
            },
            3 => {
                return format!("{:?}",Defects::Infinite_Loop).to_string();
            },
            4 => {
                return format!("{:?}",Defects::Unnecessary_Type_Conversion).to_string();
            },
            5 => {
                return format!("{:?}",Defects::Unnecessary_Bool_Judgment).to_string();
            },
            6 => {
                return format!("{:?}",Defects::Unused_Constant).to_string();
            },
            7 => {
                return format!("{:?}",Defects::Unused_Private_Functions).to_string();
            },
            _ => {
                return "".to_string();
            }
        }
    }
}