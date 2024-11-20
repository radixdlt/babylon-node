use super::ConstructionMetadata;
use crate::prelude::*;
use models::{SignatureType, SigningPayload};
use radix_transactions::prelude::ManifestBuilder;

pub(crate) async fn handle_construction_payloads(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPayloadsRequest>,
) -> Result<Json<models::ConstructionPayloadsResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let public_keys = request.public_keys.unwrap_or_default();
    let public_key = if public_keys.len() == 1 {
        extract_public_key(&public_keys[0]).map_err(|e| e.into_response_error("public_key"))?
    } else {
        return Err(
            ResponseError::from(ApiError::InvalidNumberOfPublicKeys).with_details(format!(
                "Expected 1 public key, but received {}",
                public_keys.len()
            )),
        );
    };
    let signature_type = match &public_key {
        PublicKey::Secp256k1(_) => SignatureType::EcdsaRecovery,
        PublicKey::Ed25519(_) => SignatureType::Ed25519,
    };
    let account_address = ComponentAddress::preallocated_account_from_public_key(&public_key);
    let account_identifier = to_api_account_identifier_from_public_key(
        &MappingContext::new(&state.network),
        public_key,
    )?;

    let metadata: ConstructionMetadata = request
        .metadata
        .clone()
        .and_then(|m| serde_json::from_value(m).ok())
        .ok_or(
            ResponseError::from(ApiError::InvalidMetadata)
                .with_details(format!("Invalid metadata: {:?}", request.metadata)),
        )?;

    let extraction_context = ExtractionContext::new(&state.network);
    let mut builder = ManifestBuilder::new().lock_fee(account_address, dec!(10));
    for operation in request.operations {
        let operation_type =
            MeshApiOperationType::from_str(operation._type.as_str()).map_err(|_| {
                ResponseError::from(ApiError::InvalidOperation)
                    .with_details(format!("Invalid operation: {}", operation._type))
            })?;
        match operation_type {
            MeshApiOperationType::Withdraw => {
                let account = match operation.account {
                    None => Err(ExtractionError::NotFound),
                    Some(account) => extract_radix_account_address_from_account_identifier(
                        &extraction_context,
                        &account,
                    ),
                }
                .map_err(|e| e.into_response_error("account"))?;

                let (address, quantity) =
                    extract_amount_from_option(&extraction_context, operation.amount)
                        .map_err(|e| e.into_response_error("amount"))?;
                builder = builder.withdraw_from_account(account, address, -quantity);
            }
            MeshApiOperationType::Deposit => {
                let account = match operation.account {
                    None => Err(ExtractionError::NotFound),
                    Some(account) => extract_radix_account_address_from_account_identifier(
                        &extraction_context,
                        &account,
                    ),
                }
                .map_err(|e| e.into_response_error("account"))?;
                let (address, quantity) =
                    extract_amount_from_option(&extraction_context, operation.amount)
                        .map_err(|e| e.into_response_error("amount"))?;
                let bucket = builder.generate_bucket_name("bucket");
                builder = builder.take_from_worktop(address, quantity, &bucket);
                builder = builder.try_deposit_or_abort(account, None, bucket);
            }
        }
    }
    let manifest = builder.build();

    let intent = IntentV1 {
        header: TransactionHeaderV1 {
            network_id: state.network.id,
            start_epoch_inclusive: Epoch::of(metadata.start_epoch_inclusive),
            end_epoch_exclusive: Epoch::of(metadata.end_epoch_exclusive),
            nonce: metadata.intent_discriminator,
            notary_public_key: public_key,
            notary_is_signatory: true,
            tip_percentage: metadata.tip_percentage,
        },
        instructions: InstructionsV1(manifest.instructions),
        blobs: BlobsV1 {
            blobs: Default::default(),
        },
        message: MessageV1::None,
    };

    let intent_bytes = intent.to_raw().unwrap();
    let intent_hash = PreparedIntentV1::prepare(&intent_bytes, &PreparationSettings::latest())
        .unwrap()
        .transaction_intent_hash();
    let intent_signatures_hash = hash_encoded_sbor_value(&IntentSignaturesV1 {
        signatures: Default::default(),
    });
    let signed_intent_hash = SignedTransactionIntentHash::from_hash(hash(
        [
            [
                TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                TransactionDiscriminator::V1SignedIntent as u8,
            ]
            .as_slice(),
            intent_hash.0.as_slice(),
            intent_signatures_hash.0.as_slice(),
        ]
        .concat(),
    ));

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionpayloadsresponse for field
    // definitions
    Ok(Json(models::ConstructionPayloadsResponse {
        unsigned_transaction: intent_bytes.to_hex(),
        payloads: vec![SigningPayload {
            address: None, // deprecated
            account_identifier: Some(Box::new(account_identifier)),
            hex_bytes: hex::encode(signed_intent_hash.as_bytes()),
            signature_type: Some(signature_type),
        }],
    }))
}

fn hash_encoded_sbor_value<T: ManifestEncode>(value: T) -> Hash {
    // Ignore the version byte
    hash(&manifest_encode(&value).unwrap()[1..])
}
