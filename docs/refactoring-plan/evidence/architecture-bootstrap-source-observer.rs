//! Read-only syntax observations of protected real source copies, not a checker.
use serde_json::{Value, json};
use std::path::Path;
use syn::visit::{self, Visit};

#[derive(Default)]
struct Facts(Vec<Value>);
impl<'ast> Visit<'ast> for Facts {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        // Preserve parsed structure rather than a spelling-based verdict.
        // Function bodies are visited separately to keep the record bounded.
        match item {
            syn::Item::Fn(value) => self.0.push(json!({"kind":"function", "signature":format!("{:?}",value.sig), "visibility":format!("{:?}",value.vis), "attributes":format!("{:?}",value.attrs)})),
            syn::Item::Const(value) if value.ident == "CHECKS" => self.0.push(json!({"kind":"CHECKS", "expression":format!("{:?}",value.expr)})),
            syn::Item::Enum(value) => self.0.push(json!({"kind":"enum", "name":value.ident.to_string(), "variants":format!("{:?}",value.variants)})),
            syn::Item::Mod(value) => self.0.push(json!({"kind":"module", "name":value.ident.to_string(), "attributes":format!("{:?}",value.attrs), "inline":value.content.is_some()})),
            syn::Item::Struct(value) => self.0.push(json!({"kind":"struct", "name":value.ident.to_string(), "visibility":format!("{:?}",value.vis), "attributes":format!("{:?}",value.attrs)})),
            syn::Item::Trait(value) => self.0.push(json!({"kind":"trait", "name":value.ident.to_string(), "declaration":format!("{value:?}")})),
            _ => {}
        }
        visit::visit_item(self, item);
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.0.push(json!({"kind":"method", "signature":format!("{:?}",item.sig), "attributes":format!("{:?}",item.attrs)}));
        visit::visit_impl_item_fn(self, item);
    }
    fn visit_expr_method_call(&mut self, item: &'ast syn::ExprMethodCall) {
        self.0.push(json!({"kind":"method_call", "method":item.method.to_string(), "receiver":format!("{:?}",item.receiver)}));
        visit::visit_expr_method_call(self, item);
    }
    fn visit_expr_call(&mut self, item: &'ast syn::ExprCall) {
        self.0
            .push(json!({"kind":"function_call", "callee":format!("{:?}",item.func)}));
        visit::visit_expr_call(self, item);
    }
    fn visit_expr_macro(&mut self, item: &'ast syn::ExprMacro) {
        self.0
            .push(json!({"kind":"expression_macro", "syntax":format!("{item:?}")}));
        visit::visit_expr_macro(self, item);
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        self.0
            .push(json!({"kind":"item_macro", "syntax":format!("{item:?}")}));
        visit::visit_item_macro(self, item);
    }
    fn visit_expr_unsafe(&mut self, item: &'ast syn::ExprUnsafe) {
        self.0
            .push(json!({"kind":"unsafe_block", "syntax":format!("{item:?}")}));
        visit::visit_expr_unsafe(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if item.unsafety.is_some() {
            self.0
                .push(json!({"kind":"unsafe_impl", "syntax":format!("{:?}",item.trait_)}));
        }
        visit::visit_item_impl(self, item);
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    assert_eq!(args.len(), 3, "operator supplies exact root and input list");
    let root = Path::new(&args[1]);
    let paths: Vec<String> = serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let mut observed = Vec::new();
    for path in paths {
        assert!(!Path::new(&path).is_absolute() && !path.split('/').any(|part| part == ".."));
        match std::fs::read_to_string(root.join(&path)) {
            Err(error) => {
                observed.push(json!({"path":path, "read_error":format!("{:?}",error.kind())}))
            }
            Ok(source) if path.ends_with(".rs") => match syn::parse_file(&source) {
                Err(error) => observed
                    .push(json!({"path":path,"parse_error":error.to_string(),"source":source})),
                Ok(ast) => {
                    let mut facts = Facts::default();
                    facts.visit_file(&ast);
                    observed.push(json!({"path":path,"source":source,"file_attributes":format!("{:?}",ast.attrs),"facts":facts.0}));
                }
            },
            Ok(source) => observed.push(json!({"path":path,"source":source})),
        }
    }
    println!(
        "{}",
        json!({"schema":"tc-protected-source-syntax/v1","files":observed})
    );
}
