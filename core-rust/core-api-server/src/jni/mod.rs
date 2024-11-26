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

use std::sync::MutexGuard;

use crate::core_api::{create_server, CoreApiServerConfig, CoreApiState};
use crate::jni_prelude::*;
use futures::channel::oneshot;
use futures::channel::oneshot::Sender;
use futures::FutureExt;
use tokio::runtime::Runtime;

const POINTER_JNI_FIELD_NAME: &str = "rustCoreApiServerPointer";

pub struct RunningServer {
    pub shutdown_signal_sender: Sender<()>,
}

pub struct JNICoreApiServer {
    pub config: CoreApiServerConfig,
    pub runtime: Arc<Runtime>,
    pub state: CoreApiState,
    pub running_server: Option<RunningServer>,
    pub metric_registry: Arc<MetricRegistry>,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_api_CoreApiServer_init(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    j_core_api_server: JObject,
    j_config: jbyteArray,
) {
    jni_sbor_coded_call(&env, j_config, |config: CoreApiServerConfig| {
        let jni_node_rust_env = JNINodeRustEnvironment::get(&env, j_rust_global_context);

        let jni_core_api_server = JNICoreApiServer {
            runtime: jni_node_rust_env.runtime.clone(),
            state: CoreApiState {
                network: jni_node_rust_env.network.clone(),
                flags: config.flags.clone(),
                state_manager: jni_node_rust_env.state_manager.clone(),
            },
            config,
            running_server: None,
            metric_registry: jni_node_rust_env.metric_registry.clone(),
        };

        env.set_rust_field(
            j_core_api_server,
            POINTER_JNI_FIELD_NAME,
            jni_core_api_server,
        )
        .unwrap()
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_api_CoreApiServer_start(
    env: JNIEnv,
    _class: JClass,
    j_core_api_server: JObject,
) {
    jni_call(&env, || {
        let (shutdown_signal_sender, shutdown_signal_receiver) = oneshot::channel::<()>();

        let mut jni_core_api_server: MutexGuard<JNICoreApiServer> = env
            .get_rust_field(j_core_api_server, POINTER_JNI_FIELD_NAME)
            .unwrap();

        let config = &jni_core_api_server.config;

        let state = jni_core_api_server.state.clone();
        let runtime = &jni_core_api_server.runtime;
        let metric_registry = jni_core_api_server.metric_registry.clone();

        let bind_addr = format!("{}:{}", config.bind_interface, config.port);
        runtime.spawn(async move {
            create_server(
                &bind_addr,
                shutdown_signal_receiver.map(|_| ()),
                state,
                &metric_registry,
            )
            .await;
        });

        jni_core_api_server.running_server = Some(RunningServer {
            shutdown_signal_sender,
        });
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_api_CoreApiServer_stop(
    env: JNIEnv,
    _class: JClass,
    j_core_api_server: JObject,
) {
    jni_call(&env, || {
        if let Ok(jni_core_api_server) = env.take_rust_field::<JObject, &str, JNICoreApiServer>(
            j_core_api_server,
            POINTER_JNI_FIELD_NAME,
        ) {
            if let Some(running_server) = jni_core_api_server.running_server {
                running_server.shutdown_signal_sender.send(()).unwrap();
            }
            // No-op, drop the jni_core_api_server
        }
    });
}

pub fn export_extern_functions() {}
