use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use radix_engine_interface::*;

use tracing::warn;

use crate::LoggingConfig;
use transaction::model::*;

/// A logic of an already-validated transaction, ready to be executed against an arbitrary state of
/// a substate store.
pub trait TransactionLogic<S> {
    fn execute_on(&self, store: &S) -> TransactionReceipt;
}

/// A well-known type of execution.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ConfigType {
    Genesis,
    Regular,
    Pending,
    Preview,
}

/// A preconfigured set of execution settings, allowing to turn `Executable` transactions into
/// `TransactionLogic`.
pub struct ExecutionConfigurator {
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: FeeReserveConfig,
    execution_configs: HashMap<ConfigType, ExecutionConfig>,
}

impl ExecutionConfigurator {
    pub fn new(logging_config: &LoggingConfig) -> Self {
        let trace = logging_config.engine_trace;
        Self {
            scrypto_interpreter: ScryptoInterpreter {
                wasm_engine: DefaultWasmEngine::default(),
                wasm_instrumenter: WasmInstrumenter::default(),
                wasm_metering_config: WasmMeteringConfig::default(),
            },
            fee_reserve_config: FeeReserveConfig::standard(),
            execution_configs: HashMap::from([
                (
                    ConfigType::Genesis,
                    ExecutionConfig::genesis().with_trace(trace),
                ),
                (
                    ConfigType::Regular,
                    ExecutionConfig::standard().with_trace(trace),
                ),
                (
                    ConfigType::Pending,
                    ExecutionConfig::up_to_loan_repayment().with_trace(trace),
                ),
                (ConfigType::Preview, ExecutionConfig::default()),
            ]),
        }
    }

    /// Wraps the given `Executable` with a configuration resolved from its `ConfigType`.
    pub fn wrap<'a>(
        &'a self,
        executable: Executable<'a>,
        config_type: ConfigType,
    ) -> ConfiguredExecutable<'a> {
        ConfiguredExecutable {
            executable,
            scrypto_interpreter: &self.scrypto_interpreter,
            fee_reserve_config: &self.fee_reserve_config,
            execution_config: self.execution_configs.get(&config_type).unwrap(),
        }
    }
}

/// An `Executable` transaction bound to a specific execution configuration.
pub struct ConfiguredExecutable<'a> {
    executable: Executable<'a>,
    scrypto_interpreter: &'a ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: &'a FeeReserveConfig,
    execution_config: &'a ExecutionConfig,
}

impl<'a> ConfiguredExecutable<'a> {
    /// Wraps this instance in a time-measuring decorator (which will log a `warn!` after the given
    /// runtime threshold).
    pub fn warn_after<S: Into<String>>(
        self,
        threshold: Duration,
        logged_description: S,
    ) -> TimeWarningTransactionLogic<Self> {
        TimeWarningTransactionLogic {
            underlying: self,
            threshold,
            logged_description: logged_description.into(),
        }
    }
}

impl<'a, S: ReadableSubstateStore> TransactionLogic<S> for ConfiguredExecutable<'a> {
    fn execute_on(&self, store: &S) -> TransactionReceipt {
        execute_transaction(
            store,
            self.scrypto_interpreter,
            self.fee_reserve_config,
            self.execution_config,
            &self.executable,
        )
    }
}

/// A time-measuring decorator for a `TransactionLogic`.
pub struct TimeWarningTransactionLogic<U> {
    underlying: U,
    threshold: Duration,
    logged_description: String, // for error-surfacing only
}

impl<U, S> TransactionLogic<S> for TimeWarningTransactionLogic<U>
where
    S: ReadableSubstateStore,
    U: TransactionLogic<S>,
{
    fn execute_on(&self, store: &S) -> TransactionReceipt {
        let start = Instant::now();
        let result = self.underlying.execute_on(store);
        let elapsed = start.elapsed();
        if elapsed > self.threshold {
            warn!(
                "Transaction execution took {}ms, above warning threshold of {}ms ({})",
                elapsed.as_millis(),
                self.threshold.as_millis(),
                self.logged_description
            );
        }
        result
    }
}
