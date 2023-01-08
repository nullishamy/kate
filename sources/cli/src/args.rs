use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// The classes to execute
    pub classes: Vec<String>,

    #[arg(long)]
    /// Whether to run in "test mode", which will emit more machine friendly logs
    pub test: bool,


    #[arg(long("cp"))]
    /// A list of paths to add to the classpath
    pub classpath: Vec<String>
}

#[derive(Subcommand)]
pub enum Command {
}