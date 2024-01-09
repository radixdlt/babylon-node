use crate::core_api::*;
use std::ops::Deref;
use std::time::SystemTime;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

use models::transaction_parse_request::{ParseMode, ResponseMode, ValidationMode};
use models::transaction_parse_response::TransactionParseResponse;

use state_manager::mempool::pending_transaction_result_cache::RejectionReason;
use state_manager::transaction::*;

use state_manager::store::StateManagerDatabase;
use transaction::prelude::*;

use super::{
    to_api_intent, to_api_ledger_transaction, to_api_notarized_transaction, to_api_signed_intent,
};

pub struct ParseContext<'a> {
    mapping_context: MappingContext,
    response_mode: ResponseMode,
    validation_mode: ValidationMode,
    user_transaction_validator: NotarizedTransactionValidator,
    committability_validator: &'a CommittabilityValidator<StateManagerDatabase>,
}

pub(crate) async fn handle_transaction_parse(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionParseRequest>,
) -> Result<Json<models::TransactionParseResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let bytes =
        from_hex(request.payload_hex).map_err(|err| err.into_response_error("payload_hex"))?;

    let read_commitability_validator = state.state_manager.committability_validator.read();
    let context = ParseContext {
        mapping_context: MappingContext::new(&state.network)
            .with_transaction_formats(&request.transaction_format_options),
        response_mode: request.response_mode.unwrap_or(ResponseMode::Full),
        validation_mode: request.validation_mode.unwrap_or(ValidationMode::_Static),
        user_transaction_validator: NotarizedTransactionValidator::new(ValidationConfig::default(
            state.network.id,
        )),
        committability_validator: read_commitability_validator.deref(),
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

    // Attempt 4 - Try parsing as a ledger transaction payload
    let ledger_parse_option = attempt_parsing_as_ledger_transaction(bytes);

    if let Some(parsed) = ledger_parse_option {
        return to_api_parsed_ledger_transaction(context, parsed);
    }

    Err(client_error("The payload isn't a valid notarized transaction, signed transaction intent, unsigned transaction intent or ledger transaction payload."))
}

struct ParsedNotarizedTransactionV1 {
    model: NotarizedTransactionV1,
    prepared: PreparedNotarizedTransactionV1,
    validation: Option<Result<(), RejectionReason>>,
}

fn attempt_parsing_as_notarized_transaction(
    context: &ParseContext,
    bytes: &[u8],
) -> Option<ParsedNotarizedTransactionV1> {
    let prepare_result = context
        .user_transaction_validator
        .prepare_from_payload_bytes(bytes);
    let prepared = match prepare_result {
        Ok(prepared) => prepared,
        Err(_) => return None,
    };
    let model: NotarizedTransactionV1 = match NotarizedTransactionV1::from_payload_bytes(bytes) {
        Ok(model) => model,
        Err(_) => return None,
    };

    Some(match context.validation_mode {
        ValidationMode::None => ParsedNotarizedTransactionV1 {
            model,
            prepared,
            validation: None,
        },
        ValidationMode::_Static => {
            let validation = Some(
                context
                    .user_transaction_validator
                    .validate(prepared.clone())
                    .map(|_| ())
                    .map_err(RejectionReason::ValidationError),
            );
            ParsedNotarizedTransactionV1 {
                model,
                prepared,
                validation,
            }
        }
        ValidationMode::Full => {
            let validation = Some({
                context
                    .user_transaction_validator
                    .validate(prepared.clone())
                    .map_err(RejectionReason::ValidationError)
                    .and_then(|validated| {
                        let rejection = context
                            .committability_validator
                            .check_for_rejection(&validated, SystemTime::now())
                            .rejection;
                        match rejection {
                            Some(rejection) => Err(rejection),
                            None => Ok(()),
                        }
                    })
            });
            ParsedNotarizedTransactionV1 {
                model,
                prepared,
                validation,
            }
        }
    })
}

fn to_api_parsed_notarized_transaction(
    context: &ParseContext,
    parsed: ParsedNotarizedTransactionV1,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let intent_hash = parsed.prepared.intent_hash();
    let signed_intent_hash = parsed.prepared.signed_intent_hash();
    let notarized_transaction_hash = parsed.prepared.notarized_transaction_hash();

    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_notarized_transaction(
            &context.mapping_context,
            &parsed.model,
            &intent_hash,
            &signed_intent_hash,
            &notarized_transaction_hash,
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

    let ledger_hash =
        PreparedLedgerTransactionInner::UserV1(Box::new(parsed.prepared)).get_ledger_hash();

    Ok(models::ParsedTransaction::ParsedNotarizedTransaction {
        notarized_transaction: model,
        identifiers: Box::new(models::ParsedNotarizedTransactionIdentifiers {
            intent_hash: to_api_intent_hash(&intent_hash),
            intent_hash_bech32m: to_api_hash_bech32m(&context.mapping_context, &intent_hash)?,
            signed_intent_hash: to_api_signed_intent_hash(&signed_intent_hash),
            signed_intent_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &signed_intent_hash,
            )?,
            payload_hash: to_api_notarized_transaction_hash(&notarized_transaction_hash),
            payload_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &notarized_transaction_hash,
            )?,
            ledger_hash: to_api_ledger_hash(&ledger_hash),
            ledger_hash_bech32m: to_api_hash_bech32m(&context.mapping_context, &ledger_hash)?,
        }),
        validation_error,
    })
}

