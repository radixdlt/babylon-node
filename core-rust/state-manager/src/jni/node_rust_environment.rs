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
use std::sync::Arc;

use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use node_common::environment::setup_tracing;
use node_common::java::{jni_call, jni_jbytearray_to_vector, StructFromJava};
use node_common::locks::*;
use prometheus::Registry;
use radix_engine_common::prelude::NetworkDefinition;

use node_common::scheduler::{Scheduler, UntilDropTracker};
use tokio::runtime::Runtime;

use crate::mempool_manager::MempoolManager;
use crate::mempool_relay_dispatcher::MempoolRelayDispatcher;
use crate::priority_mempool::PriorityMempool;

use crate::store::StateManagerDatabase;

use super::fatal_panic_handler::FatalPanicHandler;

use crate::{
    MainnetProtocolUpdaterFactory, ProtocolUpdaterFactory, StateComputer, StateManager,
    StateManagerConfig, TestingDefaultProtocolUpdaterFactory,
};

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
}

impl JNINodeRustEnvironment {
    pub fn init(env: &JNIEnv, j_node_rust_env: JObject, j_config: jbyteArray) {
        let config_bytes: Vec<u8> = jni_jbytearray_to_vector(env, j_config).unwrap();
        let config = StateManagerConfig::from_java(&config_bytes).unwrap();

        let network = config.network_definition.clone();

        let runtime = Arc::new(Runtime::new().unwrap());

        setup_tracing(runtime.deref(), std::env::var("JAEGER_AGENT_ENDPOINT").ok());

        let fatal_panic_handler = FatalPanicHandler::new(env, j_node_rust_env).unwrap();
        let metric_registry = Arc::new(Registry::new());
        let lock_factory = LockFactory::new("rn")
            .stopping_on_panic(move || fatal_panic_handler.handle_fatal_panic())
            .measured(metric_registry.deref());

        let scheduler = Scheduler::new("rn")
            .use_tokio(runtime.deref())
            .track_running_tasks()
            .measured(metric_registry.deref());

        let protocol_updater_factory: Box<dyn ProtocolUpdaterFactory + Send + Sync> =
            if network.id == NetworkDefinition::mainnet().id {
                Box::new(MainnetProtocolUpdaterFactory::new())
            } else {
                Box::new(TestingDefaultProtocolUpdaterFactory::new(network.clone()))
            };

        let state_manager = StateManager::new(
            config,
            Some(MempoolRelayDispatcher::new(env, j_node_rust_env).unwrap()),
            &lock_factory,
            protocol_updater_factory,
            &metric_registry,
            &scheduler,
        );

        let running_task_tracker = scheduler.into_task_tracker();

        let jni_node_rust_env = JNINodeRustEnvironment {
            runtime,
            network,
            state_manager,
            metric_registry,
            running_task_tracker,
        };

        env.set_rust_field(j_node_rust_env, POINTER_JNI_FIELD_NAME, jni_node_rust_env)
            .unwrap();
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

    pub fn get_state_computer(
        env: &JNIEnv,
        j_node_rust_env: JObject,
    ) -> Arc<StateComputer<StateManagerDatabase>> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .state_computer
            .clone()
    }

    pub fn get_database(
        env: &JNIEnv,
        j_node_rust_env: JObject,
    ) -> Arc<StateLock<StateManagerDatabase>> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .database
            .clone()
    }

    pub fn get_mempool(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<RwLock<PriorityMempool>> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .mempool
            .clone()
    }

    pub fn get_mempool_manager(env: &JNIEnv, j_node_rust_env: JObject) -> Arc<MempoolManager> {
        Self::get(env, j_node_rust_env)
            .state_manager
            .mempool_manager
            .clone()
    }
}

pub fn export_extern_functions() {}
