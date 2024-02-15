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

use std::collections::HashSet;
use std::sync::Arc;

use crate::mempool::priority_mempool::MempoolTransaction;

use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;

use crate::engine_prelude::*;
use crate::mempool::*;
use crate::MempoolAddSource;
use node_common::java::*;

use super::node_rust_environment::JNINodeRustEnvironment;
use super::transaction_preparer::JavaPreparedNotarizedTransaction;

//
// JNI Interface
//

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_add(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |transaction: RawNotarizedTransaction| -> Result<(), MempoolAddErrorJava> {
            JNINodeRustEnvironment::get_mempool_manager(&env, j_node_rust_env).add_if_committable(
                MempoolAddSource::MempoolSync,
                transaction,
                false,
            )?;
            Ok(())
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsForProposal(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |request: ProposalTransactionsRequest| -> Vec<JavaPreparedNotarizedTransaction> {
            JNINodeRustEnvironment::get_mempool_manager(&env, j_node_rust_env)
                .get_proposal_transactions(
                    request.max_count.try_into().unwrap(),
                    request.max_payload_size_bytes as u64,
                    &request.transaction_hashes_to_exclude,
                )
                .into_iter()
                .map(|mempool_transaction| mempool_transaction.into())
                .collect()
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getCount(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_no_args: ()| -> i32 {
        let mempool = JNINodeRustEnvironment::get_mempool(&env, j_node_rust_env);
        let read_mempool = mempool.read();
        read_mempool.get_count().try_into().unwrap()
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsToRelay(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |(max_num_txns, max_payload_size_bytes): (u32, u32)| -> Vec<JavaPreparedNotarizedTransaction> {
            JNINodeRustEnvironment::get_mempool_manager(&env, j_node_rust_env)
                .get_relay_transactions(
                    max_num_txns.try_into().unwrap(),
                    max_payload_size_bytes as u64,
                )
                .into_iter()
                .map(|mempool_transaction| mempool_transaction.into())
                .collect()
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_reevaluateTransactionCommittability(
    env: JNIEnv,
    _class: JClass,
    j_node_rust_env: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |max_reevaluated_count: u32| {
        let mempool_manager = JNINodeRustEnvironment::get_mempool_manager(&env, j_node_rust_env);
        mempool_manager.reevaluate_transaction_committability(max_reevaluated_count);
    })
}

//
// DTO Models + Mapping
//

#[derive(Debug, Categorize, Encode, Decode)]
pub struct ProposalTransactionsRequest {
    pub max_count: u32,
    pub max_payload_size_bytes: u32,
    pub transaction_hashes_to_exclude: HashSet<NotarizedTransactionHash>,
}

impl From<Arc<MempoolTransaction>> for JavaPreparedNotarizedTransaction {
    fn from(value: Arc<MempoolTransaction>) -> Self {
        // TODO - Once we have "EncodeAs" SBOR mapping, we can use some, type which wraps an Arc<MempoolTransaction>
        // to extract the raw payload bytes, to avoid having to clone the full bytes here.
        Self {
            notarized_transaction_bytes: value.raw.clone(),
            intent_hash: value.intent_hash(),
            signed_intent_hash: value.signed_intent_hash(),
            notarized_transaction_hash: value.notarized_transaction_hash(),
        }
    }
}

#[derive(Debug, Categorize, Encode, Decode)]
enum MempoolAddErrorJava {
    PriorityThresholdNotMet {
        min_tip_percentage_required: Option<u32>,
        tip_percentage: u32,
    },
    Duplicate(NotarizedTransactionHash),
    TransactionValidationError(String),
    Rejected(String),
}

impl From<MempoolAddError> for MempoolAddErrorJava {
    fn from(err: MempoolAddError) -> Self {
        match err {
            MempoolAddError::PriorityThresholdNotMet {
                min_tip_percentage_required,
                tip_percentage,
            } => MempoolAddErrorJava::PriorityThresholdNotMet {
                min_tip_percentage_required: min_tip_percentage_required.map(|x| x as u32),
                tip_percentage: tip_percentage as u32,
            },
            MempoolAddError::Duplicate(hash) => MempoolAddErrorJava::Duplicate(hash),
            MempoolAddError::Rejected(rejection) => {
                MempoolAddErrorJava::Rejected(rejection.reason.to_string())
            }
        }
    }
}

impl From<TransactionValidationError> for MempoolAddErrorJava {
    fn from(error: TransactionValidationError) -> Self {
        MempoolAddErrorJava::TransactionValidationError(format!("{error:?}"))
    }
}

pub fn export_extern_functions() {}
