# Node's status and health

This document focuses only on health-related Prometheus metrics: discusses their meaning in detail,
and suggests some potential "Node status dashboard" ideas.

## Health's components

There is no single scalar value that would fully capture the concept of Node's "health". We can
consider the following aspects:

### Application's lifecycle status

The Node, as an application, is not fully operational right after starting its process:
- it has to initialize its sub-systems (memory strutures, worker threads),
- if started in a pristine state, then it also has to initialize its persistent Ledger (a.k.a. run 
  the Genesis Transactions predefined for the Network).

This "spin-up" lifecycle phase - under normal circumstances - is shorter than typical Prometheus
scraping interval, so it does not publish any dedicated metrics. It would also be cumbersome to even
expose the Prometheus endpoint during this phase (due to technical details of our current spin-up
implementation).

This means:
- if the Node's process has opened the Prometheus port and returns _any_ metric, then the spin-up
  phase was finished successfully and the Node now actively tries to participate in its Network,
- if the Node's process runs, but has not yet opened the Prometheus port, then it may have
  encountered some specific problems (with the environment, or with its configuration, or with the
  Genesis Transactions execution), which requires inspecting the Node's logs.
  _Please note that such problems would be hard to unambiguously capture in the metrics anyway._

### Ledger sync / commit status

After the spin-up, one of the Node's primary goals is to keep its local Ledger "synced". Nodes
constantly exchange their Ledger status information with one another and compare the following two
metrics:

rn_sync_current_state_version
: The state version of the last transaction written to the local Ledger (either coming from a BFT
  or from a Ledger sync).

rn_sync_target_state_version
: A maximum state version among those received in the status responses from the Node's peers. It
  becomes a target which the Node will try to achieve via Ledger sync process. _Please note that
  this information is not trusted (i.e. a single malicious peer can make all Nodes believe that
  their Ledger needs syncing, but it does not affect safety)._

A `rn_sync_current_state_version < rn_sync_target_state_version` is a hint for the Node to fetch
the missing transactions (together with their end proof) from the peer that declared to have them.
Applying consecutive batches of these transactions moves the Node's "local Ledger timestamp" (see
`rn_ledger_last_update_proposer_epoch_second`) towards the wallclock (assuming the clocks of current
active Validator Set are accurate).

rn_ledger_last_update_epoch_second
: A wallclock timestamp captured at the last write to the local persistent Ledger (either coming
from a BFT commit or from a Ledger sync). If this reading grows - or, equivalently, if its value
is regularly being bumped to the current wallclock - then it means that the Node manages to
receive regular updates from the Network.

rn_ledger_last_update_proposer_epoch_second
: A proposer timestamp contained by the last Proof written to the local persistent Ledger (either
coming from a BFT commit or from a Ledger sync).

In practice, the above metrics are sufficient to distinguish between the following soft states of
the Node's sync with the Network.

#### State: Syncing

On a healthy syncing Node, the `rn_ledger_last_update_proposer_epoch_second` is significantly
behind wallclock, but should progress towards it relatively quickly.

A healthy Node is expected to eventually become synced. This will happen only if the "syncing rate"
(i.e. a progression of the Ledger timestamp in a time unit) is faster than realtime - in Prometheus
language, `rate(rn_ledger_last_update_proposer_epoch_second{...}[$rate_interval]) > 1.0`.
Values lower than this suggest that the Node's I/O or processing speed does not allow it to record
the transactions committed by the Network in realtime - and such Node cannot be considered healthy.

Additional indicators of a healthy-but-still-syncing Node:

- `rn_sync_current_state_version` is `< rn_sync_target_state_version` but approaches it.
  - This shows how many transactions still need to be ingested to become synced.

- `rn_ledger_last_update_proposer_epoch_second` is `< rn_sync_target_proposer_timestamp_epoch_second`
  but approaches it.
  - This shows how much "Ledger time" still needs to be ingested.
  - In case of healthy Validator Set, the `rn_sync_target_proposer_timestamp_epoch_second` should be
    very close to the wallclock.
  - However, in case of liveness break, the `rn_sync_target_proposer_timestamp_epoch_second` will
    lag behind the wallclock, and even a healthy Node cannot sync past that value. Technically, a 
    condition for a synced Node is `rn_ledger_last_update_proposer_epoch_second == rn_sync_target_proposer_timestamp_epoch_second`
    (or less ambiguously: `rn_sync_current_state_version == rn_sync_target_state_version`).

