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
use entity_tier::EntityTier;

/// An implementation of a [`SubstateDatabase`] viewed at a specific [`StateVersion`].
///
/// This database is backed by:
/// - a [`ReadableTreeStore`] - a versioned source of Entities / Partitions / Substates metadata,
/// - and a [`LeafSubstateValueStore`] - a store of Substate values' associated with their leafs.
pub struct StateTreeBasedSubstateDatabase<'s, DS> {
    base_store: DS,
    at_state_version: StateVersion,

    // Note: "Why do we even need to capture `'t`?"
    //
    // We want the `StateTreeBasedSubstateDatabase` struct to be ready to either own the underlying
    // `ReadableTreeStore` instance, or just reference it (depending on the caller's use-case).
    //
    // This is achieved by a classic `Deref<Target = T>, T: ReadableTreeStore` approach. However, we
    // encounter "mismatched lifetimes" issues when implementing the `SubstateDatabase`, which wants
    // to return a `dyn Iterator<Item = PartitionEntry> + '_` (i.e. return the lifetime of `&self`).
    // In Rust, there is no syntax to express a requirement for `T: '_` - and without it, the
    // compiler is not sure whether "type `T` lives long enough".
    //
    // One workaround would be to use explicit lifetimes in the `SubstateDatabase` trait definition,
    // and thus allow to explicitly require `T: 't` just within the relevant implementation here
    // (see https://users.rust-lang.org/t/trait-impl-lifetime-nightmare/54735/3). This would be
    // cumbersome, since that trait is defined by a dependency (internal one, but still). Moreover,
    // banning default/elided lifetimes (in the name of "enabling some `Deref` usage elsewhere")
    // does not seem right.
    //
    // Hence, another solution is used here: by introducing a theoretically-unused `'t` to the
    // `StateTreeBasedSubstateDatabase` struct itself, we can express `T: 't` requirement wherever
    // we need to, without touching definitions of implemented traits.
    phantom: PhantomData<&'s DS>,
}

impl<'s, S: 's, DS: Deref<Target = S>> StateTreeBasedSubstateDatabase<'s, DS> {
    /// Creates an instance backed by the given lower-level stores and scoped at the given version.
    pub fn new(base_store: DS, at_state_version: StateVersion) -> Self {
        Self {
            base_store,
            at_state_version,
            phantom: PhantomData,
        }
    }

    fn create_entity_tier(&'s self) -> EntityTier<'s, S> {
        EntityTier::new(
            self.base_store.deref(),
            Some(self.at_state_version.number()).filter(|v| *v > 0),
        )
    }
}

impl<'s, S: QueryableTransactionStore + 's, DS: Deref<Target = S>>
    StateTreeBasedSubstateDatabase<'s, DS>
{
    fn at_transaction(&self) -> (StateVersion, CommittedTransactionIdentifiers) {
        // The direct read from the unscoped underlying store is actually "historical": the
        // `QueryableTransactionStore` is append-only, and we request for a state version which
        // definitely existed at that point.
        let transaction_identifiers = self
            .base_store
            .get_committed_transaction_identifiers(self.at_state_version)
            .expect("transaction at the scoped state version");
        (self.at_state_version, transaction_identifiers)
    }
}

impl<'s, S: ReadableTreeStore + 's, DS: Deref<Target = S>> StateTreeBasedSubstateDatabase<'s, DS> {
    /// Returns an iterator over *all* Substate-Tier's leaf keys accessible from the scoped version
    /// (i.e. from all Entities/Partitions).
    /// Each Substate leaf key is accompanied by a full key of the Substate it represents.
    pub fn iter_substate_leaf_keys(
        &self,
    ) -> impl Iterator<Item = (StoredTreeNodeKey, DbSubstateKey)> + '_ {
        self.create_entity_tier()
            .into_iter_entity_partition_tiers_from(None)
            .flat_map(|partition_tier| partition_tier.into_iter_partition_substate_tiers_from(None))
            .flat_map(|substate_tier| {
                let partition_key = substate_tier.partition_key().clone();
                substate_tier
                    .into_iter_substate_summaries_from(None)
                    .map(move |summary| {
                        (
                            summary.state_tree_leaf_key,
                            (partition_key.clone(), summary.sort_key),
                        )
                    })
            })
    }
}

