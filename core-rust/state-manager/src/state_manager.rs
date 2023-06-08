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
use std::fmt::Formatter;

use crate::query::*;
use crate::staging::{ExecutionCache, ReadableStore};
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

use radix_engine::transaction::RejectResult;
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
        let transaction_root = store.read().get_top_ledger_hashes().1.transaction_root;

        StateManager {
            store,
            mempool_manager,
            execution_configurator,
            pending_transaction_result_cache,
            execution_cache: parking_lot::const_mutex(ExecutionCache::new(transaction_root)),
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
pub struct GenesisHeaderData {
    epoch: Epoch,
    round: Round,
    timestamp: i64,
    state_version: StateVersion,
}

#[derive(Debug)]
pub struct GenesisTransactionResult {
    raw: RawLedgerTransaction,
    ledger_hashes: LedgerHashes,
    next_epoch: Option<NextEpoch>,
}

impl GenesisTransactionResult {
    pub fn to_commit_request(self, header_data: &mut GenesisHeaderData) -> CommitRequest {
        header_data.state_version = header_data.state_version.next();

        let commit_request = CommitRequest {
            transactions: vec![self.raw],
            proof: LedgerProof {
                opaque: Hash([0; Hash::LENGTH]),
                ledger_header: LedgerHeader {
                    epoch: header_data.epoch,
                    round: header_data.round,
                    state_version: header_data.state_version,
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
        let (base_state_version, base_ledger_hashes) = read_store.get_top_ledger_hashes();
        let epoch_identifiers = read_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(&epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);

        let raw = LedgerTransaction::Genesis(Box::new(genesis_transaction))
            .to_raw()
            .expect("Could not encode genesis transaction");
        let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw)
            .expect("Could not prepare genesis transaction");
        let ledger_transaction_hash = prepared.ledger_transaction_hash();

        let system_transaction = prepared
            .into_genesis()
            .expect("Genesis was not a system transaction");
        let executable = system_transaction.get_executable(btreeset!());

        let mut execution_cache = self.execution_cache.lock();
        let processed = execution_cache.execute_transaction(
            self.store.read().deref(),
            &epoch_identifiers,
            base_state_version,
            &base_ledger_hashes.transaction_root,
            &ledger_transaction_hash,
            self.execution_configurator
                .wrap(executable, ConfigType::Genesis)
                .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, "genesis"),
        );

        let commit = processed.expect_commit("genesis");

        GenesisTransactionResult {
            raw,
            ledger_hashes: commit.hash_structures_diff.ledger_hashes,
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
        let (base_state_version, base_ledger_hashes) = read_store.get_top_ledger_hashes();
        let epoch_header = read_store
            .get_last_epoch_proof()
            .expect("at least genesis epoch must exist")
            .ledger_header;
        let epoch_identifiers = EpochTransactionIdentifiers::from(&epoch_header);

        let mut series_executor = TransactionSeriesExecutor::new(
            read_store.deref(),
            &self.execution_cache,
            self.execution_configurator.deref(),
            epoch_identifiers,
            base_state_version,
            base_ledger_hashes,
        );

        if prepare_request.committed_ledger_hashes != base_ledger_hashes {
            panic!(
                "state {:?} from request does not match the current ledger state {:?}",
                prepare_request.committed_ledger_hashes, base_ledger_hashes
            );
        }

        //========================================================================================
        // PART 1:
        // We execute all the ancestor transactions (on a happy path: only making sure they are in
        // our execution cache),
        //========================================================================================

        let pending_transaction_base_state = AtState::PendingPreparingVertices {
            base_committed_state_version: base_state_version,
        };

        let mut duplicate_intent_hash_detector =
            DuplicateIntentHashDetector::new(read_store.deref());

        for raw_ancestor in prepare_request.ancestor_transactions {
            // TODO(optimization-only): We could avoid the hashing, decoding, signature verification
            // and executable creation) by accessing the execution cache in a more clever way.
            let validated = self
                .ledger_transaction_validator
                .validate_user_or_round_update_from_raw(&raw_ancestor)
                .expect("Ancestor transactions should be valid");

            if let Some(intent_hash) = validated.intent_hash_if_user() {
                duplicate_intent_hash_detector.record_ancestor(intent_hash);
            }

            series_executor
                .execute(ConfigType::Regular, &validated, "ancestor")
                .expect("ancestor transaction rejected");
        }

        if &prepare_request.ancestor_ledger_hashes != series_executor.latest_ledger_hashes() {
            panic!(
                "State {:?} after ancestor transactions does not match the state {:?} from request",
                series_executor.latest_ledger_hashes(),
                prepare_request.ancestor_ledger_hashes,
            );
        }

        //========================================================================================
        // PART 2:
        // We start off the preparation by adding and executing the round change transaction
        //========================================================================================

        let mut committable_transactions = Vec::new();

        {
            // We create a separate scope to ensure any variables don't leak to later in the method.
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
            let validated = self
                .ledger_transaction_validator
                .validate_user_or_round_update_from_model(&ledger_round_update)
                .expect("expected to be able to prepare the round update transaction");

            let execute_result =
                series_executor.execute(ConfigType::Regular, &validated, "round update");
            let commit = execute_result.expect("round update rejected");
            if let DetailedTransactionOutcome::Failure(error) =
                commit.local_receipt.local_execution.outcome
            {
                panic!(
                    "update to round {} failed: {:?}",
                    prepare_request.round.number(),
                    error
                );
            }

            committable_transactions.push(CommittableTransaction {
                index: None,
                raw: ledger_round_update
                    .to_raw()
                    .expect("Expected round update to be encodable"),
                intent_hash: None,
                notarized_transaction_hash: None,
                ledger_transaction_hash: validated.ledger_transaction_hash(),
            });
        }

        //========================================================================================
        // PART 3:
        // We continue by attempting to execute the remaining transactions in the proposal
        //========================================================================================

        let mut rejected_transactions = Vec::new();
        let pending_transaction_timestamp = SystemTime::now();
        let mut pending_transaction_results = Vec::new();

        for (index, raw_user_transaction) in prepare_request
            .proposed_transactions
            .into_iter()
            .enumerate()
        {
            // Don't process any additional transactions if next epoch has occurred
            if series_executor.next_epoch().is_some() {
                break;
            }

            let prepare_results = LedgerTransaction::from_raw_user(&raw_user_transaction)
                .map_err(|err| {
                    TransactionValidationError::PrepareError(PrepareError::DecodeError(err))
                })
                .and_then(|ledger_transaction| {
                    ledger_transaction.to_raw().map_err(|err| {
                        TransactionValidationError::PrepareError(PrepareError::EncodeError(err))
                    })
                })
                .and_then(|raw_ledger_transaction| {
                    self.ledger_transaction_validator
                        .prepare_from_raw(&raw_ledger_transaction)
                        .map(|prepared_transaction| (raw_ledger_transaction, prepared_transaction))
                });

            let (raw_ledger_transaction, prepared_transaction) = match prepare_results {
                Ok(results) => results,
                Err(error) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: None,
                        notarized_transaction_hash: None,
                        ledger_transaction_hash: None,
                        error: format!("{error:?}"),
                    });
                    continue;
                }
            };

            let prepared_user_transaction = prepared_transaction
                .as_user()
                .expect("Proposed was created from user");

            let intent_hash = prepared_user_transaction.intent_hash();
            let notarized_transaction_hash = prepared_user_transaction.notarized_transaction_hash();
            let ledger_transaction_hash = prepared_transaction.ledger_transaction_hash();
            let invalid_at_epoch = prepared_user_transaction
                .signed_intent
                .intent
                .header
                .inner
                .end_epoch_exclusive;
            if let Err(with) = duplicate_intent_hash_detector.check_proposed(&intent_hash) {
                rejected_transactions.push(RejectedTransaction {
                    index: index as u32,
                    intent_hash: Some(intent_hash),
                    notarized_transaction_hash: Some(notarized_transaction_hash),
                    ledger_transaction_hash: Some(ledger_transaction_hash),
                    error: format!(
                        "Duplicate intent hash: {:?}, state: {:?}",
                        &intent_hash, with
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

            // TODO(optimization-only): We could avoid signature verification by re-using the
            // validated transaction from the mempool.
            let validate_result = self
                .ledger_transaction_validator
                .validate_user_or_round_update(prepared_transaction);

            let validated = match validate_result {
                Ok(validated) => validated,
                Err(error) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: Some(intent_hash),
                        notarized_transaction_hash: Some(notarized_transaction_hash),
                        ledger_transaction_hash: Some(ledger_transaction_hash),
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

            let execute_result =
                series_executor.execute(ConfigType::Regular, &validated, "newly proposed");
            match execute_result {
                Ok(_) => {
                    duplicate_intent_hash_detector.record_committable_proposed(intent_hash);
                    committable_transactions.push(CommittableTransaction {
                        index: Some(index as u32),
                        raw: raw_ledger_transaction,
                        intent_hash: Some(intent_hash),
                        notarized_transaction_hash: Some(notarized_transaction_hash),
                        ledger_transaction_hash,
                    });
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        notarized_transaction_hash,
                        invalid_at_epoch,
                        rejection_reason: None,
                    });
                }
                Err(RejectResult { error }) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: Some(intent_hash),
                        notarized_transaction_hash: Some(notarized_transaction_hash),
                        ledger_transaction_hash: Some(ledger_transaction_hash),
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
            committed: committable_transactions,
            rejected: rejected_transactions,
            next_epoch: series_executor.next_epoch().cloned(),
            ledger_hashes: *series_executor.latest_ledger_hashes(),
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
    state_version: StateVersion,
    ledger_hashes: LedgerHashes,
    next_epoch: Option<NextEpoch>,
}

impl StateTracker {
    pub fn initial(base_state_version: StateVersion, base_ledger_hashes: LedgerHashes) -> Self {
        Self {
            state_version: base_state_version,
            ledger_hashes: base_ledger_hashes,
            next_epoch: None,
        }
    }

    pub fn update(&mut self, result: &ProcessedCommitResult) {
        if let Some(next_epoch) = &self.next_epoch {
            panic!(
                "the {:?} has happened at {:?} (version {}) and must not be followed by {:?}",
                next_epoch,
                self.ledger_hashes,
                self.state_version,
                result.hash_structures_diff.ledger_hashes
            );
        }
        self.state_version = self.state_version.next();
        self.ledger_hashes = result.hash_structures_diff.ledger_hashes;
        self.next_epoch = result.next_epoch();
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
            state_version: StateVersion::pre_genesis(),
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
    ) -> Result<Vec<LocalTransactionReceipt>, InvalidCommitRequestError> {
        let commit_transactions_len = commit_request.transactions.len();
        if commit_transactions_len == 0 {
            panic!("cannot commit 0 transactions from request {commit_request:?}");
        }

        let commit_ledger_header = &commit_request.proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;
        let commit_request_start_state_version =
            commit_state_version.relative(-(commit_transactions_len as i128));

        // Whilst we could validate intent hash duplicates here, these are checked by validators on prepare already,
        // and the check will move into the engine at some point and we'll get it for free then...
        let prepared_transactions: Vec<_> = commit_request
            .transactions
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
        let (base_state_version, base_ledger_hashes) = write_store.get_top_ledger_hashes();
        let epoch_identifiers = write_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(&epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);
        if base_state_version != commit_request_start_state_version {
            panic!(
                "Mismatched state versions - the commit request claims {} but the database thinks we're at {}",
                commit_request_start_state_version, base_state_version
            );
        }

        let mut committed_transaction_bundles = Vec::new();
        let mut substate_store_update = SubstateStoreUpdate::new();
        let mut state_tree_update = HashTreeUpdate::new();
        let transaction_tree_len =
            AccuTreeEpochHandler::new(epoch_identifiers.state_version, base_state_version)
                .current_accu_tree_len();
        let mut transaction_tree_slice_merger = AccuTreeSliceMerger::new(transaction_tree_len);
        let mut receipt_tree_slice_merger = AccuTreeSliceMerger::new(transaction_tree_len);
        let mut intent_hashes = Vec::new();

        let mut series_executor = TransactionSeriesExecutor::new(
            write_store.deref(),
            &self.execution_cache,
            self.execution_configurator.deref(),
            epoch_identifiers,
            base_state_version,
            base_ledger_hashes,
        );

        let mut result_receipts = vec![];
        for (i, (raw, prepared)) in prepared_transactions.into_iter().enumerate() {
            let (validated, config_type) = if genesis {
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

            let (
                state_hash_tree_diff,
                transaction_tree_slice,
                receipt_tree_slice,
                local_receipt,
                database_updates,
            ) = {
                let execute_result = series_executor.execute(config_type, &validated, "prepared");
                let commit = execute_result.expect("prepared transaction not committable");
                let hash_structures_diff = &commit.hash_structures_diff;
                let state_hash_tree_diff = hash_structures_diff.state_hash_tree_diff.clone();
                let transaction_tree_slice =
                    hash_structures_diff.transaction_tree_diff.slice.clone();
                let receipt_tree_slice = hash_structures_diff.receipt_tree_diff.slice.clone();
                let local_receipt = commit.local_receipt.clone();
                let database_updates = commit.database_updates.clone();
                (
                    state_hash_tree_diff,
                    transaction_tree_slice,
                    receipt_tree_slice,
                    local_receipt,
                    database_updates,
                )
            };

            Self::check_epoch_proof_match(
                commit_ledger_header,
                series_executor.next_epoch(),
                i == (commit_transactions_len - 1),
            )?;

            if let Some(intent_hash) = validated.intent_hash_if_user() {
                intent_hashes.push(intent_hash);
            }

            substate_store_update.apply(database_updates);
            state_tree_update.add(series_executor.latest_state_version(), state_hash_tree_diff);
            transaction_tree_slice_merger.append(transaction_tree_slice);
            receipt_tree_slice_merger.append(receipt_tree_slice);

            result_receipts.push(local_receipt.clone());

            committed_transaction_bundles.push(CommittedTransactionBundle {
                state_version: series_executor.latest_state_version(),
                raw,
                receipt: local_receipt,
                identifiers: CommittedTransactionIdentifiers {
                    payload: validated.create_identifiers(),
                    resultant_ledger_hashes: *series_executor.latest_ledger_hashes(),
                },
            });
        }

        let commit_ledger_hashes = &commit_ledger_header.hashes;
        let final_ledger_hashes = series_executor.latest_ledger_hashes();
        if final_ledger_hashes != commit_ledger_hashes {
            error!(
                "computed ledger hashes at version {} differ from the ones in proof ({:?} != {:?})",
                commit_state_version, final_ledger_hashes, commit_ledger_hashes
            );
            return Err(InvalidCommitRequestError::LedgerHashesMismatch);
        }

        self.execution_cache
            .lock()
            .progress_base(&final_ledger_hashes.transaction_root);

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
            .set(commit_state_version.number() as i64);
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
        opt_transaction_next_epoch: Option<&NextEpoch>,
        is_last_transaction_in_request: bool,
    ) -> Result<(), InvalidCommitRequestError> {
        if is_last_transaction_in_request {
            match &commit_ledger_header.next_epoch {
                Some(proof_next_epoch) => {
                    if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
                        if transaction_next_epoch != proof_next_epoch {
                            error!(
                                "computed next epoch differs from the one in proof ({:?} != {:?})",
                                transaction_next_epoch, proof_next_epoch
                            );
                            return Err(InvalidCommitRequestError::EpochProofMismatch);
                        }
                    } else {
                        error!(
                            "computed no next epoch, but proof contains {:?}",
                            proof_next_epoch
                        );
                        return Err(InvalidCommitRequestError::SuperfluousEpochProof);
                    }
                }
                None => {
                    if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
                        error!(
                            "no next epoch in proof, but last transaction in batch computed {:?}",
                            transaction_next_epoch
                        );
                        return Err(InvalidCommitRequestError::MissingEpochProof);
                    }
                }
            };
        } else if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
            error!(
                "non-last transaction in batch computed {:?}",
                transaction_next_epoch
            );
            return Err(InvalidCommitRequestError::MissingEpochProof);
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

#[derive(Debug, Clone)]
enum IntentHashDuplicateWith {
    Proposed,
    Ancestor,
    Committed,
}

/// An internal implementation delegate dealing with intent hash duplicates.
// TODO: Remove after this responsibility is implemented by the Engine.
struct DuplicateIntentHashDetector<'s, S> {
    store: &'s S,
    recorded_intent_hashes: NonIterMap<IntentHash, IntentHashDuplicateWith>,
}

impl<'s, S: for<'a> TransactionIndex<&'a IntentHash>> DuplicateIntentHashDetector<'s, S> {
    pub fn new(store: &'s S) -> Self {
        Self {
            store,
            recorded_intent_hashes: NonIterMap::new(),
        }
    }

    /// Records an intent hash of an ancestor (i.e. one of already-prepared-but-not-yet-committed)
    /// transaction.
    /// Please note that duplicates are not possible for ancestor transactions (since they were all
    /// checked against this during previous prepare operations), and hence the `check_ancestor()`
    /// method does not exist.
    pub fn record_ancestor(&mut self, intent_hash: IntentHash) {
        self.recorded_intent_hashes
            .insert(intent_hash, IntentHashDuplicateWith::Ancestor);
    }

    /// Checks whether the given intent hash of a newly-proposed transaction clashes with any other
    /// transaction (i.e. an already committed one, or an ancestor, or another proposed one).
    pub fn check_proposed(
        &mut self,
        intent_hash: &IntentHash,
    ) -> Result<(), IntentHashDuplicateWith> {
        if let Some(duplicate_with) = self.recorded_intent_hashes.get(intent_hash) {
            return Err(duplicate_with.clone());
        }
        let committed_at_version = self.store.get_txn_state_version_by_identifier(intent_hash);
        if committed_at_version.is_some() {
            // we insert this to our map only as an optimization (avoid repeating the same DB read)
            self.recorded_intent_hashes
                .insert(*intent_hash, IntentHashDuplicateWith::Committed);
            return Err(IntentHashDuplicateWith::Committed);
        }
        Ok(())
    }

    /// Records an intent hash of a proposed transaction which is expected to be committed.
    /// From this point on, it will be taken into account by the `check_proposed()` method.
    pub fn record_committable_proposed(&mut self, intent_hash: IntentHash) {
        self.recorded_intent_hashes
            .insert(intent_hash, IntentHashDuplicateWith::Proposed);
    }
}

/// An internal delegate for executing a series of consecutive transactions while tracking their
/// progress.
struct TransactionSeriesExecutor<'s, S> {
    store: &'s S,
    execution_cache: &'s Mutex<ExecutionCache>,
    execution_configurator: &'s ExecutionConfigurator,
    epoch_transaction_identifiers: EpochTransactionIdentifiers,
    state_tracker: StateTracker,
}

impl<'s, S: ReadableStore> TransactionSeriesExecutor<'s, S> {
    /// Creates a new executor for a lifetime of entire transaction batch execution (i.e. for all
    /// transactions in a prepared vertex, or in a commit request).
    /// The borrowed `store` should be already locked (i.e. the epoch and base state of the ledger
    /// passed here should have been read under the same lock; and final database writes, if any,
    /// should also be performed under the same lock).
    /// The locking of the borrowed `execution_cache` will be handled by this executor.
    pub fn new(
        store: &'s S,
        execution_cache: &'s Mutex<ExecutionCache>,
        execution_configurator: &'s ExecutionConfigurator,
        epoch_transaction_identifiers: EpochTransactionIdentifiers,
        base_state_version: StateVersion,
        base_ledger_hashes: LedgerHashes,
    ) -> Self {
        Self {
            store,
            execution_cache,
            execution_configurator,
            epoch_transaction_identifiers,
            state_tracker: StateTracker::initial(base_state_version, base_ledger_hashes),
        }
    }

    /// Executes the given already-validated ledger transaction (against the borrowed `store` and
    /// `execution_cache`).
    /// Uses an internal [`StateTracker`] to track the progression of *committable* transactions.
    /// The passed description will only be used for logging/errors/panics (and will be augmented by
    /// the transaction's ledger hash).
    pub fn execute(
        &mut self,
        config_type: ConfigType,
        transaction: &ValidatedLedgerTransaction,
        description: impl Display,
    ) -> Result<ProcessedCommitResult, RejectResult> {
        let description = DescribedTransactionHash {
            ledger_hash: transaction.ledger_transaction_hash(),
            description,
        };
        let mut execution_cache = self.execution_cache.lock();
        let processed = execution_cache.execute_transaction(
            self.store,
            &self.epoch_transaction_identifiers,
            self.state_tracker.state_version,
            &self.state_tracker.ledger_hashes.transaction_root,
            &transaction.ledger_transaction_hash(),
            self.execution_configurator
                .wrap(transaction.get_executable(), config_type)
                .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, &description),
        );
        let result = processed.expect_commit_or_reject(&description).cloned();
        if let Ok(commit) = &result {
            self.state_tracker.update(commit);
        }
        result
    }

    pub fn latest_ledger_hashes(&self) -> &LedgerHashes {
        &self.state_tracker.ledger_hashes
    }

    pub fn latest_state_version(&self) -> StateVersion {
        self.state_tracker.state_version
    }

    pub fn next_epoch(&self) -> Option<&NextEpoch> {
        self.state_tracker.next_epoch.as_ref()
    }
}

struct DescribedTransactionHash<D> {
    ledger_hash: LedgerTransactionHash,
    description: D,
}

impl<D: Display> Display for DescribedTransactionHash<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} (ledger hash {})", self.description, self.ledger_hash)
    }
}
