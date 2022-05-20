use std::collections::HashSet;

pub struct VertexStore {
    in_mem_store: HashSet<Vec<u8>>
}

impl VertexStore {
    pub fn new() -> VertexStore {
        VertexStore { in_mem_store: HashSet::new() }
    }

    pub fn insert_vertex(&mut self, vertex: Vec<u8>) {
        self.in_mem_store.insert(vertex);
    }

    pub fn contains_vertex(&self, vertex: Vec<u8>) -> bool {
        self.in_mem_store.contains(&vertex)
    }
}