impl<'s, S: ReadableTreeStore + LeafSubstateValueStore + 's, DS: Deref<Target = S>> SubstateDatabase
    for StateTreeBasedSubstateDatabase<'s, DS>
{
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.create_entity_tier()
            .get_entity_partition_tier(partition_key.node_key.clone())
            .get_partition_substate_tier(partition_key.partition_num)
            .get_substate_summary(sort_key)
            .map(|substate| self.get_value(&substate.state_tree_leaf_key))
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        Box::new(
            self.create_entity_tier()
                .get_entity_partition_tier(partition_key.node_key.clone())
                .get_partition_substate_tier(partition_key.partition_num)
                .into_iter_substate_summaries_from(from_sort_key)
                .map(|substate| {
                    (
                        substate.sort_key,
                        self.get_value(&substate.state_tree_leaf_key),
                    )
                }),
        )
    }
}

impl<'s, S: LeafSubstateValueStore + 's, DS: Deref<Target = S>>
    StateTreeBasedSubstateDatabase<'s, DS>
{
    /// Returns the substate value associated with the given leaf key.
    ///
    /// The implementation makes a few assumptions and *panics* is any of them is not met:
    /// - The queried tree node represents a Substate-Tier's leaf,
    /// - The queried tree node was stored while the "state history" feature was enabled (see
    ///   [`DatabaseConfig::enable_historical_substate_values`]),
    /// - The queried tree node was not garbage-collected yet (see
    ///   [`StateTreeGcConfig::state_version_history_length`]).
    ///
    /// These assumptions are enforced by the [`VersionScopedDatabase::new`] constructor.
    fn get_value(&self, tree_leaf_key: &StoredTreeNodeKey) -> DbSubstateValue {
        let Some(value) = self.base_store.get_associated_value(tree_leaf_key) else {
            panic!(
                "DB inconsistency: associated value not found for leaf key {:?}",
                tree_leaf_key
            );
        };
        value
    }
}

/// A [`SubstateDatabase`] aware of its state version.
///
/// The implementation enum-dispatches either to the runtime store (when at current version), or to
/// a historical store.
pub enum VersionScopedDatabase<'s, S> {
    Current(S),
    Historical(StateTreeBasedSubstateDatabase<'s, S>),
}

