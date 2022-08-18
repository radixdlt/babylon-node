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

use crate::core_api::generated::models;
use crate::core_api::generated::models::{
    Bech32Hrps, NetworkConfigurationResponse, NetworkConfigurationResponseVersion,
    NetworkIdentifier,
};
use crate::core_api::generated::server::MakeService;
use crate::core_api::generated::{
    Api, StatusNetworkConfigurationPostResponse, StatusNetworkSyncPostResponse,
    TransactionPreviewPostResponse, TransactionSubmitPostResponse, API_VERSION,
};

use crate::state_manager::StateManager;
use async_trait::async_trait;

use scrypto::address::get_network_hrp_set;

use std::future::Future;
use std::marker::PhantomData;

use std::sync::{Arc, Mutex};

use swagger::ApiError;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};

pub async fn create<F>(
    bind_addr: &str,
    shutdown_signal: F,
    state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>,
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

#[derive(Clone)]
pub struct Server<C> {
    state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>,
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new(state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>) -> Self {
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
        _network_configuration_request: models::NetworkConfigurationRequest,
        _context: &C,
    ) -> Result<StatusNetworkConfigurationPostResponse, ApiError> {
        let network = &self
            .state_manager
            .lock()
            .expect("Can't acquire state manager lock")
            .network()
            .clone();

        let hrp_set = get_network_hrp_set(network);

        Ok(
            StatusNetworkConfigurationPostResponse::NetworkConfiguration(
                NetworkConfigurationResponse {
                    version: NetworkConfigurationResponseVersion {
                        core_version: env!("CARGO_PKG_VERSION").to_string(),
                        api_version: API_VERSION.to_string(),
                    },
                    network_identifier: NetworkIdentifier {
                        network: format!("{:?}", network),
                    },
                    bech32_human_readable_parts: Bech32Hrps {
                        account_hrp: hrp_set.account_component.to_string(),
                        validator_hrp: "TODO".to_string(),
                        node_hrp: "TODO".to_string(),
                        resource_hrp_suffix: hrp_set.resource.to_string(),
                    },
                },
            ),
        )
    }

    async fn status_network_sync_post(
        &self,
        _network_sync_status_request: models::NetworkSyncStatusRequest,
        _context: &C,
    ) -> Result<StatusNetworkSyncPostResponse, ApiError> {
        Err("To be implemented".into())
    }

    async fn transaction_preview_post(
        &self,
        _transaction_preview_request: models::TransactionPreviewRequest,
        _context: &C,
    ) -> Result<TransactionPreviewPostResponse, ApiError> {
        Err("To be implemented".into())
    }

    async fn transaction_submit_post(
        &self,
        _transaction_submit_request: models::TransactionSubmitRequest,
        _context: &C,
    ) -> Result<TransactionSubmitPostResponse, ApiError> {
        Err("To be implemented".into())
    }
}
