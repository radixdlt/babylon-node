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

use crate::engine_prelude::*;
use crate::transaction::*;
use crate::*;

pub struct SystemCommitRequestFactory {
    pub epoch: Epoch,
    pub timestamp: i64,
    pub state_version: StateVersion,
    pub proof_origin: LedgerProofOrigin,
}

impl SystemCommitRequestFactory {
    /// Creates a default system commit request.
    /// By default, all system transactions are required to be successful (which is later checked by
    /// the commit logic). However, this may be customized, e.g. for the Scenarios' case (see
    /// [`SystemCommitRequest::require_committed_successes()`]).
    pub fn create(&mut self, prepare_result: SystemPrepareResult) -> SystemCommitRequest {
        let SystemPrepareResult {
            committed_transactions,
            ledger_hashes,
            next_epoch,
        } = prepare_result;
        if committed_transactions.is_empty() {
            panic!("cannot commit an empty batch of system transactions");
        }
        self.state_version = self
            .state_version
            .relative(committed_transactions.len() as i128)
            .expect("Invalid next state version!");
        SystemCommitRequest {
            transactions: committed_transactions,
            proof: self.create_proof(ledger_hashes, next_epoch),
            require_committed_successes: true,
        }
    }

    fn create_proof(&self, hashes: LedgerHashes, next_epoch: Option<NextEpoch>) -> LedgerProof {
        LedgerProof {
            ledger_header: LedgerHeader {
                epoch: self.epoch,
                round: Round::zero(),
                state_version: self.state_version,
                hashes,
                consensus_parent_round_timestamp_ms: self.timestamp,
                proposer_timestamp_ms: self.timestamp,
                next_epoch,
                next_protocol_version: None,
            },
            origin: self.proof_origin.clone(),
        }
    }
}

/// An input to [`SystemCommitRequestFactory::create()`].
pub struct SystemPrepareResult {
    pub committed_transactions: Vec<RawAndValidatedTransaction>,
    pub ledger_hashes: LedgerHashes,
    pub next_epoch: Option<NextEpoch>,
}

impl SystemPrepareResult {
    /// Creates an instance for committing the given pre-validated transactions, using the current
    /// end-state of the given series executor.
    pub fn from_committed_series(
        committed_transactions: Vec<RawAndValidatedTransaction>,
        series_executor: TransactionSeriesExecutor<impl Sized>,
    ) -> Self {
        Self {
            committed_transactions,
            ledger_hashes: *series_executor.latest_ledger_hashes(),
            next_epoch: series_executor.epoch_change().map(|event| event.into()),
        }
    }
}

/// An output from [`SystemCommitRequestFactory::create()`].
pub struct SystemCommitRequest {
    pub transactions: Vec<RawAndValidatedTransaction>,
    pub proof: LedgerProof,
    pub require_committed_successes: bool,
}

impl SystemCommitRequest {
    /// Overrides the default requirement that all system transactions are successful.
    pub fn require_committed_successes(mut self, required: bool) -> Self {
        self.require_committed_successes = required;
        self
    }
}

pub struct RawAndValidatedTransaction {
    pub raw: RawLedgerTransaction,
    pub validated: ValidatedLedgerTransaction,
}
