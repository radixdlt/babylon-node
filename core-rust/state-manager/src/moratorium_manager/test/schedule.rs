use super::*;

#[test]
fn fresh_node_syncs_historical_and_upcoming_moratoriums() {
    // Arrange
    let historical = UserTransactionMoratorium {
        from_inclusive: Epoch::of(2),
        to_exclusive: Epoch::of(3),
    };
    let upcoming = UserTransactionMoratorium {
        from_inclusive: Epoch::of(4),
        to_exclusive: Epoch::of(5),
    };
    let directory = tempfile::tempdir().unwrap();
    let protocol_config = protocol_config_with_schedule([historical, upcoming]);
    let source_config = StateManagerConfig {
        protocol_config: ProtocolConfig {
            user_transaction_moratoriums: Vec::new(),
            ..protocol_config.clone()
        },
        ..StateManagerConfig::new_for_testing(directory.path().join("source").to_str().unwrap())
    };
    let source = create_bootstrapped_state_manager_with_rounds_per_epoch(source_config, 1);
    commit_round_updates_until_epoch(&source, Epoch::of(4));
    let target_config = StateManagerConfig {
        protocol_config,
        ..StateManagerConfig::new_for_testing(directory.path().join("target").to_str().unwrap())
    };

    // Act
    let target = create_bootstrapped_state_manager_with_rounds_per_epoch(target_config, 1);
    let policy_before_catch_up = target.mempool_manager.ensure_user_transactions_allowed();
    sync_available_history(&source, &target);
    let policy_during_upcoming_moratorium =
        target.mempool_manager.ensure_user_transactions_allowed();
    let caught_up_header = target
        .database
        .snapshot()
        .get_latest_proof()
        .unwrap()
        .ledger_header;
    let source_header = source
        .database
        .snapshot()
        .get_latest_proof()
        .unwrap()
        .ledger_header;
    commit_round_updates_until_epoch(&source, Epoch::of(5));
    sync_available_history(&source, &target);
    let policy_after_enactment = target.mempool_manager.ensure_user_transactions_allowed();
    let final_header = target
        .database
        .snapshot()
        .get_latest_proof()
        .unwrap()
        .ledger_header;
    let final_source_header = source
        .database
        .snapshot()
        .get_latest_proof()
        .unwrap()
        .ledger_header;

    // Assert
    assert_eq!(policy_before_catch_up, Err(historical));
    assert_eq!(policy_during_upcoming_moratorium, Err(upcoming));
    assert_eq!(caught_up_header, source_header);
    assert_eq!(policy_after_enactment, Ok(()));
    assert_eq!(final_header, final_source_header);
    assert_eq!(
        target.committability_validator.current_epoch(),
        Epoch::of(5)
    );
    assert_eq!(
        target.protocol_manager.current_protocol_version(),
        ProtocolVersionName::of("test-range-1").unwrap()
    );
}

#[test]
fn accepts_disjoint_ranges_in_reverse_epoch_order() {
    // Arrange
    let first = UserTransactionMoratorium {
        from_inclusive: Epoch::of(2),
        to_exclusive: Epoch::of(3),
    };
    let second = UserTransactionMoratorium {
        from_inclusive: Epoch::of(4),
        to_exclusive: Epoch::of(5),
    };
    let config = protocol_config_with_schedule([second, first]);

    // Act
    let manager = UserTransactionMoratoriumManager::new(config.user_transaction_moratoriums);
    let results =
        [1, 2, 3, 4, 5, 6].map(|epoch| manager.ensure_user_transactions_allowed(Epoch::of(epoch)));

    // Assert
    assert_eq!(
        results,
        [Ok(()), Err(first), Ok(()), Err(second), Ok(()), Ok(())]
    );
}

#[test]
fn touching_ranges_switch_policy_at_the_shared_epoch() {
    // Arrange
    let first = UserTransactionMoratorium {
        from_inclusive: Epoch::of(2),
        to_exclusive: Epoch::of(3),
    };
    let second = UserTransactionMoratorium {
        from_inclusive: Epoch::of(3),
        to_exclusive: Epoch::of(4),
    };
    let config = protocol_config_with_schedule([first, second]);

    // Act
    let manager = UserTransactionMoratoriumManager::new(config.user_transaction_moratoriums);
    let results = [2, 3, 4].map(|epoch| manager.ensure_user_transactions_allowed(Epoch::of(epoch)));

    // Assert
    assert_eq!(results, [Err(first), Err(second), Ok(())]);
}

/// Associates each configured range with a distinct no-op protocol update.
fn protocol_config_with_schedule(
    moratoriums: impl IntoIterator<Item = UserTransactionMoratorium>,
) -> ProtocolConfig {
    let moratoriums = moratoriums.into_iter().collect::<Vec<_>>();
    let config = ProtocolConfig::new_with_triggers(moratoriums.iter().enumerate().map(
        |(index, moratorium)| {
            (
                ProtocolVersionName::of(format!("test-range-{index}")).unwrap(),
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(
                    moratorium.to_exclusive,
                ),
            )
        },
    ));
    ProtocolConfig {
        user_transaction_moratoriums: moratoriums,
        ..config
    }
}

/// Uses served ledger proofs and normal commits, resuming each protocol update.
fn sync_available_history(source: &StateManager, target: &StateManager) {
    let source_state_version = source.database.snapshot().max_state_version();
    while target.database.snapshot().max_state_version() < source_state_version {
        let next_state_version = target
            .database
            .snapshot()
            .max_state_version()
            .next()
            .unwrap();
        let TxnsAndProof { txns, proof } = source
            .database
            .snapshot()
            .get_syncable_txns_and_proof(next_state_version, 100, 1_000_000)
            .unwrap();
        let expected_protocol_version = proof.ledger_header.next_protocol_version.clone();
        target
            .committer
            .commit(CommitRequest {
                transactions: txns,
                proof,
                vertex_store: None,
                self_validator_id: None,
            })
            .unwrap();
        if let Some(expected_protocol_version) = expected_protocol_version {
            target.apply_known_pending_protocol_updates();
            assert_eq!(
                target.protocol_manager.current_protocol_version(),
                expected_protocol_version
            );
        }
    }
}
