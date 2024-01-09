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

use prometheus::Registry;
use std::sync::Arc;

use crate::traits::QueryableProofStore;
use radix_engine::blueprints::consensus_manager::{
    ConsensusManagerConfigSubstate, ConsensusManagerConfigurationFieldPayload,
    ConsensusManagerField, VersionedConsensusManagerConfiguration,
};
use radix_engine::system::system_substates::{FieldSubstate, FieldSubstateV1, LockStatus};
use radix_engine::track::StateUpdates;
use radix_engine_common::crypto::Hash;
use radix_engine_common::network::NetworkDefinition;
use radix_engine_common::prelude::{scrypto_encode, Decimal, Epoch, CONSENSUS_MANAGER};
use radix_engine_common::types::Round;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, EpochChangeCondition,
};
use radix_engine_interface::prelude::MAIN_BASE_PARTITION;
use radix_engine_store_interface::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};

use radix_engine_store_interface::interface::DatabaseUpdate;
use sbor::HasLatestVersion;

use node_common::locks::{LockFactory, StateLock};
use node_common::scheduler::Scheduler;

use crate::ProtocolUpdateEnactmentCondition::EnactUnconditionallyAtStateVersion;
use crate::{
    CommitRequest, CommitSummary, FlashProtocolUpdater, LedgerHeader, LedgerProof,
    LedgerProofOrigin, NoStateUpdatesProtocolUpdater, PrepareRequest, PrepareResult,
    ProtocolConfig, ProtocolUpdate, ProtocolUpdateEnactmentCondition, ProtocolUpdater,
    ProtocolUpdaterFactory, RoundHistory, StateManager, StateManagerConfig, StateManagerDatabase,
    StateVersion,
};

use crate::query::TransactionIdentifierLoader;
// use crate::traits::CommitStore;

const GENESIS_PROTOCOL_VERSION: &str = "testing-genesis";
const V2_PROTOCOL_VERSION: &str = "testing-v2";

struct TestProtocolUpdaterFactory {}

impl ProtocolUpdaterFactory for TestProtocolUpdaterFactory {
    fn supports_protocol_version(&self, protocol_version_name: &str) -> bool {
        [GENESIS_PROTOCOL_VERSION, V2_PROTOCOL_VERSION].contains(&protocol_version_name)
    }

    fn updater_for(
        &self,
        protocol_version_name: &str,
        store: Arc<StateLock<StateManagerDatabase>>,
    ) -> Box<dyn ProtocolUpdater> {
        match protocol_version_name {
            GENESIS_PROTOCOL_VERSION => Box::new(NoStateUpdatesProtocolUpdater::default(
                protocol_version_name.to_string(),
                NetworkDefinition::simulator(),
                store,
            )),
            V2_PROTOCOL_VERSION => {
                let new_config = ConsensusManagerConfig {
                    max_validators: 999,
                    epoch_change_condition: EpochChangeCondition {
                        min_round_count: 3,
                        max_round_count: 3,
                        target_duration_millis: 0,
                    },
                    num_unstake_epochs: 1,
                    total_emission_xrd_per_epoch: Decimal::one(),
                    min_validator_reliability: Decimal::one(),
                    num_owner_stake_units_unlock_epochs: 2,
                    num_fee_increase_delay_epochs: 1,
                    validator_creation_usd_cost: Decimal::one(),
                };
                Box::new(FlashProtocolUpdater::new_with_default_configurator(
                    V2_PROTOCOL_VERSION.to_string(),
                    store,
                    NetworkDefinition::simulator(),
                    vec![consensus_manager_config_flash(new_config)],
                ))
            }
            _ => panic!("Unknown protocol version {:?}", protocol_version_name),
        }
    }
}

