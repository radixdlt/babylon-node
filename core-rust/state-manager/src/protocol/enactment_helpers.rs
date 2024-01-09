use crate::traits::{IterableProofStore, QueryableProofStore};
use crate::{NextEpoch, ProtocolUpdateEnactmentBound, StateVersion};
use radix_engine_common::prelude::Epoch;

/// A helper that resolves protocol update bound to a value relative
/// to the current ledger state.
#[derive(Clone, Debug)]
pub enum RelativeProtocolUpdateEnactmentBound {
    Past {
        state_version: StateVersion,
        closest_epoch_change_on_or_before: NextEpoch,
    },
    FutureStateVersion(StateVersion),
    FutureEpoch(Epoch),
}

pub fn to_relative_bound<S: QueryableProofStore + IterableProofStore>(
    store: &S,
    bound: &ProtocolUpdateEnactmentBound,
) -> RelativeProtocolUpdateEnactmentBound {
    let current_state_version = store.max_state_version();
    match bound {
        ProtocolUpdateEnactmentBound::StateVersion(state_version) => {
            if state_version <= &current_state_version {
                let closest_epoch_proof_on_or_before = store
                    .get_closest_epoch_proof_on_or_before(*state_version)
                    .expect("Invalid protocol update bound: can't be pre-genesis");
                RelativeProtocolUpdateEnactmentBound::Past {
                    state_version: *state_version,
                    closest_epoch_change_on_or_before: closest_epoch_proof_on_or_before
                        .ledger_header
                        .next_epoch
                        .expect("next_epoch is missing in epoch proof"),
                }
            } else {
                RelativeProtocolUpdateEnactmentBound::FutureStateVersion(*state_version)
            }
        }
        ProtocolUpdateEnactmentBound::Epoch(epoch) => store
            .get_epoch_proof(*epoch)
            .map(|proof| RelativeProtocolUpdateEnactmentBound::Past {
                state_version: proof.ledger_header.state_version,
                closest_epoch_change_on_or_before: proof
                    .ledger_header
                    .next_epoch
                    .expect("next_epoch is missing in epoch proof"),
            })
            .unwrap_or_else(|| RelativeProtocolUpdateEnactmentBound::FutureEpoch(*epoch)),
    }
}
