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

/// A builder of a [`CommitBundle`] from individual transactions and their commit results.
pub struct CommitBundleBuilder {
    committed_transaction_bundles: Vec<CommittedTransactionBundle>,
    substate_store_update: SubstateStoreUpdate,
    state_tree_update: StateTreeUpdate,
    new_substate_node_ancestry_records: Vec<KeyedSubstateNodeAncestryRecord>,
    new_leaf_substate_keys: Vec<LeafSubstateKeyAssociation>,
    transaction_tree_slice_merger: AccuTreeSliceMerger<TransactionTreeHash>,
    receipt_tree_slice_merger: AccuTreeSliceMerger<ReceiptTreeHash>,
}

impl CommitBundleBuilder {
    /// Starts a new build.
    ///
    /// The `epoch_state_version` vs `current_state_version` relationship is required only for
    /// proper merging of hash accumulation trees (see [`AccuTreeSliceMerger`]).
    pub fn new(epoch_state_version: StateVersion, current_state_version: StateVersion) -> Self {
        let epoch_accu_trees =
            EpochAwareAccuTreeFactory::new(epoch_state_version, current_state_version);
        Self {
            committed_transaction_bundles: Vec::new(),
            substate_store_update: SubstateStoreUpdate::new(),
            state_tree_update: StateTreeUpdate::new(),
            new_substate_node_ancestry_records: Vec::new(),
            new_leaf_substate_keys: Vec::new(),
            transaction_tree_slice_merger: epoch_accu_trees.create_merger(),
            receipt_tree_slice_merger: epoch_accu_trees.create_merger(),
        }
    }

    /// Adds another transaction execution to the bundle.
    pub fn add_executed_transaction(
        &mut self,
        state_version: StateVersion,
        proposer_timestamp_ms: i64,
        raw: RawLedgerTransaction,
        hashes: LedgerTransactionHashes,
        result: ProcessedCommitResult,
    ) {
        self.substate_store_update.apply(result.database_updates);
        let hash_structures_diff = result.hash_structures_diff;
        self.state_tree_update
            .add(state_version, hash_structures_diff.state_tree_diff);
        self.new_substate_node_ancestry_records
            .extend(result.new_substate_node_ancestry_records);
        self.new_leaf_substate_keys
            .extend(result.new_leaf_substate_keys);
        self.transaction_tree_slice_merger
            .append(hash_structures_diff.transaction_tree_diff.slice);
        self.receipt_tree_slice_merger
            .append(hash_structures_diff.receipt_tree_diff.slice);

        self.committed_transaction_bundles
            .push(CommittedTransactionBundle {
                state_version,
                raw,
                receipt: result.local_receipt,
                identifiers: CommittedTransactionIdentifiers {
                    transaction_hashes: hashes,
                    resultant_ledger_hashes: hash_structures_diff.ledger_hashes,
                    proposer_timestamp_ms,
                },
            });
    }

    /// Finalizes the build with the given proof.
    pub fn build(self, proof: LedgerProof, vertex_store: Option<Vec<u8>>) -> CommitBundle {
        CommitBundle {
            transactions: self.committed_transaction_bundles,
            proof,
            substate_store_update: self.substate_store_update,
            vertex_store: vertex_store.map(VertexStoreBlobV1),
            state_tree_update: self.state_tree_update,
            transaction_tree_slice: TransactionAccuTreeSliceV1(
                self.transaction_tree_slice_merger.into_slice(),
            ),
            receipt_tree_slice: ReceiptAccuTreeSliceV1(self.receipt_tree_slice_merger.into_slice()),
            new_substate_node_ancestry_records: self.new_substate_node_ancestry_records,
            new_leaf_substate_keys: self.new_leaf_substate_keys,
        }
    }
}
