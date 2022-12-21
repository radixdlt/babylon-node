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

use radix_engine::ledger::create_genesis;
use radix_engine::types::{
    AccessRule, PublicKey, Signature, SignatureWithPublicKey, FAUCET_COMPONENT, RADIX_TOKEN,
};
use radix_engine_interface::args;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::data::scrypto_encode;

use crate::transaction::LedgerTransaction;
use transaction::builder::ManifestBuilder;
use transaction::manifest::{compile, CompileError};
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, TransactionHeader, TransactionIntent,
    TransactionManifest,
};

pub fn create_genesis_ledger_transaction_bytes(
    validator_list: Vec<EcdsaSecp256k1PublicKey>,
    rounds_per_epoch: u64,
) -> Vec<u8> {
    let genesis = create_genesis(validator_list, rounds_per_epoch);
    scrypto_encode(&LedgerTransaction::System(genesis)).unwrap()
}

pub fn create_new_account_intent_bytes(
    network_definition: &NetworkDefinition,
    public_key: PublicKey,
) -> Vec<u8> {
    let manifest = ManifestBuilder::new(network_definition)
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(&AccessRule::AllowAll, bucket_id)
        })
        .build();

    let intent = TransactionIntent {
        header: TransactionHeader {
            version: 1,
            network_id: network_definition.id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 5,
            notary_public_key: public_key,
            notary_as_signatory: false,
            cost_unit_limit: 10_000_000,
            tip_percentage: 5,
        },
        manifest,
    };

    intent.to_bytes().unwrap()
}

pub fn create_intent_bytes(
    network_definition: &NetworkDefinition,
    header: TransactionHeader,
    manifest_str: String,
    blobs: Vec<Vec<u8>>,
) -> Result<Vec<u8>, CompileError> {
    let manifest = create_manifest(network_definition, &manifest_str, blobs)?;
    let intent = TransactionIntent { header, manifest };
    Ok(intent.to_bytes().unwrap())
}

pub fn create_manifest(
    network_definition: &NetworkDefinition,
    manifest_str: &str,
    blobs: Vec<Vec<u8>>,
) -> Result<TransactionManifest, CompileError> {
    compile(manifest_str, network_definition, blobs)
}

pub fn create_signed_intent_bytes(
    intent: TransactionIntent,
    signatures: Vec<SignatureWithPublicKey>,
) -> Vec<u8> {
    let signed_intent = SignedTransactionIntent {
        intent,
        intent_signatures: signatures,
    };
    signed_intent.to_bytes().unwrap()
}

pub fn create_notarized_bytes(
    signed_intent: SignedTransactionIntent,
    notary_signature: Signature,
) -> Vec<u8> {
    let notarized = NotarizedTransaction {
        signed_intent,
        notary_signature,
    };
    notarized.to_bytes().unwrap()
}
