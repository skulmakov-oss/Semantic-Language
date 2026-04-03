use crate::SymbolId;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SigTable {
    symbols: Vec<String>,
}

impl SigTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, name: &str) -> SymbolId {
        if let Some((idx, _)) = self
            .symbols
            .iter()
            .enumerate()
            .find(|(_, s)| s.as_str() == name)
        {
            return SymbolId(idx as u32);
        }
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(name.to_string());
        id
    }

    pub fn resolve(&self, id: SymbolId) -> Option<&str> {
        self.symbols.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }
}
