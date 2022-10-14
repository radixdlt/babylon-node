use crate::core_api::*;
use models::transaction_parse_response::TransactionParseResponse;
use scrypto::prelude::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::transaction::UserTransactionValidator;
use transaction::errors::TransactionValidationError;
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, TransactionIntent, TransactionManifest,
};

use super::{to_api_intent, to_api_manifest, to_api_notarized_transaction, to_api_signed_intent};

pub(crate) async fn handle_transaction_parse(
    state: Extension<CoreApiState>,
    request: Json<models::TransactionParseRequest>,
) -> Result<Json<models::TransactionParseResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_transaction_parse_internal)
}

fn handle_transaction_parse_internal(
    state_manager: &mut ActualStateManager,
    request: models::TransactionParseRequest,
) -> Result<models::TransactionParseResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let bytes =
        from_hex(request.payload_hex).map_err(|err| err.into_response_error("payload_hex"))?;

    let parsed = attempt_parsing_various_payload_types(
        state_manager.get_epoch(),
        &state_manager.user_transaction_validator,
        &state_manager.network,
        &bytes,
    )?;

    Ok(TransactionParseResponse {
        parsed: Some(parsed),
    })
}

fn attempt_parsing_various_payload_types(
    epoch: u64,
    validator: &UserTransactionValidator,
    network: &NetworkDefinition,
    bytes: &[u8],
) -> Result<models::ParsedTransaction, RequestHandlingError> {
    // Attempt 1 - Try parsing as NotarizedTransaction
    // If it isn't a valid notarized tranasction, this returns Ok(None)
    let notarized_parse_option = attempt_parsing_as_notarized_transaction(validator, epoch, bytes)
        .map_err(|err| ExtractionError::from(err).into_response_error("payload_hex"))?;

    if let Some(parsed) = notarized_parse_option {
        return Ok(models::ParsedTransaction::ParsedNotarizedTransaction {
            notarized_transaction: Box::new(to_api_notarized_transaction(&parsed.0, network)?),
            is_statically_valid: parsed.1.is_none(),
            validity_error: parsed.1.map(|err| format!("{:?}", err)),
        });
    }

    // Attempt 2 - Try parsing as SignedTransactionIntent
    // If it isn't a valid signed transaction intent, this returns Ok(None)
    let signed_intent_parse_option = attempt_parsing_as_signed_intent(bytes);

    if let Some(parsed) = signed_intent_parse_option {
        return Ok(models::ParsedTransaction::ParsedSignedTransactionIntent {
            signed_intent: Box::new(to_api_signed_intent(&parsed, network)?),
        });
    }

    // Attempt 3 - Try parsing as (unsigned) TransactionIntent
    // If it isn't a valid transaction intent, this returns Ok(None)
    let intent_parse_option = attempt_parsing_as_intent(bytes);

    if let Some(parsed) = intent_parse_option {
        return Ok(models::ParsedTransaction::ParsedTransactionIntent {
            intent: Box::new(to_api_intent(&parsed, network)?),
        });
    }

    // Attempt 4 - Try parsing as a manifest
    // If it isn't a valid transaction manifest, this returns Ok(None)
    let intent_parse_option = attempt_parsing_as_manifest(bytes);

    if let Some(parsed) = intent_parse_option {
        return Ok(models::ParsedTransaction::ParsedTransactionManifest {
            manifest: Box::new(to_api_manifest(&parsed, network)?),
        });
    }

    Err(client_error("The payload isn't a valid notarized transaction, signed transaction intent or unsigned transaction intent."))
}

fn attempt_parsing_as_notarized_transaction(
    validator: &UserTransactionValidator,
    epoch: u64,
    bytes: &[u8],
) -> Result<
    Option<(NotarizedTransaction, Option<TransactionValidationError>)>,
    TransactionValidationError,
> {
    let notarized_parse_result =
        UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(bytes);
    let parsed = match notarized_parse_result {
        Ok(notarized_transaction) => notarized_transaction,
        Err(err) => match err {
            // If it's too large, it's too large for all types - so emit an error
            TransactionValidationError::TransactionTooLarge => Err(err)?,
            // If it's not a valid payload, continue to the next possibility
            TransactionValidationError::DeserializationError(_) => return Ok(None),
            // Any other error is unexpected at the moment
            err => panic!("Unexpected error during unvalidated parsing: {:?}", err),
        },
    };

    let validation_result = validator.validate_user_transaction(epoch, parsed.clone());
    Ok(Some((parsed, validation_result.err())))
}

fn attempt_parsing_as_signed_intent(bytes: &[u8]) -> Option<SignedTransactionIntent> {
    match scrypto_decode::<SignedTransactionIntent>(bytes) {
        Ok(signed_intent) => Some(signed_intent),
        Err(_) => None,
    }
}

fn attempt_parsing_as_intent(bytes: &[u8]) -> Option<TransactionIntent> {
    match scrypto_decode::<TransactionIntent>(bytes) {
        Ok(intent) => Some(intent),
        Err(_) => None,
    }
}

fn attempt_parsing_as_manifest(bytes: &[u8]) -> Option<TransactionManifest> {
    match scrypto_decode::<TransactionManifest>(bytes) {
        Ok(manifest) => Some(manifest),
        Err(_) => None,
    }
}
