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

use crate::prelude::*;
use std::future::Future;
use std::sync::Arc;

use super::metrics::MeshApiMetrics;
use super::metrics_layer::MetricsLayer;
use axum::extract::State;
use axum::middleware::map_response;

use axum::{
    routing::{get, post},
    Router,
};

use prometheus::Registry;
use state_manager::state_manager::StateManager;
use tower_http::catch_panic::CatchPanicLayer;

use super::{handlers::*, ResponseError};

use crate::mesh_api::{emit_error_response_event, InternalServerErrorResponseForPanic};

#[derive(Clone)]
pub struct MeshApiState {
    pub network: NetworkDefinition,
    pub state_manager: StateManager,
}

pub async fn create_server<F>(
    bind_addr: &str,
    shutdown_signal: F,
    mesh_api_state: MeshApiState,
    metric_registry: &Registry,
) where
    F: Future<Output = ()>,
{
    let router = Router::new()
        .route("/network/status", post(handle_network_status))
        .route("/network/list", post(handle_network_list))
        .route("/network/options", post(handle_network_options))
        .route("/account/balance", post(handle_account_balance))
        // account/coins - not needed as we're not UTXO
        .route("/block", post(handle_endpoint_todo))
        .route("/block/transaction", post(handle_endpoint_todo))
        // TODO:MESH mempool
        .route("/mempool", post(handle_endpoint_todo))
        .route("/mempool/transaction", post(handle_endpoint_todo))
        .route("/construction/derive", post(handle_endpoint_todo))
        .route("/construction/preprocess", post(handle_endpoint_todo))
        .route("/construction/metadata", post(handle_endpoint_todo))
        .route("/construction/payloads", post(handle_endpoint_todo))
        .route("/construction/combine", post(handle_endpoint_todo))
        .route("/construction/hash", post(handle_endpoint_todo))
        .route("/construction/parse", post(handle_endpoint_todo))
        .route("/construction/submit", post(handle_endpoint_todo))
        // Below endpoints are optional
        .route("/call", post(handle_endpoint_not_supported))
        .route("/search/transaction", post(handle_endpoint_not_supported))
        .route("/events/blocks", post(handle_endpoint_not_supported))
        .with_state(mesh_api_state);

    let metrics = Arc::new(MeshApiMetrics::new(metric_registry));

    let prefixed_router = Router::new()
        .nest("/mesh", router)
        .route("/", get(handle_missing_mesh_path))
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

async fn handle_endpoint_not_supported(
    _state: State<MeshApiState>,
    Json(_request): Json<models::MetadataRequest>,
) -> Result<Json<()>, ResponseError> {
    Err(ResponseError::from(ApiError::EndpointNotSupported))
}

// TODO:MESH remove it when no longer needed
async fn handle_endpoint_todo(
    _state: State<MeshApiState>,
    Json(_request): Json<models::MetadataRequest>,
) -> Result<Json<()>, ResponseError> {
    todo!()
}

#[tracing::instrument]
pub(crate) async fn handle_missing_mesh_path() -> Result<(), ResponseError> {
    Err(ResponseError::from(ApiError::EndpointNotFound).with_details("Try /mesh"))
}

async fn handle_not_found(metrics: State<Arc<MeshApiMetrics>>) -> Result<(), ResponseError> {
    metrics.requests_not_found.inc();
    Err(ResponseError::from(ApiError::EndpointNotFound)
        .with_details("Please see API docs for available endpoints"))
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct MeshApiServerConfig {
    pub bind_interface: String,
    pub port: u32,
}
