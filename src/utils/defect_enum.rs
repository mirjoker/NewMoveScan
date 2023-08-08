#[derive(Debug)]
pub enum Defects {
    UncheckedReturn, 
    Overflow, 
    PrecisionLoss, 
    InfiniteLoop, 
    UnnecessaryTypeConversion, 
    UnnecessaryBoolJudgment, 
    UnusedConstant, 
    UnusedPrivateFunctions, 
}

impl Defects {
    pub fn get_defect_neme(index : usize) -> String {
        match index {
            0 => {
                return format!("{:?}",Defects::UncheckedReturn).to_string();
            },
            1 => {
                return format!("{:?}",Defects::Overflow).to_string();
            },
            2 => {
                return format!("{:?}",Defects::PrecisionLoss).to_string();
            },
            3 => {
                return format!("{:?}",Defects::InfiniteLoop).to_string();
            },
            4 => {
                return format!("{:?}",Defects::UnnecessaryTypeConversion).to_string();
            },
            5 => {
                return format!("{:?}",Defects::UnnecessaryBoolJudgment).to_string();
            },
            6 => {
                return format!("{:?}",Defects::UnusedConstant).to_string();
            },
            7 => {
                return format!("{:?}",Defects::UnusedPrivateFunctions).to_string();
            },
            _ => {
                return "".to_string();
            }
        }
    }
}