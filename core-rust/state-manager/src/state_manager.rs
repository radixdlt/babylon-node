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

use crate::accumulator_tree::slice_merger::AccuTreeSliceMerger;

use crate::query::*;
use crate::staging::{ExecutionCache, HashStructuresDiff, ReadableStore};
use crate::store::traits::*;
use crate::transaction::*;
use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::*;

use ::transaction::errors::TransactionValidationError;
use radix_engine_common::dec;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::Epoch;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, EpochChangeCondition,
};
use radix_engine_queries::typed_substate_layout::EpochChangeEvent;

use parking_lot::{Mutex, RwLock};
use prometheus::Registry;

use ::transaction::model::{IntentHash, NotarizedTransactionHash};
use ::transaction::prelude::*;
use radix_engine::types::{Categorize, ComponentAddress, Decode, Encode};

use std::ops::Deref;
use std::sync::Arc;

use crate::staging::epoch_handling::AccuTreeEpochHandler;

use crate::mempool_manager::MempoolManager;
use radix_engine::system::bootstrap::*;
use radix_engine_interface::blueprints::consensus_manager::LeaderProposalHistory;
use radix_engine_interface::constants::GENESIS_HELPER;
use radix_engine_interface::network::NetworkDefinition;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info};
use utils::rust::collections::NonIterMap;

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct LoggingConfig {
    pub engine_trace: bool,
    pub state_manager_config: StateManagerLoggingConfig,
}

// TODO: Replace this with better loglevel integration
#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct StateManagerLoggingConfig {
    pub log_on_transaction_rejection: bool,
}

const TRANSACTION_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(500);

pub struct StateManager<S> {
    store: Arc<RwLock<S>>,
    mempool_manager: Arc<MempoolManager>,
    execution_configurator: Arc<ExecutionConfigurator>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    execution_cache: Mutex<ExecutionCache>,
    ledger_transaction_validator: LedgerTransactionValidator,
    ledger_metrics: LedgerMetrics,
    logging_config: StateManagerLoggingConfig,
}

impl<S: TransactionIdentifierLoader> StateManager<S> {
    pub fn new(
        network: &NetworkDefinition,
        store: Arc<RwLock<S>>,
        mempool_manager: Arc<MempoolManager>,
        execution_configurator: Arc<ExecutionConfigurator>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
        logging_config: LoggingConfig,
        metric_registry: &Registry,
    ) -> StateManager<S> {
        let accumulator_hash = store.read().get_top_commit_identifiers().accumulator_hash;

        StateManager {
            store,
            mempool_manager,
            execution_configurator,
            pending_transaction_result_cache,
            execution_cache: parking_lot::const_mutex(ExecutionCache::new(accumulator_hash)),
            ledger_transaction_validator: LedgerTransactionValidator::new(network),
            logging_config: logging_config.state_manager_config,
            ledger_metrics: LedgerMetrics::new(metric_registry),
        }
    }
}

pub enum StateManagerRejectReason {
    TransactionValidationError(TransactionValidationError),
}

#[derive(Debug)]
enum IntentHashDuplicateWith {
    Proposed,
    Prepared,
    Committed,
}

#[derive(Debug)]
pub struct GenesisHeaderData {
    epoch: Epoch,
    round: Round,
    timestamp: i64,
    state_version: u64,
    accumulator: AccumulatorHash,
}

#[derive(Debug)]
pub struct GenesisTransactionResult {
    raw: RawLedgerTransaction,
    transaction_hashes: (LedgerPayloadHash, LegacyLedgerPayloadHash),
    ledger_hashes: LedgerHashes,
    next_epoch: Option<NextEpoch>,
}

