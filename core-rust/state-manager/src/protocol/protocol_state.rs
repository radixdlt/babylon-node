use crate::prelude::*;

// This file contains types and utilities for managing the (dynamic) protocol state of a running
// node.

pub struct ProtocolManager {
    protocol_metrics: ProtocolMetrics,
    current_protocol_version: RwLock<ProtocolVersionName>,
    protocol_state: RwLock<ProtocolState>,
    newest_protocol_version: ProtocolVersionName,
}

impl ProtocolManager {
    pub fn new(
        protocol_update_triggers: Vec<ProtocolUpdateTrigger>,
        protocol_update_overrides: &RawProtocolUpdateContentOverrides,
        protocol_update_context: ProtocolUpdateContext,
        lock_factory: &LockFactory,
        metric_registry: &MetricRegistry,
    ) -> Self {
        let initial_protocol_state = ProtocolState::compute_initial(
            protocol_update_overrides,
            protocol_update_context,
            &protocol_update_triggers,
        );
        Self {
            protocol_metrics: ProtocolMetrics::new(metric_registry, &initial_protocol_state),
            current_protocol_version: lock_factory.named("current_version").new_rwlock(
                initial_protocol_state
                    .enacted_protocol_updates
                    .last_key_value()
                    .map(|(_, protocol_version)| protocol_version.clone())
                    .unwrap_or(ProtocolVersionName::babylon()),
            ),
            protocol_state: lock_factory
                .named("state")
                .new_rwlock(initial_protocol_state),
            newest_protocol_version: protocol_update_triggers
                .last()
                .map(|protocol_update| protocol_update.next_protocol_version.clone())
                .unwrap_or(ProtocolVersionName::babylon()),
        }
    }

    pub fn protocol_state_at_version(&self, _state_version: StateVersion) -> ProtocolState {
        // TODO(strict correctness): At the moment, the protocol state is only relevant when
        // executing an epoch change (i.e. as part of a round update, during `prepare()`). In these
        // cases, we actually always need only the current protocol state. In future though, this
        // method could be called e.g. by historical transaction preview logic, or historical state
        // serving API (even if only for informational purposes), and we can cheaply avoid confusion
        // by resolving this from an in-memory map (which we almost have at `compute_initial()`).
        self.current_protocol_state()
    }

    pub fn current_protocol_state(&self) -> ProtocolState {
        self.protocol_state.read().deref().clone()
    }

    pub fn current_protocol_version(&self) -> ProtocolVersionName {
        self.current_protocol_version.read().deref().clone()
    }

    pub fn update_protocol_state_and_metrics(&self, end_state: &StateTrackerEndState) {
        self.protocol_metrics
            .update(&end_state.protocol_state, end_state.epoch_change.as_ref());
        *self.protocol_state.write() = end_state.protocol_state.clone();
    }

    pub fn newest_protocol_version(&self) -> ProtocolVersionName {
        self.newest_protocol_version.clone()
    }

