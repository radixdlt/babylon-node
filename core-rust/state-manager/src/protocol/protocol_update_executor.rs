use crate::prelude::*;

pub struct NodeProtocolUpdateExecutor {
    network_definition: NetworkDefinition,
    protocol_update_content_overrides: RawProtocolUpdateContentOverrides,
    scenarios_execution_config: ScenariosExecutionConfig,
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    system_executor: Arc<SystemExecutor>,
    transaction_validator: Arc<RwLock<TransactionValidator>>,
    genesis_data_resolver: Arc<dyn ResolveGenesisData>,
}

impl NodeProtocolUpdateExecutor {
    pub fn new(
        network: NetworkDefinition,
        protocol_update_content_overrides: RawProtocolUpdateContentOverrides,
        scenarios_execution_config: ScenariosExecutionConfig,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        system_executor: Arc<SystemExecutor>,
        transaction_validator: Arc<RwLock<TransactionValidator>>,
        genesis_data_resolver: Arc<dyn ResolveGenesisData>,
    ) -> Self {
        Self {
            network_definition: network,
            protocol_update_content_overrides,
            scenarios_execution_config,
            database,
            system_executor,
            transaction_validator,
            genesis_data_resolver,
        }
    }

    /// Executes any remaining parts of the currently-effective protocol update.
    /// This method is meant to be called during the boot-up, to support resuming after a restart.
    ///
    /// Returns the resultant protocol version.
    pub fn resume_protocol_update_if_any(&self) -> Option<ProtocolVersionName> {
        let mut latest_enacted: Option<ProtocolVersionName> = None;

        // We loop because we might need to run back-to-back protocol updates.
        loop {
            let progress = ProtocolUpdateProgress::resolve(self.database.lock().deref());
            //=========================== Helpful Notice (to future me) ============================
            // It may look sensible to inline `progress` below, but trust me, it isn't.
            // You get a deadlock because rust doesn't drop the database lock guard as soon as you'd expect.
            // This notice was installed after doing this refactor twice, 6 months apart...
            // And then having to play the fun "hunt for the deadlock" game.
            //======================================================================================
            match progress {
                ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                    protocol_version_name,
                } => {
                    info!("[UPDATE: {protocol_version_name}] Starting protocol update");
                    self.execute_protocol_update_actions(&protocol_version_name, 0, 0);
                    latest_enacted = Some(protocol_version_name.clone());
                }
                ProtocolUpdateProgress::UpdateInProgress {
                    protocol_version_name,
                    last_batch_group_index,
                    last_batch_index,
                } => {
                    info!("[UPDATE: {protocol_version_name}] Resuming protocol update");
                    self.execute_protocol_update_actions(
                        &protocol_version_name,
                        last_batch_group_index,
                        last_batch_index.checked_add(1).unwrap(),
                    );
                    latest_enacted = Some(protocol_version_name.clone());
                }
                ProtocolUpdateProgress::NotUpdating => {
                    return latest_enacted;
                }
            }
        }
    }

    /// Executes the (remaining part of the) given protocol update's transactions.
    fn execute_protocol_update_actions(
        &self,
        protocol_version: &ProtocolVersionName,
        from_batch_group_index: usize,
        from_batch_index: usize,
    ) {
        let overrides = self.protocol_update_content_overrides.get(protocol_version);
        let resolved = protocol_version.validate().unwrap_or_else(|err| {
            panic!("{protocol_version:?} is not a supported protocol version: {err:?}")
        });
        let protocol_update_definition = resolved.definition();
        let update_generator = protocol_update_definition.create_update_generator_raw(
            ProtocolUpdateContext {
                network: &self.network_definition,
                database: &self.database,
                genesis_data_resolver: &self.genesis_data_resolver,
                scenario_config: &self.scenarios_execution_config,
            },
            overrides,
        );
        let config_hash = update_generator.config_hash();
        let enable_status_reporting = update_generator.insert_status_tracking_flash_transactions();
        let mut batch_groups = update_generator.batch_groups();

        // Copies the behaviour of the engine's ProtocolUpdateExecutor.
        if enable_status_reporting {
            // The status update itself will get added when the batch is processed
            batch_groups.push(
                NodeFixedBatchGroupGenerator::named("completion")
                    .add_batch("record-completion", |_| {
                        NodeProtocolUpdateBatch::ProtocolUpdateBatch(ProtocolUpdateBatch::empty())
                    })
                    .build(),
            );
        }

        let total_batch_groups = batch_groups.len();
        let remaining_batch_groups = batch_groups
            .into_iter()
            .enumerate()
            .skip(from_batch_group_index);

        for (batch_group_index, batch_group) in remaining_batch_groups {
            let batch_group_number = batch_group_index + 1;
            let batch_group_name = batch_group.batch_group_name();
            let start_at_batch = if batch_group_index == from_batch_group_index
                && from_batch_index > 0
            {
                info!("[UPDATE: {protocol_version}] Continuing {batch_group_name} (batch group {batch_group_number}/{total_batch_groups})");
                from_batch_index
            } else {
                info!("[UPDATE: {protocol_version}] Commencing {batch_group_name} (batch group {batch_group_number}/{total_batch_groups})");
                0
            };
            let batches = {
                // In a separate block to ensure the database lock guard is dropped promptly.
                batch_group.generate_batches(self.database.lock().deref())
            };
            let total_batches = batches.len();
            let remaining_batches = batches.into_iter().enumerate().skip(start_at_batch);
            for (batch_index, batch_generator) in remaining_batches {
                let batch_number = batch_index + 1;
                let batch_name = batch_generator.batch_name().to_string();
                let batch_name = batch_name.as_str();
                info!("[UPDATE: {protocol_version}] Processing {batch_name} (batch {batch_number}/{total_batches}) in {batch_group_name} (batch group {batch_group_number}/{total_batch_groups})");
                let start_state_identifiers = self
                    .resolve_batch_ledger_state_identifiers(update_generator.genesis_start_state());
                let batch = {
                    // In a separate block to ensure the database lock guard is dropped promptly.
                    batch_generator.generate_batch(self.database.lock().deref())
                };
                self.system_executor.execute_protocol_update_action(
                    ProtocolUpdateBatchDetails {
                        protocol_version,
                        config_hash,
                        batch_group_index,
                        batch_group_name,
                        total_batch_groups,
                        batch_index,
                        batch_name,
                        total_batches,
                        enable_status_reporting,
                        start_state_identifiers,
                    },
                    batch,
                );
                {
                    // Update the transaction validator in case any of its configuration was updated.
                    // This ensures that scenarios will execute correctly.
                    *self.transaction_validator.write() = TransactionValidator::new(
                        self.database.lock().deref(),
                        &self.network_definition,
                    );
                }
            }
        }
        info!("Protocol update to {protocol_version:?} is complete");
    }

    fn resolve_batch_ledger_state_identifiers(
        &self,
        genesis_fallback: Option<StartStateIdentifiers>,
    ) -> StartStateIdentifiers {
        let db = self.database.lock();

        match db.get_latest_epoch_proof() {
            Some(latest_epoch_proof) => {
                let latest_proof = db
                    .get_latest_proof()
                    .expect("If there's an epoch proof, there must be any proof");
                StartStateIdentifiers {
                    epoch: latest_epoch_proof.ledger_header.next_epoch.unwrap().epoch,
                    proposer_timestamp_ms: latest_proof.ledger_header.proposer_timestamp_ms,
                    state_version: latest_proof.ledger_header.state_version,
                }
            }
            None => {
                // We must be mid-genesis
                let genesis_fallback = genesis_fallback
                    .expect("There is no epoch proof, but we're not executing genesis");
                match db.get_latest_proof() {
                    Some(latest_proof) => {
                        let latest_header = latest_proof.ledger_header;
                        StartStateIdentifiers {
                            epoch: genesis_fallback.epoch,
                            proposer_timestamp_ms: latest_header.proposer_timestamp_ms,
                            state_version: latest_header.state_version,
                        }
                    }
                    None => genesis_fallback,
                }
            }
        }
    }
}
