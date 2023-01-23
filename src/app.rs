use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(value_name = "path", value_hint = clap::ValueHint::DirPath)]
    pub input: std::path::PathBuf,

    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,

    #[arg(long, default_value_t = false)]
    pub show_parsed: bool,

    #[arg(long, default_value_t = false)]
    pub show_optimized: bool,

    #[arg(long, default_value_t = false)]
    pub show_llvm_ir: bool,
}
