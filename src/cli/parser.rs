use clap::{Parser,Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(short = 'p', long = "path", help = "Path to input dir/file")]
    pub path: String,

    #[clap(short = 'o', long = "output", help = "Path to output file", default_value=Some("result.json"))]
    pub output: Option<String>,
    
    #[clap(short = 'n', long = "none", help = "Print nothing on terminal")]
    pub none: bool,

    #[clap(short = 'j',long = "json",help="Print result as json on terminal")]
    pub json: bool,

    #[clap(short = 'i', long, help = "IR type",)]
    pub ir_type: Option<IR>
}

#[derive(Parser)]
#[command(author="yule liteng happytsing", version="1.0.0", about="A static analysis tool based on bytecode for move smart contracts.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommands>,

    #[clap(flatten)]
    pub args: Args,
}

#[derive(Subcommand)]
pub enum SubCommands {
    Printer,
    Detector
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum IR {
    SB, // Stackless Bytecode
    CM, // Compile Module
    CFG, // Control Flow Graph
    DU, // Tempindex def and use
    FS, // Function Signatures
    CG // Function Call Graph
}