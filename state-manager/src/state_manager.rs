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

use crate::jni::dtos::*;
use crate::mempool::{Mempool, MempoolConfig};
use crate::transaction_store::TransactionStore;
use crate::types::{PreviewError, PreviewRequest, Transaction};
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::transaction::{PreviewExecutor, PreviewResult};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter};
use sbor::DecodeError;
use scrypto::buffer::scrypto_decode;
use scrypto::core::Network;
use scrypto::crypto::EcdsaPublicKey;
use std::sync::Arc;
use std::sync::Mutex;
use transaction::model::{
    NotarizedTransaction, PreviewFlags, PreviewIntent, TransactionHeader, TransactionIntent,
    TransactionManifest,
};
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::TestIntentHashManager;

#[derive(Clone, Debug)]
pub struct StateManager<M: Mempool> {
    pub network: Network,
    pub mempool: Arc<Mutex<M>>,
    pub transaction_store: Arc<Mutex<TransactionStore>>,
}

impl<M: Mempool> StateManager<M> {
    pub fn new(mempool: M, transaction_store: TransactionStore) -> StateManager<M> {
        StateManager {
            network: Network::LocalSimulator,
            mempool: Arc::new(Mutex::new(mempool)),
            transaction_store: Arc::new(Mutex::new(transaction_store)),
        }
    }

    pub fn decode_transaction(
        &self,
        txn: &Transaction,
    ) -> Result<NotarizedTransaction, DecodeError> {
        scrypto_decode(&txn.payload)
    }

    pub fn preview(&self, preview_request: &PreviewRequest) -> Result<PreviewResult, PreviewError> {
        let manifest: TransactionManifest =
            scrypto_decode(&preview_request.manifest).map_err(|_| PreviewError::InvalidManifest)?;

        let signer_public_keys: Result<Vec<EcdsaPublicKey>, PreviewError> = preview_request
            .signer_public_keys
            .iter()
            .map(|pk| {
                EcdsaPublicKey::try_from(&pk[..]).map_err(|_| PreviewError::InvalidSignerPublicKey)
            })
            .collect();

        // not really used for preview
        let notary_private_key = EcdsaPrivateKey::from_u64(2).unwrap();

        let preview_intent = PreviewIntent {
            intent: TransactionIntent {
                header: TransactionHeader {
                    version: 1,
                    network: self.network.clone(),
                    start_epoch_inclusive: 0,
                    end_epoch_exclusive: 100,
                    nonce: preview_request.nonce,
                    notary_public_key: notary_private_key.public_key(),
                    notary_as_signatory: false,
                    cost_unit_limit: preview_request.cost_unit_limit,
                    tip_percentage: preview_request.tip_percentage,
                },
                manifest,
            },
            signer_public_keys: signer_public_keys?,
            flags: PreviewFlags {
                unlimited_loan: preview_request.flags.unlimited_loan,
            },
        };

        let mut substate_store = InMemorySubstateStore::with_bootstrap();
        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let intent_hash_manager = TestIntentHashManager::new();

        let result = PreviewExecutor::new(
            &mut substate_store,
            &mut wasm_engine,
            &mut wasm_instrumenter,
            &intent_hash_manager,
        )
        .execute(preview_intent)
        .map_err(PreviewError::EngineError)?;

        Ok(result)
    }
}

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct StateManagerConfig {
    pub mempool_config: Option<MempoolConfig>,
}

impl JavaStructure for StateManagerConfig {}