#[test]
fn flash_protocol_update_test() {
    let metrics_registry = Registry::new();

    let mut state_manager_config =
        StateManagerConfig::new_for_testing(tempfile::tempdir().unwrap().path().to_str().unwrap());
    state_manager_config.protocol_config = ProtocolConfig {
        genesis_protocol_version: GENESIS_PROTOCOL_VERSION.to_string(),
        protocol_updates: vec![ProtocolUpdate {
            next_protocol_version: V2_PROTOCOL_VERSION.to_string(),
            enactment_condition: ProtocolUpdateEnactmentCondition::EnactUnconditionallyAtEpoch(
                Epoch::of(3),
            ),
        }],
    };

    // Genesis happens to end at version 5
    let post_genesis_state_version = StateVersion::of(5);
    // We're enacting an update after another transaction commit
    let protocol_update_state_version = post_genesis_state_version.next().unwrap();
    // And expecting a single transaction committed during an update
    let expected_post_protocol_update_state_version = protocol_update_state_version.next().unwrap();

    // Updating to "testing-v2" at post_genesis_state_version + 1
    state_manager_config.protocol_config = ProtocolConfig {
        genesis_protocol_version: GENESIS_PROTOCOL_VERSION.to_string(),
        protocol_updates: vec![ProtocolUpdate {
            next_protocol_version: V2_PROTOCOL_VERSION.to_string(),
            enactment_condition: EnactUnconditionallyAtStateVersion(protocol_update_state_version),
        }],
    };
    let state_manager = StateManager::new(
        state_manager_config,
        None,
        &LockFactory::new("testing"),
        Box::new(TestProtocolUpdaterFactory {}),
        &metrics_registry,
        &Scheduler::new("testing"),
    );

    // Run the genesis
    state_manager
        .state_computer
        .execute_genesis_for_unit_tests();

    // Verify that the post-genesis state version is what we expect
    let read_db = state_manager.database.read_current();
    assert_eq!(read_db.max_state_version(), post_genesis_state_version);
    drop(read_db);

    // Commit a single round update, which should bring us to state version 6
    // and result in protocol update enactment.
    let (prepare_result, _commit_summary) = prepare_and_commit_round_update(state_manager.clone());

    assert_eq!(
        prepare_result.next_protocol_version,
        Some(V2_PROTOCOL_VERSION.to_string())
    );

    let read_db = state_manager.database.read_current();
    assert_eq!(read_db.max_state_version(), protocol_update_state_version);
    drop(read_db);

    // Now let's apply the protocol update (this would normally be called by Java)
    state_manager.apply_protocol_update(V2_PROTOCOL_VERSION);

    // Verify that a new consensus manager config has been flashed
    let read_db = state_manager.database.read_current();
    let config_substate = read_db.get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<ConsensusManagerConfigurationFieldPayload>>(
        CONSENSUS_MANAGER.as_node_id(),
        MAIN_BASE_PARTITION,
        &ConsensusManagerField::Configuration.into()
    ).unwrap();

    assert_eq!(
        config_substate
            .into_payload()
            .into_latest()
            .config
            .max_validators,
        999
    );

    assert_eq!(
        read_db.max_state_version(),
        expected_post_protocol_update_state_version
    );
}

fn prepare_and_commit_round_update(state_manager: StateManager) -> (PrepareResult, CommitSummary) {
    let read_db = state_manager.database.read_current();
    let latest_proof: LedgerProof = read_db.get_latest_proof().unwrap();
    let latest_epoch_proof: LedgerProof = read_db.get_latest_epoch_proof().unwrap();
    let (top_state_version, top_identifiers) = read_db.get_top_transaction_identifiers().unwrap();
    drop(read_db);

    // Doesn't matter which one we use, we just need some validator from the current validator set
    let proposer_address = latest_epoch_proof
        .ledger_header
        .next_epoch
        .unwrap()
        .validator_set
        .get(0)
        .unwrap()
        .address;

    let next_round = Round::of(
        latest_proof
            .ledger_header
            .round
            .number()
            .checked_add(1)
            .unwrap(),
    );

    let prepare_result = state_manager.state_computer.prepare(PrepareRequest {
        committed_ledger_hashes: top_identifiers.resultant_ledger_hashes,
        ancestor_transactions: vec![],
        ancestor_ledger_hashes: top_identifiers.resultant_ledger_hashes,
        proposed_transactions: vec![],
        round_history: RoundHistory {
            is_fallback: false,
            epoch: latest_proof.ledger_header.epoch,
            round: next_round,
            gap_round_leader_addresses: vec![],
            proposer_address,
            proposer_timestamp_ms: latest_proof.ledger_header.proposer_timestamp_ms,
        },
    });

    let txns_to_commit = prepare_result
        .committed
        .iter()
        .map(|prep| prep.raw.clone())
        .collect();

    let commit_result = state_manager
        .state_computer
        .commit(CommitRequest {
            transactions: txns_to_commit,
            proof: LedgerProof {
                ledger_header: LedgerHeader {
                    epoch: latest_proof.ledger_header.epoch,
                    round: next_round,
                    state_version: top_state_version.next().unwrap(),
                    hashes: prepare_result.ledger_hashes,
                    consensus_parent_round_timestamp_ms: latest_proof
                        .ledger_header
                        .consensus_parent_round_timestamp_ms,
                    proposer_timestamp_ms: latest_proof.ledger_header.proposer_timestamp_ms,
                    next_epoch: prepare_result.next_epoch.clone(),
                    next_protocol_version: prepare_result.next_protocol_version.clone(),
                },
                origin: LedgerProofOrigin::Consensus {
                    opaque: Hash([0u8; 32]), /* Doesn't matter */
                    timestamped_signatures: vec![],
                },
            },
            vertex_store: None,
            self_validator_id: None,
        })
        .unwrap();

    (prepare_result, commit_result)
}

fn consensus_manager_config_flash(new_config: ConsensusManagerConfig) -> StateUpdates {
    let mut state_updates = StateUpdates::default();
    state_updates
        .of_node(CONSENSUS_MANAGER.into_node_id())
        .of_partition(MAIN_BASE_PARTITION)
        .update_substates(vec![(
            ConsensusManagerField::Configuration.into(),
            DatabaseUpdate::Set(
                scrypto_encode(&FieldSubstate::V1(FieldSubstateV1 {
                    payload: ConsensusManagerConfigurationFieldPayload {
                        content: VersionedConsensusManagerConfiguration::V1(
                            ConsensusManagerConfigSubstate { config: new_config },
                        ),
                    },
                    lock_status: LockStatus::Locked,
                }))
                .unwrap(),
            ),
        )]);
    state_updates
}
