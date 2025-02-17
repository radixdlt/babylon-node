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
use jni::objects::{GlobalRef, JObject, JValue};
use jni::{JNIEnv, JavaVM};
use std::ops::Deref;

use node_common::java::*;

use super::JavaGenesisData;

pub trait ResolveGenesisData: Send + Sync + 'static {
    fn get_genesis_data_hash(&self) -> Hash {
        hash(self.get_raw_genesis_data())
    }

    fn get_raw_genesis_data(&self) -> Vec<u8>;
}

pub struct FixedGenesisDataResolver {
    raw_data: Vec<u8>,
    raw_data_hash: Hash,
}

impl FixedGenesisDataResolver {
    pub fn new(genesis_data: JavaGenesisData) -> Self {
        let raw_data = scrypto_encode(&genesis_data).unwrap();
        let raw_data_hash = hash(&raw_data);
        Self {
            raw_data,
            raw_data_hash,
        }
    }
}

impl ResolveGenesisData for FixedGenesisDataResolver {
    fn get_raw_genesis_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }

    fn get_genesis_data_hash(&self) -> Hash {
        self.raw_data_hash
    }
}

/// A Java dispatcher for reading the GenesisData lazily.
pub struct JavaGenesisDataResolver {
    jvm: JavaVM,
    j_rust_global_context_ref: GlobalRef,
}

impl ResolveGenesisData for JavaGenesisDataResolver {
    fn get_raw_genesis_data(&self) -> Vec<u8> {
        self.read_raw_genesis_data()
            .expect("Genesis data should be readable from Java")
    }

    fn get_genesis_data_hash(&self) -> Hash {
        self.read_raw_genesis_data_hash()
            .expect("Genesis hash should be readable from Java")
    }
}

// Similar to the MempoolRelayDispatcher
// TODO: If a number of Rust->Java calls grows, we could invest in a small util for building the
// "method IDs" (i.e. especially these descriptors) in some more convenient way.
impl JavaGenesisDataResolver {
    /// The Java trigger method's descriptor string (i.e. as defined by
    /// [JVMS](https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-4.html#jvms-4.3.3)).
    const NO_ARGS_RETURNS_BYTES_METHOD_DESCRIPTOR: &'static str = "()[B";

    /// Creates a long-lived dispatcher from the given short-lived JNI context and Java state
    /// manager reference.
    pub fn new(env: &JNIEnv, j_rust_global_context: JObject) -> jni::errors::Result<Self> {
        Ok(Self {
            jvm: env.get_java_vm()?,
            j_rust_global_context_ref: env.new_global_ref(j_rust_global_context)?,
        })
    }

    pub fn read_raw_genesis_data_hash(&self) -> JavaResult<Hash> {
        let bytes = self.read_byte_array("readGenesisDataHash")?;
        let array = bytes.try_into().map_err(|_| {
            JavaError("readGenesisDataHash returned the wrong number of bytes".to_string())
        })?;
        Ok(Hash::from_bytes(array))
    }

    pub fn read_raw_genesis_data(&self) -> JavaResult<Vec<u8>> {
        self.read_byte_array("readGenesisData")
    }

    fn read_byte_array(&self, method_name: &str) -> JavaResult<Vec<u8>> {
        let attachment = self.jvm.attach_current_thread()?;
        let env = attachment.deref();
        let j_rust_global_context = self.j_rust_global_context_ref.as_obj();
        let result = env.call_method(
            j_rust_global_context,
            method_name,
            Self::NO_ARGS_RETURNS_BYTES_METHOD_DESCRIPTOR,
            &[],
        );
        if result.is_err() && env.exception_check()? {
            env.exception_clear()?;
        }
        let JValue::Object(jobject) = result? else {
            panic!("Unexpected value returned from {}", method_name);
        };
        jni_jbytearray_to_vector(env, *jobject)
    }
}
