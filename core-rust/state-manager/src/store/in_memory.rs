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

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::store::traits::extensions::*;
use crate::store::traits::*;
use crate::transaction::{
    LedgerTransactionHash, RawLedgerTransaction, TypedTransactionIdentifiers,
};
use crate::{
    CommittedTransactionIdentifiers, LedgerProof, LedgerTransactionReceipt,
    LocalTransactionExecution, LocalTransactionReceipt, ReceiptTreeHash, StateVersion,
    TransactionTreeHash,
};

use crate::query::TransactionIdentifierLoader;
use crate::store::traits::scenario::{
    ExecutedGenesisScenario, ExecutedGenesisScenarioStore, ScenarioSequenceNumber,
};
use core::ops::Bound::{Included, Unbounded};
use node_common::utils::IsAccountExt;
use radix_engine_common::types::{Epoch, GlobalAddress, NodeId};
use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DbPartitionKey, DbSortKey, DbSubstateValue, PartitionEntry,
    SubstateDatabase,
};
use radix_engine_stores::hash_tree::tree_store::*;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use transaction::model::*;

#[derive(Debug)]
pub struct InMemoryStore {
    flags: DatabaseFlags,
    transactions: BTreeMap<StateVersion, RawLedgerTransaction>,
    transaction_identifiers: BTreeMap<StateVersion, CommittedTransactionIdentifiers>,
    ledger_receipts: BTreeMap<StateVersion, LedgerTransactionReceipt>,
    local_transaction_executions: BTreeMap<StateVersion, LocalTransactionExecution>,
    transaction_intent_lookup: HashMap<IntentHash, StateVersion>,
    user_payload_hash_lookup: HashMap<NotarizedTransactionHash, StateVersion>,
    ledger_payload_hash_lookup: HashMap<LedgerTransactionHash, StateVersion>,
    proofs: BTreeMap<StateVersion, LedgerProof>,
    epoch_proofs: BTreeMap<Epoch, LedgerProof>,
    vertex_store: Option<Vec<u8>>,
    substate_store: InMemorySubstateDatabase,
    node_ancestry_records: BTreeMap<NodeId, SubstateNodeAncestryRecord>,
    tree_node_store: SerializedInMemoryTreeStore,
    transaction_tree_slices: BTreeMap<StateVersion, TreeSlice<TransactionTreeHash>>,
    receipt_tree_slices: BTreeMap<StateVersion, TreeSlice<ReceiptTreeHash>>,
    account_change_index_last_state_version: StateVersion,
    account_change_index_set: HashMap<GlobalAddress, BTreeSet<StateVersion>>,
    executed_genesis_scenarios: BTreeMap<ScenarioSequenceNumber, ExecutedGenesisScenario>,
}

impl InMemoryStore {
    pub fn new(flags: DatabaseFlags) -> InMemoryStore {
        InMemoryStore {
            flags,
            transactions: BTreeMap::new(),
            transaction_identifiers: BTreeMap::new(),
            ledger_receipts: BTreeMap::new(),
            local_transaction_executions: BTreeMap::new(),
            transaction_intent_lookup: HashMap::new(),
            user_payload_hash_lookup: HashMap::new(),
            ledger_payload_hash_lookup: HashMap::new(),
            proofs: BTreeMap::new(),
            epoch_proofs: BTreeMap::new(),
            vertex_store: None,
            substate_store: InMemorySubstateDatabase::standard(),
            node_ancestry_records: BTreeMap::new(),
            tree_node_store: SerializedInMemoryTreeStore::new(),
            transaction_tree_slices: BTreeMap::new(),
            receipt_tree_slices: BTreeMap::new(),
            account_change_index_last_state_version: StateVersion::pre_genesis(),
            account_change_index_set: HashMap::new(),
            executed_genesis_scenarios: BTreeMap::new(),
        }
    }

