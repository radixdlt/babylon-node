use super::*;

#[test]
fn rejects_only_epochs_inside_the_inclusive_exclusive_range() {
    // Arrange
    let config = state_manager_config_with_moratorium_trigger(Epoch::of(3), Epoch::of(5));
    let manager =
        UserTransactionMoratoriumManager::new(config.protocol_config.user_transaction_moratoriums);
    let moratorium = UserTransactionMoratorium {
        from_inclusive: Epoch::of(3),
        to_exclusive: Epoch::of(5),
    };

    // Act
    let results =
        [2, 3, 4, 5, 6].map(|epoch| manager.ensure_user_transactions_allowed(Epoch::of(epoch)));

    // Assert
    assert_eq!(
        results,
        [Ok(()), Err(moratorium), Err(moratorium), Ok(()), Ok(())]
    );
}

#[test]
fn mainnet_has_the_incident_range_and_stokenet_has_no_moratorium() {
    // Arrange
    let mainnet = resolve_protocol_config(&NetworkDefinition::mainnet());
    let stokenet = resolve_protocol_config(&NetworkDefinition::stokenet());

    // Act
    let mainnet_manager =
        UserTransactionMoratoriumManager::new(mainnet.user_transaction_moratoriums);
    let stokenet_manager =
        UserTransactionMoratoriumManager::new(stokenet.user_transaction_moratoriums);
    let active = mainnet_manager.ensure_user_transactions_allowed(Epoch::of(339897));
    let enacted = mainnet_manager.ensure_user_transactions_allowed(Epoch::of(339898));
    let stokenet_check = stokenet_manager.ensure_user_transactions_allowed(Epoch::of(339897));

    // Assert
    assert_eq!(
        active,
        Err(UserTransactionMoratorium {
            from_inclusive: Epoch::of(339897),
            to_exclusive: Epoch::of(339898),
        })
    );
    assert_eq!(enacted, Ok(()));
    assert_eq!(stokenet_check, Ok(()));
    assert_eq!(stokenet_manager.moratoriums, Vec::new());
}
