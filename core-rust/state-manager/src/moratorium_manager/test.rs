use super::*;
use crate::test::*;

/// A no-op update used to isolate the moratorium policy from engine changes.
const MORATORIUM_TEST_PROTOCOL_VERSION: &str = "test-moratorium";

fn moratorium_test_protocol_version() -> ProtocolVersionName {
    ProtocolVersionName::of(MORATORIUM_TEST_PROTOCOL_VERSION).unwrap()
}

/// Schedules a no-op update at the moratorium's end.
fn state_manager_config_with_moratorium_trigger(
    moratorium_from_epoch: Epoch,
    enactment_epoch: Epoch,
) -> StateManagerConfig {
    let mut state_manager_config =
        StateManagerConfig::new_for_testing(tempfile::tempdir().unwrap().path().to_str().unwrap());
    state_manager_config.protocol_config = ProtocolConfig {
        user_transaction_moratoriums: vec![UserTransactionMoratorium {
            from_inclusive: moratorium_from_epoch,
            to_exclusive: enactment_epoch,
        }],
        ..ProtocolConfig::new_with_triggers(hashmap! {
            moratorium_test_protocol_version() =>
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(enactment_epoch)
        })
    };
    state_manager_config
}

/// Creates a transaction valid throughout the test epochs.
fn create_raw_user_transaction() -> RawNotarizedTransaction {
    create_raw_user_transaction_with_nonce(1)
}

/// Gives concurrent submissions distinct hashes.
fn create_raw_user_transaction_with_nonce(nonce: u32) -> RawNotarizedTransaction {
    let notary = Ed25519PrivateKey::from_u64(7).unwrap();
    TransactionV1Builder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(100),
            nonce,
            notary_public_key: notary.public_key().into(),
            notary_is_signatory: true,
            tip_percentage: 0,
        })
        .manifest(ManifestBuilder::new_v1().lock_fee_from_faucet().build())
        .notarize(&notary)
        .build()
        .to_raw()
        .unwrap()
}

#[test]
fn moratorium_trigger_enacts_update_at_start_of_enactment_epoch_and_lifts_moratorium() {
    // Arrange
    let state_manager_config =
        state_manager_config_with_moratorium_trigger(Epoch::of(2), Epoch::of(3));
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    let moratorium_before_enactment = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();

    // Act
    let (prepare_result, _commit_summary) = prepare_and_commit_round_update(&state_manager);
    let moratorium_after_enactment_commit = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();
    state_manager.apply_known_pending_protocol_updates();

    // Assert
    assert_eq!(
        moratorium_before_enactment,
        Err(UserTransactionMoratorium {
            from_inclusive: Epoch::of(2),
            to_exclusive: Epoch::of(3),
        })
    );
    assert_eq!(prepare_result.next_epoch.unwrap().epoch, Epoch::of(3));
    assert_eq!(
        prepare_result.next_protocol_version,
        Some(moratorium_test_protocol_version())
    );
    assert_eq!(moratorium_after_enactment_commit, Ok(()));
    assert_eq!(
        state_manager.protocol_manager.current_protocol_version(),
        moratorium_test_protocol_version()
    );
}

#[test]
fn moratorium_is_absent_before_its_starting_epoch_and_present_from_it() {
    // Arrange
    let state_manager_config =
        state_manager_config_with_moratorium_trigger(Epoch::of(3), Epoch::of(4));
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    let moratorium_at_epoch_two = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();

    // Act
    let (prepare_result, _commit_summary) = prepare_and_commit_round_update(&state_manager);
    let moratorium_at_epoch_three = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();

    // Assert
    assert_eq!(moratorium_at_epoch_two, Ok(()));
    assert_eq!(prepare_result.next_epoch.unwrap().epoch, Epoch::of(3));
    assert_eq!(prepare_result.next_protocol_version, None);
    assert_eq!(
        moratorium_at_epoch_three,
        Err(UserTransactionMoratorium {
            from_inclusive: Epoch::of(3),
            to_exclusive: Epoch::of(4),
        })
    );
}

