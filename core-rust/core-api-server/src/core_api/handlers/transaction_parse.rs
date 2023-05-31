use crate::core_api::*;
use models::parsed_notarized_transaction_all_of_identifiers::ParsedNotarizedTransactionAllOfIdentifiers;
use radix_engine_interface::data::manifest::manifest_decode;
use std::ops::Deref;

use models::parsed_signed_transaction_intent_all_of_identifiers::ParsedSignedTransactionIntentAllOfIdentifiers;
use models::transaction_parse_request::{ParseMode, ResponseMode, ValidationMode};
use models::transaction_parse_response::TransactionParseResponse;

use state_manager::mempool::pending_transaction_result_cache::RejectionReason;
use state_manager::transaction::{
    CommitabilityValidator, LedgerTransaction, UserTransactionValidator,
};
use state_manager::{HasIntentHash, HasLedgerPayloadHash, HasSignaturesHash, HasUserPayloadHash};

use state_manager::store::StateManagerDatabase;
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, TransactionIntent, TransactionManifest,
};

use super::{
    to_api_intent, to_api_ledger_transaction, to_api_manifest, to_api_notarized_transaction,
    to_api_signed_intent,
};

pub struct ParseContext<'a> {
    mapping_context: MappingContext,
    response_mode: ResponseMode,
    validation_mode: ValidationMode,
    user_transaction_validator: UserTransactionValidator,
    commitability_validator: &'a CommitabilityValidator<StateManagerDatabase>,
}

pub(crate) async fn handle_transaction_parse(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionParseRequest>,
) -> Result<Json<models::TransactionParseResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let bytes =
        from_hex(request.payload_hex).map_err(|err| err.into_response_error("payload_hex"))?;

    let context = ParseContext {
        mapping_context: MappingContext::new(&state.network)
            .with_transaction_formats(&request.transaction_format_options),
        response_mode: request.response_mode.unwrap_or(ResponseMode::Full),
        validation_mode: request.validation_mode.unwrap_or(ValidationMode::_Static),
        user_transaction_validator: UserTransactionValidator::new(&state.network),
        commitability_validator: state.commitability_validator.deref(),
    };

    let parse_mode = request.parse_mode.unwrap_or(ParseMode::Any);

    let parsed = match parse_mode {
        ParseMode::Any => attempt_parsing_as_any_payload_type_and_map_for_api(&context, &bytes)?,
        ParseMode::Notarized => {
            let parsed = attempt_parsing_as_notarized_transaction(&context, &bytes)
                .ok_or_else(|| client_error("The payload isn't a notarized transaction"))?;
            to_api_parsed_notarized_transaction(&context, parsed)?
        }
        ParseMode::Signed => {
            let parsed = attempt_parsing_as_signed_intent(&bytes)
                .ok_or_else(|| client_error("The payload isn't a signed transaction intent"))?;
            to_api_parsed_signed_intent(&context, parsed)?
        }
        ParseMode::Unsigned => {
            let parsed = attempt_parsing_as_intent(&bytes)
                .ok_or_else(|| client_error("The payload isn't an unsigned transaction intent"))?;
            to_api_parsed_intent(&context, parsed)?
        }
        ParseMode::Manifest => {
            let parsed = attempt_parsing_as_manifest(&bytes)
                .ok_or_else(|| client_error("The payload isn't a transaction manifest"))?;
            to_api_parsed_manifest(&context, parsed)?
        }
        ParseMode::Ledger => {
            let parsed = attempt_parsing_as_ledger_transaction(&bytes)
                .ok_or_else(|| client_error("The payload isn't a ledger transaction"))?;
            to_api_parsed_ledger_transaction(&context, parsed)?
        }
    };

    Ok(TransactionParseResponse {
        parsed: Some(parsed),
    })
    .map(Json)
}

fn attempt_parsing_as_any_payload_type_and_map_for_api(
    context: &ParseContext,
    bytes: &[u8],
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    // Attempt 1 - Try parsing as NotarizedTransaction
    let notarized_parse_option = attempt_parsing_as_notarized_transaction(context, bytes);

    if let Some(parsed) = notarized_parse_option {
        return to_api_parsed_notarized_transaction(context, parsed);
    }

    // Attempt 2 - Try parsing as SignedTransactionIntent
    let signed_intent_parse_option = attempt_parsing_as_signed_intent(bytes);

    if let Some(parsed) = signed_intent_parse_option {
        return to_api_parsed_signed_intent(context, parsed);
    }

    // Attempt 3 - Try parsing as (unsigned) TransactionIntent
    let intent_parse_option = attempt_parsing_as_intent(bytes);

    if let Some(parsed) = intent_parse_option {
        return to_api_parsed_intent(context, parsed);
    }

    // Attempt 4 - Try parsing as a manifest
    let intent_parse_option = attempt_parsing_as_manifest(bytes);

    if let Some(parsed) = intent_parse_option {
        return to_api_parsed_manifest(context, parsed);
    }

    // Attempt 5 - Try parsing as a ledger transaction payload
    let ledger_parse_option = attempt_parsing_as_ledger_transaction(bytes);

    if let Some(parsed) = ledger_parse_option {
        return to_api_parsed_ledger_transaction(context, parsed);
    }

    Err(client_error("The payload isn't a valid notarized transaction, signed transaction intent, unsigned transaction intent, transaction manifest or ledger transaction payload."))
}

