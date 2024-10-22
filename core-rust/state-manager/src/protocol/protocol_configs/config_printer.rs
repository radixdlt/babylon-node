use crate::engine_prelude::*;
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
fn print_fixed_config_code() {
    let version = ProtocolVersionName::cuttlefish();
    let start_epoch = Epoch::of(1);
    let end_epoch = Epoch::of(10000000);
    let thresholds = vec![SignalledReadinessThreshold {
        required_ratio_of_stake_supported: dec!(0.8),
        required_consecutive_completed_epochs_of_support: 10,
    }];
    output(version, start_epoch, end_epoch, thresholds, None)
}

#[test]
fn print_calculated_protocol_config_code() {
    // PARAMETERS
    let calculator = CalculationParameters {
        expected_epoch_length: Duration::minutes(5),
        // This data can come from the Core API /core/state/consensus-manager response
        base_epoch: Epoch::of(97091),
        base_epoch_effective_start: DateTime::<Utc>::from_str("2024-05-09T18:01:00.000Z").unwrap(),
    };

    let version = ProtocolVersionName::cuttlefish();
    let target_start = DateTime::<Utc>::from_str("2024-06-03T18:00:00.000Z").unwrap();
    let enactment_window = Duration::days(28);
    let proposed_thresholds = [(dec!(0.75), Duration::days(14))];

    // CALCULATE
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

    output(
        version,
        start_epoch,
        end_epoch,
        thresholds,
        Some(calculator),
    )
}

fn output(
    version: ProtocolVersionName,
    start_epoch: Epoch,
    end_epoch: Epoch,
    thresholds: Vec<SignalledReadinessThreshold>,
    calculator: Option<CalculationParameters>,
) {
    // OUTPUT
    let trigger = ProtocolUpdateTrigger::of(
        version.clone(),
        ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
            lower_bound_inclusive: start_epoch,
            upper_bound_exclusive: end_epoch,
            readiness_thresholds: thresholds.clone(),
        },
    );
    trigger
        .validate()
        .expect("Generated protocol update trigger should be valid");

    let base_indent = "        ";
    println!(
        "{base_indent}ProtocolVersionName::{version}() => EnactAtStartOfEpochIfValidatorsReady {{"
    );
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
    if let Some(calculator) = &calculator {
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
    } else {
        println!(
            "{base_indent}    lower_bound_inclusive: Epoch::of({}),",
            start_epoch.number(),
        );
        println!(
            "{base_indent}    upper_bound_exclusive: Epoch::of({}),",
            end_epoch.number(),
        );
    }

    if thresholds.is_empty() {
        println!("{base_indent}    readiness_thresholds: vec![],");
    } else {
        println!("{base_indent}    readiness_thresholds: vec![");
        for threshold in thresholds {
            let ratio = threshold.required_ratio_of_stake_supported;
            let epochs = threshold.required_consecutive_completed_epochs_of_support;
            println!("{base_indent}        SignalledReadinessThreshold {{");
            println!(
                "{base_indent}            required_ratio_of_stake_supported: dec!({}),",
                ratio
            );
            if let Some(calculator) = &calculator {
                let est_duration = calculator.estimate_duration_of_full_epochs(epochs);
                println!(
                    "{base_indent}            required_consecutive_completed_epochs_of_support: {}, // estimated: {}",
                    epochs,
                    display_duration(est_duration),
                );
            } else {
                println!(
                    "{base_indent}            required_consecutive_completed_epochs_of_support: {},",
                    epochs,
                );
            }
            println!("{base_indent}        }},");
        }
        println!("{base_indent}    ],");
    }
    println!("{base_indent}}},");
}

#[test]
fn calculate_start_of_epoch() {
    let calculator = CalculationParameters {
        expected_epoch_length: Duration::minutes(5),
        // This data can come from the Core API /core/state/consensus-manager response
        base_epoch: Epoch::of(69946),
        base_epoch_effective_start: DateTime::<Utc>::from_str("2024-02-05T11:55:57.229Z").unwrap(),
    };
    println!(
        "> Time: {}",
        calculator.estimate_start_of_epoch(Epoch::of(70575))
    );
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

// Chrono's duration display isn't very human readable.
// This is straightforward code, so didn't want to pull in another arbitrary dependency for it.
// Having custom code also allows us to tailor it for the kinds of durations and precision we want.
fn display_duration(duration: Duration) -> String {
    if duration < Duration::zero() {
        panic!("Displayed duration has to be >= 0")
    }

    let total_weeks = duration.num_weeks();
    let total_days = duration.num_days();
    let total_hours = duration.num_hours();
    let total_mins = duration.num_minutes();
    let total_secs = duration.num_seconds();

    let weeks = total_weeks;
    let days = total_days - 7 * total_weeks;
    let hours = total_hours - 24 * total_days;
    let mins = total_mins - 60 * total_hours;
    let secs = total_secs - 60 * total_mins;

    let mut parts = vec![];
    if weeks > 0 {
        if weeks == 1 {
            parts.push("1 week".to_string());
        } else {
            parts.push(format!("{weeks} weeks"));
        }
    }
    if days > 0 {
        if days == 1 {
            parts.push("1 day".to_string());
        } else {
            parts.push(format!("{days} days"));
        }
    }
    if hours > 0 {
        if hours == 1 {
            parts.push("1 hour".to_string());
        } else {
            parts.push(format!("{hours} hours"));
        }
    }
    if mins > 0 {
        if mins == 1 {
            parts.push("1 min".to_string());
        } else {
            parts.push(format!("{mins} mins"));
        }
    }
    if secs > 0 {
        if secs == 1 {
            parts.push("1 sec".to_string());
        } else {
            parts.push(format!("{secs} secs"));
        }
    }

    if !parts.is_empty() {
        parts.join(", ")
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
