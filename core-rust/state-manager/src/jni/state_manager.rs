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

use std::sync::{Arc, MutexGuard};

use crate::mempool::priority_mempool::PriorityMempool;
use crate::state_manager::{LoggingConfig, StateManager};
use crate::store::{DatabaseBackendConfig, DatabaseFlags, StateManagerDatabase};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use node_common::config::limits::VertexLimitsConfig;
use node_common::config::MempoolConfig;
use node_common::environment::setup_tracing;
use node_common::java::*;
use node_common::locks::RwLock;
use prometheus::{Encoder, Registry, TextEncoder};
use radix_engine::transaction::FeeReserveConfig;
use radix_engine_common::math::Decimal;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use tokio::runtime::Runtime;

use crate::mempool_manager::MempoolManager;
use crate::mempool_relay_dispatcher::MempoolRelayDispatcher;
use crate::transaction::{
    CachedCommittabilityValidator, CommittabilityValidator, ExecutionConfigurator,
    TransactionPreviewer,
};
use crate::PendingTransactionResultCache;

const POINTER_JNI_FIELD_NAME: &str = "rustStateManagerPointer";

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    j_config: jbyteArray,
) {
    jni_call(&env, || {
        JNIStateManager::init(&env, j_state_manager, j_config)
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
) {
    jni_call(&env, || JNIStateManager::cleanup(&env, j_state_manager));
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_prometheus_RustPrometheus_prometheusMetrics(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_no_args: ()| -> String {
        let registry = &JNIStateManager::get_state(&env, j_state_manager).metric_registry;
        let encoder = TextEncoder::new();
        let mut buffer = vec![];
        encoder.encode(&registry.gather(), &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    })
}

#[derive(Debug, ScryptoSbor)]
pub struct JavaVertexLimitsConfig {
    pub max_transaction_count: u32,
    pub max_total_transactions_size: u32,
    pub max_total_execution_cost_units_consumed: u32,
}

impl From<JavaVertexLimitsConfig> for VertexLimitsConfig {
    fn from(val: JavaVertexLimitsConfig) -> Self {
        VertexLimitsConfig {
            max_transaction_count: val.max_transaction_count,
            max_total_transactions_size: val.max_total_transactions_size as usize,
            max_total_execution_cost_units_consumed: val.max_total_execution_cost_units_consumed,
            ..VertexLimitsConfig::default()
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct StateManagerConfig {
    pub network_definition: NetworkDefinition,
    pub mempool_config: Option<MempoolConfig>,
    pub vertex_limits_config: Option<JavaVertexLimitsConfig>,
    pub database_backend_config: DatabaseBackendConfig,
    pub database_flags: DatabaseFlags,
    pub logging_config: LoggingConfig,
    pub no_fees: bool,
}

pub type ActualStateManager = StateManager<StateManagerDatabase>;

pub struct JNIStateManager {
    pub runtime: Arc<Runtime>,
    pub network: NetworkDefinition,
    pub state_manager: Arc<ActualStateManager>,
    pub database: Arc<RwLock<StateManagerDatabase>>,
    pub pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    pub mempool: Arc<RwLock<PriorityMempool>>,
    pub mempool_manager: Arc<MempoolManager>,
    pub committability_validator: Arc<CommittabilityValidator<StateManagerDatabase>>,
    pub transaction_previewer: Arc<TransactionPreviewer<StateManagerDatabase>>,
    pub metric_registry: Registry,
}

impl JNIStateManager {
    pub fn init(env: &JNIEnv, j_state_manager: JObject, j_config: jbyteArray) {
        let config_bytes: Vec<u8> = jni_jbytearray_to_vector(env, j_config).unwrap();
        let config = StateManagerConfig::from_java(&config_bytes).unwrap();

        let runtime = Runtime::new().unwrap();

        setup_tracing(&runtime, std::env::var("JAEGER_AGENT_ENDPOINT").ok());

        // Build the basic subcomponents.
        let mempool_config = match config.mempool_config {
            Some(mempool_config) => mempool_config,
            None =>
            // in general, missing mempool config should mean that mempool isn't needed
            // but for now just using a default
            {
                MempoolConfig::default()
            }
        };
        let network = config.network_definition;
        let logging_config = config.logging_config;

        let database = Arc::new(RwLock::new(StateManagerDatabase::from_config(
            config.database_backend_config,
            config.database_flags,
        )));
        let metric_registry = Registry::new();
        let mut fee_reserve_config = FeeReserveConfig::default();
        if config.no_fees {
            fee_reserve_config.cost_unit_price = Decimal::ZERO;
            fee_reserve_config.state_expansion_price = Decimal::ZERO;
        }
        let execution_configurator = Arc::new(ExecutionConfigurator::new(
            &logging_config,
            fee_reserve_config,
        ));
        let pending_transaction_result_cache = Arc::new(RwLock::new(
            PendingTransactionResultCache::new(10000, 10000),
        ));
        let committability_validator = Arc::new(CommittabilityValidator::new(
            &network,
            database.clone(),
            execution_configurator.clone(),
        ));
        let cached_committability_validator = CachedCommittabilityValidator::new(
            database.clone(),
            committability_validator.clone(),
            pending_transaction_result_cache.clone(),
        );
        let mempool = Arc::new(RwLock::new(PriorityMempool::new(mempool_config)));
        let mempool_relay_dispatcher = MempoolRelayDispatcher::new(env, j_state_manager).unwrap();
        let mempool_manager = Arc::new(MempoolManager::new(
            mempool.clone(),
            mempool_relay_dispatcher,
            cached_committability_validator,
            &metric_registry,
        ));
        let transaction_previewer = Arc::new(TransactionPreviewer::new(
            &network,
            database.clone(),
            execution_configurator.clone(),
        ));

        let vertex_limits_config = match config.vertex_limits_config {
            Some(java_vertex_limits_config) => java_vertex_limits_config.into(),
            None => VertexLimitsConfig::default(),
        };

        // Build the state manager.
        let state_manager = Arc::new(StateManager::new(
            &network,
            vertex_limits_config,
            database.clone(),
            mempool_manager.clone(),
            execution_configurator,
            pending_transaction_result_cache.clone(),
            logging_config,
            &metric_registry,
        ));

        let jni_state_manager = JNIStateManager {
            runtime: Arc::new(runtime),
            network,
            state_manager,
            database,
            pending_transaction_result_cache,
            mempool,
            mempool_manager,
            committability_validator,
            transaction_previewer,
            metric_registry,
        };

        env.set_rust_field(j_state_manager, POINTER_JNI_FIELD_NAME, jni_state_manager)
            .unwrap();
    }

    pub fn cleanup(env: &JNIEnv, j_state_manager: JObject) {
        let jni_state_manager: JNIStateManager = env
            .take_rust_field(j_state_manager, POINTER_JNI_FIELD_NAME)
            .unwrap();

        drop(jni_state_manager);
    }

    pub fn get_state<'a>(
        env: &'a JNIEnv<'a>,
        j_state_manager: JObject<'a>,
    ) -> MutexGuard<'a, JNIStateManager> {
        env.get_rust_field::<_, _, JNIStateManager>(j_state_manager, POINTER_JNI_FIELD_NAME)
            .unwrap()
    }

    pub fn get_state_manager(env: &JNIEnv, j_state_manager: JObject) -> Arc<ActualStateManager> {
        Self::get_state(env, j_state_manager).state_manager.clone()
    }

    pub fn get_database(
        env: &JNIEnv,
        j_state_manager: JObject,
    ) -> Arc<RwLock<StateManagerDatabase>> {
        Self::get_state(env, j_state_manager).database.clone()
    }

    pub fn get_mempool(env: &JNIEnv, j_state_manager: JObject) -> Arc<RwLock<PriorityMempool>> {
        Self::get_state(env, j_state_manager).mempool.clone()
    }

    pub fn get_mempool_manager(env: &JNIEnv, j_state_manager: JObject) -> Arc<MempoolManager> {
        Self::get_state(env, j_state_manager)
            .mempool_manager
            .clone()
    }
}

pub fn export_extern_functions() {}
