use crate::prelude::*;

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
    /// A (non-genesis) system protocol update transaction.
    ProtocolUpdate,
    /// A validator transaction (e.g. round update, which sometimes becomes a much larger epoch change).
    Validator,
    /// A user transaction during regular execution (e.g. prepare or commit).
    User,
    /// A user transaction during "committability check" execution (e.g. in mempool).
    UserAbortingRejectionCheck,
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
            ConfigType::Genesis | ConfigType::ProtocolUpdate | ConfigType::Validator => {
                GENESIS_TRANSACTION_RUNTIME_WARN_THRESHOLD
            }
            ConfigType::UserAbortingRejectionCheck => PENDING_UP_TO_FEE_LOAN_RUNTIME_WARN_THRESHOLD,
            ConfigType::Preview | ConfigType::PreviewNoAuth => PREVIEW_RUNTIME_WARN_THRESHOLD,
            _ => TRANSACTION_RUNTIME_WARN_THRESHOLD,
        }
    }
}

/// A preconfigured set of execution settings, allowing to turn `Executable` transactions into
/// `TransactionLogic`.
pub struct ExecutionConfigurator {
    vm_modules: DefaultVmModules,
    execution_configs: HashMap<ConfigType, ExecutionConfig>,
}

impl ExecutionConfigurator {
    pub fn new(network: &NetworkDefinition, no_fees: bool, engine_trace: bool) -> Self {
        Self {
            vm_modules: DefaultVmModules::default(),
            execution_configs: HashMap::from([
                (
                    ConfigType::Genesis,
                    ExecutionConfig::for_genesis_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::ProtocolUpdate,
                    ExecutionConfig::for_system_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::Validator,
                    ExecutionConfig::for_validator_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::User,
                    ExecutionConfig::for_notarized_transaction(network.clone())
                        .with_no_fees(no_fees)
                        .with_kernel_trace(engine_trace),
                ),
                (
                    ConfigType::UserAbortingRejectionCheck,
                    ExecutionConfig::for_notarized_transaction_rejection_check(network.clone())
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
        transaction_hashes: &LedgerTransactionHashes,
        ledger_executable: &'a LedgerExecutable,
        description: impl ToString,
    ) -> ConfiguredExecutable {
        match ledger_executable {
            LedgerExecutable::GenesisFlash => ConfiguredExecutable::SystemFlash {
                state_updates: create_system_bootstrap_flash_state_updates(),
            },
            LedgerExecutable::Flash { updates } => ConfiguredExecutable::SystemFlash {
                state_updates: updates.clone(),
            },
            LedgerExecutable::Transaction { executable } => {
                let config_type = match &transaction_hashes.kinded {
                    KindedTransactionHashes::Genesis { .. } => ConfigType::Genesis,
                    KindedTransactionHashes::User(..) => ConfigType::User,
                    KindedTransactionHashes::RoundUpdateV1 { .. } => ConfigType::Validator,
                    KindedTransactionHashes::FlashV1 { .. } => ConfigType::ProtocolUpdate,
                };

                self.wrap_transaction(executable, config_type, description.to_string())
            }
        }
    }

    pub fn wrap_pending_transaction<'a>(
        &'a self,
        executable: &'a ExecutableTransaction,
        user_hashes: &UserTransactionHashes,
    ) -> ConfiguredExecutable<'a> {
        self.wrap_transaction(
            executable,
            ConfigType::UserAbortingRejectionCheck,
            format!(
                "pending intent hash {:?}, up to fee loan",
                &user_hashes.transaction_intent_hash,
            ),
        )
    }

    pub fn wrap_preview_transaction<'a>(
        &'a self,
        executable: &'a ExecutableTransaction,
        disable_auth: bool,
    ) -> ConfiguredExecutable<'a> {
        let config_type = if disable_auth {
            ConfigType::PreviewNoAuth
        } else {
            ConfigType::Preview
        };
        self.wrap_transaction(executable, config_type, "preview".to_string())
    }

    fn wrap_transaction<'a>(
        &'a self,
        executable: &'a ExecutableTransaction,
        config_type: ConfigType,
        description: String,
    ) -> ConfiguredExecutable {
        ConfiguredExecutable::Transaction {
            executable,
            vm_modules: &self.vm_modules,
            execution_config: self.execution_configs.get(&config_type).unwrap(),
            threshold: config_type.get_transaction_runtime_warn_threshold(),
            description,
        }
    }
}

/// An `Executable` transaction bound to a specific execution configuration.
pub enum ConfiguredExecutable<'a> {
    SystemFlash {
        state_updates: StateUpdates,
    },
    Transaction {
        executable: &'a ExecutableTransaction,
        vm_modules: &'a DefaultVmModules,
        execution_config: &'a ExecutionConfig,
        threshold: Duration,
        description: String,
    },
}

impl<'a, S: SubstateDatabase> TransactionLogic<S> for ConfiguredExecutable<'a> {
    fn execute_on(self, store: &S) -> TransactionReceipt {
        match self {
            ConfiguredExecutable::SystemFlash { state_updates } => {
                FlashReceipt::from_state_updates(state_updates, store).into()
            }
            ConfiguredExecutable::Transaction {
                executable,
                vm_modules,
                execution_config,
                threshold,
                description,
            } => {
                let start = StdInstant::now();
                let result = execute_transaction(store, vm_modules, execution_config, executable);
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

/// An extension trait for easier, declarative customization of our various [`ExecutionConfig`]s.
trait CustomizedExecutionConfig {
    fn with_no_fees(self, no_fees: bool) -> Self;
}

impl CustomizedExecutionConfig for ExecutionConfig {
    fn with_no_fees(mut self, no_fees: bool) -> Self {
        if no_fees {
            self.system_overrides = Some(SystemOverrides {
                disable_costing: true,
                // Note: In practice, all ExecutionConfig's constructors set the system_overrides.
                ..self.system_overrides.unwrap_or_default()
            })
        }
        self
    }
}