fn attempt_parsing_as_signed_intent(
    bytes: &[u8],
) -> Option<(SignedIntentV1, PreparedSignedIntentV1)> {
    let signed_intent = SignedIntentV1::from_payload_bytes(bytes).ok()?;
    let prepared = PreparedSignedIntentV1::prepare_from_payload(bytes).ok()?;
    Some((signed_intent, prepared))
}

fn to_api_parsed_signed_intent(
    context: &ParseContext,
    (model, prepared): (SignedIntentV1, PreparedSignedIntentV1),
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_signed_intent(
            &context.mapping_context,
            &model,
            &prepared.intent_hash(),
            &prepared.signed_intent_hash(),
        )?)),
    };
    Ok(models::ParsedTransaction::ParsedSignedTransactionIntent {
        signed_intent: model,
        identifiers: Box::new(models::ParsedSignedTransactionIntentIdentifiers {
            intent_hash: to_api_intent_hash(&prepared.intent_hash()),
            intent_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &prepared.intent_hash(),
            )?,
            signed_intent_hash: to_api_signed_intent_hash(&prepared.signed_intent_hash()),
            signed_intent_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &prepared.signed_intent_hash(),
            )?,
        }),
    })
}

fn attempt_parsing_as_intent(bytes: &[u8]) -> Option<(IntentV1, PreparedIntentV1)> {
    let model = IntentV1::from_payload_bytes(bytes).ok()?;
    let prepared = PreparedIntentV1::prepare_from_payload(bytes).ok()?;
    Some((model, prepared))
}

fn to_api_parsed_intent(
    context: &ParseContext,
    (model, prepared): (IntentV1, PreparedIntentV1),
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let intent_hash = &prepared.intent_hash();
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_intent(
            &context.mapping_context,
            &model,
            intent_hash,
        )?)),
    };
    Ok(models::ParsedTransaction::ParsedTransactionIntent {
        intent: model,
        identifiers: Box::new(models::ParsedTransactionIntentIdentifiers {
            intent_hash: to_api_intent_hash(intent_hash),
            intent_hash_bech32m: to_api_hash_bech32m(&context.mapping_context, intent_hash)?,
        }),
    })
}

fn attempt_parsing_as_ledger_transaction(
    bytes: &[u8],
) -> Option<(
    LedgerTransaction,
    PreparedLedgerTransaction,
    RawLedgerTransaction,
)> {
    let model = LedgerTransaction::from_payload_bytes(bytes).ok()?;
    let prepared = PreparedLedgerTransaction::prepare_from_payload(bytes).ok()?;
    let raw = RawLedgerTransaction(bytes.to_vec());
    Some((model, prepared, raw))
}

fn to_api_parsed_ledger_transaction(
    context: &ParseContext,
    (model, prepared, raw): (
        LedgerTransaction,
        PreparedLedgerTransaction,
        RawLedgerTransaction,
    ),
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let identifiers = prepared.create_identifiers();
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_ledger_transaction(
            &context.mapping_context,
            &raw,
            &model,
            &identifiers,
        )?)),
    };

    let user_identifiers = identifiers.typed.user();

    Ok(models::ParsedTransaction::ParsedLedgerTransaction {
        ledger_transaction: model,
        identifiers: Box::new(models::ParsedLedgerTransactionIdentifiers {
            intent_hash: user_identifiers
                .as_ref()
                .map(|hashes| to_api_intent_hash(hashes.intent_hash)),
            intent_hash_bech32m: user_identifiers
                .as_ref()
                .map(|hashes| to_api_hash_bech32m(&context.mapping_context, hashes.intent_hash))
                .transpose()?,
            signed_intent_hash: user_identifiers
                .as_ref()
                .map(|hashes| to_api_signed_intent_hash(hashes.signed_intent_hash)),
            signed_intent_hash_bech32m: user_identifiers
                .as_ref()
                .map(|hashes| {
                    to_api_hash_bech32m(&context.mapping_context, hashes.signed_intent_hash)
                })
                .transpose()?,
            payload_hash: user_identifiers
                .as_ref()
                .map(|hashes| to_api_notarized_transaction_hash(hashes.notarized_transaction_hash)),
            payload_hash_bech32m: user_identifiers
                .as_ref()
                .map(|hashes| {
                    to_api_hash_bech32m(&context.mapping_context, hashes.notarized_transaction_hash)
                })
                .transpose()?,
            ledger_hash: to_api_ledger_hash(&prepared.ledger_transaction_hash()),
            ledger_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &prepared.ledger_transaction_hash(),
            )?,
        }),
    })
}
