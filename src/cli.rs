use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Opts {
    pub path: Option<PathBuf>,
}
