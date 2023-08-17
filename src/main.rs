#![allow(non_snake_case)]
use MoveScanner::{
    cli::parser::*,
    scanner::{detectors::Detectors, printer::Printer},
};

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(SubCommands::Printer) => {
            // todo: 代码优化
            let mut printer = Printer::new(cli.args);
            printer.run();
        }
        // 默认 Detector
        _ => {
            let mut detector = Detectors::new(cli.args);
            detector.run();
            detector.output_result();
        }
    }
}
