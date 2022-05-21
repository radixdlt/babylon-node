mod state_manager_jni;

pub use state_manager_jni::blocking_get_state_manager_from_jni;

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use tokio::runtime::Runtime as TokioRuntime;
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

pub struct StateManagerRequest {
  resp: oneshot::Sender<StateManager>
}

pub fn init(public_key: Vec<u8>) -> (Arc<TokioRuntime>, Sender<StateManagerRequest>) {
  let state_manager = StateManager::new(public_key);

  let tokio_runtime =
    Arc::new(tokio::runtime::Runtime::new().unwrap());

  let mut state_manager_channel =
    mpsc::channel::<StateManagerRequest>(1);
  
  tokio_runtime.spawn(async move {
    while let Some(request) = state_manager_channel.1.recv().await {
      let _ = request.resp.send(state_manager.clone());
    }
  });

  (tokio_runtime, state_manager_channel.0)
}

pub fn blocking_get_state_manager(
  tokio_runtime: Arc<TokioRuntime>,
  state_manager_channel_sender: Sender<StateManagerRequest>
) -> StateManager {
  let result_handle = tokio_runtime.spawn(async move {
      let (resp_tx, resp_rx) = oneshot::channel();
      let cmd = StateManagerRequest { resp: resp_tx };
      let _unused = state_manager_channel_sender.send(cmd).await;
      let res = resp_rx.await;
      res.unwrap()
  });

  tokio_runtime.block_on(result_handle).unwrap()
}