#[test]
fn mempool_rejects_user_transactions_and_proposes_none_while_moratorium_is_in_force() {
    // Arrange
    let state_manager_config =
        state_manager_config_with_moratorium_trigger(Epoch::of(2), Epoch::of(3));
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    let raw_transaction = create_raw_user_transaction();

    // Act
    let add_result = state_manager.mempool_manager.add_if_committable(
        MempoolAddSource::CoreApi,
        raw_transaction,
        false,
    );
    let proposal_transactions =
        state_manager
            .mempool_manager
            .get_proposal_transactions(10, 1_000_000, &HashSet::new());

    // Assert
    let (rejection, notarized_transaction_hash) = match add_result {
        Err(MempoolAddError::Rejected(rejection, notarized_transaction_hash)) => {
            (rejection, notarized_transaction_hash)
        }
        Err(other_error) => panic!("unexpected mempool error: {other_error:?}"),
        Ok(_) => panic!("the transaction was unexpectedly admitted to the mempool"),
    };
    assert_eq!(
        rejection.reason,
        MempoolRejectionReason::UserTransactionMoratorium(UserTransactionMoratorium {
            from_inclusive: Epoch::of(2),
            to_exclusive: Epoch::of(3),
        })
    );
    assert!(matches!(
        rejection.retry_from,
        RetryFrom::FromEpoch(epoch) if epoch == Epoch::of(3)
    ));
    assert!(!rejection.is_permanent_for_payload());
    assert!(!rejection.is_permanent_for_intent());
    assert_eq!(notarized_transaction_hash, None);
    assert!(proposal_transactions.is_empty());
    assert_eq!(state_manager.mempool_manager.get_mempool_count(), 0);
}

#[test]
fn multi_epoch_moratorium_stays_in_force_across_epoch_changes_until_enactment() {
    // Arrange
    let state_manager_config =
        state_manager_config_with_moratorium_trigger(Epoch::of(2), Epoch::of(4));
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    let moratorium_at_epoch_two = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();

    // Act
    let (first_prepare_result, _) = prepare_and_commit_round_update(&state_manager);
    let moratorium_at_epoch_three = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();
    let (second_prepare_result, _) = prepare_and_commit_round_update(&state_manager);
    let moratorium_at_epoch_four = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();

    // Assert
    let expected_moratorium = Err(UserTransactionMoratorium {
        from_inclusive: Epoch::of(2),
        to_exclusive: Epoch::of(4),
    });
    assert_eq!(moratorium_at_epoch_two, expected_moratorium);
    assert_eq!(first_prepare_result.next_epoch.unwrap().epoch, Epoch::of(3));
    assert_eq!(first_prepare_result.next_protocol_version, None);
    assert_eq!(moratorium_at_epoch_three, expected_moratorium);
    assert_eq!(
        second_prepare_result.next_epoch.unwrap().epoch,
        Epoch::of(4)
    );
    assert_eq!(
        second_prepare_result.next_protocol_version,
        Some(moratorium_test_protocol_version())
    );
    assert_eq!(moratorium_at_epoch_four, Ok(()));
}

#[test]
fn mempool_relays_and_proposes_nothing_during_moratorium_but_keeps_earlier_transactions() {
    // Arrange
    let state_manager_config =
        state_manager_config_with_moratorium_trigger(Epoch::of(3), Epoch::of(4));
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    state_manager
        .mempool_manager
        .add_if_committable(
            MempoolAddSource::CoreApi,
            create_raw_user_transaction(),
            false,
        )
        .expect("the transaction should be admitted before the moratorium starts");
    let relayed_before_moratorium = state_manager
        .mempool_manager
        .get_relay_transactions(10, 1_000_000)
        .len();

    // Act
    prepare_and_commit_round_update(&state_manager);
    let relayed_during_moratorium = state_manager
        .mempool_manager
        .get_relay_transactions(10, 1_000_000)
        .len();
    let proposed_during_moratorium = state_manager
        .mempool_manager
        .get_proposal_transactions(10, 1_000_000, &HashSet::new())
        .len();
    let held_during_moratorium = state_manager.mempool_manager.get_mempool_count();
    prepare_and_commit_round_update(&state_manager);
    state_manager.apply_known_pending_protocol_updates();
    let relayed_after_enactment = state_manager
        .mempool_manager
        .get_relay_transactions(10, 1_000_000)
        .len();
    let proposed_after_enactment = state_manager
        .mempool_manager
        .get_proposal_transactions(10, 1_000_000, &HashSet::new())
        .len();

    // Assert
    assert_eq!(relayed_before_moratorium, 1);
    assert_eq!(relayed_during_moratorium, 0);
    assert_eq!(proposed_during_moratorium, 0);
    assert_eq!(held_during_moratorium, 1);
    assert_eq!(relayed_after_enactment, 1);
    assert_eq!(proposed_after_enactment, 1);
}

