use crate::prelude::*;

use radix_engine::transaction::*;
use radix_transactions::manifest::*;

use super::*;

/// A factory of various "loaders" used by Engine State API.
///
/// Instantiating a loader is sometimes not trivial, since it may require e.g. a "staged
/// instantiation" of an uninstantiated entity - hence most endpoint handlers should interact with
/// this factory.
pub struct EngineStateLoaderFactory<'s, S> {
    network_definition: NetworkDefinition,
    database: UnmergeableSubstateDatabaseOverlay<'s, S>,
    staged_node_ids: IndexSet<NodeId>,
}

impl<'s, S: SubstateDatabase> EngineStateLoaderFactory<'s, S> {
    /// Creates a factory using the given database.
    pub fn new(network_definition: NetworkDefinition, database: &'s S) -> Self {
        Self {
            network_definition,
            database: SubstateDatabaseOverlay::new_unmergeable(database),
            staged_node_ids: index_set_new(),
        }
    }

    /// Performs a "staged instantiation" of the given entity, if required.
    ///
    /// The uninstantiated instance will be "forced" into being by executing a simple transaction
    /// that uses it (see [`create_intent_forcing_instantiation()`]). The newly-written nodes will
    /// of course live only in a staged [`SubstateDatabaseOverlay`] (passed to all loaders created
    /// by this factory).
    pub fn ensure_instantiated(mut self, node_id: &NodeId) -> Self {
        if !self.requires_instantiation(node_id) {
            return self;
        }
        let commit = self.instantiate(node_id);
        // In theory, precisely the input `node_id` should be created. However, the code below is
        // ready to fail-fast on downstream bugs (e.g. the entity not getting instantiated) and to
        // handle possible future behaviors (e.g. some extra related entities being created):
        let instantiated_node_ids = collect_instantiated_node_ids(commit);
        if !instantiated_node_ids.contains(node_id) {
            panic!(
                "forced instantiation succeeded but did not create {:?}",
                node_id
            );
        }
        self.staged_node_ids.extend(instantiated_node_ids);
        self
    }

    /// Creates a loader of an entity's type info.
    pub fn create_meta_loader(
        &'s self,
    ) -> EngineStateMetaLoader<'s, UnmergeableSubstateDatabaseOverlay<'s, S>> {
        EngineStateMetaLoader {
            reader: SystemDatabaseReader::new(&self.database),
            non_instantiated_node_ids: self.staged_node_ids.clone(),
        }
    }

    /// Creates a loader of raw data living in an object's Main module.
    pub fn create_data_loader(
        &'s self,
    ) -> EngineStateDataLoader<'s, UnmergeableSubstateDatabaseOverlay<'s, S>> {
        EngineStateDataLoader {
            reader: SystemDatabaseReader::new(&self.database),
        }
    }

    /// Creates a higher-level loader of data living in an object's Role Assignment module.
    pub fn create_object_role_assignment_loader(
        &'s self,
    ) -> ObjectRoleAssignmentLoader<'s, UnmergeableSubstateDatabaseOverlay<'s, S>> {
        ObjectRoleAssignmentLoader {
            meta_loader: self.create_meta_loader(),
            data_loader: self.create_data_loader(),
        }
    }

    /// Creates a higher-level loader of data living in an object's Royalty module.
    pub fn create_object_royalty_loader(
        &'s self,
    ) -> ObjectRoyaltyLoader<'s, UnmergeableSubstateDatabaseOverlay<'s, S>> {
        ObjectRoyaltyLoader {
            meta_loader: self.create_meta_loader(),
            data_loader: self.create_data_loader(),
        }
    }

    /// Creates a higher-level loader of data living in an object's Metadata module.
    pub fn create_object_metadata_loader(
        &'s self,
    ) -> ObjectMetadataLoader<'s, UnmergeableSubstateDatabaseOverlay<'s, S>> {
        ObjectMetadataLoader {
            loader: self.create_data_loader(),
        }
    }

