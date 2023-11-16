use radix_engine::system::bootstrap::FlashReceipt;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine::track::StateUpdates;
use radix_engine::transaction::{
    execute_transaction, CommitResult, CostingParameters, ExecutionConfig, SubstateSchemaMapper,
    SystemStructure, TransactionOutcome, TransactionReceipt,
};
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine_common::network::NetworkDefinition;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use radix_engine_interface::*;
use radix_engine_store_interface::interface::SubstateDatabase;

use tracing::warn;

use crate::LoggingConfig;
use transaction::model::*;
use utils::prelude::index_map_new;

use super::ValidatedLedgerTransaction;

/// A logic of an already-validated transaction, ready to be executed against an arbitrary state of
/// a substate store.
pub trait TransactionLogic<S>: Sized {
    fn execute_on(self, store: &S) -> TransactionReceipt;
}

/// A well-known type of execution.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ConfigType {
    /// A system genesis transaction.
    Genesis,
    /// A system transaction _other_ than genesis (e.g. round update).
    OtherSystem,
    /// A user transaction during regular execution (e.g. prepare or commit).
    Regular,
    /// A user transaction during "committability check" execution (e.g. in mempool).
    Pending,
    /// A user transaction during preview execution.
    Preview,
}

const PENDING_UP_TO_FEE_LOAN_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(100);
const TRANSACTION_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(500);
const GENESIS_TRANSACTION_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(2000);
const PREVIEW_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(500);

impl ConfigType {
    pub fn get_transaction_runtime_warn_threshold(&self) -> Duration {
        match self {
            ConfigType::Genesis => GENESIS_TRANSACTION_RUNTIME_WARN_THRESHOLD,
            ConfigType::Pending => PENDING_UP_TO_FEE_LOAN_RUNTIME_WARN_THRESHOLD,
            ConfigType::Preview => PREVIEW_RUNTIME_WARN_THRESHOLD,
            _ => TRANSACTION_RUNTIME_WARN_THRESHOLD,
        }
    }
}

/// A preconfigured set of execution settings, allowing to turn `Executable` transactions into
/// `TransactionLogic`.
pub struct ExecutionConfigurator {
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    pub(crate) costing_parameters: CostingParameters,
    pub execution_configs: HashMap<ConfigType, ExecutionConfig>,
}

impl ExecutionConfigurator {
    pub fn new(
        network: &NetworkDefinition,
        logging_config: &LoggingConfig,
        costing_parameters: CostingParameters,
    ) -> Self {
        let trace = logging_config.engine_trace;
        Self {
            scrypto_vm: ScryptoVm::<DefaultWasmEngine>::default(),
            costing_parameters,
            execution_configs: HashMap::from([
                (
                    ConfigType::Genesis,
                    ExecutionConfig::for_genesis_transaction(network.clone())
                        .with_kernel_trace(trace),
                ),
                (
                    ConfigType::OtherSystem,
                    ExecutionConfig {
                        max_number_of_events: 1_000_000,
                        ..ExecutionConfig::for_system_transaction(network.clone())
                            .with_kernel_trace(trace)
                    },
                ),
                (
                    ConfigType::Regular,
                    ExecutionConfig::for_notarized_transaction(network.clone())
                        .with_kernel_trace(trace),
                ),
                (
                    ConfigType::Pending,
                    ExecutionConfig::for_notarized_transaction(network.clone())
                        .up_to_loan_repayment(true)
                        .with_kernel_trace(trace),
                ),
                (
                    ConfigType::Preview,
                    ExecutionConfig::for_preview(network.clone()),
                ),
            ]),
        }
    }

