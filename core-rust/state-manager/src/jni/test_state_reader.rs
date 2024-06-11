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
use crate::{DetailedTransactionOutcome, LedgerTransactionOutcome, StateVersion};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use std::ops::Deref;

use crate::jni::node_rust_environment::JNINodeRustEnvironment;
use crate::query::StateManagerSubstateQueries;
use node_common::java::*;

use crate::store::traits::{
    gc::StateTreeGcStore, IterableProofStore, QueryableProofStore, QueryableTransactionStore,
    SubstateNodeAncestryStore,
};
use crate::traits::measurement::MeasurableDatabase;
use crate::transaction::LedgerTransactionHash;

//
// JNI Interface (for test purposes only)
//

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
    new_component_addresses: IndexSet<ComponentAddress>,
    new_resource_addresses: IndexSet<ResourceAddress>,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionOutcomeJava {
    Success,
    Failure,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct JavaValidatorInfo {
    pub stake_unit_resource: ResourceAddress,
    pub claim_resource: ResourceAddress,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_getTransactionAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |state_version_number: u64| -> Option<ExecutedTransaction> {
            let state_version = StateVersion::of(state_version_number);
            let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
            let database = database.snapshot();
            let committed_transaction = database.get_committed_transaction(state_version)?;
            let committed_identifiers =
                database.get_committed_transaction_identifiers(state_version)?;
            let committed_ledger_transaction_receipt =
                database.get_committed_ledger_transaction_receipt(state_version)?;
            let local_transaction_execution =
                database.get_committed_local_transaction_execution(state_version)?;

            Some(ExecutedTransaction {
                ledger_transaction_hash: committed_identifiers.payload.ledger_transaction_hash,
                outcome: match committed_ledger_transaction_receipt.outcome {
                    LedgerTransactionOutcome::Success => TransactionOutcomeJava::Success,
                    LedgerTransactionOutcome::Failure => TransactionOutcomeJava::Failure,
                },
                error_message: match local_transaction_execution.outcome {
                    DetailedTransactionOutcome::Success(_) => None,
                    DetailedTransactionOutcome::Failure(error) => Some(error.render()),
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
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_getTransactionDetailsAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |state_version_number: u64| -> Option<TransactionDetails> {
            let state_version = StateVersion::of(state_version_number);
            let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
            let committed_local_transaction_execution = database
                .snapshot()
                .get_committed_local_transaction_execution(state_version)?;

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
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_componentXrdAmount(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |component_address: ComponentAddress| -> Decimal {
            let node_id = component_address.as_node_id();
            let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
            let database = database.snapshot();

            // a quick fix for handling virtual accounts
            if database
                .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                    node_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .is_some()
            {
                let mut accounter = ResourceAccounter::new(database.deref());
                accounter.traverse(*node_id);
                let balances = accounter.close().balances;
                balances.get(&XRD).cloned().unwrap_or_else(Decimal::zero)
            } else {
                Decimal::zero()
            }
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_validatorInfo(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |validator_address: ComponentAddress| -> JavaValidatorInfo {
            let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
            let validator_state = database
                .snapshot()
                .get_mapped::<SpreadPrefixKeyMapper, ValidatorStateFieldSubstate>(
                    validator_address.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ValidatorField::State.into(),
                )
                .unwrap()
                .into_payload()
                .fully_update_and_into_latest_version();

            JavaValidatorInfo {
                stake_unit_resource: validator_state.stake_unit_resource,
                claim_resource: validator_state.claim_nft,
            }
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_epoch(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> u64 {
        let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
        let database = database.snapshot();
        database.get_epoch_and_round().0.number()
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_leastStaleStateTreeVersion(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> u64 {
        let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
        let least_stale_state_version = database
            .lock() // the `get_stale_tree_parts_iter()` is inside a trait requiring writeability
            .get_stale_tree_parts_iter()
            .next()
            .map(|(state_version, _)| state_version)
            .unwrap_or(StateVersion::pre_genesis());
        least_stale_state_version.number()
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_historicalSubstateCount(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |_: ()| -> u64 {
        let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
        let count = database
            .lock()
            .count_entries("associated_state_tree_values");
        count.try_into().expect("count out of bounds")
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_countProofsWithinEpoch(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |epoch_number: u64| -> usize {
        let epoch = Epoch::of(epoch_number);
        let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
        let database = database.snapshot();
        let epoch_proof = database.get_epoch_proof(epoch).unwrap();
        database
            .get_proof_iter(epoch_proof.ledger_header.state_version.next().unwrap())
            .take_while(|proof| proof.ledger_header.epoch == epoch)
            .count()
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_testutil_TestStateReader_getNodeGlobalRoot(
    env: JNIEnv,
    _class: JClass,
    j_rust_global_context: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |internal_address: InternalAddress| -> Option<GlobalAddress> {
            let database = JNINodeRustEnvironment::get_database(&env, j_rust_global_context);
            let node_ancestry_record = database
                .snapshot()
                .get_ancestry(internal_address.as_node_id());
            node_ancestry_record.map(|node_ancestry_record| {
                GlobalAddress::new_or_panic(node_ancestry_record.root.0 .0)
            })
        },
    )
}

pub fn export_extern_functions() {}
