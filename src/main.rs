use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use eyre::eyre;
use syn::{ItemFn, Type, visit::Visit};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct FoundFunction {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
}

pub struct FunctionFinder {
    root: PathBuf,
    results: Vec<FoundFunction>,
}

impl FunctionFinder {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            results: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for FunctionFinder {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        for input in &i.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                let ty = &*pat_type.ty;
                if is_fuzzable_type(ty) {
                    let start = i.sig.ident.span().start();
                    self.results.push(FoundFunction {
                        name: i.sig.ident.to_string(),
                        file_path: self.root.clone(),
                        line: start.line,
                    });
                }
            }
        }
        syn::visit::visit_item_fn(self, i);
    }
}

fn is_fuzzable_type(ty: &Type) -> bool {
    match ty {
        // Matches a reference type like &[u8]
        Type::Reference(ref_type) => {
            if let syn::Type::Slice(slice) = &*ref_type.elem {
                if let syn::Type::Path(type_path) = &*slice.elem {
                    return type_path.path.is_ident("u8");
                }
            }
        }
        // Matches Vec<u8> and String
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.first() {
                if segment.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(angle_bracketed) =
                        &segment.arguments
                    {
                        if let Some(syn::GenericArgument::Type(
                            syn::Type::Path(inner_type_path),
                        )) = angle_bracketed.args.first()
                        {
                            if inner_type_path.path.is_ident("u8") {
                                return true;
                            }
                        }
                    }
                }
                if segment.ident == "String" {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

fn process_file<P>(path: P) -> eyre::Result<Vec<FoundFunction>>
where
    P: AsRef<Path>,
{
    let syntax = syn::parse_file(&fs::read_to_string(&path)?)?;
    let mut finder = FunctionFinder::new(path.as_ref().to_path_buf());
    finder.visit_file(&syntax);
    Ok(finder.results)
}

#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Opts {
    path: Option<PathBuf>,
}

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
                targets.extend(process_file(entry.path()));
            }
        }
    } else if path.is_file() {
        targets.extend(process_file(&path));
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
