use std::path::PathBuf;

use clap::Parser;
use cli::Opts;
use eyre::eyre;
use search::search_file;
use walkdir::WalkDir;

pub mod cli;
pub mod search;

fn main() -> eyre::Result<()> {
    let opts = Opts::parse();

    let mut targets = Vec::new();

    let path = match opts.path {
        Some(p) => p,
        None => {
            let mut p = PathBuf::new();
            p.push(".");
            p
        }
    };

    if path.is_dir() {
        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str())
                    == Some("rs")
            {
                targets.extend(search_file(entry.path()));
            }
        }
    } else if path.is_file() {
        targets.extend(search_file(&path));
    } else {
        return Err(eyre!("Not file nor directory"));
    }

    targets.iter().flatten().for_each(|target| {
        println!(
            "Found function: {} at {}:{}",
            target.name,
            target.file_path.display(),
            target.line
        );
    });

    Ok(())
}