#[test]
#[should_panic(expected = "protocol misconfiguration")]
fn boot_fails_when_moratorium_trigger_enactment_epoch_was_already_passed_without_enacting() {
    // Arrange
    let state_manager_config =
        StateManagerConfig::new_for_testing(tempfile::tempdir().unwrap().path().to_str().unwrap());
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    commit_round_updates_until_epoch(&state_manager, Epoch::of(4));
    let network_definition = NetworkDefinition::simulator();
    let genesis_data_resolver: Arc<dyn ResolveGenesisData> =
        Arc::new(FixedGenesisDataResolver::new(JavaGenesisData::new_from(
            BabylonSettings::test_default(),
            vec![],
        )));
    let scenarios_execution_config = ScenariosExecutionConfig::default();
    let triggers = vec![ProtocolUpdateTrigger::of(
        moratorium_test_protocol_version(),
        ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(Epoch::of(3)),
    )];

    // Act
    let _ = ProtocolState::compute_initial(
        &ProtocolUpdateContentOverrides::empty().into(),
        ProtocolUpdateContext {
            network: &network_definition,
            database: &state_manager.database,
            genesis_data_resolver: &genesis_data_resolver,
            scenario_config: &scenarios_execution_config,
        },
        &triggers,
    );

    // Assert
    unreachable!("compute_initial must panic when an enactment epoch has been passed unenacted");
}

#[test]
fn epoch_changing_round_update_drops_every_proposed_user_transaction() {
    // Arrange
    let state_manager_config =
        StateManagerConfig::new_for_testing(tempfile::tempdir().unwrap().path().to_str().unwrap());
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    let database = state_manager.database.access_direct();
    let latest_proof: LedgerProof = database.get_latest_proof().unwrap();
    let latest_epoch_proof: LedgerProof = database.get_latest_epoch_proof().unwrap();
    let (_, top_identifiers) = database.get_top_transaction_identifiers().unwrap();
    let next_epoch = latest_epoch_proof
        .ledger_header
        .next_epoch
        .as_ref()
        .unwrap();
    let proposer_address = next_epoch.validator_set.first().unwrap().address;
    drop(database);

    // Act
    let prepare_result = state_manager.preparator.prepare(PrepareRequest {
        committed_ledger_hashes: top_identifiers.resultant_ledger_hashes,
        ancestor_transactions: vec![],
        ancestor_ledger_hashes: top_identifiers.resultant_ledger_hashes,
        proposed_transactions: vec![create_raw_user_transaction()],
        round_history: RoundHistory {
            is_fallback: false,
            epoch: next_epoch.epoch,
            round: Round::of(1),
            gap_round_leader_addresses: vec![],
            proposer_address,
            proposer_timestamp_ms: latest_proof.ledger_header.proposer_timestamp_ms,
        },
    });

    // Assert
    assert!(prepare_result.next_epoch.is_some());
    assert_eq!(prepare_result.committed.len(), 1);
    assert_eq!(prepare_result.committed[0].index, None);
    assert!(prepare_result.rejected.is_empty());
}

#[test]
fn transaction_rejected_during_moratorium_is_admitted_and_proposed_once_it_is_lifted() {
    // Arrange
    let state_manager_config =
        state_manager_config_with_moratorium_trigger(Epoch::of(2), Epoch::of(3));
    let state_manager =
        create_bootstrapped_state_manager_with_rounds_per_epoch(state_manager_config, 1);
    let raw_transaction = create_raw_user_transaction();
    let rejected_during_moratorium = state_manager
        .mempool_manager
        .add_if_committable(MempoolAddSource::CoreApi, raw_transaction.clone(), false)
        .is_err();

    // Act
    prepare_and_commit_round_update(&state_manager);
    state_manager.apply_known_pending_protocol_updates();
    let admitted = state_manager
        .mempool_manager
        .add_if_committable(MempoolAddSource::CoreApi, raw_transaction.clone(), false)
        .expect("the transaction should be admitted once the moratorium is lifted");
    let proposed = state_manager
        .mempool_manager
        .get_proposal_transactions(10, 1_000_000, &HashSet::new())
        .iter()
        .map(|transaction| transaction.raw.clone())
        .collect::<Vec<_>>();

    // Assert
    assert!(rejected_during_moratorium);
    assert_eq!(
        state_manager
            .mempool_manager
            .ensure_user_transactions_allowed(),
        Ok(())
    );
    assert_eq!(admitted.raw, raw_transaction);
    assert_eq!(proposed, vec![raw_transaction]);
}

