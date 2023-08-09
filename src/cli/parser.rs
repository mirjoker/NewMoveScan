use clap::{Parser,Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(short = 'p', long = "path", help = "Path to input dir/file")]
    pub path: String,

    #[clap(short = 'o', long = "output", help = "Path to output result", default_value=Some("result.json"))]
    pub output: Option<String>,

    #[clap(short = 'j',long = "json",help="Output json result on the command line")]
    pub json: bool,

    #[clap(short = 'i', long, help = "IR Type",)]
    pub ir_type: Option<IR>
}

#[derive(Parser)]
#[command(author="yule liteng happytsing", version="1.0.0", about="This is a static analysis tool for move smart contracts.", long_about = None)]
pub struct Cli {

    #[clap(flatten)]
    pub args: Args,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Printer,
    Detector
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum IR {
    SB, // Stackless Bytecode
    CM, // Compile Module
    CFG, // Control Flow Graph
    DU, // Tempindex def and use
    FNs, // Function Signatures
    CG // Function Call Graph
}