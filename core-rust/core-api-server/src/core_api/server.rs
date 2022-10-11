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

use std::future::Future;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router, extract::DefaultBodyLimit,
};
use parking_lot::RwLock;
use scrypto::prelude::*;
use state_manager::jni::state_manager::ActualStateManager;
use tower_http::limit::RequestBodyLimitLayer;

use super::{handlers::*, not_found_error, RequestHandlingError};

use handle_network_configuration as handle_provide_info_at_root_path;

#[derive(Clone)]
pub(crate) struct CoreApiState {
    pub state_manager: Arc<RwLock<ActualStateManager>>,
}
// TODO - try mapping request JSON errors into Response type
pub async fn create_server<F>(
    bind_addr: &str,
    shutdown_signal: F,
    state_manager: Arc<RwLock<ActualStateManager>>,
) where
    F: Future<Output = ()>,
{
    let core_api_state = CoreApiState { state_manager };

    // TODO - Change to remove the Tower RequestBodyLimitLayer middleware and use DefaultBodyLimit::max
    // once it is released https://github.com/tokio-rs/axum/pull/1397
    // TODO - Change this to be slightly larger than the double the max transaction payload size.
    // (We double due to the hex encoding of the payload)
    const LARGE_REQUEST_MAX_BYTES: usize = 50 * 1024 * 1024;

    let router = Router::new()
        // This only adds a route for /core, /core/ doesn't seem possible using /nest
        .route("/", get(handle_provide_info_at_root_path))
        .route(
            "/status/network-configuration",
            post(handle_network_configuration),
        )
        .route("/status/network-status", post(handle_network_status))
        .route("/transaction/submit", post(handle_transaction_submit)
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(LARGE_REQUEST_MAX_BYTES))
        )
        .route("/transaction/preview", post(handle_transaction_preview)
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(LARGE_REQUEST_MAX_BYTES))
        )
        .route("/transaction/stream", post(handle_transaction_stream))
        .route("/v0", get(handle_provide_info_at_root_path))
        .route("/v0/", get(handle_provide_info_at_root_path))
        .route(
            "/v0/status/network-configuration",
            post(handle_network_configuration),
        )
        .route("/v0/transaction/submit", post(handle_v0_transaction_submit)
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(LARGE_REQUEST_MAX_BYTES))
        )
        .route("/v0/transaction/status", post(handle_v0_transaction_status))
        .route(
            "/v0/transaction/receipt",
            post(handle_v0_transaction_receipt),
        )
        .route("/v0/state/epoch", post(handle_v0_state_epoch))
        .route("/v0/state/component", post(handle_v0_state_component))
        .route("/v0/state/resource", post(handle_v0_state_resource))
        .route("/v0/state/non-fungible", post(handle_v0_state_non_fungible))
        .route("/v0/state/package", post(handle_v0_state_package))
        .layer(Extension(core_api_state));

    let prefixed_router = Router::new()
        .nest("/core", router)
        .route("/", get(handle_no_core_path));

    let bind_addr = bind_addr.parse().expect("Failed to parse bind address");

    axum::Server::bind(&bind_addr)
        .serve(prefixed_router.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}

#[tracing::instrument(err(Debug))]
pub(crate) async fn handle_no_core_path() -> Result<(), RequestHandlingError> {
    Err(not_found_error("Try /core"))
}

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct CoreApiServerConfig {
    pub bind_interface: String,
    pub port: u32,
}
