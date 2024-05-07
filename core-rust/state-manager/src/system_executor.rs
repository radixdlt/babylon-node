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

use crate::query::*;

use crate::store::traits::*;
use crate::transaction::*;

use crate::*;

use crate::engine_prelude::*;

use node_common::locks::DbLock;

use tracing::info;

use crate::store::traits::scenario::{
    DescribedAddressRendering, ExecutedScenario, ExecutedScenarioStore, ExecutedScenarioTransaction,
};
use crate::system_commits::*;

use radix_transaction_scenarios::executor::scenarios_vector;

use crate::protocol::{ProtocolUpdateTransactionBatch, ProtocolVersionName};
use crate::traits::scenario::ExecutedScenarioV1;
use std::sync::Arc;
use std::time::Instant;

pub struct SystemExecutor {
    network: NetworkDefinition,
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    preparator: Arc<Preparator>,
    committer: Arc<Committer>,
}

impl SystemExecutor {
    pub fn new(
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        preparator: Arc<Preparator>,
        committer: Arc<Committer>,
    ) -> Self {
        Self {
            network: network.clone(),
            database,
            preparator,
            committer,
        }
    }

    /// Creates and commits a series of genesis transactions (i.e. a bootstrap, then potentially many
    /// data ingestion chunks, and then a wrap-up).
    #[allow(clippy::too_many_arguments)]
    pub fn execute_genesis(
        &self,
        genesis_data_chunks: Vec<GenesisDataChunk>,
        initial_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_timestamp_ms: i64,
        genesis_opaque_hash: Hash,
        faucet_supply: Decimal,
        genesis_scenarios: Vec<String>,
    ) -> LedgerProof {
        let start_instant = Instant::now();

        let database = self.database.lock();
        if database.get_post_genesis_epoch_proof().is_some() {
            panic!("Can't execute genesis: database already initialized")
        }
        let maybe_top_txn_identifiers = database.get_top_transaction_identifiers();
        drop(database);

        if let Some(top_txn_identifiers) = maybe_top_txn_identifiers {
            // No epoch proof, but there are some committed txns
            panic!(
                "The database is in inconsistent state: \
                there are committed transactions (up to state version {}), but there's no epoch proof. \
                This is likely caused by the the genesis data ingestion being interrupted. \
                Consider wiping your database dir and trying again.", top_txn_identifiers.0);
        }

        let mut system_commit_request_factory = SystemCommitRequestFactory {
            epoch: initial_epoch,
            timestamp: initial_timestamp_ms,
            state_version: StateVersion::pre_genesis(),
            proof_origin: LedgerProofOrigin::Genesis {
                genesis_opaque_hash,
            },
        };

        info!("Committing system flash");
        let prepare_result = self.preparator.prepare_genesis(GenesisTransaction::Flash);
        let commit_request = system_commit_request_factory.create(prepare_result);
        self.committer.commit_system(commit_request);

        info!("Committing system bootstrap");
        let transaction = create_system_bootstrap_transaction(
            initial_epoch,
            initial_config,
            initial_timestamp_ms,
            // Leader gets set to None, to be fixed at the first proper round change.
            None,
            faucet_supply,
        );
        let prepare_result = self
            .preparator
            .prepare_genesis(GenesisTransaction::Transaction(Box::new(transaction)));
        let commit_request = system_commit_request_factory.create(prepare_result);
        self.committer.commit_system(commit_request);

        let genesis_data_chunks_len = genesis_data_chunks.len();
        for (index, chunk) in genesis_data_chunks.into_iter().enumerate() {
            let chunk_type = match chunk {
                GenesisDataChunk::Validators(_) => "validators",
                GenesisDataChunk::Stakes { .. } => "stakes",
                GenesisDataChunk::Resources(_) => "resources",
                GenesisDataChunk::ResourceBalances { .. } => "resource_balances",
                GenesisDataChunk::XrdBalances(_) => "xrd_balances",
            };
            info!(
                "Committing data ingestion chunk ({}) {} of {}",
                chunk_type,
                index + 1,
                genesis_data_chunks_len
            );
            let transaction =
                create_genesis_data_ingestion_transaction(&GENESIS_HELPER, chunk, index);
            let prepare_result = self
                .preparator
                .prepare_genesis(GenesisTransaction::Transaction(Box::new(transaction)));
            let commit_request = system_commit_request_factory.create(prepare_result);
            self.committer.commit_system(commit_request);
        }

        self.execute_genesis_scenarios(&mut system_commit_request_factory, genesis_scenarios);

        info!("Committing genesis wrap-up");
        let transaction: SystemTransactionV1 = create_genesis_wrap_up_transaction();
        let prepare_result = self
            .preparator
            .prepare_genesis(GenesisTransaction::Transaction(Box::new(transaction)));
        let commit_request = system_commit_request_factory.create(prepare_result);
        let final_ledger_proof = commit_request.proof.clone();
        self.committer.commit_system(commit_request);

        info!(
            "Genesis transactions successfully executed in {:?}",
            start_instant.elapsed()
        );
        final_ledger_proof
    }

