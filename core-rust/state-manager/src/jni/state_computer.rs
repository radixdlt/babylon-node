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

use crate::{CommitSummary, LedgerProof, ProtocolState};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, EpochChangeCondition,
};

use node_common::java::*;

use crate::types::{CommitRequest, InvalidCommitRequestError, PrepareRequest, PrepareResult};

use radix_engine::system::bootstrap::GenesisDataChunk;

use super::node_rust_environment::JNINodeRustEnvironment;

//
// JNI Interface
//

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaGenesisData {
    pub initial_epoch: Epoch,
    pub initial_timestamp_ms: i64,
    pub initial_config: JavaConsensusManagerConfig,
    pub chunks: Vec<GenesisDataChunk>,
    pub faucet_supply: Decimal,
    pub scenarios_to_run: Vec<String>,
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
    pub validator_creation_usd_cost: Decimal,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_executeGenesis(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_fallible_call(
        &env,
        request_payload,
        |raw_genesis_data: Vec<u8>| -> JavaResult<LedgerProof> {
            let state_computer = JNINodeRustEnvironment::get_state_computer(&env, j_node_rust_env);
            let genesis_data_hash = hash(&raw_genesis_data);
            let genesis_data: JavaGenesisData = scrypto_decode(&raw_genesis_data)
                .map_err(|err| JavaError(format!("Invalid genesis data {:?}", err)))?;
            let config = genesis_data.initial_config;
            let resultant_proof = state_computer.execute_genesis(
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
                    validator_creation_usd_cost: config.validator_creation_usd_cost,
                },
                genesis_data.initial_timestamp_ms,
                genesis_data_hash,
                genesis_data.faucet_supply,
                genesis_data.scenarios_to_run,
            );
            Ok(resultant_proof)
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_prepare(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |prepare_request: PrepareRequest| -> PrepareResult {
            let state_computer = JNINodeRustEnvironment::get_state_computer(&env, j_node_rust_env);
            state_computer.prepare(prepare_request)
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_commit(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |commit_request: CommitRequest| -> Result<CommitSummary, InvalidCommitRequestError> {
            let state_computer = JNINodeRustEnvironment::get_state_computer(&env, j_node_rust_env);
            state_computer.commit(commit_request)
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_newestProtocolVersion(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> String {
        let env = JNINodeRustEnvironment::get(&env, j_node_rust_env);
        env.state_manager.newest_protocol_version()
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statecomputer_RustStateComputer_protocolState(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> ProtocolState {
        let env = JNINodeRustEnvironment::get(&env, j_node_rust_env);
        env.state_manager.state_computer.protocol_state()
    })
}

pub fn export_extern_functions() {}
