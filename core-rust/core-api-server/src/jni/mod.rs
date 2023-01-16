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

use crate::core_api::{create_server, CoreApiServerConfig};
use futures::channel::oneshot;
use futures::channel::oneshot::Sender;
use futures::FutureExt;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;

use opentelemetry_otlp::WithExportConfig;
use parking_lot::RwLock;
use state_manager::jni::java_structure::JavaStructure;
use state_manager::jni::state_manager::{ActualStateManager, JNIStateManager};
use state_manager::jni::utils::*;
use std::str;
use std::sync::{Arc, MutexGuard};
use tokio::runtime::Runtime as TokioRuntime;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const POINTER_JNI_FIELD_NAME: &str = "rustCoreApiServerPointer";

pub struct RunningServer {
    pub tokio_runtime: TokioRuntime,
    pub shutdown_signal_sender: Sender<()>,
}

pub struct JNICoreApiServer {
    pub config: CoreApiServerConfig,
    pub state_manager: Arc<ActualStateManager>,
    pub running_server: Option<RunningServer>,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_api_CoreApiServer_init(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    j_core_api_server: JObject,
    j_config: jbyteArray,
) {
    let state_manager = JNIStateManager::get_state_manager(&env, j_state_manager);
    let config_bytes: Vec<u8> = jni_jbytearray_to_vector(&env, j_config).unwrap();
    let config = CoreApiServerConfig::from_java(&config_bytes).unwrap();
    let jni_core_api_server = JNICoreApiServer {
        config,
        state_manager,
        running_server: None,
    };

    env.set_rust_field(
        j_core_api_server,
        POINTER_JNI_FIELD_NAME,
        jni_core_api_server,
    )
    .unwrap();
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_api_CoreApiServer_start(
    env: JNIEnv,
    _class: JClass,
    j_core_api_server: JObject,
) {
    let tokio_runtime = TokioRuntime::new().unwrap();

    let (shutdown_signal_sender, shutdown_signal_receiver) = oneshot::channel::<()>();

    let mut jni_core_api_server: MutexGuard<JNICoreApiServer> = env
        .get_rust_field(j_core_api_server, POINTER_JNI_FIELD_NAME)
        .unwrap();

    let config = &jni_core_api_server.config;

    let state_manager = jni_core_api_server.state_manager.clone();

    let bind_addr = format!("{}:{}", config.bind_interface, config.port);
    tokio_runtime.spawn(async move {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".into(), format!("Basic: eyJrIjoiMWYxMzIyZThlZTA5YmYyNWM0ZmMwYmE1ZTUzNTI0YmZhYTI2NWM1ZSIsIm4iOiJvcGVudGVsZW1ldHJ5LXRlc3Qta2V5LWRlbGV0ZW1lIiwiaWQiOjE1NDc2OX0="));
        let tracer = opentelemetry_otlp::new_pipeline().tracing().with_exporter(opentelemetry_otlp::new_exporter().http().with_endpoint("https://otlp-gateway-prod-eu-west-0.grafana.net/otlp").with_headers(headers)).install_batch(opentelemetry::runtime::Tokio).unwrap();
                // let tracer = opentelemetry_jaeger::new_collector_pipeline().with_reqwest().with_username("154769").with_endpoint("https://otlp-gateway-prod-eu-central-0.grafana.net/otlp").with_password("a6cf97964aec1b81dd833d904eaab1c4fe97125d")
                //     .with_service_name("core_api")
                //     .install_batch(opentelemetry::runtime::Tokio)
                //     .unwrap();

                let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

                // Trying to initialize a global logger here, and carry on if this fails.
                let _ = tracing_subscriber::registry()
                    .with(tracing_subscriber::filter::LevelFilter::INFO)
                    .with(opentelemetry)
                    .with(tracing_subscriber::fmt::layer())
                    .try_init();
        // match std::env::var("JAEGER_AGENT_ENDPOINT") {
        //     Ok(jaeger_agent_endpoint) => {
        //     }
        //     Err(_) => {
        //         let _ = tracing_subscriber::registry()
        //             .with(tracing_subscriber::filter::LevelFilter::INFO)
        //             .with(tracing_subscriber::fmt::layer())
        //             .try_init();
        //     }
        // }

        create_server(
            &bind_addr,
            shutdown_signal_receiver.map(|_| ()),
            state_manager,
        )
        .await;
    });

    jni_core_api_server.running_server = Some(RunningServer {
        tokio_runtime,
        shutdown_signal_sender,
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_api_CoreApiServer_stop(
    env: JNIEnv,
    _class: JClass,
    j_core_api_server: JObject,
) {
    if let Ok(jni_core_api_server) = env.take_rust_field::<JObject, &str, JNICoreApiServer>(
        j_core_api_server,
        POINTER_JNI_FIELD_NAME,
    ) {
        if let Some(running_server) = jni_core_api_server.running_server {
            running_server.shutdown_signal_sender.send(()).unwrap();
        }
        // No-op, drop the jni_core_api_server
    }
}

pub fn export_extern_functions() {}