    pub fn execute_protocol_update_batch(
        &self,
        protocol_version: &ProtocolVersionName,
        batch_idx: u32,
        batch: ProtocolUpdateTransactionBatch,
    ) {
        let database = self.database.lock();
        let latest_header = database
            .get_latest_proof()
            .expect("Pre-genesis protocol updates are currently not supported")
            .ledger_header;
        drop(database);

        // Currently, protocol updates are always executed at epoch boundary. This means that:
        // - at the update's first batch, we assume the latest header to contain an epoch change,
        //   and we advance to the next epoch;
        // - at any consecutive batches, we will simply use the epoch from the latest header (i.e.
        //   the one generated for the first batch).
        let epoch = latest_header
            .next_epoch
            .map(|next_epoch| next_epoch.epoch)
            .unwrap_or(latest_header.epoch);

        let mut system_commit_request_factory = SystemCommitRequestFactory {
            epoch,
            timestamp: latest_header.proposer_timestamp_ms,
            state_version: latest_header.state_version,
            proof_origin: LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name: protocol_version.clone(),
                batch_idx,
            },
        };

        match batch {
            ProtocolUpdateTransactionBatch::FlashTransactions(flash_transactions) => {
                let prepare_result = self.preparator.prepare_protocol_update(flash_transactions);
                let commit_request = system_commit_request_factory.create(prepare_result);
                self.committer.commit_system(commit_request);
            }
            ProtocolUpdateTransactionBatch::Scenario(scenario) => {
                // Note: here we re-use the batch_idx as a nonce, since we need a growing number
                // that survives Node restarts. It is not going to be the same as the manual nonce
                // used for Genesis Scenarios:
                // - it is not guaranteed to start with 0 (in practice, it will be 1);
                // - it will be incremented only by 1 for each consecutive Scenario (while during
                //   Genesis Scenarios, the next Scenario gets a nonce incremented by the number of
                //   transactions of the previous Scenario).
                // TODO(resolve during review): is the above ok?
                let protocol_update_specific_nonce = batch_idx;
                self.execute_scenario(
                    &mut system_commit_request_factory,
                    scenario.as_str(),
                    protocol_update_specific_nonce,
                );
            }
        }
    }

    // NOTE:
    // Execution of Genesis Scenarios differs in one important detail from regular Scenarios: they
    // are simply executed in a series of commits that happen right after Genesis, without any
    // progress being tracked in the database (neither intermediate, nor final). Actually, the same
    // is true about the Genesis itself. Thus, if this process is interrupted e.g. by a Node
    // restart, it may result in an inconsistent DB state (e.g. incomplete Scenario results, or even
    // panics on boot-up, requiring ledger wipe). We are currently fine with this, since a wipe of
    // an empty ledger does not hurt that much.
    fn execute_genesis_scenarios(
        &self,
        system_commit_request_factory: &mut SystemCommitRequestFactory,
        scenarios: Vec<String>,
    ) {
        if !scenarios.is_empty() {
            info!("Running {} scenarios", scenarios.len());
            let mut next_nonce: u32 = 0;
            for scenario in scenarios {
                next_nonce = self.execute_scenario(
                    system_commit_request_factory,
                    scenario.as_str(),
                    next_nonce,
                );
            }
            info!("Scenarios finished");
        }
    }

    fn execute_scenario(
        &self,
        system_commit_request_factory: &mut SystemCommitRequestFactory,
        scenario_name: &str,
        nonce: u32,
    ) -> u32 {
        let scenario =
            self.find_scenario(system_commit_request_factory.epoch, nonce, scenario_name);
        info!("Running scenario: {}", scenario_name);
        let (
            prepare_result,
            PreparedScenarioMetadata {
                committed_transaction_names,
                end_state:
                    EndState {
                        next_unused_nonce,
                        output,
                    },
            },
        ) = self.preparator.prepare_scenario(scenario_name, scenario);

        // Note:
        // We want to store the information on each executed scenario in the DB (for inspection).
        // Ideally, we should write it atomically in the same batch as the `commit_system()` call
        // below - however, it would require breaking some abstraction (i.e. atomicity is currently
        // driven by the DB layer) or making an API exception for the Scenarios (i.e. accept some
        // extra params in the commit request). Since Scenarios only exist for test purposes, we
        // chose to simply write it non-atomically here, before commit. Worst case (i.e. if Node is
        // restarted right after this line), we will see a duplicate entry in the informative-only
        // "executed Scenarios" table.
        let executed_scenario = self.create_executed_scenario_entry(
            scenario_name,
            &prepare_result.committed_transactions,
            committed_transaction_names,
            output,
        );
        log_executed_scenario_details(&executed_scenario);
        let database = self.database.lock();
        database.put_next_scenario(executed_scenario);
        drop(database);

        if prepare_result.committed_transactions.is_empty() {
            info!(
                "No committable transaction for Scenario {}; skipping it",
                scenario_name
            );
        } else {
            let commit_request = system_commit_request_factory
                .create(prepare_result)
                .require_committed_successes(false);
            self.committer.commit_system(commit_request);
        }

        next_unused_nonce
    }

    fn find_scenario(
        &self,
        epoch: Epoch,
        next_nonce: u32,
        scenario_name: &str,
    ) -> Box<dyn ScenarioInstance> {
        for (_protocol_version, scenario_builder) in scenarios_vector() {
            let scenario =
                scenario_builder(ScenarioCore::new(self.network.clone(), epoch, next_nonce));
            if scenario.metadata().logical_name == scenario_name {
                return scenario;
            }
        }
        panic!(
            "Could not find scenario with logical name: {}",
            scenario_name
        );
    }

    fn create_executed_scenario_entry(
        &self,
        logical_name: &str,
        committed_transactions: &[RawAndValidatedTransaction],
        committed_transaction_names: Vec<(StateVersion, String)>,
        output: ScenarioOutput,
    ) -> ExecutedScenario {
        let encoder = AddressBech32Encoder::new(&self.network);
        ExecutedScenarioV1 {
            logical_name: logical_name.to_string(),
            committed_transactions: committed_transactions
                .iter()
                .zip(committed_transaction_names)
                .map(
                    |(transaction, (state_version, logical_name))| ExecutedScenarioTransaction {
                        logical_name,
                        state_version,
                        intent_hash: transaction.validated.intent_hash_if_user().unwrap(),
                    },
                )
                .collect(),
            addresses: output
                .interesting_addresses
                .0
                .into_iter()
                .map(|(descriptor, address)| DescribedAddressRendering {
                    logical_name: descriptor,
                    rendered_address: match address {
                        DescribedAddress::Global(address) => address.to_string(&encoder),
                        DescribedAddress::Internal(address) => address.to_string(&encoder),
                        DescribedAddress::NonFungible(nf_global_id) => {
                            nf_global_id.to_string(&encoder)
                        }
                    },
                })
                .collect(),
        }
    }
}

