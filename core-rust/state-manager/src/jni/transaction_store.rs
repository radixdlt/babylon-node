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

use crate::jni::state_manager::ActualStateManager;
use crate::store::traits::*;

use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::transaction::TransactionOutcome;
use radix_engine::types::{scrypto_encode, ComponentAddress};
use radix_engine_interface::scrypto;
use sbor::{Decode, Encode, TypeId};

use super::mempool::JavaPayloadHash;
use super::utils::jni_state_manager_sbor_read_call;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
struct ExecutedTransaction {
    outcome: TransactionOutcomeJava,
    ledger_receipt_bytes: Vec<u8>,
    transaction_bytes: Vec<u8>,
    /// Used by some Java tests, consider removing at some point as it doesn't really fit here
    new_component_addresses: Vec<ComponentAddress>,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TransactionOutcomeJava {
    Success(Vec<Vec<u8>>),
    Failure(String),
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getTransactionAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(
        env,
        j_state_manager,
        request_payload,
        do_get_transaction_at_state_version,
    )
}

fn do_get_transaction_at_state_version(
    state_manager: &ActualStateManager,
    state_version: u64,
) -> Option<ExecutedTransaction> {
    let payload_hash = state_manager
        .staged_store
        .root
        .get_payload_hash(state_version)?;

    let (stored_transaction, ledger_receipt, _) = state_manager
        .staged_store
        .root
        .get_committed_transaction(&payload_hash)?;

    let ledger_receipt_bytes = scrypto_encode(&ledger_receipt).unwrap();

    Some(ExecutedTransaction {
        outcome: match ledger_receipt.outcome {
            TransactionOutcome::Success(output) => TransactionOutcomeJava::Success(output),
            TransactionOutcome::Failure(err) => {
                TransactionOutcomeJava::Failure(format!("{:?}", err))
            }
        },
        ledger_receipt_bytes,
        transaction_bytes: stored_transaction.create_payload().unwrap(),
        new_component_addresses: ledger_receipt.entity_changes.new_component_addresses,
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getNextProof(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, request_payload, do_get_next_proof)
}

#[tracing::instrument(skip_all)]
fn do_get_next_proof(
    state_manager: &ActualStateManager,
    state_version: u64,
) -> Option<(Vec<JavaPayloadHash>, Vec<u8>)> {
    let (payload_hashes, proof) = state_manager
        .staged_store
        .root
        .get_next_proof(state_version)?;

    let payload_hashes = payload_hashes.into_iter().map(|hash| hash.into()).collect();

    Some((payload_hashes, proof))
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_REv2TransactionAndProofStore_getLastProof(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, request_payload, do_get_last_proof)
}

#[tracing::instrument(skip_all)]
fn do_get_last_proof(state_manager: &ActualStateManager, _args: ()) -> Option<Vec<u8>> {
    state_manager.staged_store.root.get_last_proof()
}

pub fn export_extern_functions() {}
