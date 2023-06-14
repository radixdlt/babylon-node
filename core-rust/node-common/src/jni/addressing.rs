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

use crate::java::utils::jni_sbor_coded_call;
use bech32::{FromBase32, ToBase32, Variant};
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::ComponentAddress;
use radix_engine_common::prelude::{Bech32Encoder, NetworkDefinition};
use radix_engine_common::types::{EntityType, NodeId, ResourceAddress};
use radix_engine_interface::crypto::Secp256k1PublicKey;

#[no_mangle]
extern "system" fn Java_com_radixdlt_identifiers_Bech32mCoder_encodeAddress(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |(network_definition, address_data): (NetworkDefinition, Vec<u8>)| -> Result<String, String> {
            if address_data.len() != NodeId::LENGTH {
                return Err(format!("Raw address length must be {}", NodeId::LENGTH));
            }

            Bech32Encoder::new(&network_definition).encode(&address_data)
                .map_err(|err| format!("{err:?}"))
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_identifiers_Bech32mCoder_encodeBech32m(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |(hrp, full_data): (String, Vec<u8>)| -> Result<String, String> {
            let base32_data = full_data.to_base32();

            let address = bech32::encode(&hrp, base32_data, Variant::Bech32m)
                .map_err(|e| format!("Unable to encode bech32m address: {e:?}"))?;

            Ok(address)
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_identifiers_Bech32mCoder_decodeBech32m(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |address: String| -> Result<(String, Vec<u8>), String> {
            let (hrp, base32_data, variant) = bech32::decode(&address)
                .map_err(|e| format!("Unable to decode bech32 address: {e:?}"))?;

            check_variant_is_bech32m(variant)?;

            let data = Vec::<u8>::from_base32(&base32_data).map_err(|e| {
                format!("Unable to decode bech32 data from 5 bits to 8 bits: {e:?}")
            })?;

            Ok((hrp, data))
        },
    )
}

fn check_variant_is_bech32m(variant: Variant) -> Result<(), String> {
    match variant {
        Variant::Bech32 => Err("Address was bech32 encoded, not bech32m".to_owned()),
        Variant::Bech32m => Ok(()),
    }
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_identifiers_Address_nativeVirtualAccountAddress(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |unchecked_public_key_bytes: [u8; Secp256k1PublicKey::LENGTH]| {
            // Note: the bytes may represent a non-existent point on a curve (an invalid public key) -
            // this is okay. We need to support this because there are such accounts on Olympia.
            let public_key = Secp256k1PublicKey(unchecked_public_key_bytes);
            ComponentAddress::virtual_account_from_public_key(&public_key)
        },
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_identifiers_Address_nativeGlobalFungible(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |address_bytes_without_entity_id: [u8; NodeId::UUID_LENGTH]| {
            ResourceAddress::new_or_panic(
                NodeId::new(
                    EntityType::GlobalFungibleResourceManager as u8,
                    &address_bytes_without_entity_id,
                )
                .0,
            )
        },
    )
}

pub fn export_extern_functions() {}
