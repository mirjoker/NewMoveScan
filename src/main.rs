#![allow(non_snake_case)]
#[allow(unused_imports)]
use MoveScanner::{
    scanner::{
        detector::Detector,
        printer::Printer
    },
    cli::parser::*,
};

use clap::Parser;

fn main() {
    // 命令行参数解析
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Printer)=> {
            // todo
            println!("Printer: todo");
        },
        // 默认 Detector
        _ =>{
            let mut detector = Detector::new(cli.args);
            detector.run();
            detector.output_result();
        }
    }
    
}
