use crate::prelude::*;
use node_common::scheduler::Scheduler;

// A bunch of test utils

pub fn create_bootstrapped_state_manager_with_rounds_per_epoch(
    config: StateManagerConfig,
    rounds_per_epoch: u64,
) -> StateManager {
    let epoch_change_condition = EpochChangeCondition {
        min_round_count: rounds_per_epoch,
        max_round_count: rounds_per_epoch,
        target_duration_millis: 100000,
    };
    let consensus_config =
        ConsensusManagerConfig::test_default().with_epoch_change_condition(epoch_change_condition);
    let babylon_settings =
        BabylonSettings::test_default().with_consensus_manager_config(consensus_config);
    create_bootstrapped_state_manager(config, babylon_settings)
}

pub fn create_bootstrapped_state_manager(
    config: StateManagerConfig,
    babylon_settings: BabylonSettings,
) -> StateManager {
    let genesis_data = JavaGenesisData::new_from(babylon_settings, vec![]);
    StateManager::new(
        config,
        None,
        Arc::new(FixedGenesisDataResolver::new(genesis_data)),
        &LockFactory::new("testing"),
        &MetricRegistry::new(),
        &Scheduler::new("testing"),
    )
}

pub fn commit_round_updates_until_epoch(state_manager: &StateManager, epoch: Epoch) {
    loop {
        let (prepare_result, _) = prepare_and_commit_round_update(state_manager);
        if prepare_result.next_protocol_version.is_some() {
            state_manager.apply_known_pending_protocol_update();
        }
        if let Some(next_epoch) = prepare_result.next_epoch {
            if next_epoch.epoch == epoch {
                break;
            }
        }
    }
}

pub fn prepare_and_commit_round_update(
    state_manager: &StateManager,
) -> (PrepareResult, CommitSummary) {
    let database = state_manager.database.access_direct();
    let latest_proof: LedgerProof = database.get_latest_proof().unwrap();
    let latest_epoch_proof: LedgerProof = database.get_latest_epoch_proof().unwrap();
    let (top_state_version, top_identifiers) = database.get_top_transaction_identifiers().unwrap();

    // Doesn't matter which one we use, we just need some validator from the current validator set
    let proposer_address = latest_epoch_proof
        .ledger_header
        .next_epoch
        .as_ref()
        .unwrap()
        .validator_set
        .first()
        .unwrap()
        .address;

    let latest_non_protocol_update_proof = match &latest_proof.origin {
        LedgerProofOrigin::Consensus { .. } => &latest_proof,
        LedgerProofOrigin::ProtocolUpdate { .. } => &latest_epoch_proof,
    };

    let (next_round, epoch) =
        if let Some(next_epoch) = &latest_non_protocol_update_proof.ledger_header.next_epoch {
            (Round::of(1), next_epoch.epoch)
        } else {
            (
                Round::of(
                    latest_proof
                        .ledger_header
                        .round
                        .number()
                        .checked_add(1)
                        .unwrap(),
                ),
                latest_proof.ledger_header.epoch,
            )
        };

    let prepare_result = state_manager.preparator.prepare(PrepareRequest {
        committed_ledger_hashes: top_identifiers.resultant_ledger_hashes,
        ancestor_transactions: vec![],
        ancestor_ledger_hashes: top_identifiers.resultant_ledger_hashes,
        proposed_transactions: vec![],
        round_history: RoundHistory {
            is_fallback: false,
            epoch,
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
        .committer
        .commit(CommitRequest {
            transactions: txns_to_commit,
            proof: LedgerProof {
                ledger_header: LedgerHeader {
                    epoch,
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
