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

use crate::transaction::RawLedgerTransaction;
use crate::{
    AccumulatorHash, AccumulatorState, LedgerHashes, LedgerHeader, LedgerProof, PreviousVertex,
    RejectedTransaction, TimestampedValidatorSignature,
};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, EpochChangeCondition,
};
use radix_engine_queries::query::ResourceAccounter;
use std::ops::Deref;
use transaction::model::*;

use crate::jni::state_manager::JNIStateManager;
use crate::query::StateManagerSubstateQueries;
use node_common::java::*;

use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::{CommitError, NextEpoch};
use radix_engine::blueprints::consensus_manager::ValidatorSubstate;
use radix_engine::system::bootstrap::GenesisDataChunk;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;

use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};

//
// JNI Interface
//

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaGenesisData {
    pub initial_epoch: Epoch,
    pub initial_config: JavaConsensusManagerConfig,
    pub initial_timestamp_ms: i64,
    pub chunks: Vec<GenesisDataChunk>,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaConsensusManagerConfig {
    pub max_validators: u32,
    pub epoch_min_round_count: u64,
    pub epoch_max_round_count: u64,
    pub epoch_target_duration_millis: u64,
    pub num_unstake_epochs: u64,
    pub total_emission_xrd_per_epoch: Decimal,
    pub min_validator_reliability: Decimal,
    pub num_owner_stake_units_unlock_epochs: u64,
    pub num_fee_increase_delay_epochs: u64,
}

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
            let config = genesis_data.initial_config;
            let result = state_manager.execute_genesis(
                genesis_data.chunks,
                genesis_data.initial_epoch,
                ConsensusManagerConfig {
                    max_validators: config.max_validators,
                    epoch_change_condition: EpochChangeCondition {
                        min_round_count: config.epoch_min_round_count,
                        max_round_count: config.epoch_max_round_count,
                        target_duration_millis: config.epoch_target_duration_millis,
                    },
                    num_unstake_epochs: config.num_unstake_epochs,
                    total_emission_xrd_per_epoch: config.total_emission_xrd_per_epoch,
                    min_validator_reliability: config.min_validator_reliability,
                    num_owner_stake_units_unlock_epochs: config.num_owner_stake_units_unlock_epochs,
                    num_fee_increase_delay_epochs: config.num_fee_increase_delay_epochs,
                },
                genesis_data.initial_timestamp_ms,
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
                stake_unit_resource: validator_substate.stake_unit_resource,
                unstake_receipt_resource: validator_substate.unstake_nft,
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
        read_store.get_epoch().number()
    })
}

pub fn export_extern_functions() {}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaCommitRequest {
    pub transactions: Vec<RawLedgerTransaction>,
    pub proof: JavaLedgerProof,
    pub vertex_store: Option<Vec<u8>>,
}

impl From<JavaCommitRequest> for CommitRequest {
    fn from(commit_request: JavaCommitRequest) -> Self {
        CommitRequest {
            transaction_payloads: commit_request.transactions,
            proof: commit_request.proof.into(),
            vertex_store: commit_request.vertex_store,
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaPrepareRequest {
    pub parent_accumulator_hash: AccumulatorHash,
    pub previous_vertices: Vec<PreviousVertex>,
    pub proposed: Vec<RawNotarizedTransaction>,
    pub is_fallback: bool,
    pub epoch: Epoch,
    pub round: Round,
    pub gap_round_leader_addresses: Vec<ComponentAddress>,
    pub proposer_address: ComponentAddress,
    pub proposer_timestamp_ms: i64,
}

impl From<JavaPrepareRequest> for PrepareRequest {
    fn from(prepare_request: JavaPrepareRequest) -> Self {
        PrepareRequest {
            parent_accumulator: prepare_request.parent_accumulator_hash,
            prepared_vertices: prepare_request.previous_vertices,
            proposed_payloads: prepare_request.proposed,
            is_fallback: prepare_request.is_fallback,
            epoch: prepare_request.epoch,
            round: prepare_request.round,
            gap_round_leader_addresses: prepare_request.gap_round_leader_addresses,
            proposer_address: prepare_request.proposer_address,
            proposer_timestamp_ms: prepare_request.proposer_timestamp_ms,
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaPrepareResult {
    pub committed: Vec<RawLedgerTransaction>,
    pub rejected: Vec<RejectedTransaction>,
    pub next_epoch: Option<NextEpoch>,
    pub ledger_hashes: LedgerHashes,
}

impl From<PrepareResult> for JavaPrepareResult {
    fn from(prepare_results: PrepareResult) -> Self {
        JavaPrepareResult {
            committed: prepare_results.committed,
            rejected: prepare_results.rejected,
            next_epoch: prepare_results.next_epoch,
            ledger_hashes: prepare_results.ledger_hashes,
        }
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaValidatorInfo {
    pub stake_unit_resource: ResourceAddress,
    pub unstake_receipt_resource: ResourceAddress,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaLedgerProof {
    pub opaque: Hash,
    pub ledger_header: JavaLedgerHeader,
    pub timestamped_signatures: Vec<TimestampedValidatorSignature>,
}

impl From<LedgerProof> for JavaLedgerProof {
    fn from(ledger_proof: LedgerProof) -> Self {
        Self {
            opaque: ledger_proof.opaque,
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
    pub epoch: Epoch,
    pub round: Round,
    pub accumulator_state: AccumulatorState,
    pub hashes: LedgerHashes,
    pub consensus_parent_round_timestamp_ms: i64,
    pub proposer_timestamp_ms: i64,
    pub next_epoch: Option<NextEpoch>,
}

impl From<LedgerHeader> for JavaLedgerHeader {
    fn from(ledger_header: LedgerHeader) -> Self {
        Self {
            epoch: ledger_header.epoch,
            round: ledger_header.round,
            accumulator_state: ledger_header.accumulator_state,
            hashes: ledger_header.hashes,
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
            accumulator_state: ledger_header.accumulator_state,
            hashes: ledger_header.hashes,
            consensus_parent_round_timestamp_ms: ledger_header.consensus_parent_round_timestamp_ms,
            proposer_timestamp_ms: ledger_header.proposer_timestamp_ms,
            next_epoch: ledger_header.next_epoch,
        }
    }
}
