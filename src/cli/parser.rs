use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author="yule liteng", version="0.01", about="This is a static analysis tool for move smart contracts.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // #[clap(short = 'f', long, default_value_t = ("/home/yule/Movebit/detect/sources/unchecked_return.move").to_string(), help = "The project under this dir will be analyzed")]
    #[clap(short = 'f', long, help = "The project under this dir will be analyzed")]
    pub filedir: String,

    #[clap(short = 'j', long, help = "Write the result in json", default_value=Some("result.json"))]
    pub json_file: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Printer { printer: Option<Infos> },
    Detection { detection:  Option<Defects> },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Defects {
    UncheckedReturn, //detect1
    Overflow, //detect2
    PrecisionLoss, //detect3
    InfiniteLoop, //detect4
    UnusedConstant, //detect5
    UnusedPrivateFunctions, //detect6
    UnnecessaryTypeConversion, //detect7
    UnnecessaryBoolJudgment, //detect8
    // AllIn, //all detects
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Infos {
    IR, // Stackless Bytecode IR
    CM, // Compile Module
    CFG, // Control Flow Graph
    DU, // Tempindex def and use
    FNs, // Function Signatures
    CG // Function Call Graph
}