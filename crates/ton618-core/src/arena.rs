use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaId(pub usize);

#[derive(Debug, Clone)]
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn alloc(&mut self, value: T) -> ArenaId {
        let id = ArenaId(self.items.len());
        self.items.push(value);
        id
    }

    pub fn get(&self, id: ArenaId) -> Option<&T> {
        self.items.get(id.0)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