/// An error that may happen during opening [`VersionScopedDatabase`] at a past version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateHistoryError {
    StateHistoryDisabled,
    StateVersionInTooDistantPast {
        first_available_version: StateVersion,
    },
    StateVersionInFuture {
        current_version: StateVersion,
    },
}

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>>
    VersionScopedDatabase<'s, DS>
{
    /// Creates an instance backed by the given database, and depending on a requested version.
    pub fn new(
        database: DS,
        requested_version: Option<StateVersion>,
    ) -> Result<Self, StateHistoryError> {
        let current_version = database.max_state_version();
        let Some(requested_version) = requested_version else {
            return Ok(Self::Current(database)); // implicit "use current version"
        };
        if requested_version == current_version {
            return Ok(Self::Current(database)); // explicit "use current version"
        }

        if !database.is_state_history_enabled() {
            return Err(StateHistoryError::StateHistoryDisabled);
        };
        let first_available_version = database.get_first_stored_historical_state_version();
        if requested_version < first_available_version {
            return Err(StateHistoryError::StateVersionInTooDistantPast {
                first_available_version,
            });
        }
        if requested_version > current_version {
            return Err(StateHistoryError::StateVersionInFuture { current_version });
        }

        return Ok(Self::Historical(StateTreeBasedSubstateDatabase::new(
            database,
            requested_version,
        )));
    }

    /// Returns the summary of the ledger's state at which this store is scoped.
    ///
    /// Note: this will be based on an actual ledger proof only if it exists at the scoped state
    /// version (i.e. always in case of "current top of ledger", and also sometimes for incidental
    /// historical ledger states). Otherwise, it will be composed based on the relevant Substates
    /// read at the scoped version.
    pub fn at_ledger_state(&self) -> LedgerStateSummary {
        match self {
            VersionScopedDatabase::Current(database) => database
                .get_latest_proof()
                .expect("proof for current top of ledger")
                .ledger_header
                .into(),
            VersionScopedDatabase::Historical(database) => {
                let (state_version, transaction_identifiers) = database.at_transaction();
                if let Some(exact_proof) = database.base_store.get_proof(state_version) {
                    // If an actual header is available at this version, we can use it:
                    return exact_proof.ledger_header.into();
                }
                let (epoch, round) = database.get_epoch_and_round();
                LedgerStateSummary {
                    epoch,
                    round,
                    state_version,
                    hashes: transaction_identifiers.resultant_ledger_hashes,
                    proposer_timestamp_ms: transaction_identifiers.proposer_timestamp_ms,
                }
            }
        }
    }
}

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>> SubstateDatabase
    for VersionScopedDatabase<'s, DS>
{
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        match self {
            VersionScopedDatabase::Current(database) => {
                database.get_raw_substate_by_db_key(partition_key, sort_key)
            }
            VersionScopedDatabase::Historical(database) => {
                database.get_raw_substate_by_db_key(partition_key, sort_key)
            }
        }
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        match self {
            VersionScopedDatabase::Current(database) => {
                database.list_raw_values_from_db_key(partition_key, from_sort_key)
            }
            VersionScopedDatabase::Historical(database) => {
                database.list_raw_values_from_db_key(partition_key, from_sort_key)
            }
        }
    }
}

// Apart from the meaty `SubstateDatabase`, we implement a couple of other stores typically used by
// clients interested in historical state:

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>>
    SubstateNodeAncestryStore for VersionScopedDatabase<'s, DS>
{
    fn batch_get_ancestry<'a>(
        &self,
        node_ids: impl IntoIterator<Item = &'a NodeId>,
    ) -> Vec<Option<SubstateNodeAncestryRecord>> {
        // Unfortunately, there is no easy way to filter out the "future" here
        self.underlying().batch_get_ancestry(node_ids)
    }
}

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>> EntityListingIndex
    for VersionScopedDatabase<'s, DS>
{
    fn get_created_entity_iter(
        &self,
        entity_type: EntityType,
        from_creation_id: Option<&CreationId>,
    ) -> Box<dyn Iterator<Item = (CreationId, EntityBlueprintId)> + '_> {
        match self {
            VersionScopedDatabase::Current(current) => {
                Box::new(current.get_created_entity_iter(entity_type, from_creation_id))
            }
            VersionScopedDatabase::Historical(historical) => Box::new(
                historical
                    .base_store
                    .get_created_entity_iter(entity_type, from_creation_id)
                    .take_while(|(id, _)| id.state_version <= historical.at_state_version),
            ),
        }
    }

    fn get_blueprint_entity_iter(
        &self,
        blueprint_id: &BlueprintId,
        from_creation_id: Option<&CreationId>,
    ) -> Box<dyn Iterator<Item = (CreationId, EntityBlueprintId)> + '_> {
        match self {
            VersionScopedDatabase::Current(current) => {
                Box::new(current.get_blueprint_entity_iter(blueprint_id, from_creation_id))
            }
            VersionScopedDatabase::Historical(historical) => Box::new(
                historical
                    .base_store
                    .get_blueprint_entity_iter(blueprint_id, from_creation_id)
                    .take_while(|(id, _)| id.state_version <= historical.at_state_version),
            ),
        }
    }
}

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>> ConfigurableDatabase
    for VersionScopedDatabase<'s, DS>
{
    fn is_account_change_index_enabled(&self) -> bool {
        self.underlying().is_account_change_index_enabled()
    }

    fn is_local_transaction_execution_index_enabled(&self) -> bool {
        self.underlying()
            .is_local_transaction_execution_index_enabled()
    }

    fn are_entity_listing_indices_enabled(&self) -> bool {
        self.underlying().are_entity_listing_indices_enabled()
    }

    fn is_state_history_enabled(&self) -> bool {
        self.underlying().is_state_history_enabled()
    }

    fn get_first_stored_historical_state_version(&self) -> StateVersion {
        self.underlying()
            .get_first_stored_historical_state_version()
    }
}

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>>
    VersionScopedDatabase<'s, DS>
{
    /// Accesses the underlying [`StateManagerDatabase`] directly.
    ///
    /// This is an implementation detail for store implementations capable of accessing "runtime"
    /// tables in a "historical" way (e.g. in case of append-only tables).
    fn underlying(&self) -> &StateManagerDatabase<R> {
        match self {
            VersionScopedDatabase::Current(current) => current.deref(),
            VersionScopedDatabase::Historical(historical) => historical.base_store.deref(),
        }
    }
}