impl GenesisTransactionResult {
    pub fn to_commit_request(self, header_data: &mut GenesisHeaderData) -> CommitRequest {
        header_data.state_version += 1;
        header_data.accumulator = header_data
            .accumulator
            .accumulate(&self.transaction_hashes.1);

        let commit_request = CommitRequest {
            transaction_payloads: vec![self.raw],
            proof: LedgerProof {
                opaque: Hash([0; Hash::LENGTH]),
                ledger_header: LedgerHeader {
                    epoch: header_data.epoch,
                    round: header_data.round,
                    accumulator_state: AccumulatorState {
                        state_version: header_data.state_version,
                        accumulator_hash: header_data.accumulator,
                    },
                    hashes: self.ledger_hashes,
                    consensus_parent_round_timestamp_ms: header_data.timestamp,
                    proposer_timestamp_ms: header_data.timestamp,
                    next_epoch: self.next_epoch.clone(),
                },
                timestamped_signatures: vec![],
            },
            vertex_store: None,
        };
        if let Some(epoch) = self.next_epoch {
            header_data.epoch = epoch.epoch;
        }
        commit_request
    }
}

impl<S> StateManager<S>
where
    S: ReadableStore,
    S: for<'a> TransactionIndex<&'a IntentHash>,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    pub fn prepare_genesis(
        &self,
        genesis_transaction: SystemTransactionV1,
    ) -> GenesisTransactionResult {
        let read_store = self.store.read();
        let base_transaction_identifiers = read_store.get_top_commit_identifiers();
        let epoch_identifiers = read_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(&epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);

        let mut state_tracker = StateTracker::initial(base_transaction_identifiers);

        let raw = LedgerTransaction::Genesis(Box::new(genesis_transaction))
            .to_raw()
            .expect("Could not encode genesis transaction");
        let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw)
            .expect("Could not prepare genesis transaction");
        let payload_hash = prepared.ledger_payload_hash();
        let legacy_hash = prepared.legacy_ledger_payload_hash();

        let system_transaction = prepared
            .into_genesis()
            .expect("Genesis was not a system transaction");
        let executable = system_transaction.get_executable(btreeset!());

        let mut locked_execution_cache = self.execution_cache.lock();
        let processed = locked_execution_cache.execute_transaction(
            self.store.read().deref(),
            &epoch_identifiers,
            state_tracker.latest_transaction_identifiers(),
            &legacy_hash,
            self.execution_configurator
                .wrap(executable, ConfigType::Genesis)
                .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, || {
                    format!("prepare genesis {}", payload_hash)
                }),
        );

        let commit = processed.expect_commit(format!("prepare genesis {}", payload_hash));
        state_tracker.update(&commit.hash_structures_diff);

        GenesisTransactionResult {
            raw,
            transaction_hashes: (payload_hash, legacy_hash),
            ledger_hashes: *state_tracker.latest_ledger_hashes(),
            next_epoch: commit.next_epoch(),
        }
    }

    pub fn prepare(&self, prepare_request: PrepareRequest) -> PrepareResult {
        //========================================================================================
        // NOTE:
        // In this method, "already prepared" transactions that live between the commit point and the current
        // proposal will be referred to as "ancestor" - to distinguish them from "preparation" of the transactions
        // themselves, which is part of the validation process
        //========================================================================================

        let read_store = self.store.read();
        let base_transaction_identifiers = read_store.get_top_commit_identifiers();
        let epoch_header = read_store
            .get_last_epoch_proof()
            .expect("at least genesis epoch must exist")
            .ledger_header;
        let epoch_identifiers = EpochTransactionIdentifiers::from(&epoch_header);

        debug_assert_eq!(
            base_transaction_identifiers.accumulator_hash,
            prepare_request.parent_accumulator
        );

        //========================================================================================
        // PART 1:
        // We check all the proposed transactions to see if their intent hash has already been committed
        // so that we can mark them as duplicates when we come to execute them.
        //
        // TODO - Remove when this check lives in the engine/executor.
        //========================================================================================

        let prepared_proposed_results: Vec<_> = prepare_request
            .proposed_payloads
            .iter()
            .map(
                |raw| -> Result<
                    (RawLedgerTransaction, PreparedLedgerTransaction),
                    TransactionValidationError,
                > {
                    let ledger_transaction = LedgerTransaction::from_raw_user(raw)
                        .map_err(|err| {
                            TransactionValidationError::PrepareError(PrepareError::DecodeError(err))
                        })?
                        .to_raw()?;
                    let prepared = self
                        .ledger_transaction_validator
                        .prepare_from_raw(&ledger_transaction)?;
                    Ok((ledger_transaction, prepared))
                },
            )
            .collect();

        let mut intent_hash_potential_conflicts =
            HashMap::<IntentHash, IntentHashDuplicateWith>::new();

        for proposed_result in prepared_proposed_results.iter() {
            if let Ok((_, prepared)) = proposed_result {
                let intent_hash = prepared
                    .as_user()
                    .expect("Proposed was created from user")
                    .intent_hash();
                if read_store
                    .get_txn_state_version_by_identifier(&intent_hash)
                    .is_some()
                {
                    intent_hash_potential_conflicts
                        .insert(intent_hash, IntentHashDuplicateWith::Committed);
                }
            }
        }

        //========================================================================================
        // PART 2:
        // We ensure all the ancestor transactions are in our execution cache, ready to execute
        // the ancestor transactions
        //========================================================================================

        let pending_transaction_base_state = AtState::PendingPreparingVertices {
            base_committed_state_version: base_transaction_identifiers.state_version,
        };

        let mut state_tracker = StateTracker::initial(base_transaction_identifiers);

        let ancestor_transactions: Vec<_> = prepare_request
            .prepared_vertices
            .into_iter()
            .flat_map(|v| v.transaction_payloads)
            .collect();

        for raw_ancestor in ancestor_transactions {
            // TODO - By passing through the accumulator / tree hash, avoid doing all this validation if the
            // transactions are already in the execution cache!
            let validated = self
                .ledger_transaction_validator
                .validate_user_or_round_update_from_raw(&raw_ancestor)
                .expect("Already prepared transactions should be valid");

            if let Some(intent_hash) = validated.intent_hash_if_user() {
                intent_hash_potential_conflicts
                    .insert(intent_hash, IntentHashDuplicateWith::Prepared);
            }

            let legacy_hash = validated.legacy_ledger_payload_hash();
            let payload_hash = validated.ledger_payload_hash();
            let executable = validated.get_executable();
            {
                let mut execution_cache = self.execution_cache.lock();
                let processed = execution_cache.execute_transaction(
                    read_store.deref(),
                    &epoch_identifiers,
                    state_tracker.latest_transaction_identifiers(),
                    &legacy_hash,
                    self.execution_configurator
                        .wrap(executable, ConfigType::Regular)
                        .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, || {
                            format!("already prepared {}", payload_hash)
                        }),
                );

                let commit = processed.expect_commit(format!("already prepared {}", payload_hash));
                // TODO: Do we need to check that next epoch request has been prepared?
                state_tracker.update(&commit.hash_structures_diff);
            }
        }

        //========================================================================================
        // PART 3:
        // We start off the preparation by adding and executing the round change transaction
        //========================================================================================

        let mut committed = Vec::new();

        // Round Update
        // TODO: Unify this with the proposed payloads execution
        let validator_index_by_address = Self::to_validator_set_index(epoch_header);
        let round_update = RoundUpdateTransactionV1 {
            proposer_timestamp_ms: prepare_request.proposer_timestamp_ms,
            epoch: prepare_request.epoch,
            round: prepare_request.round,
            leader_proposal_history: LeaderProposalHistory {
                gap_round_leaders: prepare_request
                    .gap_round_leader_addresses
                    .iter()
                    .map(|leader_address| {
                        *validator_index_by_address
                            .get(leader_address)
                            .expect("gap round leader must belong to the validator set")
                    })
                    .collect::<Vec<_>>(),
                current_leader: *validator_index_by_address
                    .get(&prepare_request.proposer_address)
                    .expect("proposer must belong to the validator set"),
                is_fallback: prepare_request.is_fallback,
            },
        };
        let ledger_round_update = LedgerTransaction::RoundUpdateV1(Box::new(round_update));
        let prepared = self
            .ledger_transaction_validator
            .validate_user_or_round_update_from_model(&ledger_round_update)
            .expect("expected to be able to prepare the round update transaction");
        let legacy_hash = prepared.legacy_ledger_payload_hash();
        let executable = prepared.get_executable();

        let mut next_epoch = {
            let mut lock_execution_cache = self.execution_cache.lock();
            let processed_round_update = lock_execution_cache.execute_transaction(
                read_store.deref(),
                &epoch_identifiers,
                state_tracker.latest_transaction_identifiers(),
                &legacy_hash,
                self.execution_configurator
                    .wrap(executable, ConfigType::Regular)
                    .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, || {
                        format!("round update {}", prepare_request.round.number())
                    }),
            );

            let round_update_commit = processed_round_update
                .expect_commit(format!("round update {}", prepare_request.round.number()));
            round_update_commit
                .check_success(format!("round update {}", prepare_request.round.number()));
            state_tracker.update(&round_update_commit.hash_structures_diff);
            round_update_commit.next_epoch()
        };

        committed.push(
            ledger_round_update
                .to_raw()
                .expect("Expected round update to be encodable"),
        );

        //========================================================================================
        // PART 4:
        // We continue by attempting to execute the remaining transactions in the proposal
        //========================================================================================

        let mut rejected_transactions = Vec::new();
        let pending_transaction_timestamp = SystemTime::now();
        let mut pending_transaction_results = Vec::new();

        for (i, proposed_prepare_result) in prepared_proposed_results.into_iter().enumerate() {
            // Don't process any additional transactions if next epoch has occurred
            if next_epoch.is_some() {
                break;
            }

            let (raw, prepared) = match proposed_prepare_result {
                Ok(prepared) => prepared,
                Err(error) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: i as u32,
                        hashes: None,
                        error: format!("{error:?}"),
                    });
                    continue;
                }
            };

            let prepared_user = prepared.as_user().expect("Proposed was created from user");

            let intent_hash = prepared_user.intent_hash();
            let notarized_transaction_hash = prepared_user.notarized_transaction_hash();
            let ledger_hash = prepared.ledger_payload_hash();
            let legacy_hash = prepared.legacy_ledger_payload_hash();
            let invalid_at_epoch = prepared_user
                .signed_intent
                .intent
                .header
                .inner
                .end_epoch_exclusive;
            if let Some(state) = intent_hash_potential_conflicts.get(&intent_hash) {
                rejected_transactions.push(RejectedTransaction {
                    index: i as u32,
                    hashes: Some((intent_hash, notarized_transaction_hash, ledger_hash)),
                    error: format!(
                        "Duplicate intent hash: {:?}, state: {:?}",
                        &intent_hash, state
                    ),
                });
                pending_transaction_results.push(PendingTransactionResult {
                    intent_hash,
                    notarized_transaction_hash,
                    invalid_at_epoch,
                    rejection_reason: Some(RejectionReason::IntentHashCommitted),
                });
                continue;
            }

            // TODO - consider saving signature verification by re-using the validated transaction
            // in the mempool if it's already been verified there
            let validate_result = self
                .ledger_transaction_validator
                .validate_user_or_round_update(prepared);

            let validated = match validate_result {
                Ok(validated) => validated,
                Err(error) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: i as u32,
                        hashes: Some((intent_hash, notarized_transaction_hash, ledger_hash)),
                        error: format!("{:?}", &error),
                    });
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        notarized_transaction_hash,
                        invalid_at_epoch,
                        rejection_reason: Some(RejectionReason::ValidationError(
                            error.into_user_validation_error(),
                        )),
                    });
                    continue;
                }
            };

            let executable = validated.get_executable();

            {
                // Take out lock for the context
                let mut execution_cache = self.execution_cache.lock();

                let processed = execution_cache.execute_transaction(
                    read_store.deref(),
                    &epoch_identifiers,
                    state_tracker.latest_transaction_identifiers(),
                    &legacy_hash,
                    self.execution_configurator
                        .wrap(executable, ConfigType::Regular)
                        .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, || {
                            format!("newly proposed {}", ledger_hash)
                        }),
                );

                match processed.expect_commit_or_reject(format!("newly proposed {}", ledger_hash)) {
                    Ok(commit) => {
                        state_tracker.update(&commit.hash_structures_diff);
                        next_epoch = commit.next_epoch();

                        intent_hash_potential_conflicts
                            .insert(intent_hash, IntentHashDuplicateWith::Proposed);
                        committed.push(raw);
                        pending_transaction_results.push(PendingTransactionResult {
                            intent_hash,
                            notarized_transaction_hash,
                            invalid_at_epoch,
                            rejection_reason: None,
                        });
                    }
                    Err(reject) => {
                        let error = reject.error.clone();
                        rejected_transactions.push(RejectedTransaction {
                            index: i as u32,
                            hashes: Some((intent_hash, notarized_transaction_hash, ledger_hash)),
                            error: format!("{:?}", &error),
                        });
                        pending_transaction_results.push(PendingTransactionResult {
                            intent_hash,
                            notarized_transaction_hash,
                            invalid_at_epoch,
                            rejection_reason: Some(RejectionReason::FromExecution(Box::new(error))),
                        });
                    }
                }
            }
        }

        if self.logging_config.log_on_transaction_rejection {
            for rejection in rejected_transactions.iter() {
                info!("TXN INVALID: {}", &rejection.error);
            }
        }

        let pending_rejected_transactions = pending_transaction_results
            .iter()
            .filter(|pending_result| pending_result.rejection_reason.is_some())
            .map(|pending_result| {
                (
                    &pending_result.intent_hash,
                    &pending_result.notarized_transaction_hash,
                )
            })
            .collect::<Vec<_>>();
        self.mempool_manager
            .remove_rejected(&pending_rejected_transactions);

        let mut write_pending_transaction_result_cache =
            self.pending_transaction_result_cache.write();
        for pending_transaction_result in pending_transaction_results {
            let attempt = TransactionAttempt {
                rejection: pending_transaction_result.rejection_reason,
                against_state: pending_transaction_base_state.clone(),
                timestamp: pending_transaction_timestamp,
            };
            write_pending_transaction_result_cache.track_transaction_result(
                pending_transaction_result.intent_hash,
                pending_transaction_result.notarized_transaction_hash,
                Some(pending_transaction_result.invalid_at_epoch),
                attempt,
            );
        }
        drop(write_pending_transaction_result_cache);

        PrepareResult {
            committed,
            rejected: rejected_transactions,
            next_epoch,
            ledger_hashes: *state_tracker.latest_ledger_hashes(),
        }
    }

    fn to_validator_set_index(epoch_header: LedgerHeader) -> NonIterMap<ComponentAddress, u8> {
        epoch_header
            .next_epoch
            .expect("epoch header must contain next epoch information")
            .validator_set
            .into_iter()
            .enumerate()
            .map(|(validator_index, validator_info)| {
                (
                    validator_info.address,
                    ValidatorIndex::try_from(validator_index)
                        .expect("validator set size limit guarantees this"),
                )
            })
            .collect::<NonIterMap<_, _>>()
    }
}

