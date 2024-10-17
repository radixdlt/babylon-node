/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

use crate::prelude::*;

mod mempool_manager;
mod mempool_relay_dispatcher;
mod metrics;
mod pending_transaction_result_cache;
mod priority_mempool;

pub use mempool_manager::*;
pub use mempool_relay_dispatcher::*;
pub use metrics::*;
pub use pending_transaction_result_cache::*;
pub use priority_mempool::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MempoolAddSource {
    CoreApi,
    MempoolSync,
}

#[derive(Debug, Clone)]
pub enum MempoolAddError {
    PriorityThresholdNotMet {
        min_tip_basis_points_required: Option<u32>,
        tip_basis_points: u32,
    },
    Duplicate(NotarizedTransactionHash),
    Rejected(MempoolAddRejection),
}

#[derive(Debug, Clone)]
pub struct MempoolAddRejection {
    pub reason: MempoolRejectionReason,
    pub against_state: AtState,
    pub retry_from: RetryFrom,
    pub was_cached: bool,
    /// The epoch when the payload will definitely be permanently rejected
    /// This isn't always provided if the rejection is permanent
    pub invalid_from_epoch: Option<Epoch>,
}

impl MempoolAddRejection {
    pub fn for_static_rejection(validation_error: TransactionValidationError) -> Self {
        Self {
            reason: MempoolRejectionReason::ValidationError(validation_error),
            against_state: AtState::Static,
            retry_from: RetryFrom::Never,
            was_cached: false,
            invalid_from_epoch: None,
        }
    }

    pub fn is_permanent_for_payload(&self) -> bool {
        match &self.against_state {
            AtState::Specific(specific) => match specific {
                AtSpecificState::Committed { .. } => self.reason.is_permanent_for_payload(),
                AtSpecificState::PendingPreparingVertices { .. } => false,
            },
            AtState::Static => self.reason.is_permanent_for_payload(),
        }
    }

    pub fn is_permanent_for_intent(&self) -> bool {
        match &self.against_state {
            AtState::Specific(specific) => match specific {
                AtSpecificState::Committed { .. } => self.reason.is_permanent_for_intent(),
                AtSpecificState::PendingPreparingVertices { .. } => false,
            },
            AtState::Static => self.reason.is_permanent_for_payload(),
        }
    }

    pub fn is_rejected_because_intent_already_committed(&self) -> bool {
        match &self.against_state {
            AtState::Specific(specific) => match specific {
                AtSpecificState::Committed { .. } => {
                    self.reason.is_rejected_because_intent_already_committed()
                }
                AtSpecificState::PendingPreparingVertices { .. } => false,
            },
            AtState::Static => false,
        }
    }
}

impl Display for MempoolAddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MempoolAddError::PriorityThresholdNotMet {
                min_tip_basis_points_required,
                tip_basis_points,
            } => match min_tip_basis_points_required {
                None => {
                    write!(f, "Priority Threshold not met. There is no known tip to guarantee mempool submission.")
                }
                Some(min_tip_percentage_required) => {
                    write!(f, "Priority Threshold not met: tip is {tip_basis_points} basis points while min tip required {min_tip_percentage_required} basis points")
                }
            },
            MempoolAddError::Duplicate(_) => write!(f, "Duplicate Entry"),
            MempoolAddError::Rejected(rejection) => write!(f, "{}", rejection.reason),
        }
    }
}
