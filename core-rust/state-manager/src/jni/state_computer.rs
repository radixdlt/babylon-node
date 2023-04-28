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
    AccumulatorHash, AccumulatorState, ActiveValidatorInfo, LedgerHashes, LedgerHeader,
    LedgerProof, PreviousVertex, ReceiptTreeHash, StateHash, TimestampedValidatorSignature,
    TransactionTreeHash,
};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::*;
use radix_engine_queries::query::ResourceAccounter;
use std::ops::Deref;

use crate::jni::common_types::JavaHashCode;
use crate::jni::state_manager::JNIStateManager;
use crate::jni::utils::*;
use crate::query::StateManagerSubstateQueries;
use crate::store::traits::QueryableTransactionStore;
use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::{CommitError, NextEpoch, PrepareGenesisRequest, PrepareGenesisResult};
use radix_engine::blueprints::epoch_manager::ValidatorSubstate;
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtMapper;

use super::state_manager::ActualStateManager;

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
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_execute_genesis)
}

#[tracing::instrument(skip_all)]
fn do_execute_genesis(
    state_manager: &mut ActualStateManager,
    args: JavaGenesisData,
) -> JavaLedgerProof {
    let genesis_data = args;

    let result = state_manager.execute_genesis(
        vec![], /* TODO */
        genesis_data.initial_epoch,
        genesis_data.max_validators,
        genesis_data.rounds_per_epoch,
        genesis_data.num_unstake_epochs,
    );

    result.into()
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_prepare(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_prepare)
}

#[tracing::instrument(skip_all)]
fn do_prepare(
    state_manager: &mut ActualStateManager,
    args: JavaPrepareRequest,
) -> JavaPrepareResult {
    let prepare_request = args;

    let result = state_manager.prepare(prepare_request.into());

    result.into()
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_commit(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_commit)
}

#[tracing::instrument(skip_all)]
fn do_commit(
    state_manager: &mut ActualStateManager,
    args: JavaCommitRequest,
) -> Result<(), CommitError> {
    let commit_request = args;

    state_manager
        .commit(commit_request.into())
        .map(|_unused| ())
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
            let mut accounter = ResourceAccounter::new(read_store.deref());
            accounter.traverse(*node_id);
            let balances = accounter.close().balances;
            balances
                .get(&RADIX_TOKEN)
                .cloned()
                .unwrap_or_else(Decimal::zero)
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_faucetAddress(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |()| -> ComponentAddress {
        // TODO: this won't work with the current genesis
        // update radix engine genesis so that faucet is created in the
        // system bootstrap txn (rather than the wrap up txn)
        let database = JNIStateManager::get_database(&env, j_state_manager);
        let read_store = database.read();
        let system_bootstrap_receipt = read_store.get_committed_transaction_receipt(1).unwrap();
        *system_bootstrap_receipt
            .local_execution
            .state_update_summary
            .new_components
            .last()
            .unwrap()
    })
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
                .get_mapped_substate::<JmtMapper, ValidatorSubstate>(
                    validator_address.as_node_id(),
                    SysModuleId::Object.into(),
                    &ValidatorOffset::Validator.into(),
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
pub struct JavaGenesisDataChunk {}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaGenesisData {
    pub chunks: Vec<JavaGenesisDataChunk>,
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

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaPrepareRequest {
    pub parent_accumulator_hash: JavaHashCode,
    pub previous_vertices: Vec<JavaPreviousVertex>,
    pub proposed: Vec<JavaRawTransaction>,
    pub consensus_epoch: u64,
    pub round_number: u64,
    pub proposer_timestamp_ms: i64,
}

impl From<JavaPrepareRequest> for PrepareRequest {
    fn from(prepare_request: JavaPrepareRequest) -> Self {
        PrepareRequest {
            parent_accumulator: prepare_request.parent_accumulator_hash.into(),
            prepared_vertices: prepare_request
                .previous_vertices
                .into_iter()
                .map(|t| t.into())
                .collect(),
            proposed_payloads: prepare_request
                .proposed
                .into_iter()
                .map(|t| t.payload)
                .collect(),
            consensus_epoch: prepare_request.consensus_epoch,
            round_number: prepare_request.round_number,
            proposer_timestamp_ms: prepare_request.proposer_timestamp_ms,
        }
    }
}

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaPreviousVertex {
    pub transactions: Vec<JavaRawTransaction>,
    pub resultant_accumulator_hash: JavaHashCode,
}

impl From<JavaPreviousVertex> for PreviousVertex {
    fn from(previous_vertex: JavaPreviousVertex) -> Self {
        PreviousVertex {
            transaction_payloads: previous_vertex
                .transactions
                .into_iter()
                .map(|v| v.payload)
                .collect(),
            resultant_accumulator: previous_vertex.resultant_accumulator_hash.into(),
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
}

impl From<PrepareResult> for JavaPrepareResult {
    fn from(prepare_results: PrepareResult) -> Self {
        JavaPrepareResult {
            committed: prepare_results.committed,
            rejected: prepare_results.rejected,
            next_epoch: prepare_results.next_epoch,
            ledger_hashes: prepare_results.ledger_hashes.into(),
        }
    }
}

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaPrepareGenesisRequest {
    pub genesis: JavaRawTransaction,
}

impl From<JavaPrepareGenesisRequest> for PrepareGenesisRequest {
    fn from(prepare_genesis_request: JavaPrepareGenesisRequest) -> Self {
        PrepareGenesisRequest {
            genesis: prepare_genesis_request.genesis.payload,
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaPrepareGenesisResult {
    pub validator_set: Option<Vec<ActiveValidatorInfo>>,
    pub ledger_hashes: JavaLedgerHashes,
}

impl From<PrepareGenesisResult> for JavaPrepareGenesisResult {
    fn from(prepare_result: PrepareGenesisResult) -> Self {
        JavaPrepareGenesisResult {
            validator_set: prepare_result.validator_set,
            ledger_hashes: prepare_result.ledger_hashes.into(),
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
