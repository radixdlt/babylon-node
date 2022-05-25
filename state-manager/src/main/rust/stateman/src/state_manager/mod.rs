mod state_manager_jni;

pub use state_manager_jni::get_state_manager_from_jni_env;

use std::sync::Arc;
use std::sync::Mutex;
use crate::vertex_store::VertexStore;
use crate::transaction_store::TransactionStore;

#[derive(Clone, Debug)]
pub struct StateManager {
  pub public_key: Vec<u8>,
  pub vertex_store: Arc<Mutex<VertexStore>>,
  pub transaction_store: Arc<Mutex<TransactionStore>>,
}

impl StateManager {
  pub fn new(public_key: Vec<u8>) -> StateManager {
    StateManager {
        public_key: public_key,
        vertex_store: Arc::new(Mutex::new(VertexStore::new())),
        transaction_store: Arc::new(Mutex::new(TransactionStore::new()))
    }
  }
}
