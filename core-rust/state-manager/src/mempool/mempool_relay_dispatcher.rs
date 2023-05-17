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

use jni::errors::Result;
use jni::objects::{GlobalRef, JObject, JValue};
use jni::{JNIEnv, JavaVM};
use radix_engine_common::data::scrypto::scrypto_encode;
use std::ops::Deref;

use node_common::java::*;
use transaction::model::NotarizedTransaction;

use crate::jni::mempool::JavaRawTransaction;

/// A Java dispatcher for a "new transaction from Core API" event.
pub struct MempoolRelayDispatcher {
    jvm: JavaVM,
    j_state_manager_ref: GlobalRef,
}

impl MempoolRelayDispatcher {
    /// Name of the "mempool relay trigger" method on the Java side.
    const TRIGGER_METHOD_NAME: &'static str = "triggerMempoolRelay";

    /// The Java trigger method's descriptor string (i.e. as defined by
    /// [JVMS](https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-4.html#jvms-4.3.3)).
    const TRIGGER_METHOD_DESCRIPTOR: &'static str = "([B)V";

    // TODO: If a number of Rust->Java calls grows, we could invest in a small util for building the
    // "method IDs" (i.e. especially these descriptors) in some more convenient way.

    /// Creates a long-lived dispatcher from the given short-lived JNI context and Java state
    /// manager reference.
    pub fn new(env: &JNIEnv, j_state_manager: JObject) -> Result<Self> {
        Ok(Self {
            jvm: env.get_java_vm()?,
            j_state_manager_ref: env.new_global_ref(j_state_manager)?,
        })
    }

    /// Triggers a relay of the given transaction.
    /// This is implemented using an event dispatch on the Java side, so this method should not
    /// block. Any Java exceptions will be returned as `Err` (i.e. their Java exception status will
    /// be cleared, as if this method was catching them).
    pub fn trigger_relay(&self, transaction: NotarizedTransaction) -> Result<()> {
        let attachment = self.jvm.attach_current_thread()?;
        let env = attachment.deref();
        let j_state_manager = self.j_state_manager_ref.as_obj();
        let serialized_transaction =
            scrypto_encode(&JavaRawTransaction::from(transaction)).unwrap();
        let result = env.call_method(
            j_state_manager,
            MempoolRelayDispatcher::TRIGGER_METHOD_NAME,
            MempoolRelayDispatcher::TRIGGER_METHOD_DESCRIPTOR,
            &[JValue::Object(JObject::from(jni_slice_to_jbytearray(
                env,
                &serialized_transaction,
            )))],
        );
        if result.is_err() && env.exception_check()? {
            env.exception_clear()?;
        }
        result.map(|_| ())
    }
}
