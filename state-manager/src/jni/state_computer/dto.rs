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

use radix_engine::fee::FeeSummary;
use radix_engine::transaction::{PreviewResult, TransactionOutcome, TransactionResult};
use sbor::{Decode, Encode, TypeId};
use scrypto::component::{ComponentAddress, PackageAddress};
use scrypto::core::Level;
use scrypto::math::Decimal;
use scrypto::prelude::ResourceAddress;

use crate::result::StateManagerResult;
use crate::types::PreviewError;

#[derive(Debug, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct PreviewErrorJava {
    message: String,
}

impl From<PreviewError> for StateManagerResult<PreviewErrorJava> {
    fn from(err: PreviewError) -> Self {
        let msg: String = match err {
            PreviewError::InvalidManifest => "Invalid manifest".to_string(),
            PreviewError::InvalidSignerPublicKey => "Invalid signer public key".to_string(),
            PreviewError::EngineError(engine_preview_error) => {
                format!("Preview execution failed: {:?}", engine_preview_error)
            }
        };
        Ok(PreviewErrorJava { message: msg })
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum TransactionStatusJava {
    Rejected(String),
    Succeeded(Vec<Vec<u8>>),
    Failed(String),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct FeeSummaryJava {
    pub loan_fully_repaid: bool,
    pub cost_unit_limit: u32,
    pub cost_units_consumed: u32,
    pub cost_unit_price: Decimal,
    pub tip_percentage: u32,
    pub burned: Decimal,
    pub tipped: Decimal,
}

impl From<FeeSummary> for FeeSummaryJava {
    fn from(fee_summary: FeeSummary) -> Self {
        FeeSummaryJava {
            loan_fully_repaid: fee_summary.loan_fully_repaid,
            cost_unit_limit: fee_summary.cost_unit_limit,
            cost_units_consumed: fee_summary.cost_unit_consumed,
            cost_unit_price: fee_summary.cost_unit_price,
            tip_percentage: fee_summary.tip_percentage,
            burned: fee_summary.burned,
            tipped: fee_summary.tipped,
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PreviewResultJava {
    status: TransactionStatusJava,
    fee_summary: FeeSummaryJava,
    application_logs: Vec<(Level, String)>,
    new_package_addresses: Vec<PackageAddress>,
    new_component_addresses: Vec<ComponentAddress>,
    new_resource_addresses: Vec<ResourceAddress>,
}

impl From<PreviewResult> for PreviewResultJava {
    fn from(result: PreviewResult) -> Self {
        let receipt = result.receipt;

        let (status, entity_changes) = match receipt.result {
            TransactionResult::Commit(commit) => match commit.outcome {
                TransactionOutcome::Success(output) => (
                    TransactionStatusJava::Succeeded(output),
                    Some(commit.entity_changes),
                ),
                TransactionOutcome::Failure(error) => (
                    TransactionStatusJava::Failed(error.to_string()),
                    Some(commit.entity_changes),
                ),
            },
            TransactionResult::Reject(reject) => (
                TransactionStatusJava::Rejected(reject.error.to_string()),
                None,
            ),
        };

        let (new_package_addresses, new_component_addresses, new_resource_addresses) =
            match entity_changes {
                Some(ec) => (
                    ec.new_package_addresses,
                    ec.new_component_addresses,
                    ec.new_resource_addresses,
                ),
                None => (Vec::new(), Vec::new(), Vec::new()),
            };

        let (fee_summary, application_logs) = {
            let execution = receipt.execution;
            (execution.fee_summary, execution.application_logs)
        };

        PreviewResultJava {
            status,
            fee_summary: fee_summary.into(),
            application_logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
        }
    }
}