Please bear in mind that both `rn_sync_target_proposer_timestamp_epoch_second` and
`rn_sync_target_state_version` come from the Node's peers and are not verified in any way, so the
"synced node" condition described above cannot be trusted blindly.

#### State: Synced

On a healthy synced Node, the `rn_ledger_last_update_proposer_epoch_second` should always stay
close to the wallclock, since a healthy Network always keeps committing to the Ledger (even in case
of no "business" transactions, the consensus epoch/round updates are being committed - otherwise
there must be a liveness break).

A synced Node may either belong to an active Validator Set (and perform BFT commits), or receive
timely Ledger sync updates.

#### State: Out-of-sync

On an unhealthy, out-of-sync Node, the `rn_ledger_last_update_proposer_epoch_second` keeps moving
further and further into the past, despite the rest of the Network (most importantly: Validator Set)
being healthy.

Of course, a local connectivity loss can be hard to distinguish from the Network's liveness break,
but some other metrics can narrow it down:

- A non-progressing `rn_sync_target_proposer_timestamp_epoch_second` and `rn_sync_target_state_version`
  are strong indicators of the Network's liveness problems (unless _all_ connected peers coordinated
  to lie to the Node).
- A growing `rn_sync_unexpected_responses_received_total` hints at network latency/timeout issues.
- A growing `rn_sync_invalid_responses_received_total` (especially when caused by
  `MISMATCHED_TRANSACTION_ROOT`) hints at malformed local Ledger state.
- _(if the Node is in active Validator Set)_ A growing `rn_bft_rejected_consensus_events_total`
  hints at byzantine behavior of the other Validators.

### Consensus status

The consensus health is harder to categorize into distinct states. It may be required to interpret
each related metric individually: 

rn_misc_wallclock_epoch_second
: A current wallclock, as seen by the Node precisely at the moment of rendering the Prometheus
endpoint response. All metrics which record timestamps (i.e. named `*_epoch_second`) should be
interpreted as relative to this one. A Node with a seriously misconfigured clock will most
likely not be a healthy Validator, since timestamps from BFT messages are verified against it.

rn_misc_config_info
: A Prometheus `info` metric containing a few items related to the Node's resolved configuration.

The labels of `rn_misc_config_info` related to the Node's consensus health:
- `post_genesis_epoch_state_hash`
  A state hash captured after the execution of all the Genesis Transactions. This should be constant
  throughout the Node's lifetime, and the same across the entire Network. 
- `configured_validator_address`
  A logical Validator component address (within the Engine). Only present (i.e. non-empty) if it 
  was statically configured or resolved from Genesis.

rn_bft_in_validator_set
: A "boolean" gauge, showing `1.0` when this Node belongs to an active Validator Set of the
_current_ epoch (or `0.0` otherwise).
The `1.0` here is only possible when `rn_misc_config_info.configured_validator_address` is
non-empty.
Please note that the _current_ here means "according to the last epoch header written to the Node's
local Ledger" - which makes this reading stale if the Node is not synced.

rn_bft_proposal_timestamp_difference_seconds { key=..., component_address=... }
: The difference between the specified Validator's wallclock and this Node's wallclock, as captured
during the proposal creation / proposal handling.
This "clock skew" actually includes the network delay too - but this is the number we care about,
since we accept/reject proposals based on this clock difference. The protocol defines the acceptable
bounds.
If all the numbers observed here exceed the upper bound, then the Node's clock may be delayed.
If all the numbers observed here exceed the lower bound, then either the Node's clock is rushing, or
the Node's latency to its peers is too high.

rn_ledger_consensus_rounds_committed { leader_component_address=..., round_resolution=... }
: A historical record of the specified Validator's proposal reliability. Each round has a leader
responsible for the corresponding proposal, and may resolve as `Successful`, `MissedByFallback` or
`MissedByGap`. If the Node belonged to the active Validator Set of a certain Epoch, then it was a
leader of at least one round of that Epoch, and a healthy Validator should always be successful.
Fallback rounds denote problems with just this Node, and gaps in round numbers most likely denote
problems with liveness of the entire Network.
