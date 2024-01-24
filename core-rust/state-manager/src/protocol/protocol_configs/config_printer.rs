use radix_engine::prelude::*;

use chrono::prelude::*;
use chrono::Duration;

use super::*;

// ============================================================================
// EXPLANATION
// ============================================================================
// This file is intended to assist with easy creation of consistent protocol config files.
// - If you need to generate bounds / target epochs, run print_protocol_config_code()
// - If you want to generate the header for a manually crafted range, run print_readiness_signals()
//
// NOTE:
// While we could use macros for this, I much rather have the code be clearly readable/explicit,
// as people like to manually read / verify the exact conditions.
// ============================================================================

#[test]
fn print_protocol_config_code() {
    // PARAMETERS
    let calculator = CalculationParameters {
        expected_epoch_length: Duration::minutes(5),
        // This data can come from the Core API /core/state/consensus-manager response
        base_epoch: Epoch::of(66516),
        base_epoch_effective_start: DateTime::<Utc>::from_str("2024-01-24T14:05:57.229Z").unwrap(),
    };

    let version = ANEMONE_PROTOCOL_VERSION;
    let version_const = stringify!(ANEMONE_PROTOCOL_VERSION);
    let target_start = DateTime::<Utc>::from_str("2024-02-05T18:00:00.000Z").unwrap();
    let enactment_window = Duration::days(14);
    let proposed_thresholds = [(dec!(0.8), Duration::days(4))];

    // OUTPUT
    let start_epoch = calculator
        .estimate_current_epoch_at(target_start)
        .next()
        .unwrap();
    let end_epoch = start_epoch
        .after(calculator.estimate_full_epochs_in_duration(enactment_window))
        .unwrap();
    let thresholds = proposed_thresholds
        .into_iter()
        .map(|(threshold, duration)| {
            let epochs = calculator.estimate_full_epochs_in_duration(duration);
            SignalledReadinessThreshold {
                required_ratio_of_stake_supported: threshold,
                required_consecutive_completed_epochs_of_support: epochs,
            }
        })
        .collect::<Vec<_>>();

    let trigger = ProtocolUpdateTrigger::of(
        version,
        ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
            lower_bound_inclusive: start_epoch,
            upper_bound_exclusive: end_epoch,
            readiness_thresholds: thresholds.clone(),
        },
    );

    let base_indent = "        ";
    println!("{base_indent}{version_const} => EnactAtStartOfEpochIfValidatorsReady {{");
    println!(
        "{base_indent}    // ================================================================="
    );
    println!("{base_indent}    // PROTOCOL_VERSION: \"{version}\"");
    println!(
        "{base_indent}    // READINESS_SIGNAL: \"{}\"",
        trigger.readiness_signal_name()
    );
    println!(
        "{base_indent}    // ================================================================="
    );
    println!("{base_indent}    // The below estimates are based off:");
    println!(
        "{base_indent}    // - Calculating relative to epoch {}",
        calculator.base_epoch.number(),
    );
    println!(
        "{base_indent}    // - Using that epoch {} started at {}",
        calculator.base_epoch.number(),
        display_instant(calculator.base_epoch_effective_start),
    );
    println!(
        "{base_indent}    // - Assuming epoch length will be {}",
        display_duration(calculator.expected_epoch_length),
    );
    println!(
        "{base_indent}    // ================================================================="
    );
    println!(
        "{base_indent}    lower_bound_inclusive: Epoch::of({}), // estimated: {}",
        start_epoch.number(),
        display_instant(calculator.estimate_start_of_epoch(start_epoch)),
    );
    println!(
        "{base_indent}    upper_bound_exclusive: Epoch::of({}), // estimated: {}",
        end_epoch.number(),
        display_instant(calculator.estimate_start_of_epoch(end_epoch)),
    );

    if thresholds.is_empty() {
        println!("{base_indent}    readiness_thresholds: vec![],");
    } else {
        println!("{base_indent}    readiness_thresholds: vec![");
        for threshold in thresholds {
            let ratio = threshold.required_ratio_of_stake_supported;
            let epochs = threshold.required_consecutive_completed_epochs_of_support;
            let est_duration = calculator.estimate_duration_of_full_epochs(epochs);
            println!("{base_indent}        SignalledReadinessThreshold {{");
            println!(
                "{base_indent}            required_ratio_of_stake_supported: dec!({}),",
                ratio
            );
            println!(
                "{base_indent}            required_consecutive_completed_epochs_of_support: {}, // estimated: {}",
                epochs,
                display_duration(est_duration),
            );
            println!("{base_indent}        }},");
        }
        println!("{base_indent}    ],");
    }
    println!("{base_indent}}},");
}

