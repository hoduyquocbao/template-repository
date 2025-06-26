use std::fs;
use syn::{visit::Visit, Item, ItemStruct, ItemTrait, ItemEnum, ItemUnion, ItemType, ItemFn, ItemConst, ItemStatic, ImplItem, ItemImpl, ItemMacro};
use crate::rules::Config;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Violation {
    pub line: Option<usize>,
    pub name: String,
    pub kind: &'static str,
}

pub fn scan(file: &str, config: &Config) -> Result<Vec<Violation>, String> {
    let src = fs::read_to_string(file).map_err(|e| format!("Không mở được file {file}: {e}"))?;
    let ast = syn::parse_file(&src).map_err(|e| format!("Lỗi parse file {file}: {e}"))?;
    let mut visitor = Visitor {
        config,
        violations: Vec::new(),
    };
    visitor.visit_file(&ast);
    // Kiểm tra duplicate identifier
    let mut counts = std::collections::HashMap::new();
    for v in &visitor.violations {
        *counts.entry(&v.name).or_insert(0) += 1;
    }
    let mut all = visitor.violations.clone();
    for v in &visitor.violations {
        if let Some(c) = counts.get(&v.name) {
            if *c > 1 {
                all.push(Violation {
                    line: v.line,
                    name: v.name.clone(),
                    kind: "Duplicate",
                });
            }
        }
    }
    Ok(all)
}

struct Visitor<'a> {
    config: &'a Config,
    violations: Vec<Violation>,
}

impl<'a, 'ast> Visit<'ast> for Visitor<'a> {
    fn visit_item(&mut self, i: &'ast Item) {
        match i {
            Item::Struct(ItemStruct { ident, fields, .. }) => {
                self.check(ident, "PascalCase");
                for f in fields {
                    if let Some(id) = &f.ident {
                        self.check(id, "Field");
                    }
                }
            }
            Item::Trait(ItemTrait { ident, .. })
            | Item::Union(ItemUnion { ident, .. })
            | Item::Type(ItemType { ident, .. }) => {
                self.check(ident, "PascalCase");
            }
            Item::Enum(ItemEnum { ident, variants, .. }) => {
                self.check(ident, "PascalCase");
                for v in variants {
                    self.check(&v.ident, "Variant");
                }
            }
            Item::Fn(ItemFn { sig, .. }) => {
                self.check(&sig.ident, "Fn");
            }
            Item::Const(ItemConst { ident, .. }) => {
                self.check(ident, "Const");
            }
            Item::Static(ItemStatic { ident, .. }) => {
                self.check(ident, "Static");
            }
            Item::Impl(ItemImpl { items, .. }) => {
                for impl_item in items {
                    if let ImplItem::Fn(m) = impl_item {
                        self.check(&m.sig.ident, "Method");
                    }
                }
            }
            Item::Macro(ItemMacro { .. }) => {
                // Bỏ qua macro
            }
            _ => {}
        }
        syn::visit::visit_item(self, i);
    }
}

impl<'a> Visitor<'a> {
    fn check(&mut self, ident: &syn::Ident, kind: &'static str) {
        let name = ident.to_string();
        if let Some(black) = &self.config.blacklist {
            if black.iter().any(|b| b == &name) {
                self.violations.push(Violation {
                    line: None,
                    name,
                    kind: "Blacklist",
                });
                return;
            }
        }
        if let Some(white) = &self.config.whitelist {
            if white.iter().any(|w| w == &name) {
                return;
            }
        }
        // Kiểm tra enable rule
        if kind == "PascalCase" && self.config.pascal == Some(false) {
            return;
        }
        if kind == "Variant" && self.config.pascal == Some(false) {
            return;
        }
        if kind == "camelCase" && self.config.camel == Some(false) {
            return;
        }
        if kind == "snake_case" && self.config.snake == Some(false) {
            return;
        }
        // Kiểm tra độ dài định danh
        if self.config.length.unwrap_or(true) {
            if let Some(min) = self.config.min {
                if name.len() < min {
                    self.violations.push(Violation {
                        line: None,
                        name: name.clone(),
                        kind: "Length",
                    });
                }
            }
            if let Some(max) = self.config.max {
                if name.len() > max {
                    self.violations.push(Violation {
                        line: None,
                        name: name.clone(),
                        kind: "Length",
                    });
                }
            }
        }
        // Kiểm tra pattern
        if kind == "PascalCase" && hub(&name) > 1 {
            self.violations.push(Violation {
                line: None,
                name,
                kind: "PascalCase",
            });
        } else if kind == "Variant" && hub(&name) > 1 {
            self.violations.push(Violation {
                line: None,
                name,
                kind: "Variant",
            });
        } else if camel(&name) {
            self.violations.push(Violation {
                line: None,
                name,
                kind: "camelCase",
            });
        } else if snake(&name) {
            self.violations.push(Violation {
                line: None,
                name,
                kind: "snake_case",
            });
        }
    }
}

fn hub(name: &str) -> usize {
    name.chars().filter(|c| c.is_uppercase()).count()
}
fn camel(name: &str) -> bool {
    name.chars().any(|c| c.is_uppercase()) && name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
}
fn snake(name: &str) -> bool {
    name.contains('_')
} 