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
use std::ops::Deref;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::str::FromStr;

use crate::jni_prelude::*;
use node_common::environment::setup_tracing;
use prometheus::Registry;

use node_common::scheduler::{Scheduler, UntilDropTracker};
use tokio::runtime::Runtime;
use tracing::Level;

use super::fatal_panic_handler::FatalPanicHandler;

use crate::protocol::ProtocolManager;
use crate::store::rocks_db::ActualStateManagerDatabase;
use crate::transaction::Preparator;
use p2p::rocks_db::{ActualAddressBookDatabase, ActualSafetyStoreDatabase};

const POINTER_JNI_FIELD_NAME: &str = "rustNodeRustEnvironmentPointer";

#[no_mangle]
extern "system" fn Java_com_radixdlt_environment_NodeRustEnvironment_init(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    j_config: jbyteArray,
) {
    jni_call(&env, || {
        JNINodeRustEnvironment::init(&env, j_node_rust_env, j_config)
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_environment_NodeRustEnvironment_cleanup(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
) {
    jni_call(&env, || {
        JNINodeRustEnvironment::cleanup(&env, j_node_rust_env)
    });
}

pub struct JNINodeRustEnvironment {
    pub runtime: Arc<Runtime>,
    pub network: NetworkDefinition,
    pub state_manager: StateManager,
    pub metric_registry: Arc<Registry>,
    pub running_task_tracker: UntilDropTracker,
    pub address_book_store: Arc<ActualAddressBookDatabase>,
    pub safety_store_store: Arc<ActualSafetyStoreDatabase>,
}

impl JNINodeRustEnvironment {
    pub fn init(env: &JNIEnv, j_node_rust_env: JObject, j_config: jbyteArray) {
        let (base_path, config) =
            Self::prepare_config(&jni_jbytearray_to_vector(env, j_config).unwrap());
        let network = config.network_definition.clone();
        let runtime = Arc::new(Runtime::new().unwrap());

        setup_tracing(
            runtime.deref(),
            std::env::var("JAEGER_AGENT_ENDPOINT").ok(),
            std::env::var("RADIXDLT_LOG_LEVEL")
                .ok()
                .and_then(|level| Level::from_str(level.as_str()).ok())
                .unwrap_or(Level::INFO),
        );

        let fatal_panic_handler = FatalPanicHandler::new(env, j_node_rust_env).unwrap();
        let metric_registry = Arc::new(Registry::new());
        let lock_factory = LockFactory::new("rn")
            .stopping_on_panic(move || fatal_panic_handler.handle_fatal_panic())
            .measured(metric_registry.deref());

        let scheduler = Scheduler::new("rn")
            .use_tokio(runtime.deref())
            .track_running_tasks()
            .measured(metric_registry.deref());

        let state_manager = StateManager::new(
            config,
            Some(MempoolRelayDispatcher::new(env, j_node_rust_env).unwrap()),
            Arc::new(JavaGenesisDataResolver::new(env, j_node_rust_env).unwrap()),
            &lock_factory,
            &metric_registry,
            &scheduler,
        );

        let running_task_tracker = scheduler.into_task_tracker();

        let address_book_db_path = Self::combine(&base_path, "address_book");
        let safety_store_db_path = Self::combine(&base_path, "consensus_safety_store");
        let jni_node_rust_env = JNINodeRustEnvironment {
            runtime,
            network,
            state_manager,
            metric_registry,
            running_task_tracker,
            address_book_store: Arc::new(ActualAddressBookDatabase::new(address_book_db_path)),
            safety_store_store: Arc::new(ActualSafetyStoreDatabase::new(safety_store_db_path)),
        };

        env.set_rust_field(j_node_rust_env, POINTER_JNI_FIELD_NAME, jni_node_rust_env)
            .unwrap();
    }

    fn prepare_config(config_bytes: &[u8]) -> (String, StateManagerConfig) {
        let config = StateManagerConfig::valid_from_java(config_bytes).unwrap();
        let base_path = config.database_backend_config.rocks_db_path.clone();
        let mut state_manager_db_path = config.database_backend_config.rocks_db_path.clone();

        state_manager_db_path.push(MAIN_SEPARATOR);
        state_manager_db_path.push_str("state_manager");

        let config = StateManagerConfig {
            database_backend_config: DatabaseBackendConfig {
                rocks_db_path: state_manager_db_path,
            },
            ..config
        };

        (base_path, config)
    }

    fn combine(base: &String, ext: &str) -> PathBuf {
        [base, ext].iter().collect()
    }

    pub fn cleanup(env: &JNIEnv, j_node_rust_env: JObject) {
        let jni_node_rust_env: JNINodeRustEnvironment = env
            .take_rust_field(j_node_rust_env, POINTER_JNI_FIELD_NAME)
            .unwrap();

        drop(jni_node_rust_env);
    }

    pub fn get<'a>(
        env: &'a JNIEnv<'a>,
        j_node_rust_env: JObject<'a>,
    ) -> std::sync::MutexGuard<'a, JNINodeRustEnvironment> {
        env.get_rust_field::<_, _, JNINodeRustEnvironment>(j_node_rust_env, POINTER_JNI_FIELD_NAME)
            .unwrap()
    }

    pub fn get_system_executor(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<SystemExecutor> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .system_executor
            .clone()
    }

    pub fn get_formatter(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<Formatter> {
        let env = Self::get(env, j_node_rust_env);
        env.state_manager.formatter.clone()
    }

    pub fn get_database(
        env: &JNIEnv,
        j_node_rust_env: JObject,
    ) -> Arc<DbLock<ActualStateManagerDatabase>> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .database
            .clone()
    }

    pub fn get_address_book_database(
        env: &JNIEnv,
        j_node_rust_env: JObject,
    ) -> Arc<ActualAddressBookDatabase> {
        Self::get(env, j_node_rust_env).address_book_store.clone()
    }

    pub fn get_safety_store_database(
        env: &JNIEnv,
        j_node_rust_env: JObject,
    ) -> Arc<ActualSafetyStoreDatabase> {
        Self::get(env, j_node_rust_env).safety_store_store.clone()
    }

    pub fn get_mempool_manager(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<MempoolManager> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .mempool_manager
            .clone()
    }

    pub fn get_preparator(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<Preparator> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .preparator
            .clone()
    }

    pub fn get_committer(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<Committer> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .committer
            .clone()
    }

    pub fn get_protocol_manager(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<ProtocolManager> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .protocol_manager
            .clone()
    }

    pub fn get_ledger_metrics(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<LedgerMetrics> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .ledger_metrics
            .clone()
    }
}

pub fn export_extern_functions() {}