    fn insert_transaction(
        &mut self,
        state_version: StateVersion,
        transaction: RawLedgerTransaction,
        receipt: LocalTransactionReceipt,
        identifiers: CommittedTransactionIdentifiers,
    ) {
        if self.is_account_change_index_enabled() {
            self.update_account_change_index_from_receipt(state_version, &receipt.local_execution);
        }

        if let TypedTransactionIdentifiers::User {
            intent_hash,
            notarized_transaction_hash,
            ..
        } = &identifiers.payload.typed
        {
            let key_already_exists = self.transaction_intent_lookup.get(intent_hash);
            if let Some(existing_payload_hash) = key_already_exists {
                panic!(
                    "Attempted to save intent hash which already exists: {existing_payload_hash:?}"
                );
            }
            self.transaction_intent_lookup
                .insert(*intent_hash, state_version);
            self.user_payload_hash_lookup
                .insert(*notarized_transaction_hash, state_version);
        }

        self.ledger_payload_hash_lookup
            .insert(identifiers.payload.ledger_payload_hash, state_version);

        self.transactions.insert(state_version, transaction);
        self.ledger_receipts
            .insert(state_version, receipt.on_ledger);
        self.local_transaction_executions
            .insert(state_version, receipt.local_execution);
        self.transaction_identifiers
            .insert(state_version, identifiers);
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new(DatabaseFlags::default())
    }
}

impl ConfigurableDatabase for InMemoryStore {
    fn read_flags_state(&self) -> DatabaseFlagsState {
        DatabaseFlagsState {
            account_change_index_enabled: None,
            local_transaction_execution_index_enabled: None,
        }
    }

    fn write_flags(&mut self, _flags: &DatabaseFlags) {
        // We don't need to do anything for in memory store
    }

    fn is_account_change_index_enabled(&self) -> bool {
        self.flags.enable_account_change_index
    }

    fn is_local_transaction_execution_index_enabled(&self) -> bool {
        self.flags.enable_local_transaction_execution_index
    }
}

impl WriteableVertexStore for InMemoryStore {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        self.vertex_store = Some(vertex_store_bytes);
    }
}

impl RecoverableVertexStore for InMemoryStore {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        self.vertex_store.clone()
    }
}

impl TransactionIndex<&IntentHash> for InMemoryStore {
    fn get_txn_state_version_by_identifier(&self, identifier: &IntentHash) -> Option<StateVersion> {
        self.transaction_intent_lookup.get(identifier).cloned()
    }
}

impl TransactionIndex<&NotarizedTransactionHash> for InMemoryStore {
    fn get_txn_state_version_by_identifier(
        &self,
        identifier: &NotarizedTransactionHash,
    ) -> Option<StateVersion> {
        self.user_payload_hash_lookup.get(identifier).cloned()
    }
}

impl TransactionIndex<&LedgerTransactionHash> for InMemoryStore {
    fn get_txn_state_version_by_identifier(
        &self,
        identifier: &LedgerTransactionHash,
    ) -> Option<StateVersion> {
        self.ledger_payload_hash_lookup.get(identifier).cloned()
    }
}

impl SubstateDatabase for InMemoryStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.substate_store.get_substate(partition_key, sort_key)
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.substate_store.list_entries(partition_key)
    }
}

impl SubstateNodeAncestryStore for InMemoryStore {
    fn get_ancestry(&self, node_id: &NodeId) -> Option<SubstateNodeAncestryRecord> {
        self.node_ancestry_records.get(node_id).cloned()
    }
}

impl ReadableTreeStore for InMemoryStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.tree_node_store.get_node(key)
    }
}

impl ReadableAccuTreeStore<StateVersion, TransactionTreeHash> for InMemoryStore {
    fn get_tree_slice(
        &self,
        state_version: &StateVersion,
    ) -> Option<TreeSlice<TransactionTreeHash>> {
        self.transaction_tree_slices.get(state_version).cloned()
    }
}

impl ReadableAccuTreeStore<StateVersion, ReceiptTreeHash> for InMemoryStore {
    fn get_tree_slice(&self, state_version: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.receipt_tree_slices.get(state_version).cloned()
    }
}

