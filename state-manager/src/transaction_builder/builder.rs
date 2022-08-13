use scrypto::core::Network;
use scrypto::crypto::EcdsaPublicKey;
use scrypto::prelude::{AccessRule, EcdsaSignature, RADIX_TOKEN, SYSTEM_COMPONENT};
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;
use transaction::model::{NotarizedTransaction, SignedTransactionIntent, TransactionHeader, TransactionIntent};


pub fn create_new_account_unsigned_manifest(public_key: EcdsaPublicKey) -> Vec<u8> {
    let manifest = ManifestBuilder::new(Network::InternalTestnet)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(SYSTEM_COMPONENT, "free_xrd", to_struct!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(&AccessRule::AllowAll, bucket_id)
        })
        .build();

    let intent = TransactionIntent {
        header: TransactionHeader {
            version: 1,
            network: Network::InternalTestnet,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 5,
            notary_public_key: public_key,
            notary_as_signatory: false,
            cost_unit_limit: 1_000_000,
            tip_percentage: 5,
        },
        manifest
    };

    intent.to_bytes()
}

pub fn combine_for_notary(intent: TransactionIntent, public_key: EcdsaPublicKey, signature: EcdsaSignature) -> Vec<u8>{
    let signed_intent = SignedTransactionIntent {
        intent,
        intent_signatures: vec![(public_key, signature)]
    };
    signed_intent.to_bytes()
}

pub fn combine(signed_intent: SignedTransactionIntent, notary_signature: EcdsaSignature) -> Vec<u8> {
    let notarized = NotarizedTransaction {
        signed_intent,
        notary_signature,
    };
    notarized.to_bytes()
}