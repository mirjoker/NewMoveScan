use crate::cli::parser::Args;
use crate::scanner::result::Result;

pub struct Printer {
    pub args: Args,
    pub result: Result,
}