//! Protected ADJ-13 syntax observations. This instrument supplies no verdict.
use serde_json::{Value, json};
use std::path::Path;
use syn::visit::{self, Visit};

fn ty(value: &syn::Type) -> Value {
    match value {
        syn::Type::Path(value) if value.qself.is_none() => json!({
            "path": value.path.segments.iter().map(|segment| json!({
                "name": segment.ident.to_string(),
                "arguments": match &segment.arguments {
                    syn::PathArguments::None => Vec::new(),
                    syn::PathArguments::AngleBracketed(args) => args.args.iter().map(|arg| match arg {
                        syn::GenericArgument::Type(value) => ty(value),
                        _ => json!({"other": format!("{arg:?}")}),
                    }).collect(),
                    _ => vec![json!({"unsupported": format!("{:?}", segment.arguments)})],
                }
            })).collect::<Vec<_>>()
        }),
        syn::Type::Reference(value) => {
            json!({"reference": ty(&value.elem), "mutable": value.mutability.is_some()})
        }
        syn::Type::Array(value) => {
            json!({"array": ty(&value.elem), "length": format!("{:?}", value.len)})
        }
        syn::Type::Slice(value) => json!({"slice": ty(&value.elem)}),
        syn::Type::Tuple(value) => json!({"tuple": value.elems.iter().map(ty).collect::<Vec<_>>()}),
        _ => json!({"unsupported": format!("{value:?}")}),
    }
}

fn attrs(values: &[syn::Attribute]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("{:?}", value.meta))
        .collect()
}

#[derive(Default)]
struct Facts {
    rows: Vec<Value>,
    modules: Vec<Value>,
    function_depth: usize,
}

impl Facts {
    fn push(&mut self, mut row: Value) {
        row["modules"] = json!(self.modules);
        row["function_depth"] = json!(self.function_depth);
        self.rows.push(row);
    }
}

impl<'ast> Visit<'ast> for Facts {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.push(json!({"kind":"module", "name":item.ident.to_string(), "attributes":attrs(&item.attrs), "inline":item.content.is_some()}));
        self.modules.push(json!({"name":item.ident.to_string(), "attributes":attrs(&item.attrs), "visibility":format!("{:?}",item.vis)}));
        visit::visit_item_mod(self, item);
        self.modules.pop();
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        self.push(json!({"kind":"static", "name":item.ident.to_string(), "type":ty(&item.ty),
            "mutable":matches!(item.mutability, syn::StaticMutability::Mut(_)),
            "visibility":format!("{:?}",item.vis), "attributes":attrs(&item.attrs), "initializer":format!("{:?}",item.expr)}));
        visit::visit_item_static(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.push(json!({"kind":"struct", "name":item.ident.to_string(), "visibility":format!("{:?}",item.vis),
            "attributes":attrs(&item.attrs), "fields":item.fields.iter().map(|field| json!({
                "name":field.ident.as_ref().map(ToString::to_string), "type":ty(&field.ty),
                "visibility":format!("{:?}",field.vis), "attributes":attrs(&field.attrs)
            })).collect::<Vec<_>>()}));
        visit::visit_item_struct(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.push(json!({"kind":"alias", "name":item.ident.to_string(), "type":ty(&item.ty), "attributes":attrs(&item.attrs)}));
        visit::visit_item_type(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.push(json!({"kind":"use", "tree":format!("{:?}",item.tree), "visibility":format!("{:?}",item.vis), "attributes":attrs(&item.attrs)}));
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.push(json!({"kind":"function", "signature":format!("{:?}",item.sig), "visibility":format!("{:?}",item.vis), "attributes":attrs(&item.attrs)}));
        self.function_depth += 1;
        visit::visit_item_fn(self, item);
        self.function_depth -= 1;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.push(json!({"kind":"method", "signature":format!("{:?}",item.sig), "visibility":format!("{:?}",item.vis), "attributes":attrs(&item.attrs)}));
        self.function_depth += 1;
        visit::visit_impl_item_fn(self, item);
        self.function_depth -= 1;
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.push(json!({"kind":"macro", "path":format!("{:?}",item.path), "tokens":item.tokens.to_string()}));
        visit::visit_macro(self, item);
    }
}

fn run() -> Result<Value, Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("operator must supply root and exact input inventory".into());
    }
    let root = Path::new(&args[1]);
    let paths: Vec<String> = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let mut files = Vec::new();
    for path in paths {
        if Path::new(&path).is_absolute() || path.split('/').any(|part| part == "..") {
            return Err("unsafe input path".into());
        }
        let source = std::fs::read_to_string(root.join(&path))?;
        let row = match syn::parse_file(&source) {
            Err(error) => json!({"path":path, "source":source, "parse_error":error.to_string()}),
            Ok(parsed) => {
                let mut facts = Facts::default();
                facts.visit_file(&parsed);
                json!({"path":path, "source":source, "file_attributes":attrs(&parsed.attrs), "facts":facts.rows})
            }
        };
        files.push(row);
    }
    Ok(json!({"schema":"tc-protected-source-syntax/v1", "files":files}))
}

fn main() {
    match run() {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("broker syntax observation: {error}");
            std::process::exit(2);
        }
    }
}
