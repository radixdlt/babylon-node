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

use std::sync::Arc;

use crate::jni::java_structure::JavaStructure;
use crate::jni::utils::*;

use crate::store::{DatabaseConfig, StateManagerDatabase};
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use parking_lot::RwLock;

use radix_engine_interface::*;

const POINTER_JNI_FIELD_NAME: &str = "rustDatabasePointer";

#[no_mangle]
extern "system" fn Java_com_radixdlt_database_Database_init(
    env: JNIEnv,
    _class: JClass,
    j_database: JObject,
    j_config: jbyteArray,
) {
    JNIDatabase::init(&env, j_database, j_config);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_database_Database_cleanup(
    env: JNIEnv,
    _class: JClass,
    j_database: JObject,
) {
    JNIDatabase::cleanup(&env, j_database);
}

pub struct JNIDatabase {
    pub database: Arc<RwLock<StateManagerDatabase>>,
}

impl JNIDatabase {
    pub fn init(env: &JNIEnv, j_database: JObject, j_config: jbyteArray) {
        let config_bytes: Vec<u8> = jni_jbytearray_to_vector(env, j_config).unwrap();
        let config = DatabaseConfig::from_java(&config_bytes).unwrap();
        let database = Arc::new(parking_lot::const_rwlock(
            StateManagerDatabase::from_config(config),
        ));
        let jni_database = JNIDatabase { database };
        env.set_rust_field(j_database, POINTER_JNI_FIELD_NAME, jni_database)
            .unwrap();
    }

    pub fn cleanup(env: &JNIEnv, j_database: JObject) {
        env.take_rust_field::<_, _, JNIDatabase>(j_database, POINTER_JNI_FIELD_NAME)
            .unwrap();
    }

    pub fn get_database(env: &JNIEnv, j_database: JObject) -> Arc<RwLock<StateManagerDatabase>> {
        env.get_rust_field::<_, _, JNIDatabase>(j_database, POINTER_JNI_FIELD_NAME)
            .unwrap()
            .database
            .clone()
    }
}

pub fn export_extern_functions() {}
