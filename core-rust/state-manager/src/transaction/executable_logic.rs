use radix_engine::system::bootstrap::{create_substate_flash_for_genesis, FlashReceipt};
use radix_engine::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::ScryptoVm;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use radix_engine_interface::*;
use radix_engine_store_interface::interface::SubstateDatabase;

use tracing::warn;

use crate::LoggingConfig;
use transaction::model::*;

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
    scrypto_interpreter: ScryptoVm<DefaultWasmEngine>,
    fee_reserve_config: FeeReserveConfig,
    pub execution_configs: HashMap<ConfigType, ExecutionConfig>,
}

impl ExecutionConfigurator {
    pub fn new(logging_config: &LoggingConfig, fee_reserve_config: FeeReserveConfig) -> Self {
        let trace = logging_config.engine_trace;
        Self {
            scrypto_interpreter: ScryptoVm::<DefaultWasmEngine>::default(),
            fee_reserve_config,
            execution_configs: HashMap::from([
                (
                    ConfigType::Genesis,
                    ExecutionConfig::for_genesis_transaction().with_kernel_trace(trace),
                ),
                (
                    ConfigType::OtherSystem,
                    ExecutionConfig {
                        // Explanation: The first epoch change in Stokenet panicked with
                        // `SystemModuleError(TransactionLimitsError(TooManyEntriesInTrack))` - so instead
                        // we override these limits with the genesis limit until they can be fixed upstream.
                        //
                        // TODO(rcnet-v3) - Remove these down when we understand what the limit should be,
                        // and have adjusted ExecutionConfig::for_system_transaction() in the engine.
                        max_number_of_substates_in_track: 50_000,
                        max_number_of_substates_in_heap: 50_000,
                        max_number_of_events: 1_000_000,
                        ..ExecutionConfig::for_system_transaction().with_kernel_trace(trace)
                    },
                ),
                (
                    ConfigType::Regular,
                    ExecutionConfig::for_notarized_transaction().with_kernel_trace(trace),
                ),
                (
                    ConfigType::Pending,
                    ExecutionConfig::for_notarized_transaction()
                        .up_to_loan_repayment(true)
                        .with_kernel_trace(trace),
                ),
                (ConfigType::Preview, ExecutionConfig::for_preview()),
            ]),
        }
    }

    /// Wraps the given `Executable` with a configuration resolved from its `ConfigType`.
    pub fn wrap_ledger_transaction<'a>(
        &'a self,
        transaction: &'a ValidatedLedgerTransaction,
        description: impl ToString,
    ) -> ConfiguredExecutable<'a> {
        if transaction.as_genesis_flash().is_some() {
            return ConfiguredExecutable::Flash {
                flash_receipt: create_substate_flash_for_genesis(),
            };
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
            scrypto_interpreter: &self.scrypto_interpreter,
            fee_reserve_config: &self.fee_reserve_config,
            execution_config: self.execution_configs.get(&config_type).unwrap(),
            threshold: config_type.get_transaction_runtime_warn_threshold(),
            description,
        }
    }
}

/// An `Executable` transaction bound to a specific execution configuration.
pub enum ConfiguredExecutable<'a> {
    Flash {
        flash_receipt: FlashReceipt,
    },
    Transaction {
        executable: Executable<'a>,
        scrypto_interpreter: &'a ScryptoVm<DefaultWasmEngine>,
        fee_reserve_config: &'a FeeReserveConfig,
        execution_config: &'a ExecutionConfig,
        threshold: Duration,
        description: String,
    },
}

impl<'a, S: SubstateDatabase> TransactionLogic<S> for ConfiguredExecutable<'a> {
    fn execute_on(self, store: &S) -> TransactionReceipt {
        match self {
            ConfiguredExecutable::Flash { flash_receipt } => flash_receipt.into(),
            ConfiguredExecutable::Transaction {
                executable,
                scrypto_interpreter,
                fee_reserve_config,
                execution_config,
                threshold,
                description,
            } => {
                let start = Instant::now();
                let result = execute_transaction(
                    store,
                    scrypto_interpreter,
                    fee_reserve_config,
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
