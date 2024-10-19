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

use crate::prelude::*;

use crate::system_commits::*;

use radix_transaction_scenarios::scenarios::get_scenario;

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

    pub fn execute_protocol_update_action(
        &self,
        batch_details: ProtocolUpdateBatchDetails,
        batch: ProtocolUpdateNodeBatch,
    ) {
        match batch {
            ProtocolUpdateNodeBatch::ProtocolUpdateBatch(batch) => {
                let prepare_result = self
                    .preparator
                    .prepare_protocol_update(&batch_details, batch);
                let commit_request = batch_details
                    .create_system_commit_factory()
                    .create(prepare_result);
                self.committer.commit_system(commit_request);
            }
            ProtocolUpdateNodeBatch::Scenario(scenario) => {
                let starting_nonce =
                    if batch_details.protocol_version != &ProtocolVersionName::babylon() {
                        // For non-genesis, we use the top-of-ledger's state version as a starting nonce for the
                        // Scenario's transactions. This behavior is different than the Engine's default
                        // Scenario executor's (which increments the last nonce used by the previous
                        // Scenario).
                        // But the ever-incrementing state version is good enough for transaction deduplication purposes.
                        // And if the state version ever gets larger than a u32, we are fine with this wrapping around.
                        batch_details.start_state_identifiers.state_version.number() as u32
                    } else {
                        // Annoyingly, genesis scenarios used a different strategy for their nonces,
                        // so for backwards compatibility, we need to resolve them from <previous transaction + 1>
                        let database = self.database.lock();
                        let top_of_ledger = database.get_top_transaction_identifiers().unwrap().0;
                        let raw_last_transaction =
                            database.get_committed_transaction(top_of_ledger).unwrap();
                        let typed = LedgerTransaction::from_raw(&raw_last_transaction).unwrap();

                        match typed {
                            LedgerTransaction::UserV1(user) => {
                                user.signed_intent.intent.header.nonce + 1
                            }
                            _ => 0,
                        }
                    };
                self.execute_scenario(batch_details, scenario.as_str(), starting_nonce);
            }
        }
    }

    fn execute_scenario(
        &self,
        batch_details: ProtocolUpdateBatchDetails,
        scenario_name: &str,
        starting_nonce: u32,
    ) -> u32 {
        let scenario = self.find_scenario(
            batch_details.start_state_identifiers.epoch,
            starting_nonce,
            scenario_name,
        );
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
        ) = self.preparator.prepare_scenario(
            batch_details.to_batch_situation(),
            scenario_name,
            scenario,
        );

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
            let commit_request = batch_details
                .create_system_commit_factory()
                .create(prepare_result)
                .require_committed_successes(false);
            self.committer.commit_system(commit_request);
        }

        next_unused_nonce
    }

    fn find_scenario(
        &self,
        epoch: Epoch,
        starting_nonce: u32,
        scenario_name: &str,
    ) -> Box<dyn ScenarioInstance> {
        get_scenario(scenario_name).create(ScenarioCore::new(
            self.network.clone(),
            epoch,
            starting_nonce,
        ))
    }

    fn create_executed_scenario_entry(
        &self,
        logical_name: &str,
        committed_transactions: &[ProcessedLedgerTransaction],
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
                        transaction_intent_hash: transaction
                            .hashes
                            .as_user()
                            .unwrap()
                            .transaction_intent_hash,
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
            transaction_intent_hash: intent_hash,
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

pub struct ProtocolUpdateBatchDetails<'a> {
    pub protocol_version: &'a ProtocolVersionName,
    pub config_hash: Hash,
    pub batch_group_index: usize,
    pub batch_group_descriptor: &'a str,
    pub batch_index: usize,
    pub is_final_batch: bool,
    pub start_state_identifiers: StartStateIdentifiers,
}

impl<'a> ProtocolUpdateBatchDetails<'a> {
    fn create_system_commit_factory(&self) -> SystemCommitRequestFactory {
        SystemCommitRequestFactory {
            epoch: self.start_state_identifiers.epoch,
            timestamp: self.start_state_identifiers.proposer_timestamp_ms,
            state_version: self.start_state_identifiers.state_version,
            proof_origin: self.to_ledger_proof_origin(),
            batch_situation: self.to_batch_situation(),
        }
    }

    fn to_ledger_proof_origin(&self) -> LedgerProofOrigin {
        LedgerProofOrigin::ProtocolUpdate {
            protocol_version_name: self.protocol_version.clone(),
            config_hash: Some(self.config_hash),
            batch_group_index: self.batch_group_index,
            batch_group_descriptor: self.batch_group_descriptor.to_string(),
            batch_index: self.batch_index,
            is_end_of_update: self.is_final_batch,
        }
    }

    pub fn to_batch_situation(&self) -> BatchSituation {
        BatchSituation::ProtocolUpdate {
            update: self.protocol_version.clone(),
            is_final_batch: self.is_final_batch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::*;
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
                        ComponentAddress::preallocated_account_from_public_key(
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
        let state_manager = create_bootstrapped_state_manager_with_rounds_per_epoch(config, 3);
        let epoch_proof = state_manager
            .database
            .lock()
            .get_post_genesis_epoch_proof()
            .unwrap();
        (epoch_proof, state_manager)
    }

    fn prepare_with_vertex_limits(
        tmp: &TempDir,
        vertex_limits_config: VertexLimitsConfig,
        proposed_transactions: Vec<RawNotarizedTransaction>,
    ) -> PrepareResult {
        let (epoch_proof, state_manager) = setup_state_manager(tmp, vertex_limits_config);
        state_manager
            .preparator
            .prepare(build_unit_test_prepare_request(
                &epoch_proof,
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

        let round_update = create_round_update_transaction(
            series_executor.epoch_header(),
            &prepare_request.round_history,
        );
        let ledger_round_update = LedgerTransaction::RoundUpdateV1(Box::new(round_update));
        let round_update_executable = ledger_round_update
            .to_raw()
            .unwrap()
            .create_identifiable_ledger_executable(
                state_manager.transaction_validator.read().deref(),
                AcceptedLedgerTransactionKind::UserOrValidator,
            )
            .expect("expected to be able to prepare the round update transaction");

        let round_update_result = series_executor
            .execute_and_update_state(
                &round_update_executable.executable,
                &round_update_executable.hashes,
                "cost computation - round update",
            )
            .expect("round update rejected");

        prepare_request
            .proposed_transactions
            .iter()
            .map(|raw_user_transaction| {
                let (_, executable, hashes) = state_manager
                    .preparator
                    .prepare_known_valid_raw_user_transaction(raw_user_transaction);

                let execute_result = series_executor.execute_and_update_state(
                    &executable,
                    &hashes,
                    "cost computation",
                );

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