#[test]
fn print_readiness_signals() {
    let base_indent = "        ";
    for (network, config) in generate_network_configs() {
        if !config.protocol_update_triggers.is_empty() {
            println!("> Network: {network}");
        }
        for trigger in config.protocol_update_triggers {
            if let ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
                ..
            } = trigger.enactment_condition
            {
                println!("{base_indent}    // =================================================================");
                println!(
                    "{base_indent}    // PROTOCOL_VERSION: \"{}\"",
                    trigger.next_protocol_version
                );
                println!(
                    "{base_indent}    // READINESS_SIGNAL: \"{}\"",
                    trigger.readiness_signal_name()
                );
                println!("{base_indent}    // =================================================================");
                println!();
            }
        }
    }
}

fn display_duration(duration: Duration) -> String {
    if duration < Duration::zero() {
        panic!("Duration has to be >= 0")
    }
    let days = duration.num_days();
    let hours = duration.num_hours() - 24 * duration.num_days();
    let mins = duration.num_minutes() - 60 * duration.num_hours();
    let secs = duration.num_seconds() - 60 * duration.num_minutes();
    if days > 0 {
        format!("{days} days, {hours} hours, {mins} mins, {secs} secs")
    } else if hours > 0 {
        format!("{hours} hours, {mins} mins, {secs} secs")
    } else if mins > 0 {
        format!("{mins} mins, {secs} secs")
    } else if secs > 0 {
        format!("{secs} secs")
    } else {
        "Immediate".to_string()
    }
}

fn display_instant(instant: Instant) -> String {
    instant.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn generate_network_configs() -> Vec<(&'static str, ProtocolConfig)> {
    vec![
        (
            "mainnet",
            mainnet_protocol_config::mainnet_protocol_config(),
        ),
        (
            "stokenet",
            stokenet_protocol_config::stokenet_protocol_config(),
        ),
        (
            "dumunet",
            dumunet_protocol_config::dumunet_protocol_config(),
        ),
        (
            "testnet",
            testnet_protocol_config::testnet_protocol_config(),
        ),
    ]
}

type Instant = DateTime<Utc>;

struct CalculationParameters {
    expected_epoch_length: Duration,
    base_epoch: Epoch,
    base_epoch_effective_start: Instant,
}

impl CalculationParameters {
    fn estimate_start_of_epoch(&self, epoch: Epoch) -> Instant {
        self.base_epoch_effective_start
            + self.expected_epoch_length * ((epoch.number() - self.base_epoch.number()) as i32)
    }

    fn estimate_current_epoch_at(&self, instant: Instant) -> Epoch {
        Epoch::of(
            self.base_epoch.number()
                + self.estimate_full_epochs_in_duration(instant - self.base_epoch_effective_start),
        )
    }

    fn estimate_duration_of_full_epochs(&self, epochs: u64) -> Duration {
        self.expected_epoch_length * (epochs as i32)
    }

    fn estimate_full_epochs_in_duration(&self, duration: Duration) -> u64 {
        self.floor_div(duration, self.expected_epoch_length) as u64
    }

    fn floor_div(&self, duration1: Duration, duration2: Duration) -> i32 {
        (duration1.num_milliseconds() / duration2.num_milliseconds())
            .try_into()
            .unwrap()
    }
}
