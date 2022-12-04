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

use crate::jni::java_structure::*;
use crate::jni::utils::*;
use crate::mempool::simple_mempool::SimpleMempool;
use crate::mempool::MempoolConfig;
use crate::state_manager::{LoggingConfig, StateManager};
use crate::store::{DatabaseConfig, StateManagerDatabase};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use parking_lot::RwLock;
use radix_engine_interface::core::NetworkDefinition;

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

fn do_prometheus_metrics(state_manager: &ActualStateManager, _args: ()) -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = vec![];
    encoder
        .encode(&state_manager.prometheus_registry.gather(), &mut buffer)
        .unwrap();

    String::from_utf8(buffer).unwrap()
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_prometheus_StateManagerPrometheus_prometheusMetrics(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    args: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, args, do_prometheus_metrics)
}

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct StateConfig {
    pub rounds_per_epoch: u64,
}

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct StateManagerConfig {
    pub network_definition: NetworkDefinition,
    pub state_config: StateConfig,
    pub mempool_config: Option<MempoolConfig>,
    pub db_config: DatabaseConfig,
    pub logging_config: LoggingConfig,
}

pub type ActualStateManager = StateManager<StateManagerDatabase>;

pub struct JNIStateManager {
    pub state_manager: Arc<RwLock<ActualStateManager>>,
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

        let store = StateManagerDatabase::from_config(config.db_config);
        let mempool = SimpleMempool::new(mempool_config);

        // Build the state manager.
        let state_manager = Arc::new(parking_lot::const_rwlock(StateManager::new(
            config.network_definition,
            config.state_config.rounds_per_epoch,
            mempool,
            store,
            config.logging_config,
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
    ) -> Arc<RwLock<ActualStateManager>> {
        let jni_state_manager: MutexGuard<JNIStateManager> = env
            .get_rust_field(j_state_manager, POINTER_JNI_FIELD_NAME)
            .unwrap();
        jni_state_manager.state_manager.clone()
    }
}

pub fn export_extern_functions() {}