#[test]
fn moratorium_stays_active_until_the_epoch_transition_is_committed() {
    // Arrange
    let config = state_manager_config_with_moratorium_trigger(Epoch::of(3), Epoch::of(4));
    let state_manager = create_bootstrapped_state_manager_with_rounds_per_epoch(config, 1);
    let transaction = create_raw_user_transaction();
    state_manager
        .mempool_manager
        .add_if_committable(MempoolAddSource::CoreApi, transaction.clone(), false)
        .unwrap();
    prepare_and_commit_round_update(&state_manager);
    let committed_version = state_manager.database.snapshot().max_state_version();
    let (paused_sender, paused_receiver) = std::sync::mpsc::channel();
    let (resume_sender, resume_receiver) = std::sync::mpsc::channel();
    state_manager
        .committer
        .before_next_database_commit(move || {
            paused_sender.send(()).unwrap();
            resume_receiver
                .recv_timeout(Duration::from_secs(30))
                .unwrap();
        });
    let committing_state_manager = state_manager.clone();
    let submitted_while_paused = create_raw_user_transaction_with_nonce(2);

    // Act
    let worker =
        std::thread::spawn(move || prepare_and_commit_round_update(&committing_state_manager));
    paused_receiver
        .recv_timeout(Duration::from_secs(30))
        .unwrap();
    let moratorium_while_paused = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();
    let relay_while_paused = state_manager
        .mempool_manager
        .get_relay_transactions(10, 1_000_000);
    let proposals_while_paused =
        state_manager
            .mempool_manager
            .get_proposal_transactions(10, 1_000_000, &HashSet::new());
    let admission_while_paused = state_manager.mempool_manager.add_if_committable(
        MempoolAddSource::CoreApi,
        submitted_while_paused.clone(),
        false,
    );
    let snapshot_while_paused = state_manager.database.snapshot();
    let epoch_while_paused = snapshot_while_paused.get_epoch_and_round().0;
    let version_while_paused = snapshot_while_paused.max_state_version();
    drop(snapshot_while_paused);
    resume_sender.send(()).unwrap();
    let (prepare_result, _) = worker.join().unwrap();
    state_manager.apply_known_pending_protocol_updates();
    let moratorium_after_commit = state_manager
        .mempool_manager
        .ensure_user_transactions_allowed();
    let relayed_after_commit = state_manager
        .mempool_manager
        .get_relay_transactions(10, 1_000_000)
        .iter()
        .map(|entry| entry.raw.clone())
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(epoch_while_paused, Epoch::of(3));
    assert_eq!(version_while_paused, committed_version);
    assert_eq!(
        moratorium_while_paused,
        Err(UserTransactionMoratorium {
            from_inclusive: Epoch::of(3),
            to_exclusive: Epoch::of(4),
        })
    );
    assert!(relay_while_paused.is_empty());
    assert!(proposals_while_paused.is_empty());
    assert_eq!(prepare_result.next_epoch.unwrap().epoch, Epoch::of(4));
    assert_eq!(moratorium_after_commit, Ok(()));
    assert_eq!(relayed_after_commit, vec![transaction]);
    let Err(MempoolAddError::Rejected(rejection, payload_hash)) = admission_while_paused else {
        panic!("the paused commit must not admit a new user transaction");
    };
    assert_eq!(
        rejection.reason,
        MempoolRejectionReason::UserTransactionMoratorium(UserTransactionMoratorium {
            from_inclusive: Epoch::of(3),
            to_exclusive: Epoch::of(4)
        })
    );
    assert_eq!(payload_hash, None);
    assert_eq!(rejection.retry_from, RetryFrom::FromEpoch(Epoch::of(4)));
    assert!(!rejection.is_permanent_for_payload());
    assert!(!rejection.is_permanent_for_intent());
}

mod policy;
mod schedule;
