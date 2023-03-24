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
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use parking_lot::RwLock;
use radix_engine::types::{Categorize, Decode, Encode};
use state_manager::jni::state_manager::ActualStateManager;

use super::{constants::LARGE_REQUEST_MAX_BYTES, handlers::*, not_found_error, ResponseError};

use handle_status_network_configuration as handle_provide_info_at_root_path;

#[derive(Clone)]
pub(crate) struct CoreApiState {
    pub state_manager: Arc<RwLock<ActualStateManager>>,
}

pub async fn create_server<F>(
    bind_addr: &str,
    shutdown_signal: F,
    state_manager: Arc<RwLock<ActualStateManager>>,
) where
    F: Future<Output = ()>,
{
    let core_api_state = CoreApiState { state_manager };

    let router = Router::new()
        // This only adds a route for /core, /core/ doesn't seem possible using /nest
        .route("/", get(handle_provide_info_at_root_path))
        // Release Candidate backward compatible Sub-API
        .route(
            "/lts/transaction/construction",
            post(lts::handle_rc_transaction_construction),
        )
        .route(
            "/lts/transaction/status",
            post(lts::handle_rc_transaction_status),
        )
        .route(
            "/lts/transaction/submit",
            post(lts::handle_rc_transaction_submit),
        )
        .route(
            "/lts/stream/transactions-basic-outcomes",
            post(lts::handle_rc_stream_transactions_basic_outcomes),
        )
        .route(
            "/lts/state/account-all-fungible-resource-balances",
            post(lts::handle_rc_state_account_all_fungible_resource_balances),
        )
        .route(
            "/lts/state/account-fungible-resource-balance",
            post(lts::handle_rc_state_account_fungible_resource_balance),
        )
        // Status Sub-API
        .route(
            "/status/network-configuration",
            post(handle_status_network_configuration),
        )
        .route("/status/network-status", post(handle_status_network_status))
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
        .route("/state/epoch", post(handle_state_epoch))
        .route("/state/clock", post(handle_state_clock))
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
pub(crate) async fn handle_no_core_path() -> Result<(), ResponseError<()>> {
    Err(not_found_error("Try /core"))
}

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct CoreApiServerConfig {
    pub bind_interface: String,
    pub port: u32,
}