/// An extension trait for more convenient construction of version-scoped [`StateManagerDatabase`].
///
/// Note: an implementation for `Deref<Target = StateManagerDatabase<R>>,  R: ReadableRocks` is
/// provided - and it is appropriate for all instances of `StateManagerDatabase` obtained from the
/// DB lock. Callers are not expected to implement it for any other type.
pub trait VersionScopingSupport<'s, R>: Sized {
    /// Returns a database scoped at the requested state version.
    ///
    /// Note: this method consumes `self`, but since it is only implemented for [`Deref`], it can be
    /// used both for owning the underlying [`StateManagerDatabase`] and for referencing it.
    fn scoped_at(
        self,
        requested_state_version: Option<StateVersion>,
    ) -> Result<VersionScopedDatabase<'s, Self>, StateHistoryError>;
}

impl<'s, R: ReadableRocks + 's, DS: Deref<Target = StateManagerDatabase<R>>>
    VersionScopingSupport<'s, R> for DS
{
    fn scoped_at(
        self,
        requested_state_version: Option<StateVersion>,
    ) -> Result<VersionScopedDatabase<'s, Self>, StateHistoryError> {
        VersionScopedDatabase::new(self, requested_state_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updated_substate_has_different_values_across_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores
            .put_substate_changes(vec![change(8, 7, 2, Some(8)), change(8, 1, 9, Some(1))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, Some(from_seed(8)));
    }

    #[test]
    fn reset_substate_has_different_values_across_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 =
            test_stores.reset_partition(&partition_key(8, 7), vec![(sort_key(2), from_seed(9))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, Some(from_seed(9)));
    }

    #[test]
    fn unchanged_substate_has_same_value_across_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores
            .put_substate_changes(vec![change(8, 7, 2, Some(8)), change(8, 1, 9, Some(1))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_raw_substate_by_db_key(&partition_key(8, 6), &sort_key(3));
        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_raw_substate_by_db_key(&partition_key(8, 6), &sort_key(3));
        assert_eq!(value_v2, value_v1);
    }

    #[test]
    fn substate_created_later_has_no_value_earlier() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores
            .put_substate_changes(vec![change(8, 7, 2, Some(8)), change(8, 1, 9, Some(1))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_raw_substate_by_db_key(&partition_key(8, 1), &sort_key(9));
        assert_eq!(value_v1, None);

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_raw_substate_by_db_key(&partition_key(8, 1), &sort_key(9));
        assert_eq!(value_v2, Some(from_seed(1)));
    }

    #[test]
    fn substate_deleted_later_still_has_value_earlier() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 =
            test_stores.put_substate_changes(vec![change(8, 7, 2, None), change(8, 1, 9, Some(1))]);

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, None);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));
    }

    #[test]
    fn substate_inside_deleted_partition_still_has_value_earlier() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores.reset_partition(&partition_key(8, 7), vec![]);

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, None);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_raw_substate_by_db_key(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));
    }

    #[test]
    fn lists_partition_substates_at_different_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 7, 2, Some(7)), // to be changed
            change(8, 7, 4, Some(9)), // to be unchanged
            change(8, 7, 1, Some(3)), // to be deleted
            change(8, 6, 3, Some(6)), // unrelated partition
            change(9, 2, 5, Some(2)), // unrelated node
        ]);
        let v2 = test_stores.put_substate_changes(vec![
            change(8, 7, 2, Some(8)), // changed value
            change(8, 7, 9, Some(1)), // added substate
            change(8, 7, 1, None),    // deleted substate
            change(8, 1, 8, Some(2)), // unrelated change
        ]);

        let subject_v1 = test_stores.create_subject(v1);
        let substates_v1 = subject_v1
            .list_raw_values_from_db_key(&partition_key(8, 7), None)
            .collect::<Vec<_>>();
        assert_eq!(
            substates_v1,
            vec![
                (sort_key(1), from_seed(3)),
                (sort_key(2), from_seed(7)),
                (sort_key(4), from_seed(9)),
            ]
        );

        let subject_v2 = test_stores.create_subject(v2);
        let substates_v2 = subject_v2
            .list_raw_values_from_db_key(&partition_key(8, 7), None)
            .collect::<Vec<_>>();
        assert_eq!(
            substates_v2,
            vec![
                (sort_key(2), from_seed(8)),
                (sort_key(4), from_seed(9)),
                (sort_key(9), from_seed(1)),
            ]
        );
    }

    #[test]
    fn lists_partition_substates_from_different_starting_keys() {
        let mut test_stores = TestStores::new_empty();
        let the_only_version = test_stores.put_substate_changes(vec![
            change(8, 7, 1, Some(3)),
            change(8, 7, 2, Some(5)),
            change(8, 7, 4, Some(7)),
            change(8, 7, 7, Some(9)),
        ]);

        let subject = test_stores.create_subject(the_only_version);
        let all_substates = subject
            .list_raw_values_from_db_key(&partition_key(8, 7), None)
            .collect::<Vec<_>>();
        assert_eq!(
            all_substates,
            vec![
                (sort_key(1), from_seed(3)),
                (sort_key(2), from_seed(5)),
                (sort_key(4), from_seed(7)),
                (sort_key(7), from_seed(9)),
            ]
        );

        let from_existent = subject
            .list_raw_values_from_db_key(&partition_key(8, 7), Some(&sort_key(2)))
            .collect::<Vec<_>>();
        assert_eq!(from_existent, all_substates[1..]);

        let from_non_existent = subject
            .list_raw_values_from_db_key(&partition_key(8, 7), Some(&sort_key(3)))
            .collect::<Vec<_>>();
        assert_eq!(from_non_existent, all_substates[2..]);

        let from_lt_min = subject
            .list_raw_values_from_db_key(&partition_key(8, 7), Some(&sort_key(0)))
            .collect::<Vec<_>>();
        assert_eq!(from_lt_min, all_substates);

        let from_gt_max = subject
            .list_raw_values_from_db_key(&partition_key(8, 7), Some(&sort_key(9)))
            .collect::<Vec<_>>();
        assert_eq!(from_gt_max, vec![]);
    }

    // Only test utils below:

    type SingleSubstateChange = (DbSubstateKey, DatabaseUpdate);

    fn change(
        node_key_seed: u8,
        partition_num: u8,
        sort_key_seed: u8,
        value_seed: Option<u8>,
    ) -> SingleSubstateChange {
        change_exact(
            node_key(node_key_seed),
            partition_num,
            from_seed(sort_key_seed),
            value_seed.map(from_seed),
        )
    }

    pub fn change_exact(
        node_key: Vec<u8>,
        partition_num: u8,
        sort_key: Vec<u8>,
        value: Option<Vec<u8>>,
    ) -> SingleSubstateChange {
        (
            (
                DbPartitionKey {
                    node_key,
                    partition_num,
                },
                DbSortKey(sort_key),
            ),
            value
                .map(DatabaseUpdate::Set)
                .unwrap_or(DatabaseUpdate::Delete),
        )
    }

    fn node_key(node_key_seed: u8) -> DbNodeKey {
        const RANDOM_ENTITY_TYPES: [EntityType; 3] = [
            EntityType::GlobalAccount,
            EntityType::GlobalPackage,
            EntityType::GlobalValidator,
        ];
        let entity_type = RANDOM_ENTITY_TYPES[node_key_seed as usize % RANDOM_ENTITY_TYPES.len()];
        let node_id = NodeId::new(entity_type as u8, &[node_key_seed; NodeId::RID_LENGTH]);
        SpreadPrefixKeyMapper::to_db_node_key(&node_id)
    }

    fn partition_key(node_key_seed: u8, partition_num: u8) -> DbPartitionKey {
        DbPartitionKey {
            node_key: node_key(node_key_seed),
            partition_num,
        }
    }

    fn sort_key(sort_key_seed: u8) -> DbSortKey {
        DbSortKey(from_seed(sort_key_seed))
    }

    fn from_seed(seed: u8) -> Vec<u8> {
        vec![seed; seed as usize]
    }

    struct TestStores {
        tree_store: TypedInMemoryTreeStore,
        current_version: StateVersion,
    }

    impl TestStores {
        pub fn new_empty() -> Self {
            Self {
                tree_store: TypedInMemoryTreeStore::new().storing_associated_substates(),
                current_version: StateVersion::pre_genesis(),
            }
        }

        pub fn put_substate_changes(
            &mut self,
            changes: impl IntoIterator<Item = SingleSubstateChange>,
        ) -> StateVersion {
            self.apply_database_updates(&DatabaseUpdates::from_delta_maps(
                Self::index_to_delta_maps(changes),
            ))
        }

        pub fn reset_partition(
            &mut self,
            partition_key: &DbPartitionKey,
            values: impl IntoIterator<Item = (DbSortKey, DbSubstateValue)>,
        ) -> StateVersion {
            let DbPartitionKey {
                node_key,
                partition_num,
            } = partition_key.clone();
            self.apply_database_updates(&DatabaseUpdates {
                node_updates: indexmap!(
                    node_key => NodeDatabaseUpdates {
                        partition_updates: indexmap!(
                            partition_num => PartitionDatabaseUpdates::Reset {
                                new_substate_values: values.into_iter().collect()
                            }
                        )
                    }
                ),
            })
        }

        pub fn create_subject(
            &self,
            at_state_version: StateVersion,
        ) -> StateTreeBasedSubstateDatabase<&TypedInMemoryTreeStore> {
            StateTreeBasedSubstateDatabase::new(&self.tree_store, at_state_version)
        }

        fn apply_database_updates(&mut self, database_updates: &DatabaseUpdates) -> StateVersion {
            let current_version = self.current_version;
            put_at_next_version(
                &self.tree_store,
                Some(current_version.number()).filter(|number| *number > 0u64),
                database_updates,
            );
            let next_version = current_version.next().expect("too high version in a test");
            self.current_version = next_version;
            self.current_version
        }

        fn index_to_delta_maps(
            changes: impl IntoIterator<Item = SingleSubstateChange>,
        ) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>> {
            let mut delta_maps =
                index_map_new::<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>>();
            for change in changes {
                let ((partition_key, sort_key), update) = change;
                delta_maps
                    .entry(partition_key)
                    .or_default()
                    .insert(sort_key, update);
            }
            delta_maps
        }
    }

    impl LeafSubstateValueStore for TypedInMemoryTreeStore {
        fn get_associated_value(
            &self,
            state_tree_leaf_key: &StoredTreeNodeKey,
        ) -> Option<DbSubstateValue> {
            self.associated_substates
                .borrow()
                .get(state_tree_leaf_key)
                .and_then(|(_key, value)| value.clone())
        }
    }
}
