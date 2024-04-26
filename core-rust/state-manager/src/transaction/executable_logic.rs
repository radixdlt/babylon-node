use crate::engine_prelude::wasm::*;
use crate::engine_prelude::*;

use std::collections::HashMap;

use std::time::{Duration, Instant};

use tracing::warn;

use super::ValidatedLedgerTransaction;

/// A logic of an already-validated transaction, ready to be executed against an arbitrary state of
/// a substate store.
pub trait TransactionLogic<S> {
    fn execute_on(self, store: &S) -> TransactionReceipt;
}

/// A well-known type of execution.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ConfigType {
    /// A system genesis transaction.
    Genesis,
    /// A user transaction during regular execution (e.g. prepare or commit).
    Regular,
    /// A user transaction during "committability check" execution (e.g. in mempool).
    Pending,
    /// A user transaction during preview execution.
    Preview,
    /// A user transaction during preview execution with auth module disabled.
    PreviewNoAuth,
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
            ConfigType::Preview | ConfigType::PreviewNoAuth => PREVIEW_RUNTIME_WARN_THRESHOLD,
            _ => TRANSACTION_RUNTIME_WARN_THRESHOLD,
        }
    }
}

/// A preconfigured set of execution settings, allowing to turn `Executable` transactions into
/// `TransactionLogic`.
pub struct ExecutionConfigurator {
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    execution_configs: HashMap<ConfigType, ExecutionConfig>,
}

impl ExecutionConfigurator {
    pub fn new(network: &NetworkDefinition, no_fees: bool, engine_trace: bool) -> Self {
        Self {
            scrypto_vm: ScryptoVm::<DefaultWasmEngine>::default(),
            execution_configs: HashMap::from([
                (
                    ConfigType::Genesis,
                    ExecutionConfig::for_genesis_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::Regular,
                    ExecutionConfig::for_notarized_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::Pending,
                    ExecutionConfig::for_notarized_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::Preview,
                    ExecutionConfig::for_preview(network.clone()).with_no_fees(no_fees),
                ),
                (
                    ConfigType::PreviewNoAuth,
                    ExecutionConfig::for_preview_no_auth(network.clone()).with_no_fees(no_fees),
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
            transaction.get_executable().abort_when_loan_repaid(),
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
        let config_type = if validated_preview_intent.flags.disable_auth {
            ConfigType::PreviewNoAuth
        } else {
            ConfigType::Preview
        };
        self.wrap_transaction(
            validated_preview_intent.get_executable(),
            config_type,
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
                let application_events = Vec::new();
                let system_structure =
                    SystemStructure::resolve(store, &state_updates, &application_events);
                let new_node_ids = collect_new_node_ids(&state_updates);
                let state_update_summary =
                    StateUpdateSummary::new(store, new_node_ids, &state_updates);

                let commit_result = CommitResult {
                    state_updates,
                    state_update_summary,
                    fee_source: Default::default(),
                    fee_destination: Default::default(),
                    outcome: TransactionOutcome::Success(vec![]),
                    application_events,
                    application_logs: vec![],
                    system_structure,
                    execution_trace: None,
                };

                TransactionReceipt::empty_with_commit(commit_result)
            }
            ConfiguredExecutable::Transaction {
                executable,
                scrypto_interpreter,
                execution_config,
                threshold,
                description,
            } => {
                let start = Instant::now();
                let result = execute_transaction_with_configuration::<_, Vm<_, _>>(
                    store,
                    VmInit {
                        scrypto_vm: scrypto_interpreter,
                        native_vm_extension: NoExtension,
                    },
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

/// Traverses the given [`StateUpdates`] and returns [`NodeId`]s of the newly-created entities.
///
/// Note: this assumes that the [`TYPE_INFO_FIELD_PARTITION`] is mandatory and immutable, i.e. it is
/// written to exactly once, at the creation of its entity.
fn collect_new_node_ids(state_updates: &StateUpdates) -> IndexSet<NodeId> {
    state_updates
        .by_node
        .iter()
        .filter(|(_node_id, node_state_updates)| {
            let NodeStateUpdates::Delta { by_partition } = node_state_updates;
            by_partition.contains_key(&TYPE_INFO_FIELD_PARTITION)
        })
        .map(|(node_id, _node_state_updates)| *node_id)
        .collect()
}

/// An extension trait for easier, declarative customization of our various [`ExecutionConfig`]s.
trait CustomizedExecutionConfig {
    fn with_no_fees(self, no_fees: bool) -> Self;
}

impl CustomizedExecutionConfig for ExecutionConfig {
    fn with_no_fees(self, no_fees: bool) -> Self {
        let ExecutionConfig {
            enable_kernel_trace,
            enable_cost_breakdown,
            execution_trace,
            system_overrides,
        } = self;
        ExecutionConfig {
            enable_kernel_trace,
            enable_cost_breakdown,
            execution_trace,
            system_overrides: Some(SystemOverrides {
                disable_costing: no_fees,
                // Note: In practice, all ExecutionConfig's constructors set the system_overrides.
                ..system_overrides.unwrap_or_default()
            }),
        }
    }
}