    /// Checks whether the given entity should be automatically instantiated on access, given the
    /// current database state.
    fn requires_instantiation(&self, node_id: &NodeId) -> bool {
        if !node_id.is_global_preallocated() {
            // No matter if it exists or not, we shouldn't instantiate:
            return false;
        }
        // We check whether it is missing by trying to load the `TypeInfo`:
        return SystemDatabaseReader::new(&self.database)
            .get_type_info(node_id)
            .is_err();
    }

    /// Triggers automatic instantiation of the given entity, using a handcrafted transaction on the
    /// internally-staged database.
    fn instantiate(&mut self, node_id: &NodeId) -> CommitResult {
        let intent = create_intent_forcing_instantiation(node_id);

        // Note: here, we initialize a new `ScryptoVm` instance every time - however, this is
        // currently a very lightweight operation. At the same time, we would NOT be able to feel
        // the benefits of caching an instance (since we do not use any WASM here). If these
        // assumptions change in future, then it will make most sense to turn the `Self` into a
        // long-lived service holding a `ScryptoVm` as its dependency.
        let vm_modules = DefaultVmModules::default();

        let receipt = execute_and_commit_transaction(
            &mut self.database,
            &vm_modules,
            &ExecutionConfig::for_preview(self.network_definition.clone()),
            intent.create_executable(),
        );
        let TransactionResult::Commit(commit) = receipt.result else {
            panic!("failed to force instantiation: {:?}", receipt);
        };

        commit
    }
}

/// Traverses the state updates in order to discover all actually instantiated [`NodeId`]s.
fn collect_instantiated_node_ids(commit: CommitResult) -> IndexSet<NodeId> {
    let mut staged_node_ids = index_set_new();
    let StateUpdateSummary {
        new_packages,
        new_components,
        new_resources,
        new_vaults,
        vault_balance_changes: _,
    } = &commit.state_update_summary;
    staged_node_ids.extend(new_packages.iter().map(|addr| *addr.as_node_id()));
    staged_node_ids.extend(new_components.iter().map(|addr| *addr.as_node_id()));
    staged_node_ids.extend(new_resources.iter().map(|addr| *addr.as_node_id()));
    staged_node_ids.extend(new_vaults.iter().map(|addr| *addr.as_node_id()));
    staged_node_ids
}

/// Creates a minimal transaction intent which forces an instantiation of the given Entity.
fn create_intent_forcing_instantiation(node_id: &NodeId) -> ValidatedPreviewIntent {
    let address =
        GlobalAddress::try_from(*node_id).expect("should only be reachable for global entities");
    // Use dummy key, as for a regular preview:
    let notary_public_key =
        PublicKey::Secp256k1(Secp256k1PrivateKey::from_u64(1).unwrap().public_key());
    let intent = IntentV1 {
        header: TransactionHeaderV1 {
            // Use dummy values, since we disable these checks anyway:
            network_id: 0,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::zero(),
            nonce: 0,
            notary_public_key,
            notary_is_signatory: true,
            tip_percentage: 0,
        },
        instructions: InstructionsV1(vec![
            // Call any basic, non-invasive method, which is enough to force instantiation:
            // Note: the "get metadata" call below seems universal and future-proof enough, but
            // a more elegant solution could be based on some hypothetical "ensure exists" method?
            InstructionV1::CallMetadataMethod(CallMetadataMethod {
                address: DynamicGlobalAddress::Static(address),
                method_name: "get".to_string(),
                args: ManifestValue::Tuple {
                    fields: vec![ManifestValue::String {
                        value: "dummy name".to_string(),
                    }],
                },
            }),
        ]),
        blobs: BlobsV1::default(),
        message: MessageV1::None,
    };

    ValidatedPreviewIntent {
        intent: intent
            .prepare(&PreparationSettings::latest())
            .expect("hardcoded"),
        encoded_instructions: manifest_encode(&intent.instructions.0).expect("hardcoded"),
        signer_public_keys: vec![],
        flags: PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: true,
        },
    }
}
