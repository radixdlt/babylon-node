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

use crate::jni::state_manager::JNIStateManager;
use crate::store::traits::*;
use crate::transaction::{LedgerTransactionHash, RawLedgerTransaction};
use crate::{DetailedTransactionOutcome, LedgerProof, LedgerTransactionOutcome, StateVersion};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use node_common::java::*;
use radix_engine::types::*;

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct ExecutedTransaction {
    ledger_transaction_hash: LedgerTransactionHash,
    outcome: TransactionOutcomeJava,
    error_message: Option<String>,
    consensus_receipt_bytes: Vec<u8>,
    transaction_bytes: Vec<u8>,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct TransactionDetails {
    new_component_addresses: Vec<ComponentAddress>,
    new_resource_addresses: Vec<ResourceAddress>,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionOutcomeJava {
    Success,
    Failure,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct TxnsAndProofRequest {
    start_state_version_inclusive: u64,
    max_number_of_txns_if_more_than_one_proof: u32,
    max_payload_size_in_bytes: u32,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct TxnsAndProof {
    transactions: Vec<RawLedgerTransaction>,
    proof: LedgerProof,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getTransactionAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |state_version_number: u64| -> Option<ExecutedTransaction> {
            let state_version = StateVersion::of(state_version_number);
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let read_database = database.read();
            let committed_transaction = read_database.get_committed_transaction(state_version)?;
            let committed_identifiers =
                read_database.get_committed_transaction_identifiers(state_version)?;
            let committed_ledger_transaction_receipt =
                read_database.get_committed_ledger_transaction_receipt(state_version)?;
            let local_transaction_execution =
                read_database.get_committed_local_transaction_execution(state_version)?;

            Some(ExecutedTransaction {
                ledger_transaction_hash: committed_identifiers.payload.ledger_payload_hash,
                outcome: match committed_ledger_transaction_receipt.outcome {
                    LedgerTransactionOutcome::Success => TransactionOutcomeJava::Success,
                    LedgerTransactionOutcome::Failure => TransactionOutcomeJava::Failure,
                },
                error_message: match local_transaction_execution.outcome {
                    DetailedTransactionOutcome::Success(_) => None,
                    DetailedTransactionOutcome::Failure(err) => Some(format!("{err:?}")),
                },
                consensus_receipt_bytes: scrypto_encode(
                    &committed_ledger_transaction_receipt.get_consensus_receipt(),
                )
                .unwrap(),
                transaction_bytes: committed_transaction.0,
            })
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getTransactionDetailsAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |state_version_number: u64| -> Option<TransactionDetails> {
            let state_version = StateVersion::of(state_version_number);
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let read_database = database.read();
            let committed_local_transaction_execution =
                read_database.get_committed_local_transaction_execution(state_version)?;

            Some(TransactionDetails {
                new_component_addresses: committed_local_transaction_execution
                    .state_update_summary
                    .new_components,
                new_resource_addresses: committed_local_transaction_execution
                    .state_update_summary
                    .new_resources,
            })
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getTxnsAndProof(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |request: TxnsAndProofRequest| -> Option<TxnsAndProof> {
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let txns_and_proof = database.read().get_txns_and_proof(
                StateVersion::of(request.start_state_version_inclusive),
                request.max_number_of_txns_if_more_than_one_proof,
                request.max_payload_size_in_bytes,
            );
            txns_and_proof.map(|(transactions, proof)| TxnsAndProof {
                transactions,
                proof,
            })
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getFirstEpochProof(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> Option<LedgerProof> {
        let database = JNIStateManager::get_database(&env, j_state_manager);
        let proof = database.read().get_first_epoch_proof();
        proof
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getEpochProof(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |epoch: Epoch| -> Option<LedgerProof> {
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let proof = database.read().get_epoch_proof(epoch);
            proof
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getLastProof(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |_no_args: ()| -> Option<LedgerProof> {
            let database = JNIStateManager::get_database(&env, j_state_manager);
            let proof = database.read().get_last_proof();
            proof
        },
    )
}

pub fn export_extern_functions() {}
