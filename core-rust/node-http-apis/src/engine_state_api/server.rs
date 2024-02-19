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

use super::metrics::EngineStateApiMetrics;
use super::metrics_layer::MetricsLayer;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::map_response;

use axum::{
    routing::{get, post},
    Router,
};

use crate::engine_prelude::*;
use prometheus::Registry;
use state_manager::StateManager;
use tower_http::catch_panic::CatchPanicLayer;

use super::{handlers::*, ResponseError};

use crate::engine_state_api::{emit_error_response_event, InternalServerErrorResponseForPanic};

#[derive(Clone)]
pub struct EngineStateApiState {
    pub network: NetworkDefinition,
    pub state_manager: StateManager,
}

pub async fn create_server<F>(
    bind_addr: &str,
    shutdown_signal: F,
    engine_state_api_state: EngineStateApiState,
    metric_registry: &Registry,
) where
    F: Future<Output = ()>,
{
    let router = Router::new()
        .route("/blueprint/info", post(handle_blueprint_info))
        .route("/entity/iterator", post(handle_entity_iterator))
        .route("/entity/info", post(handle_entity_info))
        .route("/object/field", post(handle_object_field))
        .route(
            "/object/collection/iterator",
            post(handle_object_collection_iterator),
        )
        .route(
            "/object/collection/entry",
            post(handle_object_collection_entry),
        )
        .route(
            "/object/attached-modules/metadata/iterator",
            post(handle_object_metadata_iterator),
        )
        .route(
            "/object/attached-modules/metadata/entry",
            post(handle_object_metadata_entry),
        )
        .route(
            "/object/attached-modules/role-assignment",
            post(handle_object_role_assignment),
        )
        .route(
            "/object/attached-modules/royalty",
            post(handle_object_royalty),
        )
        .route("/kv-store/iterator", post(handle_kv_store_iterator))
        .route("/kv-store/entry", post(handle_kv_store_entry))
        .route("/entity/schema/entry", post(handle_entity_schema_entry))
        .with_state(engine_state_api_state);

    let metrics = Arc::new(EngineStateApiMetrics::new(metric_registry));

    let prefixed_router = Router::new()
        .nest("/engine-state", router)
        .route("/", get(handle_missing_engine_state_path))
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
pub(crate) async fn handle_missing_engine_state_path() -> Result<(), ResponseError> {
    Err(ResponseError::new(
        StatusCode::NOT_FOUND,
        "Try /engine-state",
    ))
}

async fn handle_not_found(metrics: State<Arc<EngineStateApiMetrics>>) -> Result<(), ResponseError> {
    metrics.requests_not_found.inc();
    Err(ResponseError::new(
        StatusCode::NOT_FOUND,
        "Please see API docs for available endpoints",
    ))
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct EngineStateApiServerConfig {
    pub bind_interface: String,
    pub port: u32,
}