impl CommitStore for InMemoryStore {
    fn commit(&mut self, commit_bundle: CommitBundle) {
        let commit_ledger_header = &commit_bundle.proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;

        for bundle in commit_bundle.transactions.into_iter() {
            let CommittedTransactionBundle {
                state_version,
                raw,
                receipt,
                identifiers,
            } = bundle;
            self.insert_transaction(state_version, raw, receipt, identifiers);
        }

        if let Some(next_epoch) = &commit_ledger_header.next_epoch {
            self.epoch_proofs
                .insert(next_epoch.epoch, commit_bundle.proof.clone());
        }
        self.proofs
            .insert(commit_state_version, commit_bundle.proof);

        self.substate_store
            .commit(&commit_bundle.substate_store_update.updates);

        if let Some(vertex_store) = commit_bundle.vertex_store {
            self.save_vertex_store(vertex_store)
        }

        let state_hash_tree_update = commit_bundle.state_tree_update;
        for (key, node) in state_hash_tree_update.new_nodes {
            self.tree_node_store.insert_node(key, node);
        }

        for (node_ids, record) in commit_bundle.new_substate_node_ancestry_records {
            for node_id in node_ids {
                self.node_ancestry_records.insert(node_id, record.clone());
            }
        }

        self.transaction_tree_slices
            .insert(commit_state_version, commit_bundle.transaction_tree_slice);
        self.receipt_tree_slices
            .insert(commit_state_version, commit_bundle.receipt_tree_slice);
    }
}

impl ExecutedGenesisScenarioStore for InMemoryStore {
    fn put_scenario(&mut self, number: ScenarioSequenceNumber, scenario: ExecutedGenesisScenario) {
        self.executed_genesis_scenarios.insert(number, scenario);
    }

    fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedGenesisScenario)> {
        self.executed_genesis_scenarios
            .iter()
            .map(|(number, scenario)| (*number, scenario.clone()))
            .collect()
    }
}

pub struct InMemoryCommittedTransactionBundleIterator<'a> {
    state_version: StateVersion,
    store: &'a InMemoryStore,
}

impl<'a> InMemoryCommittedTransactionBundleIterator<'a> {
    fn new(from_state_version: StateVersion, store: &'a InMemoryStore) -> Self {
        InMemoryCommittedTransactionBundleIterator {
            state_version: from_state_version,
            store,
        }
    }
}

impl Iterator for InMemoryCommittedTransactionBundleIterator<'_> {
    type Item = CommittedTransactionBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let state_version = self.state_version;
        self.state_version = self.state_version.next();
        match self.store.transactions.get(&state_version) {
            None => None,
            Some(transaction) => Some(CommittedTransactionBundle {
                state_version,
                raw: transaction.clone(),
                receipt: LocalTransactionReceipt {
                    on_ledger: self
                        .store
                        .ledger_receipts
                        .get(&state_version)
                        .unwrap()
                        .clone(),
                    local_execution: self
                        .store
                        .local_transaction_executions
                        .get(&state_version)
                        .unwrap()
                        .clone(),
                },
                identifiers: self
                    .store
                    .transaction_identifiers
                    .get(&state_version)
                    .unwrap()
                    .clone(),
            }),
        }
    }
}

impl IterableTransactionStore for InMemoryStore {
    fn get_committed_transaction_bundle_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = CommittedTransactionBundle> + '_> {
        // This is to align behaviour with the RocksDB implementation. See comment there.
        debug_assert!(self.is_local_transaction_execution_index_enabled());

        Box::new(InMemoryCommittedTransactionBundleIterator::new(
            from_state_version,
            self,
        ))
    }
}

impl QueryableTransactionStore for InMemoryStore {
    fn get_committed_transaction(
        &self,
        state_version: StateVersion,
    ) -> Option<RawLedgerTransaction> {
        Some(self.transactions.get(&state_version)?.clone())
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: StateVersion,
    ) -> Option<CommittedTransactionIdentifiers> {
        Some(self.transaction_identifiers.get(&state_version)?.clone())
    }