fn log_executed_scenario_details(executed_scenario: &ExecutedScenario) {
    let ExecutedScenarioV1 {
        logical_name,
        committed_transactions,
        addresses,
    } = executed_scenario;
    for committed_transaction in committed_transactions {
        let ExecutedScenarioTransaction {
            logical_name,
            state_version,
            intent_hash,
        } = committed_transaction;
        info!(
            "Committed {} at state version {} ({:?})",
            logical_name, state_version, intent_hash
        );
    }
    info!(
        "Completed committing {} transactions for scenario {}, with resultant addresses:\n{}",
        committed_transactions.len(),
        logical_name,
        addresses
            .iter()
            .map(|address| format!("  - {}: {}", address.logical_name, address.rendered_address))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ONLY TESTS BELOW

#[cfg(test)]
impl SystemExecutor {
    /// Performs an [`execute_genesis()`] with a hardcoded genesis data meant for test purposes.
    pub fn execute_genesis_for_unit_tests_with_config(
        &self,
        consensus_manager_config: ConsensusManagerConfig,
    ) -> LedgerProof {
        // Roughly copied from bootstrap_test_default in scrypto
        let genesis_validator: GenesisValidator = Secp256k1PublicKey([0; 33]).into();
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
        let initial_timestamp_ms = 1;
        self.execute_genesis(
            genesis_chunks,
            initial_epoch,
            consensus_manager_config,
            initial_timestamp_ms,
            Hash([0; Hash::LENGTH]),
            *DEFAULT_TESTING_FAUCET_SUPPLY,
            vec![],
        )
    }

    /// Performs an [`execute_genesis_for_unit_tests_with_config()`] with a hardcoded config.
    pub fn execute_genesis_for_unit_tests_with_default_config(&self) -> LedgerProof {
        let default_config = ConsensusManagerConfig {
            max_validators: 10,
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
        self.execute_genesis_for_unit_tests_with_config(default_config)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::engine_prelude::*;
    use crate::transaction::{LedgerTransaction, RoundUpdateTransactionV1};
    use crate::{
        LedgerProof, PrepareRequest, PrepareResult, RoundHistory, StateManager, StateManagerConfig,
    };
    use node_common::config::limits::VertexLimitsConfig;

    use crate::test::create_state_manager;
    use tempfile::TempDir;

    // TODO: maybe move/refactor testing infra as we add more Rust tests
    fn build_unit_test_round_history(proof: &LedgerProof) -> RoundHistory {
        RoundHistory {
            is_fallback: false,
            epoch: proof.ledger_header.epoch,
            round: Round::of(proof.ledger_header.round.number() + 1),
            gap_round_leader_addresses: Vec::new(),
            proposer_address: proof
                .ledger_header
                .next_epoch
                .clone()
                .unwrap()
                .validator_set[0]
                .address,
            proposer_timestamp_ms: proof.ledger_header.proposer_timestamp_ms,
        }
    }

    fn build_unit_test_prepare_request(
        proof: &LedgerProof,
        proposed_transactions: Vec<RawNotarizedTransaction>,
    ) -> PrepareRequest {
        PrepareRequest {
            committed_ledger_hashes: proof.ledger_header.hashes,
            ancestor_transactions: Vec::new(),
            ancestor_ledger_hashes: proof.ledger_header.hashes,
            proposed_transactions,
            round_history: build_unit_test_round_history(proof),
        }
    }

    fn build_committable_transaction(epoch: Epoch, nonce: u32) -> RawNotarizedTransaction {
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let notary_private_key = Secp256k1PrivateKey::from_u64(2).unwrap();

        TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: epoch,
                end_epoch_exclusive: epoch.after(100).unwrap(),
                nonce,
                notary_public_key: notary_private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .get_free_xrd_from_faucet()
                    .try_deposit_entire_worktop_or_abort(
                        ComponentAddress::virtual_account_from_public_key(
                            &sig_1_private_key.public_key(),
                        ),
                        None,
                    )
                    .build(),
            )
            .sign(&sig_1_private_key)
            .notarize(&notary_private_key)
            .build()
            .to_raw()
            .unwrap()
    }

    fn build_rejected_transaction(epoch: Epoch, nonce: u32) -> RawNotarizedTransaction {
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let notary_private_key = Secp256k1PrivateKey::from_u64(2).unwrap();

        TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: epoch,
                end_epoch_exclusive: epoch.after(100).unwrap(),
                nonce,
                notary_public_key: notary_private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new().get_free_xrd_from_faucet().build())
            .sign(&sig_1_private_key)
            .notarize(&notary_private_key)
            .build()
            .to_raw()
            .unwrap()
    }

    fn setup_state_manager(
        tmp: &TempDir,
        vertex_limits_config: VertexLimitsConfig,
    ) -> (LedgerProof, StateManager) {
        let config = StateManagerConfig {
            vertex_limits_config: Some(vertex_limits_config),
            ..StateManagerConfig::new_for_testing(tmp.path().to_str().unwrap())
        };
        let state_manager = create_state_manager(config);

        let proof = state_manager
            .system_executor
            .execute_genesis_for_unit_tests_with_default_config();

        (proof, state_manager)
    }

    fn prepare_with_vertex_limits(
        tmp: &TempDir,
        vertex_limits_config: VertexLimitsConfig,
        proposed_transactions: Vec<RawNotarizedTransaction>,
    ) -> PrepareResult {
        let (proof, state_manager) = setup_state_manager(tmp, vertex_limits_config);
        state_manager
            .preparator
            .prepare(build_unit_test_prepare_request(
                &proof,
                proposed_transactions,
            ))
    }

    fn compute_consumed_execution_units(
        state_manager: &StateManager,
        prepare_request: PrepareRequest,
    ) -> u32 {
        let database = state_manager.database.snapshot();
        let mut series_executor = state_manager
            .transaction_executor_factory
            .start_series_execution(database.deref());

        let round_update = RoundUpdateTransactionV1::new(
            series_executor.epoch_header(),
            &prepare_request.round_history,
        );
        let ledger_round_update = LedgerTransaction::RoundUpdateV1(Box::new(round_update));
        let validated_round_update = state_manager
            .ledger_transaction_validator
            .validate_user_or_round_update_from_model(&ledger_round_update)
            .expect("expected to be able to prepare the round update transaction");

        let round_update_result = series_executor
            .execute_and_update_state(&validated_round_update, "cost computation - round update")
            .expect("round update rejected");

        prepare_request
            .proposed_transactions
            .iter()
            .map(|raw_user_transaction| {
                let (_, prepared_transaction) = state_manager
                    .preparator
                    .try_prepare_ledger_transaction_from_user_transaction(raw_user_transaction)
                    .unwrap();

                let validated = state_manager
                    .ledger_transaction_validator
                    .validate_user_or_round_update(prepared_transaction)
                    .unwrap();

                let execute_result =
                    series_executor.execute_and_update_state(&validated, "cost computation");

                match execute_result {
                    Ok(commit) => {
                        commit
                            .local_receipt
                            .local_execution
                            .fee_summary
                            .total_execution_cost_units_consumed
                    }
                    Err(reject) => reject.fee_summary.total_execution_cost_units_consumed,
                }
            })
            .sum::<u32>()
            + round_update_result
                .local_receipt
                .local_execution
                .fee_summary
                .total_execution_cost_units_consumed
    }

    #[test]
    fn test_prepare_vertex_limits() {
        let tmp = tempfile::tempdir().unwrap();
        let (proof, state_manager) = setup_state_manager(&tmp, VertexLimitsConfig::max());

        let mut proposed_transactions = Vec::new();
        let epoch = proof.ledger_header.epoch;
        proposed_transactions.push(build_committable_transaction(epoch, 1));
        proposed_transactions.push(build_committable_transaction(epoch, 2));
        proposed_transactions.push(build_rejected_transaction(epoch, 1));
        proposed_transactions.push(build_committable_transaction(epoch, 3));
        proposed_transactions.push(build_committable_transaction(epoch, 4));
        proposed_transactions.push(build_rejected_transaction(epoch, 2));
        proposed_transactions.push(build_committable_transaction(epoch, 5));
        proposed_transactions.push(build_rejected_transaction(epoch, 3));
        proposed_transactions.push(build_committable_transaction(epoch, 6));
        proposed_transactions.push(build_committable_transaction(epoch, 7));
        proposed_transactions.push(build_rejected_transaction(epoch, 4));
        proposed_transactions.push(build_committable_transaction(epoch, 8));
        proposed_transactions.push(build_committable_transaction(epoch, 9));
        proposed_transactions.push(build_rejected_transaction(epoch, 5));

        let prepare_result = state_manager
            .preparator
            .prepare(build_unit_test_prepare_request(
                &proof,
                proposed_transactions.clone(),
            ));

        assert_eq!(prepare_result.committed.len(), 10); // 9 committable transactions + 1 round update transaction
        assert_eq!(prepare_result.rejected.len(), 5); // 5 rejected transactions

        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig {
                max_transaction_count: 6,
                ..VertexLimitsConfig::max()
            },
            proposed_transactions.clone(),
        );

        assert_eq!(prepare_result.committed.len(), 6); // same as the limit
                                                       // only first 7 (5 committable) transactions are executed before the limit is hit, at which point we have encountered only 2 rejected transactions
        assert_eq!(prepare_result.rejected.len(), 2);

        let limited_proposal_ledger_hashes = prepare_result.ledger_hashes;

        // We now compute PrepareResult only for the first 7 transactions in order to test that indeed resultant states are the same.
        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig::max(),
            proposed_transactions.clone()[0..7].to_vec(),
        );
        assert_eq!(prepare_result.committed.len(), 6);
        assert_eq!(prepare_result.rejected.len(), 2);
        assert_eq!(prepare_result.ledger_hashes, limited_proposal_ledger_hashes);

        // Transaction size/count only tests `check_pre_execution`. We also need to test `try_next_transaction`.
        let tmp = tempfile::tempdir().unwrap();
        let cost_for_first_9_user_transactions = compute_consumed_execution_units(
            &setup_state_manager(&tmp, VertexLimitsConfig::max()).1,
            build_unit_test_prepare_request(&proof, proposed_transactions.clone()[0..9].to_vec()),
        );
        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig {
                // We add an extra cost unit in order to not trigger the LimitExceeded right at 9th transaction.
                max_total_execution_cost_units_consumed: cost_for_first_9_user_transactions + 1,
                ..VertexLimitsConfig::max()
            },
            proposed_transactions.clone(),
        );
        assert_eq!(prepare_result.committed.len(), 7); // in the first 9 proposed transactions we have 6 that gets committed + 1 round update transaction
        assert_eq!(prepare_result.rejected.len(), 4); // 3 rejected transactions + last one that is committable but gets discarded due to limits

        let limited_proposal_ledger_hashes = prepare_result.ledger_hashes;
        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig::max(),
            proposed_transactions.clone()[0..9].to_vec(),
        );

        // Should be identical to previous prepare run (cost limited)
        assert_eq!(prepare_result.committed.len(), 7);
        assert_eq!(prepare_result.rejected.len(), 3);
        assert_eq!(prepare_result.ledger_hashes, limited_proposal_ledger_hashes);
    }
}