struct ParsedNotarizedTransaction {
    transaction: NotarizedTransaction,
    validation: Option<Result<(), RejectionReason>>,
}

fn attempt_parsing_as_notarized_transaction(
    context: &ParseContext,
    bytes: &[u8],
) -> Option<ParsedNotarizedTransaction> {
    let notarized_parse_result =
        UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(bytes);
    let parsed = match notarized_parse_result {
        Ok(notarized_transaction) => notarized_transaction,
        Err(_) => return None,
    };

    Some(match context.validation_mode {
        ValidationMode::None => ParsedNotarizedTransaction {
            transaction: parsed,
            validation: None,
        },
        ValidationMode::_Static => {
            let validation = Some(
                context
                    .user_transaction_validator
                    .validate_and_create_executable(&parsed, bytes.len())
                    .map(|_| ())
                    .map_err(RejectionReason::ValidationError),
            );
            ParsedNotarizedTransaction {
                transaction: parsed,
                validation,
            }
        }
        ValidationMode::Full => {
            let validation = Some(
                context
                    .commitability_validator
                    .check_for_rejection(&parsed, bytes.len()),
            );
            ParsedNotarizedTransaction {
                transaction: parsed,
                validation,
            }
        }
    })
}

fn to_api_parsed_notarized_transaction(
    context: &ParseContext,
    parsed: ParsedNotarizedTransaction,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_notarized_transaction(
            &context.mapping_context,
            &parsed.transaction,
        )?)),
    };

    let validation_error = parsed
        .validation
        .and_then(|result| result.err())
        .map(|error| {
            Box::new(models::ParsedNotarizedTransactionAllOfValidationError {
                reason: format!("{error:?}"),
                is_permanent: error.is_permanent_for_payload(),
            })
        });

    Ok(models::ParsedTransaction::ParsedNotarizedTransaction {
        notarized_transaction: model,
        identifiers: Box::new(ParsedNotarizedTransactionAllOfIdentifiers {
            intent_hash: to_api_intent_hash(&parsed.transaction.intent_hash()),
            signatures_hash: to_api_signed_intent_hash(&parsed.transaction.signatures_hash()),
            payload_hash: to_api_notarized_transaction_hash(&parsed.transaction.user_payload_hash()),
            ledger_hash: to_api_ledger_hash(&parsed.transaction.ledger_payload_hash()),
        }),
        validation_error,
    })
}

fn attempt_parsing_as_signed_intent(bytes: &[u8]) -> Option<SignedTransactionIntent> {
    manifest_decode::<SignedTransactionIntent>(bytes).ok()
}

fn to_api_parsed_signed_intent(
    context: &ParseContext,
    parsed: SignedTransactionIntent,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_signed_intent(
            &context.mapping_context,
            &parsed,
        )?)),
    };
    Ok(models::ParsedTransaction::ParsedSignedTransactionIntent {
        signed_intent: model,
        identifiers: Box::new(ParsedSignedTransactionIntentAllOfIdentifiers {
            intent_hash: to_api_intent_hash(&parsed.intent_hash()),
            signatures_hash: to_api_signed_intent_hash(&parsed.signatures_hash()),
        }),
    })
}

fn attempt_parsing_as_intent(bytes: &[u8]) -> Option<TransactionIntent> {
    manifest_decode::<TransactionIntent>(bytes).ok()
}

fn to_api_parsed_intent(
    context: &ParseContext,
    parsed: TransactionIntent,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_intent(&context.mapping_context, &parsed)?)),
    };
    Ok(models::ParsedTransaction::ParsedTransactionIntent {
        intent: model,
        identifiers: Box::new(models::ParsedTransactionIntentAllOfIdentifiers {
            intent_hash: to_api_intent_hash(&parsed.intent_hash()),
        }),
    })
}

fn attempt_parsing_as_manifest(bytes: &[u8]) -> Option<TransactionManifest> {
    manifest_decode::<TransactionManifest>(bytes).ok()
}

fn to_api_parsed_manifest(
    context: &ParseContext,
    parsed: TransactionManifest,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_manifest(
            &context.mapping_context,
            &parsed,
        )?)),
    };
    Ok(models::ParsedTransaction::ParsedTransactionManifest { manifest: model })
}

fn attempt_parsing_as_ledger_transaction(bytes: &[u8]) -> Option<LedgerTransaction> {
    LedgerTransaction::from_slice(bytes).ok()
}

fn to_api_parsed_ledger_transaction(
    context: &ParseContext,
    parsed: LedgerTransaction,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_ledger_transaction(
            &context.mapping_context,
            &parsed,
        )?)),
    };
    let notarized = parsed.user();
    Ok(models::ParsedTransaction::ParsedLedgerTransaction {
        ledger_transaction: model,
        identifiers: Box::new(models::ParsedLedgerTransactionAllOfIdentifiers {
            intent_hash: notarized.map(|tx| to_api_intent_hash(&tx.intent_hash())),
            signatures_hash: notarized.map(|tx| to_api_signed_intent_hash(&tx.signatures_hash())),
            payload_hash: notarized.map(|tx| to_api_notarized_transaction_hash(&tx.user_payload_hash())),
            ledger_hash: to_api_ledger_hash(&parsed.ledger_payload_hash()),
        }),
    })
}
