use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Opts {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Lists viable fuzzing targets
    #[command(arg_required_else_help = true)]
    List {
        /// Path to Rust code to search
        path: Option<PathBuf>,
    },
    /// Write fuzzing tests
    Generate {
        /// Path to Rust code to search
        inpath: Option<PathBuf>,
        /// Path to write generated fuzzing tests to
        outpath: Option<PathBuf>,
    },
}
