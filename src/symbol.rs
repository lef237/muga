use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

impl Symbol {
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct SymbolTable {
    names: Vec<String>,
    index: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(symbol) = self.index.get(name) {
            return *symbol;
        }
        let symbol = Symbol(self.names.len() as u32);
        let owned = name.to_string();
        self.names.push(owned.clone());
        self.index.insert(owned, symbol);
        symbol
    }

    pub fn lookup(&self, name: &str) -> Option<Symbol> {
        self.index.get(name).copied()
    }

    pub fn resolve(&self, symbol: Symbol) -> &str {
        self.names[symbol.as_u32() as usize].as_str()
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }
}