    /// Wraps the given `Executable` with a configuration resolved from its `ConfigType`.
    pub fn wrap_ledger_transaction<'a>(
        &'a self,
        transaction: &'a ValidatedLedgerTransaction,
        description: impl ToString,
    ) -> ConfiguredExecutable<'a> {
        if let Some(executable) = transaction.as_flash() {
            return executable;
        }

        self.wrap_transaction(
            transaction.get_executable(),
            transaction.config_type(),
            description.to_string(),
        )
    }

    pub fn wrap_pending_transaction<'a>(
        &'a self,
        transaction: &'a ValidatedNotarizedTransactionV1,
    ) -> ConfiguredExecutable<'a> {
        self.wrap_transaction(
            transaction.get_executable(),
            ConfigType::Pending,
            format!(
                "pending intent hash {:?}, up to fee loan",
                transaction.prepared.intent_hash()
            ),
        )
    }

    pub fn wrap_preview_transaction<'a>(
        &'a self,
        validated_preview_intent: &'a ValidatedPreviewIntent,
    ) -> ConfiguredExecutable<'a> {
        self.wrap_transaction(
            validated_preview_intent.get_executable(),
            ConfigType::Preview,
            "preview".to_string(),
        )
    }

    fn wrap_transaction<'a>(
        &'a self,
        executable: Executable<'a>,
        config_type: ConfigType,
        description: String,
    ) -> ConfiguredExecutable<'a> {
        ConfiguredExecutable::Transaction {
            executable,
            scrypto_interpreter: &self.scrypto_vm,
            costing_parameters: &self.costing_parameters,
            execution_config: self.execution_configs.get(&config_type).unwrap(),
            threshold: config_type.get_transaction_runtime_warn_threshold(),
            description,
        }
    }
}

/// An `Executable` transaction bound to a specific execution configuration.
pub enum ConfiguredExecutable<'a> {
    GenesisFlash {
        flash_receipt: FlashReceipt,
    },
    SystemFlash {
        state_updates: StateUpdates,
    },
    Transaction {
        executable: Executable<'a>,
        scrypto_interpreter: &'a ScryptoVm<DefaultWasmEngine>,
        costing_parameters: &'a CostingParameters,
        execution_config: &'a ExecutionConfig,
        threshold: Duration,
        description: String,
    },
}

impl<'a, S: SubstateDatabase> TransactionLogic<S> for ConfiguredExecutable<'a> {
    fn execute_on(self, store: &S) -> TransactionReceipt {
        match self {
            ConfiguredExecutable::GenesisFlash { flash_receipt } => flash_receipt.into(),
            ConfiguredExecutable::SystemFlash { state_updates } => {
                let mut substate_schema_mapper =
                    SubstateSchemaMapper::new(SystemDatabaseReader::new(store));
                substate_schema_mapper.add_for_all_individually_updated(&state_updates);
                let substate_system_structures = substate_schema_mapper.done();

                // Sanity check that all updates are to existing nodes so that
                // we can assure there are no new entities in the receipt
                let reader = SystemDatabaseReader::new(store);
                for (node_id, ..) in &state_updates.by_node {
                    reader
                        .get_object_info(*node_id)
                        .expect("Substate flash is currently only supported for existing nodes.");
                }

                let commit_result = CommitResult {
                    state_updates,
                    state_update_summary: Default::default(),
                    fee_source: Default::default(),
                    fee_destination: Default::default(),
                    outcome: TransactionOutcome::Success(vec![]),
                    application_events: vec![],
                    application_logs: vec![],
                    system_structure: SystemStructure {
                        substate_system_structures,
                        event_system_structures: index_map_new(),
                    },
                    execution_trace: None,
                };

                TransactionReceipt::empty_with_commit(commit_result)
            }
            ConfiguredExecutable::Transaction {
                executable,
                scrypto_interpreter,
                costing_parameters,
                execution_config,
                threshold,
                description,
            } => {
                let start = Instant::now();
                let result = execute_transaction(
                    store,
                    Vm {
                        scrypto_vm: scrypto_interpreter,
                        native_vm: DefaultNativeVm::new(),
                    },
                    costing_parameters,
                    execution_config,
                    &executable,
                );
                let elapsed = start.elapsed();
                if elapsed > threshold {
                    warn!(
                        "Execution of {} took {}ms, above warning threshold of {}ms",
                        description,
                        elapsed.as_millis(),
                        threshold.as_millis(),
                    );
                }
                result
            }
        }
    }
}
