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

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::engine_prelude::*;
use crate::jni::LedgerSyncLimitsConfig;
use crate::protocol::{
    ProtocolConfig, ProtocolManager, ProtocolUpdateExecutor, ProtocolVersionName,
};
use crate::store::jmt_gc::StateTreeGcConfig;
use crate::store::proofs_gc::{LedgerProofsGc, LedgerProofsGcConfig};
use crate::store::traits::proofs::QueryableProofStore;
use crate::traits::DatabaseConfigValidationError;
use crate::transaction::{
    ExecutionConfigurator, LedgerTransactionValidator, Preparator, TransactionExecutorFactory,
};
use crate::{
    mempool_manager::MempoolManager,
    mempool_relay_dispatcher::MempoolRelayDispatcher,
    priority_mempool::PriorityMempool,
    store::{jmt_gc::StateTreeGc, DatabaseBackendConfig, DatabaseConfig, RawDbMetricsCollector},
    transaction::{CachedCommittabilityValidator, CommittabilityValidator, TransactionPreviewer},
    ActualStateManagerDatabase, Committer, ExecutionCacheManager, LedgerMetrics,
    PendingTransactionResultCache, ProtocolUpdateResult, StateManagerDatabase, SystemExecutor,
};
use node_common::java::{JavaError, JavaResult, StructFromJava};
use node_common::scheduler::{Metrics, Scheduler, Spawner, Tracker};
use node_common::{
    config::{limits::VertexLimitsConfig, MempoolConfig},
    locks::*,
};
use prometheus::Registry;

/// An interval between time-intensive measurement of raw DB metrics.
/// Some of our raw DB metrics take ~a few milliseconds to collect. We cannot afford the overhead of
/// updating them every time they change (i.e. on every DB commit) and we also should not perform
/// this considerable I/O within the Prometheus' exposition servlet thread - hence, a periodic task
/// (which in practice still runs more often than Prometheus' scraping).
const RAW_DB_MEASUREMENT_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct StateManagerConfig {
    pub network_definition: NetworkDefinition,
    pub mempool_config: Option<MempoolConfig>,
    pub vertex_limits_config: Option<VertexLimitsConfig>,
    pub database_backend_config: DatabaseBackendConfig,
    pub database_config: DatabaseConfig,
    pub logging_config: LoggingConfig,
    pub state_tree_gc_config: StateTreeGcConfig,
    pub ledger_proofs_gc_config: LedgerProofsGcConfig,
    pub ledger_sync_limits_config: LedgerSyncLimitsConfig,
    pub protocol_config: ProtocolConfig,
    pub no_fees: bool,
    pub scenarios_execution_config: ScenariosExecutionConfig,
}

#[derive(Debug, Clone, Default, Sbor)]
pub struct ScenariosExecutionConfig {
    pub after_protocol_updates: Vec<ProtocolUpdateScenarios>,
}

impl ScenariosExecutionConfig {
    pub fn to_run_after_protocol_update(
        &self,
        protocol_version: &ProtocolVersionName,
    ) -> Vec<String> {
        self.after_protocol_updates
            .iter()
            .filter(|config| config.protocol_version_name.as_str() == protocol_version.as_str())
            .flat_map(|config| config.scenario_names.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Default, Sbor)]
pub struct ProtocolUpdateScenarios {
    pub protocol_version_name: String,
    pub scenario_names: Vec<String>,
}

#[derive(Debug, Clone, Default, Sbor)]
pub struct LoggingConfig {
    pub engine_trace: bool,
}

impl StateManagerConfig {
    pub fn new_for_testing(rocks_db_path: impl Into<String>) -> Self {
        StateManagerConfig {
            network_definition: NetworkDefinition::simulator(),
            mempool_config: Some(MempoolConfig::new_for_testing()),
            vertex_limits_config: None,
            database_backend_config: DatabaseBackendConfig {
                rocks_db_path: rocks_db_path.into(),
            },
            database_config: DatabaseConfig::default(),
            logging_config: LoggingConfig::default(),
            state_tree_gc_config: StateTreeGcConfig::default(),
            ledger_proofs_gc_config: LedgerProofsGcConfig::default(),
            ledger_sync_limits_config: LedgerSyncLimitsConfig::default(),
            protocol_config: ProtocolConfig::new_with_no_updates(),
            no_fees: false,
            scenarios_execution_config: ScenariosExecutionConfig::default(),
        }
    }