    fn get_committed_ledger_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LedgerTransactionReceipt> {
        Some(self.ledger_receipts.get(&state_version)?.clone())
    }

    fn get_committed_local_transaction_execution(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionExecution> {
        Some(
            self.local_transaction_executions
                .get(&state_version)?
                .clone(),
        )
    }

    fn get_committed_local_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionReceipt> {
        Some(LocalTransactionReceipt {
            on_ledger: self.ledger_receipts.get(&state_version)?.clone(),
            local_execution: self
                .local_transaction_executions
                .get(&state_version)?
                .clone(),
        })
    }
}

impl TransactionIdentifierLoader for InMemoryStore {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)> {
        self.transaction_identifiers
            .iter()
            .next_back()
            .map(|(state_version, value)| (*state_version, value.clone()))
    }
}

impl QueryableProofStore for InMemoryStore {
    fn max_state_version(&self) -> StateVersion {
        self.transactions
            .iter()
            .next_back()
            .map(|(k, _v)| *k)
            .unwrap_or_else(StateVersion::pre_genesis)
    }

    /// In memory implementation doesn't need to respect the limits
    fn get_txns_and_proof(
        &self,
        start_state_version_inclusive: StateVersion,
        _max_number_of_txns_if_more_than_one_proof: u32,
        _max_payload_size_in_bytes: u32,
    ) -> Option<(Vec<RawLedgerTransaction>, LedgerProof)> {
        self.proofs
            .range(start_state_version_inclusive..)
            .next()
            .map(|(v, proof)| {
                let mut txns = Vec::new();
                for (_, txn) in self.transactions.range(start_state_version_inclusive..=*v) {
                    txns.push(txn.clone());
                }
                (txns, proof.clone())
            })
    }

    fn get_epoch_proof(&self, epoch: Epoch) -> Option<LedgerProof> {
        self.epoch_proofs.get(&epoch).cloned()
    }

    fn get_first_proof(&self) -> Option<LedgerProof> {
        self.proofs.values().next().cloned()
    }

    fn get_post_genesis_epoch_proof(&self) -> Option<LedgerProof> {
        self.epoch_proofs.values().next().cloned()
    }

    fn get_last_proof(&self) -> Option<LedgerProof> {
        self.proofs.values().next_back().cloned()
    }

    fn get_last_epoch_proof(&self) -> Option<LedgerProof> {
        self.epoch_proofs.values().next_back().cloned()
    }
}

impl InMemoryStore {
    fn update_account_change_index_from_receipt(
        &mut self,
        state_version: StateVersion,
        receipt: &LocalTransactionExecution,
    ) {
        for (address, _) in receipt.global_balance_changes.iter() {
            if !address.is_account() {
                continue;
            }
            self.account_change_index_set
                .entry(*address)
                .or_insert(BTreeSet::new())
                .insert(state_version);
        }
        self.account_change_index_last_state_version = state_version;
    }
}

impl AccountChangeIndexExtension for InMemoryStore {
    fn account_change_index_last_processed_state_version(&self) -> StateVersion {
        self.account_change_index_last_state_version
    }

    fn catchup_account_change_index(&mut self) {
        let last_state_version = self.max_state_version();
        let last_processed_state_version = self.account_change_index_last_processed_state_version();

        for state_version in last_processed_state_version
            .next()
            .to(last_state_version.next())
        {
            self.update_account_change_index_from_receipt(
                state_version,
                &self
                    .local_transaction_executions
                    .get(&state_version)
                    .unwrap()
                    .clone(),
            );
        }
    }
}

impl IterableAccountChangeIndex for InMemoryStore {
    fn get_state_versions_for_account_iter(
        &self,
        account: GlobalAddress,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = StateVersion> + '_> {
        let Some(index) = self.account_change_index_set.get(&account) else { return Box::new(vec![].into_iter()); };
        return Box::new(
            index
                .range((Included(from_state_version), Unbounded))
                .cloned(),
        );
    }
}
