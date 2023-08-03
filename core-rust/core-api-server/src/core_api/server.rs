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

use super::metrics::CoreApiMetrics;
use super::metrics_layer::MetricsLayer;
use axum::extract::State;
use axum::http::{StatusCode, Uri};
use axum::middleware::map_response;
use axum::response::Response;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};

use prometheus::Registry;
use radix_engine::types::{Categorize, Decode, Encode};
use radix_engine_common::network::NetworkDefinition;
use state_manager::jni::rust_global_context::RadixNode;
use tower_http::catch_panic::CatchPanicLayer;
use tracing::{debug, error, info, trace, warn, Level};

use super::{constants::LARGE_REQUEST_MAX_BYTES, handlers::*, not_found_error, ResponseError};

use crate::core_api::models::ErrorResponse;
use crate::core_api::InternalServerErrorResponseForPanic;
use handle_status_network_configuration as handle_provide_info_at_root_path;

#[derive(Clone)]
pub struct CoreApiState {
    pub network: NetworkDefinition,
    pub radix_node: RadixNode,
}

pub async fn create_server<F>(
    bind_addr: &str,
    shutdown_signal: F,
    core_api_state: CoreApiState,
    metric_registry: &Registry,
) where
    F: Future<Output = ()>,
{
    let router = Router::new()
        // This only adds a route for /core, /core/ doesn't seem possible using /nest
        .route("/", get(handle_provide_info_at_root_path))
        // Release Candidate backward compatible Sub-API
        .route(
            "/lts/transaction/construction",
            post(lts::handle_lts_transaction_construction),
        )
        .route(
            "/lts/transaction/status",
            post(lts::handle_lts_transaction_status),
        )
        .route(
            "/lts/transaction/submit",
            post(lts::handle_lts_transaction_submit),
        )
        .route(
            "/lts/state/account-all-fungible-resource-balances",
            post(lts::handle_lts_state_account_all_fungible_resource_balances),
        )
        .route(
            "/lts/state/account-fungible-resource-balance",
            post(lts::handle_lts_state_account_fungible_resource_balance),
        )
        .route(
            "/lts/stream/transaction-outcomes",
            post(lts::handle_lts_stream_transaction_outcomes),
        )
        .route(
            "/lts/stream/account-transaction-outcomes",
            post(lts::handle_lts_stream_account_transaction_outcomes),
        )
        // Status Sub-API
        .route(
            "/status/network-configuration",
            post(handle_status_network_configuration),
        )
        .route("/status/network-status", post(handle_status_network_status))
        .route("/status/scenarios", post(handle_status_scenarios))
        // Mempool Sub-API
        .route("/mempool/list", post(handle_mempool_list))
        .route("/mempool/transaction", post(handle_mempool_transaction))
        // Transaction Sub-API
        .route(
            "/transaction/parse",
            post(handle_transaction_parse).layer(DefaultBodyLimit::max(LARGE_REQUEST_MAX_BYTES)),
        )
        .route(
            "/transaction/submit",
            post(handle_transaction_submit).layer(DefaultBodyLimit::max(LARGE_REQUEST_MAX_BYTES)),
        )
        .route("/transaction/status", post(handle_transaction_status))
        .route("/transaction/receipt", post(handle_transaction_receipt))
        .route(
            "/transaction/preview",
            post(handle_transaction_preview).layer(DefaultBodyLimit::max(LARGE_REQUEST_MAX_BYTES)),
        )
        .route(
            "/transaction/call-preview",
            post(handle_transaction_callpreview),
        )
        // Stream Sub-API
        .route("/stream/transactions", post(handle_stream_transactions))
        // State Sub-API
        .route(
            "/state/consensus-manager",
            post(handle_state_consensus_manager),
        )
        .route("/state/account", post(handle_state_account))
        .route("/state/component", post(handle_state_component))
        .route("/state/validator", post(handle_state_validator))
        .route(
            "/state/access-controller",
            post(handle_state_access_controller),
        )
        .route("/state/package", post(handle_state_package))
        .route("/state/resource", post(handle_state_resource))
        .route("/state/non-fungible", post(handle_state_non_fungible))
        .with_state(core_api_state);

    let metrics = Arc::new(CoreApiMetrics::new(metric_registry));

    let prefixed_router = Router::new()
        .nest("/core", router)
        .route("/", get(handle_no_core_path))
        .layer(CatchPanicLayer::custom(InternalServerErrorResponseForPanic))
        // Note: it is important to run the metrics middleware only on router matched paths to avoid out of memory crash
        // of node or full storage for prometheus server.
        .route_layer(MetricsLayer::new(metrics.clone()))
        .layer(map_response(emit_error_response_event))
        .fallback(handle_not_found)
        .with_state(metrics);

    let bind_addr = bind_addr.parse().expect("Failed to parse bind address");

    axum::Server::bind(&bind_addr)
        .serve(prefixed_router.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}

#[tracing::instrument]
pub(crate) async fn handle_no_core_path() -> Result<(), ResponseError<()>> {
    Err(not_found_error("Try /core"))
}

async fn handle_not_found(metrics: State<Arc<CoreApiMetrics>>) -> Result<(), ResponseError<()>> {
    metrics.requests_not_found.inc();
    Err(not_found_error("Not found!"))
}

/// A function (to be used within a `map_response` layer) in order to emit more customized events
/// when top-level `ErrorResponse` is returned.
/// In short, it is supposed to replace an `err(Debug)` within `#[tracing::instrument(...)]` of
/// every handler function which returns `Result<_, ResponseError<_>>`. It emits almost the same
/// information (except for emitting the path instead of the handler function name), but is
/// capable of choosing the `Level` based on the HTTP status code (see `resolve_level()`).
async fn emit_error_response_event(uri: Uri, response: Response) -> Response {
    let error_response = response.extensions().get::<ErrorResponse>();
    if let Some(error_response) = error_response {
        let level = resolve_level(response.status());
        // the `event!(level, ...)` macro does not accept non-constant levels, hence we unroll:
        match level {
            Level::TRACE => trace!(path = uri.path(), error = debug(error_response)),
            Level::DEBUG => debug!(path = uri.path(), error = debug(error_response)),
            Level::INFO => info!(path = uri.path(), error = debug(error_response)),
            Level::WARN => warn!(path = uri.path(), error = debug(error_response)),
            Level::ERROR => error!(path = uri.path(), error = debug(error_response)),
        }
    }
    response
}

fn resolve_level(status_code: StatusCode) -> Level {
    if status_code.is_server_error() {
        Level::WARN
    } else {
        Level::DEBUG
    }
}

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct CoreApiServerConfig {
    pub bind_interface: String,
    pub port: u32,
}
