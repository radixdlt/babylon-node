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

use crate::jni::mempool::JavaRawTransaction;

use crate::{
    AccumulatorHash, AccumulatorState, LedgerHashes, LedgerHeader, LedgerProof, ReceiptTreeHash,
    StateHash, TimestampedValidatorSignature, TransactionTreeHash,
};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::*;
use radix_engine_queries::query::ResourceAccounter;
use std::ops::Deref;

use crate::jni::state_manager::JNIStateManager;
use crate::query::StateManagerSubstateQueries;
use node_common::java::*;

use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::{CommitError, NextEpoch};
use radix_engine::blueprints::epoch_manager::ValidatorSubstate;
use radix_engine::system::bootstrap::GenesisDataChunk;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;

use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};

//
// JNI Interface
//

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_executeGenesis(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |genesis_data: JavaGenesisData| -> JavaLedgerProof {
            let state_manager = JNIStateManager::get_state_manager(&env, j_state_manager);
            let result = state_manager.execute_genesis(
                genesis_data.chunks,
                genesis_data.initial_epoch,
                genesis_data.max_validators,
                genesis_data.rounds_per_epoch,
                genesis_data.num_unstake_epochs,
            );
            result.into()
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_prepare(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |prepare_request: JavaPrepareRequest| -> JavaPrepareResult {
            let state_manager = JNIStateManager::get_state_manager(&env, j_state_manager);
            let result = state_manager.prepare(prepare_request.into());
            result.into()
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_commit(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |commit_request: JavaCommitRequest| -> Result<(), CommitError> {
            let state_manager = JNIStateManager::get_state_manager(&env, j_state_manager);
            state_manager
                .commit(commit_request.into(), false)
                .map(|_unused| ())
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_componentXrdAmount(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |component_address: ComponentAddress| -> Decimal {
            let node_id = component_address.as_node_id();
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let read_store = database.read();

            // a quick fix for handling virtual accounts
            // TODO: fix upstream
            if read_store
                .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                    node_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .is_some()
            {
                let mut accounter = ResourceAccounter::new(read_store.deref());
                accounter.traverse(*node_id);
                let balances = accounter.close().balances;
                balances
                    .get(&RADIX_TOKEN)
                    .cloned()
                    .unwrap_or_else(Decimal::zero)
            } else {
                Decimal::zero()
            }
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_validatorInfo(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |validator_address: ComponentAddress| -> JavaValidatorInfo {
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let read_store = database.read();
            let validator_substate: ValidatorSubstate = read_store
                .get_mapped::<SpreadPrefixKeyMapper, ValidatorSubstate>(
                    validator_address.as_node_id(),
                    OBJECT_BASE_PARTITION,
                    &ValidatorField::Validator.into(),
                )
                .unwrap();

            JavaValidatorInfo {
                lp_token_address: validator_substate.liquidity_token,
                unstake_resource: validator_substate.unstake_nft,
            }
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_epoch(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> u64 {
        let database = JNIStateManager::get_database(&env, j_state_manager);
        let read_store = database.read();
        read_store.get_epoch()
    })
}

pub fn export_extern_functions() {}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaGenesisData {
    pub chunks: Vec<GenesisDataChunk>,
    pub initial_epoch: u64,
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaCommitRequest {
    pub transactions: Vec<JavaRawTransaction>,
    pub proof: JavaLedgerProof,
    pub vertex_store: Option<Vec<u8>>,
}

impl From<JavaCommitRequest> for CommitRequest {
    fn from(commit_request: JavaCommitRequest) -> Self {
        CommitRequest {
            transaction_payloads: commit_request
                .transactions
                .into_iter()
                .map(|t| t.payload)
                .collect(),
            proof: commit_request.proof.into(),
            vertex_store: commit_request.vertex_store,
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaPrepareRequest {
    pub committed_accumulator_state: JavaAccumulatorState,
    pub prepared_uncommitted_transactions: Vec<JavaRawTransaction>,
    pub prepared_uncommitted_accumulator_state: JavaAccumulatorState,
    pub proposed_transactions: Vec<JavaRawTransaction>,
    pub is_fallback: bool,
    pub epoch: u64,
    pub round: u64,
    pub gap_round_leader_addresses: Vec<ComponentAddress>,
    pub proposer_address: ComponentAddress,
    pub proposer_timestamp_ms: i64,
}

impl From<JavaPrepareRequest> for PrepareRequest {
    fn from(prepare_request: JavaPrepareRequest) -> Self {
        PrepareRequest {
            committed_accumulator_state: prepare_request.committed_accumulator_state.into(),
            prepared_uncommitted_payloads: prepare_request
                .prepared_uncommitted_transactions
                .into_iter()
                .map(|t| t.payload)
                .collect(),
            prepared_uncommitted_accumulator_state: prepare_request
                .prepared_uncommitted_accumulator_state
                .into(),
            proposed_payloads: prepare_request
                .proposed_transactions
                .into_iter()
                .map(|t| t.payload)
                .collect(),
            is_fallback: prepare_request.is_fallback,
            epoch: prepare_request.epoch,
            round: prepare_request.round,
            gap_round_leader_addresses: prepare_request.gap_round_leader_addresses,
            proposer_address: prepare_request.proposer_address,
            proposer_timestamp_ms: prepare_request.proposer_timestamp_ms,
        }
    }
}

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaLedgerHashes {
    pub state_root: JavaHashCode,
    pub transaction_root: JavaHashCode,
    pub receipt_root: JavaHashCode,
}

impl From<LedgerHashes> for JavaLedgerHashes {
    fn from(ledger_hashes: LedgerHashes) -> Self {
        Self {
            state_root: JavaHashCode::from_bytes(ledger_hashes.state_root.into_bytes()),
            transaction_root: JavaHashCode::from_bytes(ledger_hashes.transaction_root.into_bytes()),
            receipt_root: JavaHashCode::from_bytes(ledger_hashes.receipt_root.into_bytes()),
        }
    }
}

impl From<JavaLedgerHashes> for LedgerHashes {
    fn from(ledger_hashes: JavaLedgerHashes) -> Self {
        Self {
            state_root: StateHash::from_raw_bytes(ledger_hashes.state_root.into_bytes()),
            transaction_root: TransactionTreeHash::from_raw_bytes(
                ledger_hashes.transaction_root.into_bytes(),
            ),
            receipt_root: ReceiptTreeHash::from_raw_bytes(ledger_hashes.receipt_root.into_bytes()),
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaPrepareResult {
    pub committed: Vec<Vec<u8>>,
    pub rejected: Vec<(Vec<u8>, String)>,
    pub next_epoch: Option<NextEpoch>,
    pub ledger_hashes: JavaLedgerHashes,
    pub accumulator_state: JavaAccumulatorState,
}

impl From<PrepareResult> for JavaPrepareResult {
    fn from(prepare_result: PrepareResult) -> Self {
        JavaPrepareResult {
            committed: prepare_result.committed_payloads,
            rejected: prepare_result.rejected_payloads,
            next_epoch: prepare_result.next_epoch,
            ledger_hashes: prepare_result.ledger_hashes.into(),
            accumulator_state: prepare_result.accumulator_state.into(),
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaValidatorInfo {
    pub lp_token_address: ResourceAddress,
    pub unstake_resource: ResourceAddress,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaLedgerProof {
    pub opaque: JavaHashCode,
    pub ledger_header: JavaLedgerHeader,
    pub timestamped_signatures: Vec<TimestampedValidatorSignature>,
}

impl From<LedgerProof> for JavaLedgerProof {
    fn from(ledger_proof: LedgerProof) -> Self {
        Self {
            opaque: JavaHashCode::from_bytes(ledger_proof.opaque.0),
            ledger_header: ledger_proof.ledger_header.into(),
            timestamped_signatures: ledger_proof.timestamped_signatures,
        }
    }
}

impl From<JavaLedgerProof> for LedgerProof {
    fn from(ledger_proof: JavaLedgerProof) -> Self {
        Self {
            opaque: Hash(ledger_proof.opaque.into_bytes()),
            ledger_header: ledger_proof.ledger_header.into(),
            timestamped_signatures: ledger_proof.timestamped_signatures,
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaLedgerHeader {
    pub epoch: u64,
    pub round: u64,
    pub accumulator_state: JavaAccumulatorState,
    pub hashes: JavaLedgerHashes,
    pub consensus_parent_round_timestamp_ms: i64,
    pub proposer_timestamp_ms: i64,
    pub next_epoch: Option<NextEpoch>,
}

impl From<LedgerHeader> for JavaLedgerHeader {
    fn from(ledger_header: LedgerHeader) -> Self {
        Self {
            epoch: ledger_header.epoch,
            round: ledger_header.round,
            accumulator_state: ledger_header.accumulator_state.into(),
            hashes: ledger_header.hashes.into(),
            consensus_parent_round_timestamp_ms: ledger_header.consensus_parent_round_timestamp_ms,
            proposer_timestamp_ms: ledger_header.proposer_timestamp_ms,
            next_epoch: ledger_header.next_epoch,
        }
    }
}

impl From<JavaLedgerHeader> for LedgerHeader {
    fn from(ledger_header: JavaLedgerHeader) -> Self {
        Self {
            epoch: ledger_header.epoch,
            round: ledger_header.round,
            accumulator_state: ledger_header.accumulator_state.into(),
            hashes: ledger_header.hashes.into(),
            consensus_parent_round_timestamp_ms: ledger_header.consensus_parent_round_timestamp_ms,
            proposer_timestamp_ms: ledger_header.proposer_timestamp_ms,
            next_epoch: ledger_header.next_epoch,
        }
    }
}

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaAccumulatorState {
    pub state_version: u64,
    pub accumulator_hash: JavaHashCode,
}

impl From<JavaAccumulatorState> for AccumulatorState {
    fn from(accumulator_state: JavaAccumulatorState) -> Self {
        Self {
            state_version: accumulator_state.state_version,
            accumulator_hash: AccumulatorHash::from_raw_bytes(
                accumulator_state.accumulator_hash.into_bytes(),
            ),
        }
    }
}

impl From<AccumulatorState> for JavaAccumulatorState {
    fn from(accumulator_state: AccumulatorState) -> Self {
        Self {
            state_version: accumulator_state.state_version,
            accumulator_hash: JavaHashCode::from_bytes(
                accumulator_state.accumulator_hash.into_bytes(),
            ),
        }
    }
}
