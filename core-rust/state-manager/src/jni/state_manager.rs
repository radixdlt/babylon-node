/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

use crate::jni::dtos::*;
use crate::jni::utils::*;
use crate::mempool::simple::SimpleMempool;
use crate::mempool::MempoolConfig;
use crate::state_manager::{DatabaseConfig, StateManager, StateManagerConfig};
use crate::store::{InMemoryTransactionStore, RocksDBTransactionStore, TransactionStore};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use std::path::PathBuf;

use crate::receipt::LedgerTransactionReceipt;
use crate::types::{TId, Transaction};
use radix_engine_stores::memory_db::SerializedInMemorySubstateStore;
use std::sync::{Arc, Mutex, MutexGuard};

const POINTER_JNI_FIELD_NAME: &str = "rustStateManagerPointer";

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    j_config: jbyteArray,
) {
    JNIStateManager::init(&env, j_state_manager, j_config);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
) {
    JNIStateManager::cleanup(&env, j_state_manager);
}

pub enum SupportedStateManagerStore {
    InMemory(InMemoryTransactionStore),
    RocksDB(RocksDBTransactionStore),
    None,
}

impl SupportedStateManagerStore {
    pub fn from_config(config: DatabaseConfig) -> Self {
        match config {
            DatabaseConfig::InMemory => {
                SupportedStateManagerStore::InMemory(InMemoryTransactionStore::new())
            }
            DatabaseConfig::RocksDB(path) => SupportedStateManagerStore::RocksDB(
                RocksDBTransactionStore::new(PathBuf::from(path)),
            ),
            DatabaseConfig::None => SupportedStateManagerStore::None,
        }
    }
}

impl TransactionStore for SupportedStateManagerStore {
    fn insert_transactions(&mut self, transactions: Vec<(&Transaction, LedgerTransactionReceipt)>) {
        match self {
            SupportedStateManagerStore::InMemory(store) => store.insert_transactions(transactions),
            SupportedStateManagerStore::RocksDB(store) => store.insert_transactions(transactions),
            SupportedStateManagerStore::None => panic!("Unexpected call to no state manager store"),
        }
    }

    fn get_transaction(&self, tid: &TId) -> (Vec<u8>, LedgerTransactionReceipt) {
        match self {
            SupportedStateManagerStore::InMemory(store) => store.get_transaction(tid),
            SupportedStateManagerStore::RocksDB(store) => store.get_transaction(tid),
            SupportedStateManagerStore::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

pub type ActualStateManager =
    StateManager<SimpleMempool, SerializedInMemorySubstateStore, SupportedStateManagerStore>;

pub struct JNIStateManager {
    pub state_manager: Arc<Mutex<ActualStateManager>>,
}

impl JNIStateManager {
    pub fn init(env: &JNIEnv, j_state_manager: JObject, j_config: jbyteArray) {
        let config_bytes: Vec<u8> = jni_jbytearray_to_vector(env, j_config).unwrap();
        let config = StateManagerConfig::from_java(&config_bytes).unwrap();

        // Build the basic subcomponents.
        let mempool_config = match config.mempool_config {
            Some(mempool_config) => mempool_config,
            None =>
            // in general, missing mempool config should mean that mempool isn't needed
            // but for now just using a default
            {
                MempoolConfig { max_size: 10 }
            }
        };

        let transaction_store = SupportedStateManagerStore::from_config(config.db_config);
        let mempool = SimpleMempool::new(mempool_config);
        let substate_store = SerializedInMemorySubstateStore::with_bootstrap();

        // Build the state manager.
        let state_manager = Arc::new(Mutex::new(StateManager::new(
            config.network_definition,
            mempool,
            transaction_store,
            substate_store,
        )));

        let jni_state_manager = JNIStateManager { state_manager };

        env.set_rust_field(j_state_manager, POINTER_JNI_FIELD_NAME, jni_state_manager)
            .unwrap();
    }

    pub fn cleanup(env: &JNIEnv, j_state_manager: JObject) {
        let jni_state_manager: JNIStateManager = env
            .take_rust_field(j_state_manager, POINTER_JNI_FIELD_NAME)
            .unwrap();

        drop(jni_state_manager);
    }

    pub fn get_state_manager(
        env: &JNIEnv,
        j_state_manager: JObject,
    ) -> Arc<Mutex<ActualStateManager>> {
        let state_manager = {
            let jni_state_manager: MutexGuard<JNIStateManager> = env
                .get_rust_field(j_state_manager, POINTER_JNI_FIELD_NAME)
                .unwrap();
            let state_manager = jni_state_manager.state_manager.clone();
            // Ensure the JNI mutex lock is released
            drop(jni_state_manager);
            state_manager
        };

        state_manager
    }
}

pub fn export_extern_functions() {}
