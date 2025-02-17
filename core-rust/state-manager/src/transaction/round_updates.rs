use crate::prelude::*;

pub fn create_round_update_transaction(
    epoch_header: Option<&LedgerHeader>,
    round_history: &RoundHistory,
) -> RoundUpdateTransactionV1 {
    let validator_index_by_address = iterate_validators(epoch_header)
        .enumerate()
        .map(|(validator_index, validator_id)| {
            (
                validator_id.component_address,
                ValidatorIndex::try_from(validator_index)
                    .expect("validator set size limit guarantees this"),
            )
        })
        .collect::<NonIterMap<_, _>>();
    RoundUpdateTransactionV1 {
        proposer_timestamp_ms: round_history.proposer_timestamp_ms,
        epoch: round_history.epoch,
        round: round_history.round,
        leader_proposal_history: LeaderProposalHistory {
            gap_round_leaders: round_history
                .gap_round_leader_addresses
                .iter()
                .map(|leader_address| {
                    *validator_index_by_address
                        .get(leader_address)
                        .expect("gap round leader must belong to the validator set")
                })
                .collect::<Vec<_>>(),
            current_leader: *validator_index_by_address
                .get(&round_history.proposer_address)
                .expect("proposer must belong to the validator set"),
            is_fallback: round_history.is_fallback,
        },
    }
}

/// A builder of "successful/timeout/gap rounds by leader" counter update, for metrics purposes.
#[derive(Default)]
pub struct LeaderRoundCountersBuilder {
    /// Counters per validator (of the corresponding [`ValidatorIndex`]).
    /// Implementation note:
    /// This structure starts as an empty one (on the builder's initialization via `default()`).
    /// Then, it is lazily-initialized by any larger-than-previously-observed [`ValidatorIndex`]
    /// (within `update()`; see `get_counter_mut()`). The size of this vector is thus effectively
    /// bounded by `u8::MAX` (even if validator indices found in proposal history were invalid).
    counters_by_index: Vec<LeaderRoundCounter>,
}

impl LeaderRoundCountersBuilder {
    /// Increments the counters according to the information from proposal history.
    /// This will at least update the current round leader's entry (either being successful or
    /// fallback), and potentially many gap rounds (that were missed by validators since the
    /// previously reported round change).
    pub fn update(&mut self, leader_proposal_history: &LeaderProposalHistory) {
        let current_leader_counter = self.get_counter_mut(&leader_proposal_history.current_leader);
        if leader_proposal_history.is_fallback {
            current_leader_counter.missed_by_fallback += 1;
        } else {
            current_leader_counter.successful += 1;
        }
        for gap_round_leader_index in leader_proposal_history.gap_round_leaders.iter() {
            let gap_round_leader_counter = self.get_counter_mut(gap_round_leader_index);
            gap_round_leader_counter.missed_by_gap += 1;
        }
    }

    /// Finalizes the build of the counters per validator.
    /// Resolves the validator [`ComponentAddress`]es from the given epoch header.
    /// Returns only the entries of validators for which some counts have changed.
    pub fn build(
        self,
        epoch_header: Option<&LedgerHeader>,
    ) -> Vec<(ValidatorId, LeaderRoundCounter)> {
        self.counters_by_index
            .into_iter()
            .zip(iterate_validators(epoch_header))
            .filter(|(counter, _)| counter.is_non_zero())
            .map(|(counter, validator_id)| (validator_id, counter))
            .collect()
    }

    fn get_counter_mut(&mut self, validator_index: &ValidatorIndex) -> &mut LeaderRoundCounter {
        let index = *validator_index as usize;
        if self.counters_by_index.len() <= index {
            self.counters_by_index
                .resize_with(index + 1, LeaderRoundCounter::default);
        }
        self.counters_by_index
            .get_mut(index)
            .expect("ensured by the branch above")
    }
}

/// A set of counters of rounds led by a concrete leader.
#[derive(Default, Clone, Debug, ScryptoSbor)]
pub struct LeaderRoundCounter {
    pub successful: usize,
    pub missed_by_fallback: usize,
    pub missed_by_gap: usize,
}

impl LeaderRoundCounter {
    /// Returns a sum of both kinds of missed rounds.
    pub fn missed(&self) -> usize {
        self.missed_by_fallback + self.missed_by_gap
    }

    /// Returns true if *any* of the counters is non-zero.
    pub fn is_non_zero(&self) -> bool {
        self.successful != 0 || self.missed_by_fallback != 0 || self.missed_by_gap != 0
    }
}

/// Extracts an iterator of validator IDs (in their [`ValidatorIndex`] order) from
/// the given epoch header (i.e. assumes that it was found and contains the "next epoch").
fn iterate_validators(
    epoch_header: Option<&LedgerHeader>,
) -> impl Iterator<Item = ValidatorId> + '_ {
    epoch_header
        .expect("at least genesis epoch is expected")
        .next_epoch
        .as_ref()
        .expect("epoch header must contain next epoch information")
        .validator_set
        .iter()
        .map(ValidatorId::from)
}
