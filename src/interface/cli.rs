use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(
    author = "nullishamy",
    version = "0.1",
    about = "A simple JVM build in Rust"
)]
pub struct Cli {
    #[clap(short, long)]
    pub tui: bool,

    #[clap(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    #[clap(about = "Runs a single class file")]
    Run {
        #[clap(value_name = "FILE")]
        file: String,
    },
}