struct StateTracker {
    transaction_identifiers: CommitBasedIdentifiers,
    ledger_hashes: Option<LedgerHashes>,
}

impl StateTracker {
    pub fn initial(base_transaction_identifiers: CommitBasedIdentifiers) -> Self {
        Self {
            transaction_identifiers: base_transaction_identifiers,
            ledger_hashes: None,
        }
    }

    pub fn latest_transaction_identifiers(&self) -> &CommitBasedIdentifiers {
        &self.transaction_identifiers
    }

    pub fn update(&mut self, hash_structures_diff: &HashStructuresDiff) {
        self.transaction_identifiers.state_version += 1;
        self.transaction_identifiers.accumulator_hash =
            hash_structures_diff.transaction_accumulator_hash;
        self.ledger_hashes = Some(hash_structures_diff.ledger_hashes);
    }

    pub fn latest_ledger_hashes(&self) -> &LedgerHashes {
        self.ledger_hashes.as_ref().expect("no update yet")
    }
}

impl<S> StateManager<S>
where
    S: CommitStore,
    S: ReadableStore,
    S: for<'a> TransactionIndex<&'a IntentHash>,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    pub fn execute_test_genesis(&self) -> LedgerProof {
        // Roughly copied from bootstrap_test_default in scrypto
        let genesis_validator: GenesisValidator = EcdsaSecp256k1PublicKey([0; 33]).into();
        let genesis_chunks = vec![
            GenesisDataChunk::Validators(vec![genesis_validator.clone()]),
            GenesisDataChunk::Stakes {
                accounts: vec![ComponentAddress::virtual_account_from_public_key(
                    &genesis_validator.key,
                )],
                allocations: vec![(
                    genesis_validator.key,
                    vec![GenesisStakeAllocation {
                        account_index: 0,
                        xrd_amount: dec!("100"),
                    }],
                )],
            },
        ];
        let initial_epoch = Epoch::of(1);
        let initial_config = ConsensusManagerConfig {
            max_validators: 10,
            epoch_change_condition: EpochChangeCondition {
                min_round_count: 1,
                max_round_count: 1,
                target_duration_millis: 0,
            },
            num_unstake_epochs: 1,
            total_emission_xrd_per_epoch: Decimal::one(),
            min_validator_reliability: Decimal::one(),
            num_owner_stake_units_unlock_epochs: 2,
            num_fee_increase_delay_epochs: 1,
        };
        let initial_timestamp_ms = 1;
        self.execute_genesis(
            genesis_chunks,
            initial_epoch,
            initial_config,
            initial_timestamp_ms,
        )
    }

    pub fn execute_genesis(
        &self,
        genesis_data_chunks: Vec<GenesisDataChunk>,
        initial_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_timestamp_ms: i64,
    ) -> LedgerProof {
        let mut header_data = GenesisHeaderData {
            epoch: initial_epoch,
            round: Round::of(0),
            timestamp: initial_timestamp_ms,
            state_version: 0,
            accumulator: AccumulatorHash::pre_genesis(),
        };

        // System bootstrap
        let system_bootstrap_transaction = create_system_bootstrap_transaction(
            initial_epoch,
            initial_config,
            initial_timestamp_ms,
        );

        let commit_request = self
            .prepare_genesis(system_bootstrap_transaction)
            .to_commit_request(&mut header_data);

        let system_bootstrap_receipt = self
            .commit(commit_request, true)
            .expect("System bootstrap commit failed")
            .remove(0);

        match system_bootstrap_receipt.on_ledger.outcome {
            LedgerTransactionOutcome::Success => {}
            LedgerTransactionOutcome::Failure => {
                panic!("Genesis system bootstrap txn didn't succeed"); // TODO(genesis): better error handling?
            }
        }

        // Data ingestion
        for (chunk_number, chunk) in genesis_data_chunks.into_iter().enumerate() {
            let genesis_data_ingestion_transaction =
                create_genesis_data_ingestion_transaction(&GENESIS_HELPER, chunk, chunk_number);

            let commit_request = self
                .prepare_genesis(genesis_data_ingestion_transaction)
                .to_commit_request(&mut header_data);

            let genesis_data_ingestion_commit_receipt = self
                .commit(commit_request, true)
                .expect("Genesis data ingestion commit failed")
                .remove(0);

            match genesis_data_ingestion_commit_receipt.on_ledger.outcome {
                LedgerTransactionOutcome::Success => {}
                LedgerTransactionOutcome::Failure => {
                    panic!("Genesis data ingestion txn didn't succeed"); // TODO(genesis): better error handling?
                }
            }
        }

        // Wrap up
        let genesis_wrap_up_transaction = create_genesis_wrap_up_transaction();

        let commit_request = self
            .prepare_genesis(genesis_wrap_up_transaction)
            .to_commit_request(&mut header_data);

        let final_ledger_proof = commit_request.proof.clone();

        let genesis_wrap_up_receipt = self
            .commit(commit_request, true)
            .expect("Genesis wrap up commit failed")
            .remove(0);

        match genesis_wrap_up_receipt.on_ledger.outcome {
            LedgerTransactionOutcome::Success => {}
            LedgerTransactionOutcome::Failure => {
                panic!("Genesis wrap up txn didn't succeed"); // TODO(genesis): better error handling?
            }
        }

        final_ledger_proof
    }

    pub fn commit(
        &self,
        commit_request: CommitRequest,
        genesis: bool,
    ) -> Result<Vec<LocalTransactionReceipt>, CommitError> {
        let commit_transactions_len = commit_request.transaction_payloads.len();
        if commit_transactions_len == 0 {
            panic!("cannot commit 0 transactions from request {commit_request:?}");
        }

        let commit_ledger_header = &commit_request.proof.ledger_header;
        let commit_accumulator_state = &commit_ledger_header.accumulator_state;
        let commit_state_version = commit_accumulator_state.state_version;
        let commit_request_start_state_version =
            commit_state_version - (commit_transactions_len as u64);

        // Whilst we could validate intent hash duplicates here, these are checked by validators on prepare already,
        // and the check will move into the engine at some point and we'll get it for free then...
        let prepared_transactions: Vec<_> = commit_request
            .transaction_payloads
            .into_iter()
            .map(|raw| -> (RawLedgerTransaction, PreparedLedgerTransaction) {
                let prepared = self.ledger_transaction_validator.prepare_from_raw(&raw)
                    .unwrap_or_else(|error| {
                        panic!("Committed transaction cannot be prepared - likely byzantine quorum: {error:?}");
                    });
                (raw, prepared)
            })
            .collect();

        let mut write_store = self.store.write();
        let base_transaction_identifiers = write_store.get_top_commit_identifiers();
        let epoch_identifiers = write_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(&epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);
        let base_state_version = base_transaction_identifiers.state_version;
        if base_state_version != commit_request_start_state_version {
            panic!(
                "Mismatched state versions - the commit request claims {} but the database thinks we're at {}",
                commit_request_start_state_version, base_state_version
            );
        }

        let mut state_tracker = StateTracker::initial(base_transaction_identifiers);
        let mut committed_transaction_bundles = Vec::new();
        let mut substate_store_update = SubstateStoreUpdate::new();
        let mut state_tree_update = HashTreeUpdate::new();
        let transaction_tree_len =
            AccuTreeEpochHandler::new(epoch_identifiers.state_version, base_state_version)
                .current_accu_tree_len();
        let mut transaction_tree_slice_merger = AccuTreeSliceMerger::new(transaction_tree_len);
        let mut receipt_tree_slice_merger = AccuTreeSliceMerger::new(transaction_tree_len);
        let mut intent_hashes = Vec::new();

        let mut result_receipts = vec![];
        for (i, (raw, prepared)) in prepared_transactions.into_iter().enumerate() {
            let (validated, execution_type) = if genesis {
                (
                    self.ledger_transaction_validator.validate_genesis(prepared),
                    ConfigType::Genesis,
                )
            } else {
                (
                    self
                        .ledger_transaction_validator
                        .validate_user_or_round_update(prepared)
                        .unwrap_or_else(|error| {
                            panic!(
                                "Committed transaction is not valid - likely byzantine quorum: {error:?}"
                            );
                        }),
                    ConfigType::Regular,
                )
            };

            let payload_hash = validated.ledger_payload_hash();
            let legacy_hash = validated.legacy_ledger_payload_hash();
            let executable = validated.get_executable();

            let (
                next_epoch,
                state_hash_tree_diff,
                transaction_tree_slice,
                receipt_tree_slice,
                local_receipt,
                database_updates,
            ) = {
                let mut lock_execution_cache = self.execution_cache.lock();
                let processed = lock_execution_cache.execute_transaction(
                    write_store.deref(),
                    &epoch_identifiers,
                    state_tracker.latest_transaction_identifiers(),
                    &legacy_hash,
                    self.execution_configurator
                        .wrap(executable, execution_type)
                        .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, || {
                            format!("committing {}", payload_hash)
                        }),
                );
                let commit = processed.expect_commit(format!("committing {}", payload_hash));

                let hash_structures_diff = &commit.hash_structures_diff;
                state_tracker.update(hash_structures_diff);
                let next_epoch = commit.next_epoch();
                let state_hash_tree_diff = hash_structures_diff.state_hash_tree_diff.clone();
                let transaction_tree_slice =
                    hash_structures_diff.transaction_tree_diff.slice.clone();
                let receipt_tree_slice = hash_structures_diff.receipt_tree_diff.slice.clone();
                let local_receipt = commit.local_receipt.clone();
                let database_updates = commit.database_updates.clone();
                (
                    next_epoch,
                    state_hash_tree_diff,
                    transaction_tree_slice,
                    receipt_tree_slice,
                    local_receipt,
                    database_updates,
                )
            };

            Self::check_epoch_proof_match(
                commit_ledger_header,
                next_epoch,
                i == (commit_transactions_len - 1),
            )?;

            if let Some(intent_hash) = validated.intent_hash_if_user() {
                intent_hashes.push(intent_hash);
            }

            let commit_based_identifiers = state_tracker.latest_transaction_identifiers().clone();

            substate_store_update.apply(database_updates);
            state_tree_update.add(commit_based_identifiers.state_version, state_hash_tree_diff);
            transaction_tree_slice_merger.append(transaction_tree_slice);
            receipt_tree_slice_merger.append(receipt_tree_slice);

            result_receipts.push(local_receipt.clone());

            committed_transaction_bundles.push(CommittedTransactionBundle {
                raw,
                receipt: local_receipt,
                identifiers: CommittedTransactionIdentifiers {
                    payload: validated.create_identifiers(),
                    at_commit: commit_based_identifiers,
                },
            });
        }

        let commit_ledger_hashes = &commit_ledger_header.hashes;
        let final_ledger_hashes = state_tracker.latest_ledger_hashes();
        if *final_ledger_hashes != *commit_ledger_hashes {
            error!(
                "computed ledger hashes at version {} differ from the ones in proof ({:?} != {:?})",
                commit_accumulator_state.state_version, final_ledger_hashes, commit_ledger_hashes
            );
            return Err(CommitError::LedgerHashesMismatch);
        }

        let final_transaction_identifiers = state_tracker.latest_transaction_identifiers().clone();

        self.execution_cache
            .lock()
            .progress_root(&final_transaction_identifiers.accumulator_hash);

        write_store.commit(CommitBundle {
            transactions: committed_transaction_bundles,
            proof: commit_request.proof,
            substate_store_update,
            vertex_store: commit_request.vertex_store,
            state_tree_update,
            transaction_tree_slice: transaction_tree_slice_merger.into_slice(),
            receipt_tree_slice: receipt_tree_slice_merger.into_slice(),
        });
        drop(write_store);

        self.ledger_metrics
            .state_version
            .set(final_transaction_identifiers.state_version as i64);
        self.ledger_metrics
            .transactions_committed
            .inc_by(commit_transactions_len as u64);
        self.ledger_metrics.last_update_epoch_second.set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );

        self.mempool_manager.remove_committed(&intent_hashes);

        self.pending_transaction_result_cache
            .write()
            .track_committed_transactions(
                SystemTime::now(),
                commit_request_start_state_version,
                intent_hashes,
            );

        Ok(result_receipts)
    }

    fn check_epoch_proof_match(
        commit_ledger_header: &LedgerHeader,
        opt_transaction_next_epoch: Option<NextEpoch>,
        is_last_transaction_in_request: bool,
    ) -> Result<(), CommitError> {
        if is_last_transaction_in_request {
            match &commit_ledger_header.next_epoch {
                Some(proof_next_epoch) => {
                    if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
                        if transaction_next_epoch != *proof_next_epoch {
                            error!(
                                "computed next epoch differs from the one in proof ({:?} != {:?})",
                                transaction_next_epoch, proof_next_epoch
                            );
                            return Err(CommitError::EpochProofMismatch);
                        }
                    } else {
                        error!(
                            "computed no next epoch, but proof contains {:?}",
                            proof_next_epoch
                        );
                        return Err(CommitError::SuperfluousEpochProof);
                    }
                }
                None => {
                    if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
                        error!(
                            "no next epoch in proof, but last transaction in batch computed {:?}",
                            transaction_next_epoch
                        );
                        return Err(CommitError::MissingEpochProof);
                    }
                }
            };
        } else if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
            error!(
                "non-last transaction in batch computed {:?}",
                transaction_next_epoch
            );
            return Err(CommitError::MissingEpochProof);
        }
        Ok(())
    }
}

impl From<EpochChangeEvent> for NextEpoch {
    fn from(epoch_change_event: EpochChangeEvent) -> Self {
        NextEpoch {
            validator_set: epoch_change_event
                .validator_set
                .validators_by_stake_desc
                .into_iter()
                .map(|(address, validator)| ActiveValidatorInfo {
                    address,
                    key: validator.key,
                    stake: validator.stake,
                })
                .collect(),
            epoch: epoch_change_event.epoch,
        }
    }
}

struct PendingTransactionResult {
    pub intent_hash: IntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
    pub invalid_at_epoch: Epoch,
    pub rejection_reason: Option<RejectionReason>,
}
