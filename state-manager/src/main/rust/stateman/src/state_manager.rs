use crate::mempool::Mempool;
use crate::transaction_store::TransactionStore;
use crate::vertex_store::VertexStore;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct StateManager<M: Mempool> {
    pub public_key: Vec<u8>,
    pub mempool: Arc<Mutex<M>>,
    pub vertex_store: Arc<Mutex<VertexStore>>,
    pub transaction_store: Arc<Mutex<TransactionStore>>,
}

impl<M: Mempool> StateManager<M> {
    pub fn new(
        mempool: M,
        vertex_store: VertexStore,
        transaction_store: TransactionStore,
    ) -> StateManager<M> {
        StateManager {
            public_key: Vec::new(),
            mempool: Arc::new(Mutex::new(mempool)),
            vertex_store: Arc::new(Mutex::new(vertex_store)),
            transaction_store: Arc::new(Mutex::new(transaction_store)),
        }
    }
}