    pub fn set_current_protocol_version(&self, protocol_version: &ProtocolVersionName) {
        *self.current_protocol_version.write() = protocol_version.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProtocolState {
    /// A list of all protocol updates that have been enacted.
    pub enacted_protocol_updates: BTreeMap<StateVersion, ProtocolVersionName>,
    /// A list of protocol updates that haven't yet been enacted, but still can be in the future.
    pub pending_protocol_updates: IndexMap<ProtocolVersionName, PendingProtocolUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct PendingProtocolUpdate {
    pub protocol_update: ProtocolUpdateTrigger,
    pub state: PendingProtocolUpdateState,
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub enum PendingProtocolUpdateState {
    ForSignalledReadinessSupportCondition {
        thresholds_state: Vec<(
            // Thresholds here are kept just for convenience,
            // they duplicate the ones in `protocol_update.enactment_condition`.
            SignalledReadinessThreshold,
            SignalledReadinessThresholdState,
        )>,
    },
    // Empty placeholder for all other stateless conditions
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct SignalledReadinessThresholdState {
    /// A number of consecutive epochs on or above the threshold,
    /// including the current (uncompleted) epoch.
    pub consecutive_started_epochs_of_support: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InitialProtocolUpdateStatus {
    ExpectedToHaveBeenEnactedAtStateVersion(StateVersion),
    Pending(PendingProtocolUpdateState),
    ExpiredUnenacted,
}

fn compute_initial_protocol_update_status<
    S: QueryableProofStore + IterableProofStore + QueryableTransactionStore,
>(
    store: &S,
    protocol_update_trigger: &ProtocolUpdateTrigger,
) -> InitialProtocolUpdateStatus {
    match &protocol_update_trigger.enactment_condition {
        ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
            lower_bound_inclusive,
            upper_bound_exclusive,
            readiness_thresholds,
        } => compute_initial_signalled_readiness_protocol_update_status(
            store,
            protocol_update_trigger,
            lower_bound_inclusive,
            upper_bound_exclusive,
            readiness_thresholds,
        ),
        ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(epoch) => {
            compute_initial_at_epoch_protocol_update_status(store, *epoch)
        }
        ProtocolUpdateEnactmentCondition::EnactImmediatelyAfterEndOfProtocolUpdate {
            trigger_after,
        } => compute_initial_after_protocol_update_status(store, trigger_after),
    }
}

fn compute_initial_signalled_readiness_protocol_update_status<
    S: QueryableProofStore + IterableProofStore + QueryableTransactionStore,
>(
    store: &S,
    protocol_update_trigger: &ProtocolUpdateTrigger,
    lower_bound_inclusive: &Epoch,
    upper_bound_exclusive: &Epoch,
    thresholds: &[SignalledReadinessThreshold],
) -> InitialProtocolUpdateStatus {
    // Mutable var for the initial state that we'll compute in this fn
    let mut thresholds_state: Vec<(
        SignalledReadinessThreshold,
        SignalledReadinessThresholdState,
    )> = thresholds
        .iter()
        .map(|threshold| {
            (
                threshold.clone(),
                SignalledReadinessThresholdState {
                    consecutive_started_epochs_of_support: 0,
                },
            )
        })
        .collect();

    // Highest number of required epochs of support from all thresholds
    let max_required_consecutive_epochs_of_support = thresholds
        .iter()
        .map(|threshold| threshold.required_consecutive_completed_epochs_of_support)
        .max()
        .expect("No thresholds found in signalled readiness protocol update");

    // The earliest epoch where we need to consider the readiness signals
    let earliest_relevant_epoch = Epoch::of(
        lower_bound_inclusive
            .number()
            .saturating_sub(max_required_consecutive_epochs_of_support),
    );

    // Start iterating from the earliest relevant epoch
    // (or the earliest epoch we have, i.e. genesis)
    let epoch_change_event_iter = epoch_change_iter(store, earliest_relevant_epoch);

    for (state_version, epoch_change_event) in epoch_change_event_iter {
        // Update the thresholds
        update_thresholds_state_at_epoch_change(
            protocol_update_trigger,
            &epoch_change_event,
            &mut thresholds_state,
        );

        // Check if any threshold passes
        let any_threshold_passes = any_threshold_passes(&thresholds_state);
        if !any_threshold_passes {
            continue;
        }

        // Check if we're on or above the lower bound
        let on_or_above_lower_bound =
            epoch_change_event.epoch.number() >= lower_bound_inclusive.number();
        if !on_or_above_lower_bound {
            continue;
        }

        // Check if we're below the upper bound
        let below_upper_bound = epoch_change_event.epoch.number() < upper_bound_exclusive.number();
        if !below_upper_bound {
            continue;
        }

        // It's a match! This protocol update enacts at exactly the state version
        // corresponding to the current epoch change.
        return InitialProtocolUpdateStatus::ExpectedToHaveBeenEnactedAtStateVersion(state_version);
    }

    // The protocol update wasn't enacted yet, so just return the latest computed state
    // as the initial state.
    InitialProtocolUpdateStatus::Pending(
        PendingProtocolUpdateState::ForSignalledReadinessSupportCondition { thresholds_state },
    )
}

fn compute_initial_at_epoch_protocol_update_status<S: QueryableProofStore>(
    store: &S,
    epoch: Epoch,
) -> InitialProtocolUpdateStatus {
    if let Some(epoch_proof) = store.get_epoch_proof(epoch) {
        InitialProtocolUpdateStatus::ExpectedToHaveBeenEnactedAtStateVersion(
            epoch_proof.ledger_header.state_version,
        )
    } else {
        InitialProtocolUpdateStatus::Pending(PendingProtocolUpdateState::Empty)
    }
}

fn compute_initial_after_protocol_update_status<S: QueryableProofStore + IterableProofStore>(
    store: &S,
    triggered_after_name: &ProtocolVersionName,
) -> InitialProtocolUpdateStatus {
    let mut triggered_updates: IndexMap<ProtocolVersionName, StateVersion> = store
        .get_protocol_update_init_proof_iter(StateVersion::pre_genesis())
        .map(|proof| {
            (
                proof.ledger_header.next_protocol_version.unwrap(),
                proof.ledger_header.state_version,
            )
        })
        .collect();
    triggered_updates.insert(ProtocolVersionName::babylon(), StateVersion::pre_genesis());

    if let Some(triggered_at) = triggered_updates.get(triggered_after_name) {
        let latest_execution_proof =
            latest_execution_proof(store, *triggered_at, triggered_after_name);
        if let Some(latest_execution_proof) = latest_execution_proof {
            if let LedgerProofOrigin::ProtocolUpdate {
                is_end_of_update, ..
            } = &latest_execution_proof.origin
            {
                if *is_end_of_update {
                    return InitialProtocolUpdateStatus::ExpectedToHaveBeenEnactedAtStateVersion(
                        latest_execution_proof.ledger_header.state_version,
                    );
                }
            }
        }
    }
    InitialProtocolUpdateStatus::Pending(PendingProtocolUpdateState::Empty)
}

fn latest_execution_proof(
    store: &impl IterableProofStore,
    init_state_version: StateVersion,
    version_name: &ProtocolVersionName,
) -> Option<LedgerProofV3> {
    let first_execution_proof = init_state_version.next().unwrap();
    store.get_protocol_update_execution_proof_iter(first_execution_proof)
        .take_while(|proof| {
            matches!(
                &proof.origin,
                LedgerProofOrigin::ProtocolUpdate { protocol_version_name, .. } if protocol_version_name == version_name
            )
        })
        .last()
}

impl ProtocolState {
    pub fn compute_initial(
        raw_protocol_overrides: &RawProtocolUpdateContentOverrides,
        protocol_update_context: ProtocolUpdateContext,
        protocol_update_triggers: &[ProtocolUpdateTrigger],
    ) -> ProtocolState {
        // For each configured allowed protocol update we calculate its expected status against
        // the current state of the ledger, regardless of any information stored
        // about the protocol updates that were actually enacted.
        // This is then juxtaposed with the protocol updates that have actually been enacted,
        // to catch any inconsistencies.
        // This serves mainly to protect from misconfiguration (e.g.
        // running a node with a configured protocol update for the past state version,
        // which hasn't been executed on the local ledger at the right time).
        // This also provides the initial state for stateful (readiness-based)
        // protocol update conditions.
        let initial_statuses: Vec<_> = protocol_update_triggers
            .iter()
            .map(|protocol_update| {
                (
                    protocol_update,
                    compute_initial_protocol_update_status(
                        protocol_update_context.database.lock().deref(),
                        protocol_update,
                    ),
                )
            })
            .collect();

        let expected_already_enacted_protocol_updates: BTreeMap<StateVersion, ProtocolVersionName> =
            initial_statuses
                .iter()
                .flat_map(|(protocol_update, status)| match status {
                    InitialProtocolUpdateStatus::ExpectedToHaveBeenEnactedAtStateVersion(
                        state_version,
                    ) => Some((
                        *state_version,
                        protocol_update.next_protocol_version.clone(),
                    )),
                    InitialProtocolUpdateStatus::Pending(_)
                    | InitialProtocolUpdateStatus::ExpiredUnenacted => None,
                })
                .collect();

        let enacted_protocol_updates: BTreeMap<StateVersion, ProtocolVersionName> =
            protocol_update_context
                .database
                .lock()
                .get_protocol_update_init_proof_iter(StateVersion::pre_genesis())
                .map(|proof| {
                    let init_state_version = proof.ledger_header.state_version;
                    let version_name = ProtocolVersionName::of_unchecked(
                        proof
                            .ledger_header
                            .next_protocol_version
                            .expect("next_protocol_version is missing in protocol update proof"),
                    );
                    (init_state_version, version_name)
                })
                .collect();

        if expected_already_enacted_protocol_updates != enacted_protocol_updates {
            panic!(
                "State computer couldn't be initialized, protocol misconfiguration: \
             according to the current configuration and the ledger state the following \
             protocol updates should have been enacted: {:?}, but the following \
             updates were actually enacted: {:?}.",
                expected_already_enacted_protocol_updates, enacted_protocol_updates,
            );
        }

        for (init_state_version, protocol_version) in expected_already_enacted_protocol_updates {
            let stored_config_hash = latest_execution_proof(
                protocol_update_context.database.lock().deref(),
                init_state_version,
                &protocol_version,
            )
            .and_then(|proof| {
                if let LedgerProofOrigin::ProtocolUpdate { config_hash, .. } = &proof.origin {
                    *config_hash
                } else {
                    None
                }
            });
            if let Some(stored_config_hash) = stored_config_hash {
                let resolved_config_hash = {
                    let overrides = raw_protocol_overrides.get(&protocol_version);
                    let resolved = protocol_version.validate().unwrap_or_else(|err| {
                        panic!(
                            "{} is not a supported protocol version: {err:?}",
                            protocol_version.as_str()
                        )
                    });
                    let protocol_update_definition = resolved.definition();
                    protocol_update_definition
                        .resolve_config_hash(protocol_update_context, overrides)
                };
                if stored_config_hash != resolved_config_hash {
                    if protocol_version == ProtocolVersionName::babylon() {
                        panic!("\
                            The genesis data (of hash {resolved_config_hash}) doesn't match the genesis data that has previously been \
                            used to initialize the database ({stored_config_hash}). \
                            Make sure your configuration is correct (check `network.id` and/or `network.genesis_data` \
                            and/or `network.genesis_data_file`).\
                        ")
                    } else {
                        panic!("\
                            The overrides for {protocol_version:?} have changed since the update was run. \
                            The current hash is {resolved_config_hash}, but the stored hash is {stored_config_hash}.
                        ")
                    }
                }
            }
        }

        let pending_protocol_updates = initial_statuses
            .into_iter()
            .flat_map(|(protocol_update, status)| match status {
                InitialProtocolUpdateStatus::ExpectedToHaveBeenEnactedAtStateVersion(_) => None,
                InitialProtocolUpdateStatus::Pending(state) => {
                    let name = protocol_update.next_protocol_version.clone();
                    let pending_update = PendingProtocolUpdate {
                        protocol_update: protocol_update.clone(),
                        state,
                    };
                    Some((name, pending_update))
                }
                InitialProtocolUpdateStatus::ExpiredUnenacted => None,
            })
            .collect();

        ProtocolState {
            enacted_protocol_updates,
            pending_protocol_updates,
        }
    }

    /// Computes a new protocol state if a protocol update has been enacted.
    ///
    /// The correctness of protocol updating relies on the fact that any trigger below can only be triggered by:
    /// * End of an epoch
    /// * End of a commit batch
    ///
    /// And we check in [`StateTracker::update`] that end of an epoch can only occur at the end of a commit batch.
    pub fn check_for_update_trigger_at_end_of_batch(
        &mut self,
        next_epoch_event: Option<&EpochChangeEvent>,
        post_execute_state_version: StateVersion,
        batch_situation: BatchSituation,
    ) -> Option<ProtocolVersionName> {
        let mut enactable_protocol_updates = index_set_new();
        let mut expired_protocol_updates = index_set_new();

        for (protocol_version_name, pending_protocol_update) in
            self.pending_protocol_updates.iter_mut()
        {
            match &pending_protocol_update.protocol_update.enactment_condition {
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
                    lower_bound_inclusive,
                    upper_bound_exclusive,
                    ..
                } => {
                    let Some(epoch_change_event) = next_epoch_event else {
                        continue;
                    };
                    let post_execute_epoch = epoch_change_event.epoch;

                    let PendingProtocolUpdateState::ForSignalledReadinessSupportCondition {
                        ref mut thresholds_state,
                    } = &mut pending_protocol_update.state
                    else {
                        panic!("Invalid protocol state");
                    };

                    update_thresholds_state_at_epoch_change(
                        &pending_protocol_update.protocol_update,
                        epoch_change_event,
                        thresholds_state,
                    );

                    let on_or_above_lower_bound = post_execute_epoch >= *lower_bound_inclusive;
                    let below_upper_bound = post_execute_epoch < *upper_bound_exclusive;

                    if on_or_above_lower_bound
                        && below_upper_bound
                        && any_threshold_passes(thresholds_state)
                    {
                        enactable_protocol_updates.insert(protocol_version_name.clone());
                    } else if !below_upper_bound {
                        expired_protocol_updates.insert(protocol_version_name.clone());
                    }
                }
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(
                    enactment_epoch,
                ) => {
                    let Some(new_epoch) = next_epoch_event.map(|e| e.epoch) else {
                        continue;
                    };
                    match new_epoch.cmp(enactment_epoch) {
                        Ordering::Less => {}
                        Ordering::Equal => {
                            enactable_protocol_updates.insert(protocol_version_name.clone());
                        }
                        Ordering::Greater => {
                            panic!("FATAL: Previous protocol update enactment missed. There is configuration to enact {protocol_version_name} at the start of epoch {enactment_epoch:?}, but the node is about to start epoch {new_epoch:?}, and the protocol update has not already been enacted. This is an inconsistency, so the node is halting.")
                        }
                    }
                }
                ProtocolUpdateEnactmentCondition::EnactImmediatelyAfterEndOfProtocolUpdate {
                    trigger_after,
                } => match batch_situation {
                    BatchSituation::ProtocolUpdate {
                        ref update,
                        is_final_batch,
                    } if update == trigger_after && is_final_batch => {
                        enactable_protocol_updates.insert(protocol_version_name.clone());
                    }
                    _ => {}
                },
            }
        }

        for expired_protocol_update in expired_protocol_updates {
            // This isn't really a right place for this log, but will do for now
            info!(
                "Protocol update {:?} expires unenacted at state version {:?}",
                expired_protocol_update, post_execute_state_version,
            );
            self.pending_protocol_updates
                .shift_remove(&expired_protocol_update);
        }

        match enactable_protocol_updates.len() {
            0 => None,
            1 => {
                let enacted_protocol_version =
                    enactable_protocol_updates.into_iter().next().unwrap();
                self.pending_protocol_updates
                    .shift_remove(&enacted_protocol_version);
                self.enacted_protocol_updates
                    .insert(post_execute_state_version, enacted_protocol_version.clone());

                Some(enacted_protocol_version.clone())
            }
            _ => {
                panic!(
                    "Invalid state: more than one protocol update is enactable at state version {:?}",
                    post_execute_state_version
                )
            }
        }
    }
}

fn any_threshold_passes(
    thresholds_state: &[(
        SignalledReadinessThreshold,
        SignalledReadinessThresholdState,
    )],
) -> bool {
    thresholds_state.iter().any(|(threshold, threshold_state)| {
        // Note: `consecutive_started_epochs_of_support` must be strictly greater because
        // it includes the current (uncompleted) epoch, while the threshold condition
        // specifies fully completed epochs.
        threshold_state.consecutive_started_epochs_of_support
            > threshold.required_consecutive_completed_epochs_of_support
    })
}

fn update_thresholds_state_at_epoch_change(
    protocol_update: &ProtocolUpdateTrigger,
    epoch_change_event: &EpochChangeEvent,
    thresholds_state: &mut Vec<(
        SignalledReadinessThreshold,
        SignalledReadinessThresholdState,
    )>,
) {
    let signalled_stake_readiness = epoch_change_event
        .significant_protocol_update_readiness
        .get(&protocol_update.readiness_signal_name())
        .cloned()
        .unwrap_or_else(Decimal::zero);

    let total_stake = epoch_change_event
        .validator_set
        .total_active_stake_xrd()
        .expect("Failed to calculate the total stake");

    // Update each threshold according to its required_percentage_stake_supported
    for (threshold, threshold_state) in &mut *thresholds_state {
        let required_stake = total_stake
            .checked_mul(threshold.required_ratio_of_stake_supported)
            .expect("Failed to calculate required stake for a protocol update");
        if signalled_stake_readiness >= required_stake {
            // Support on or above threshold: inc num of consecutive epochs
            threshold_state.consecutive_started_epochs_of_support = threshold_state
                .consecutive_started_epochs_of_support
                .saturating_add(1);
        } else {
            // Not enough support: reset to 0
            threshold_state.consecutive_started_epochs_of_support = 0;
        }
    }
}

/// A helper that iterates epoch proofs and extracts
/// EpochChangeEvents from corresponding ledger receipts.
pub fn epoch_change_iter<'s, S: IterableProofStore + QueryableTransactionStore>(
    store: &'s S,
    from_epoch: Epoch,
) -> Box<dyn Iterator<Item = (StateVersion, EpochChangeEvent)> + 's> {
    let epoch_iter = store.get_next_epoch_proof_iter(from_epoch);
    Box::new(epoch_iter.map(|epoch_proof| {
        let next_epoch = epoch_proof
            .ledger_header
            .epoch
            .next()
            .expect("Epoch overflow");
        let state_version = epoch_proof.ledger_header.state_version;
        let epoch_receipt = store
            .get_committed_ledger_transaction_receipt(state_version)
            .unwrap_or_else(|| {
                panic!(
                    "Missing transaction receipt for epoch change transaction \
                        (next_epoch={}, state_version={})",
                    next_epoch.number(),
                    state_version
                )
            });

        let epoch_change_event = epoch_receipt
            .application_events
            .iter()
            .filter(|ev| {
                ev.type_id.0 == Emitter::Method(CONSENSUS_MANAGER.into_node_id(), ModuleId::Main)
                    && ev.type_id.1 == "EpochChangeEvent"
            })
            .map(|ev| {
                scrypto_decode::<EpochChangeEvent>(&ev.data)
                    .expect("Could not decode EpochChangeEvent")
            })
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "Epoch change transaction receipt does not contain an EpochChangeEvent \
                    (next_epoch={}, state_version={}",
                    next_epoch.number(),
                    state_version
                )
            });
        (state_version, epoch_change_event)
    }))
}
