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

use crate::core_api::generated::server::MakeService;
use crate::core_api::generated::{models, TransactionsPostResponse};
use crate::core_api::generated::{
    Api, StatusNetworkConfigurationPostResponse, TransactionPreviewPostResponse,
    TransactionSubmitPostResponse,
};

use async_trait::async_trait;

use std::future::Future;
use std::marker::PhantomData;

use std::sync::{Arc, Mutex};

use crate::core_api::generated::models::CommittedTransactionsRequest;
use crate::core_api::{network_configuration, preview, transactions};
use scrypto::prelude::*;
use state_manager::jni::state_manager::ActualStateManager;
use swagger::ApiError;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};

pub async fn create<F>(
    bind_addr: &str,
    shutdown_signal: F,
    state_manager: Arc<Mutex<ActualStateManager>>,
) where
    F: Future<Output = ()>,
{
    let server = Server::new(state_manager);

    let service = MakeService::new(server);
    let service =
        crate::core_api::generated::context::MakeAddContext::<_, EmptyContext>::new(service);

    let bind_addr = bind_addr.parse().expect("Failed to parse bind address");
    hyper::server::Server::bind(&bind_addr)
        .serve(service)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}

pub struct Server<C> {
    state_manager: Arc<Mutex<ActualStateManager>>,
    marker: PhantomData<C>,
}

/// Keeping this for the future when, at some point we'll potentially replace ActualStateManager with a generic type.
/// The `derive` macro doesn't always work reliably in such cases.
/// See: https://users.rust-lang.org/t/why-does-deriving-clone-not-work-in-this-case-but-implementing-manually-does/29075
impl<C> Clone for Server<C> {
    fn clone(&self) -> Self {
        Server {
            state_manager: self.state_manager.clone(),
            marker: self.marker,
        }
    }
}

impl<C> Server<C> {
    pub fn new(state_manager: Arc<Mutex<ActualStateManager>>) -> Self {
        Server {
            state_manager,
            marker: PhantomData,
        }
    }
}

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    async fn status_network_configuration_post(
        &self,
        _context: &C,
    ) -> Result<StatusNetworkConfigurationPostResponse, ApiError> {
        network_configuration::handle_network_configuration(self.state_manager.clone())
    }

    async fn transaction_preview_post(
        &self,
        request: models::TransactionPreviewRequest,
        _context: &C,
    ) -> Result<TransactionPreviewPostResponse, ApiError> {
        preview::handle_preview(self.state_manager.clone(), request)
    }

    async fn transaction_submit_post(
        &self,
        request: models::TransactionSubmitRequest,
        _context: &C,
    ) -> Result<TransactionSubmitPostResponse, ApiError> {
        transactions::handle_submit_transaction(self.state_manager.clone(), request)
    }

    async fn transactions_post(
        &self,
        request: CommittedTransactionsRequest,
        _context: &C,
    ) -> Result<TransactionsPostResponse, ApiError> {
        transactions::handle_transactions(self.state_manager.clone(), request)
    }
}

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct CoreApiServerConfig {
    pub bind_interface: String,
    pub port: u32,
}