    /// Parses the [`Self`] (see [`StructFromJava`]) and performs static validation of all
    /// applicable configuration components.
    pub fn valid_from_java(data: &[u8]) -> JavaResult<Self> {
        let instance = Self::from_java(data)?;
        instance.protocol_config.validate().map_err(JavaError)?;
        Ok(instance)
    }
}

#[derive(Clone)]
pub struct StateManager {
    pub database: Arc<DbLock<ActualStateManagerDatabase>>,
    pub mempool: Arc<RwLock<PriorityMempool>>,
    pub mempool_manager: Arc<MempoolManager>,
    pub ledger_transaction_validator: Arc<LedgerTransactionValidator>,
    pub committability_validator: Arc<RwLock<CommittabilityValidator>>,
    pub transaction_previewer: Arc<TransactionPreviewer>,
    pub preparator: Arc<Preparator>,
    pub committer: Arc<Committer>,
    pub transaction_executor_factory: Arc<TransactionExecutorFactory>,
    pub execution_cache_manager: Arc<ExecutionCacheManager>,
    pub system_executor: Arc<SystemExecutor>,
    pub pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    pub protocol_manager: Arc<ProtocolManager>,
    pub protocol_update_executor: Arc<ProtocolUpdateExecutor>,
    pub ledger_metrics: Arc<LedgerMetrics>,
}

impl StateManager {
    pub fn new(
        config: StateManagerConfig,
        mempool_relay_dispatcher: Option<MempoolRelayDispatcher>,
        lock_factory: &LockFactory,
        metrics_registry: &Registry,
        scheduler: &Scheduler<impl Spawner, impl Tracker, impl Metrics>,
    ) -> Self {
        let StateManagerConfig {
            network_definition,
            mempool_config,
            vertex_limits_config,
            database_backend_config,
            database_config,
            logging_config,
            state_tree_gc_config,
            ledger_proofs_gc_config,
            ledger_sync_limits_config,
            protocol_config,
            no_fees,
            scenarios_execution_config,
        } = config;
        let ProtocolConfig {
            genesis_protocol_version,
            protocol_update_triggers,
            protocol_update_content_overrides,
        } = protocol_config;
        let db_path = PathBuf::from(database_backend_config.rocks_db_path);
        let raw_db = match StateManagerDatabase::new(db_path, database_config, &network_definition) {
            Ok(db) => db,
            Err(error) => {
                match error {
                    DatabaseConfigValidationError::AccountChangeIndexRequiresLocalTransactionExecutionIndex => {
                        panic!("Local transaction execution index needs to be enabled in order for account change index to work.");
                    },
                    DatabaseConfigValidationError::LocalTransactionExecutionIndexChanged => {
                        panic!("Local transaction execution index can not be changed once configured.\nIf you need to change it, please wipe ledger data and resync.");
                    }
                }
            }
        };

        let database = Arc::new(lock_factory.named("database").new_db_lock(raw_db));

        let ledger_transaction_validator = Arc::new(
            LedgerTransactionValidator::default_from_network(&network_definition),
        );

        let protocol_manager = Arc::new(ProtocolManager::new(
            genesis_protocol_version,
            protocol_update_triggers,
            database.lock().deref(),
            &lock_factory.named("protocol_manager"),
            metrics_registry,
        ));

        let execution_configurator = Arc::new(ExecutionConfigurator::new(
            &network_definition,
            no_fees,
            logging_config.engine_trace,
        ));

        let mempool = Arc::new(
            lock_factory
                .named("mempool")
                .new_rwlock(PriorityMempool::new(
                    mempool_config.unwrap_or_default(),
                    metrics_registry,
                )),
        );
        let pending_transaction_result_cache =
            Arc::new(lock_factory.named("pending_cache").new_rwlock(
                PendingTransactionResultCache::new(mempool.clone(), 10000, 10000),
            ));
        let validation_config = ValidationConfig::default(network_definition.id);
        let committability_validator =
            Arc::new(lock_factory.named("committability_validator").new_rwlock(
                CommittabilityValidator::new(
                    database.clone(),
                    execution_configurator.clone(),
                    NotarizedTransactionValidator::new(validation_config),
                ),
            ));
        let cached_committability_validator = CachedCommittabilityValidator::new(
            database.clone(),
            committability_validator.clone(),
            pending_transaction_result_cache.clone(),
        );
        let mempool_manager = Arc::new(match mempool_relay_dispatcher {
            None => MempoolManager::new_for_testing(
                mempool.clone(),
                cached_committability_validator,
                metrics_registry,
            ),
            Some(mempool_relay_dispatcher) => MempoolManager::new(
                mempool.clone(),
                mempool_relay_dispatcher,
                cached_committability_validator,
                metrics_registry,
            ),
        });

        let transaction_previewer = Arc::new(TransactionPreviewer::new(
            database.clone(),
            execution_configurator.clone(),
            validation_config,
        ));

        let execution_cache_manager =
            Arc::new(ExecutionCacheManager::new(database.clone(), lock_factory));
        let transaction_executor_factory = Arc::new(TransactionExecutorFactory::new(
            execution_configurator.clone(),
            execution_cache_manager.clone(),
            protocol_manager.clone(),
        ));
        let preparator = Arc::new(Preparator::new(
            database.clone(),
            transaction_executor_factory.clone(),
            pending_transaction_result_cache.clone(),
            ledger_transaction_validator.clone(),
            vertex_limits_config.unwrap_or_default(),
            metrics_registry,
        ));

        let ledger_metrics = Arc::new(LedgerMetrics::new(
            &network_definition,
            database.lock().deref(),
            // We deliberately opt-out of measuring the "technical" locks used inside these metrics:
            &lock_factory.named("ledger_metrics").not_measured(),
            metrics_registry,
        ));

        let committer = Arc::new(Committer::new(
            database.clone(),
            transaction_executor_factory.clone(),
            ledger_transaction_validator.clone(),
            mempool_manager.clone(),
            execution_cache_manager.clone(),
            pending_transaction_result_cache.clone(),
            protocol_manager.clone(),
            ledger_metrics.clone(),
        ));

        let system_executor = Arc::new(SystemExecutor::new(
            &network_definition,
            database.clone(),
            preparator.clone(),
            committer.clone(),
        ));

        let protocol_update_executor = Arc::new(ProtocolUpdateExecutor::new(
            network_definition.clone(),
            protocol_update_content_overrides,
            scenarios_execution_config,
            database.clone(),
            system_executor.clone(),
        ));

        // If we are booting mid-protocol-update, ensure all required transactions are committed:
        protocol_update_executor.execute_protocol_update(
            &protocol_manager
                .current_protocol_state()
                .current_protocol_version,
        );

        // Register the periodic background task for collecting the costly raw DB metrics...
        let raw_db_metrics_collector =
            RawDbMetricsCollector::new(database.clone(), metrics_registry);
        scheduler
            .named("raw_db_measurement")
            .start_periodic(RAW_DB_MEASUREMENT_INTERVAL, move || {
                raw_db_metrics_collector.run()
            });

        // ... and for deleting the stale state hash tree nodes (a.k.a. "JMT GC")...
        let state_tree_gc = StateTreeGc::new(database.clone(), state_tree_gc_config);
        scheduler
            .named("state_tree_gc")
            .start_periodic(state_tree_gc.interval(), move || state_tree_gc.run());

        // ... and for deleting the old, non-critical ledger proofs (a.k.a. "Proofs GC"):
        let ledger_proofs_gc = LedgerProofsGc::new(
            database.clone(),
            ledger_proofs_gc_config,
            ledger_sync_limits_config,
        );
        scheduler.named("ledger_proofs_gc").start_periodic(
            ledger_proofs_gc.interval(),
            move || {
                ledger_proofs_gc.run();
            },
        );

        Self {
            database,
            mempool,
            mempool_manager,
            ledger_transaction_validator,
            committability_validator,
            transaction_previewer,
            preparator,
            committer,
            transaction_executor_factory,
            execution_cache_manager,
            system_executor,
            pending_transaction_result_cache,
            protocol_manager,
            protocol_update_executor,
            ledger_metrics,
        }
    }

    /// Executes the actual protocol update transactions (on-ledger) and performs any changes to the
    /// services (off-ledger) affected by the protocol update.
    /// Note: This method is only called from Java, after the consensus makes sure that the ledger
    /// is in particular state and ready for protocol update. Hence, we trust the input here and
    /// unconditionally update the internally-maintained protocol version to its new value.
    pub fn apply_protocol_update(
        &self,
        protocol_version_name: &ProtocolVersionName,
    ) -> ProtocolUpdateResult {
        self.protocol_update_executor
            .execute_protocol_update(protocol_version_name);
        self.protocol_manager
            .set_current_protocol_version(protocol_version_name);

        // Protocol update might change transaction execution rules, so we need to clear the cache:
        self.execution_cache_manager.clear();

        ProtocolUpdateResult {
            post_update_proof: self
                .database
                .lock()
                .get_latest_proof()
                .expect("Missing post protocol update proof"),
        }
    }
}
