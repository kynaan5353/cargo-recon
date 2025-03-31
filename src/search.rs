use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use syn::{ItemFn, Type, visit::Visit};

#[derive(Clone, Debug)]
pub struct Target {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.file_path.display(),
            self.line,
            self.name
        )
    }
}

pub struct FunctionFinder {
    root: PathBuf,
    results: Vec<Target>,
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
                    self.results.push(Target {
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

pub fn search_file<P>(path: P) -> eyre::Result<Vec<Target>>
where
    P: AsRef<Path>,
{
    let syntax = syn::parse_file(&fs::read_to_string(&path)?)?;
    let mut finder = FunctionFinder::new(path.as_ref().to_path_buf());
    finder.visit_file(&syntax);
    Ok(finder.results)
}
