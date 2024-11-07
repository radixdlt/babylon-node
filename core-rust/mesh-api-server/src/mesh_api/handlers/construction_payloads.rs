use crate::prelude::*;
use models::{AccountIdentifier, SignatureType, SigningPayload};
use radix_transactions::prelude::ManifestBuilder;
use rand::Rng;

pub(crate) async fn handle_construction_payloads(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPayloadsRequest>,
) -> Result<Json<models::ConstructionPayloadsResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let public_key = if let Some(public_keys) = request.public_keys {
        if public_keys.len() == 1 {
            assert_public_key(&public_keys[0])?
        } else {
            return Err(client_error(
                format!("Expected 1 public key, but received {}", public_keys.len()),
                false,
            ));
        }
    } else {
        return Err(client_error("Missing public_keys", false));
    };
    let address = state
        .address_encoder()
        .encode(ComponentAddress::preallocated_account_from_public_key(&public_key).as_bytes())
        .expect("Failed to encode account address");

    let mut builder = ManifestBuilder::new();
    for operation in request.operations {
        match operation._type.as_str() {
            "LockFee" => {
                let account = assert_account_from_option(&state.network, operation.account)?;
                let (address, quantity) = assert_amount_from_option(operation.amount.clone())?;
                if address != XRD {
                    return Err(client_error(
                        format!("LockFee only supports XRD: actual = {:?}", operation.amount),
                        false,
                    ));
                }
                builder = builder.lock_fee(account, quantity);
            }
            "Withdraw" => {
                let account = assert_account_from_option(&state.network, operation.account)?;
                let (address, quantity) = assert_amount_from_option(operation.amount)?;
                builder = builder.withdraw_from_account(account, address, quantity);
            }
            "Deposit" => {
                let account = assert_account_from_option(&state.network, operation.account)?;
                let (address, quantity) = assert_amount_from_option(operation.amount)?;
                let bucket = builder.generate_bucket_name("bucket");
                builder = builder.take_from_worktop(address, quantity, &bucket);
                builder = builder.try_deposit_or_abort(account, None, bucket);
            }
            _ => {
                return Err(ResponseError::from(ApiError::InvalidRequest)
                    .with_details(format!("Invalid operation: {}", operation._type)))
            }
        }
    }
    let manifest = builder.build();

    let database = state.state_manager.database.snapshot();
    let current_epoch = database
        .get_latest_epoch_proof()
        .unwrap()
        .ledger_header
        .epoch;
    let intent = IntentV1 {
        header: TransactionHeaderV1 {
            network_id: state.network.id,
            start_epoch_inclusive: current_epoch, // TODO:MESH move variables to metadata for deterministic payloads
            end_epoch_exclusive: current_epoch.after(100).unwrap(),
            nonce: rand::thread_rng().gen(),
            notary_public_key: public_key,
            notary_is_signatory: true,
            tip_percentage: 0,
        },
        instructions: InstructionsV1(manifest.instructions),
        blobs: BlobsV1 {
            blobs: Default::default(),
        },
        message: MessageV1::None,
    };

    let intent_bytes = intent.to_raw().unwrap();
    let prepared_intent =
        PreparedIntentV1::prepare(&intent_bytes, &PreparationSettings::latest()).unwrap();
    let intent_hash = prepared_intent.transaction_intent_hash();
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
            account_identifier: Some(Box::new(AccountIdentifier {
                address,
                sub_account: None,
                metadata: None,
            })),
            hex_bytes: hex::encode(signed_intent_hash.as_bytes()),
            signature_type: Some(SignatureType::Ecdsa),
        }],
    }))
}

fn hash_encoded_sbor_value<T: ManifestEncode>(value: T) -> Hash {
    // Ignore the version byte
    hash(&manifest_encode(&value).unwrap()[1..])
}
