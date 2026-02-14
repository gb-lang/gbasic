use gbasic_common::types::Type;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Type,
    #[allow(dead_code)]
    pub mutable: bool,
}

/// Nested-scope symbol table using a stack of hashmaps.
pub struct SymbolTable {
    scopes: Vec<IndexMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![IndexMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(IndexMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }
}
