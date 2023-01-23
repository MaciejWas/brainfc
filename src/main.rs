use clap::Parser;
use std::path::PathBuf;
use colored::Colorize;
use optimizer::Optimizer;

pub mod lexer;
pub mod parser;
pub mod optimizations;
pub mod app;
pub mod llvm_ir_gen;
pub mod optimizer;

fn report_error(msg: String) {
    eprintln!("{} {}", "error: ".red().bold().to_string(), msg);
}

fn report_info(msg: String) {
    eprintln!("{} {}", "info: ".bold().to_string(), msg);
}

fn main() {
    std_logger::Config::logfmt().init();

    let args = app::Args::parse();
    log::debug!("parsed args: {:?}", args);

    let Ok(program_text) = std::fs::read_to_string(&args.input) else {
        report_error(format!("Could not open {:?}", args.input));
        return;
    };

    report_info("Parsing...".to_string());
    let parsed = parser::parse(lexer::parse(&program_text));
    if let Err(msg) = parsed {
        report_error(msg);
        return;
    }
    let parsed = parsed.unwrap();

    if args.show_parsed {
        println!("{:?}", parsed);
    }

    report_info("Optimizing...".to_string());
    let optimized = Optimizer::new().optimize(parsed);
    if args.show_optimized {
        println!("{:?}", optimized);
    }

    let output_file = args.output
        .as_ref()
        .cloned()
        .unwrap_or(PathBuf::new().with_file_name("out"));

    let object_file = output_file.clone()
        .with_extension("o");

    report_info("Compiling...".to_string());
    llvm_ir_gen::compile(optimized, args);

    report_info("Linking with gcc...".to_string());
    std::process::Command::new("gcc")
        .args([object_file.to_str().unwrap(), "-o", output_file.to_str().unwrap()])
        .spawn().unwrap().wait().unwrap();

    report_info(format!("Done ({})", output_file.as_path().as_os_str().to_str().unwrap()));
}


#[cfg(test)]
mod test {
    use super::*;

    macro_rules! build_ir_test
    {
        ($name:ident) => {
            #[test]
            pub fn $name() {
                let input = std::fs::read_to_string(format!("tests/ir_tests/{}.input", stringify!($name))).unwrap();
                let expected_output = std::fs::read_to_string(format!("tests/ir_tests/{}.output", stringify!($name))).unwrap();
                let parsed = parser::parse(lexer::parse(&input)).unwrap();
                assert_eq!(  format!("{:?}\n", parsed),  expected_output);
            }
        };
    }
}
