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
use crate::transaction::UserTransactionValidator;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::{
    scrypto, Categorize, ComponentAddress, Decimal, Decode, Encode, RADIX_TOKEN,
};
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use std::collections::HashSet;

use crate::jni::utils::*;
use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::{CommitError, NextEpoch, PrepareGenesisRequest, PrepareGenesisResult};

use super::state_manager::ActualStateManager;

//
// JNI Interface
//

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_verify(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, request_payload, do_verify)
}

fn do_verify(state_manager: &ActualStateManager, args: JavaRawTransaction) -> Result<(), String> {
    let transaction = args;

    let parsed = UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
        &transaction.payload,
    )
    .map_err(|err| format!("{:?}", err))?;

    let _static_validation = state_manager
        .user_transaction_validator
        .validate_and_create_executable(&parsed, transaction.payload.len())
        .map_err(|err| format!("{:?}", err))?;

    Ok(())
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_saveVertexStore(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_save_vertex_store)
}

#[tracing::instrument(skip_all)]
fn do_save_vertex_store(state_manager: &mut ActualStateManager, args: Vec<u8>) {
    let vertex_store_bytes: Vec<u8> = args;
    state_manager.save_vertex_store(vertex_store_bytes);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_prepareGenesis(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_prepare_genesis)
}

#[tracing::instrument(skip_all)]
fn do_prepare_genesis(
    state_manager: &mut ActualStateManager,
    args: JavaPrepareGenesisRequest,
) -> JavaPrepareGenesisResult {
    let prepare_request = args;

    let result = state_manager.prepare_genesis(prepare_request.into());

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

    state_manager.commit(commit_request.into())
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_componentXrdAmount(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, request_payload, get_component_xrd)
}

fn get_component_xrd(state_manager: &ActualStateManager, args: ComponentAddress) -> Decimal {
    let component_address = args;
    let resources = state_manager.get_component_resources(component_address);

    resources
        .map(|r| r.get(&RADIX_TOKEN).cloned().unwrap_or_else(Decimal::zero))
        .unwrap_or_else(Decimal::zero)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_epoch(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, request_payload, do_get_epoch)
}

fn do_get_epoch(state_manager: &ActualStateManager, _args: ()) -> u64 {
    state_manager.get_epoch()
}

pub fn export_extern_functions() {}

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaCommitRequest {
    pub transactions: Vec<JavaRawTransaction>,
    pub state_version: u64,
    pub proof: Vec<u8>,
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
            proof_state_version: commit_request.state_version,
            proof: commit_request.proof,
            vertex_store: commit_request.vertex_store,
        }
    }
}

#[derive(Debug, Decode, Encode, Categorize)]
pub struct JavaPrepareRequest {
    pub already_prepared: Vec<JavaRawTransaction>,
    pub proposed: Vec<JavaRawTransaction>,
    pub consensus_epoch: u64,
    pub round_number: u64,
    pub proposer_timestamp_ms: i64,
}

impl From<JavaPrepareRequest> for PrepareRequest {
    fn from(prepare_request: JavaPrepareRequest) -> Self {
        PrepareRequest {
            already_prepared_payloads: prepare_request
                .already_prepared
                .into_iter()
                .map(|t| t.payload)
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

#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct JavaPrepareResult {
    pub committed: Vec<Vec<u8>>,
    pub rejected: Vec<(Vec<u8>, String)>,
    pub next_epoch: Option<NextEpoch>,
}

impl From<PrepareResult> for JavaPrepareResult {
    fn from(prepare_results: PrepareResult) -> Self {
        JavaPrepareResult {
            committed: prepare_results.committed,
            rejected: prepare_results.rejected,
            next_epoch: prepare_results.next_epoch,
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

#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct JavaPrepareGenesisResult {
    pub validator_set: Option<HashSet<EcdsaSecp256k1PublicKey>>,
}

impl From<PrepareGenesisResult> for JavaPrepareGenesisResult {
    fn from(prepare_results: PrepareGenesisResult) -> Self {
        JavaPrepareGenesisResult {
            validator_set: prepare_results.validator_set,
        }
    }
}
