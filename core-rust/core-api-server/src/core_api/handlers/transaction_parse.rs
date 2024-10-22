use crate::prelude::*;

use models::transaction_parse_request::{ParseMode, ResponseMode, ValidationMode};
use models::transaction_parse_response::TransactionParseResponse;

use super::{
    to_api_intent_v1, to_api_ledger_transaction, to_api_notarized_transaction_v1,
    to_api_notarized_transaction_v2, to_api_signed_intent,
};

pub struct ParseContext<'a> {
    mapping_context: MappingContext,
    response_mode: ResponseMode,
    validation_mode: ValidationMode,
    transaction_validator: TransactionValidator,
    committability_validator: &'a CommittabilityValidator,
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
        transaction_validator: *state.state_manager.transaction_validator.read(),
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
            let parsed = attempt_parsing_as_signed_intent(&context, &bytes)
                .ok_or_else(|| client_error("The payload isn't a signed transaction intent"))?;
            to_api_parsed_signed_intent(&context, parsed)?
        }
        ParseMode::Unsigned => {
            let parsed = attempt_parsing_as_intent(&context, &bytes)
                .ok_or_else(|| client_error("The payload isn't an unsigned transaction intent"))?;
            to_api_parsed_intent(&context, parsed)?
        }
        ParseMode::Ledger => {
            let parsed = attempt_parsing_as_ledger_transaction(&context, &bytes)
                .ok_or_else(|| client_error("The payload isn't a ledger transaction"))?;
            to_api_parsed_ledger_transaction(&context, parsed)?
        }
    };

    Ok(Json(TransactionParseResponse {
        parsed: Some(parsed),
    }))
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
    let signed_intent_parse_option = attempt_parsing_as_signed_intent(context, bytes);

    if let Some(parsed) = signed_intent_parse_option {
        return to_api_parsed_signed_intent(context, parsed);
    }

    // Attempt 3 - Try parsing as (unsigned) TransactionIntent
    let intent_parse_option = attempt_parsing_as_intent(context, bytes);

    if let Some(parsed) = intent_parse_option {
        return to_api_parsed_intent(context, parsed);
    }

    // Attempt 4 - Try parsing as a ledger transaction payload
    let ledger_parse_option = attempt_parsing_as_ledger_transaction(context, bytes);

    if let Some(parsed) = ledger_parse_option {
        return to_api_parsed_ledger_transaction(context, parsed);
    }

    Err(client_error("The payload isn't a valid notarized transaction, signed transaction intent, unsigned transaction intent or ledger transaction payload."))
}

struct ParsedNotarizedTransaction {
    model: UserTransaction,
    hashes: UserTransactionHashes,
    prepared: PreparedUserTransaction,
    validation: Option<Result<(), MempoolRejectionReason>>,
}

fn attempt_parsing_as_notarized_transaction(
    context: &ParseContext,
    bytes: &[u8],
) -> Option<ParsedNotarizedTransaction> {
    let raw = RawNotarizedTransaction::from_slice(bytes);
    let prepared = raw
        .prepare(context.transaction_validator.preparation_settings())
        .ok()?;

    let hashes = prepared.hashes();

    let model = UserTransaction::from_raw(&raw).ok()?;

    Some(match context.validation_mode {
        ValidationMode::None => ParsedNotarizedTransaction {
            model,
            hashes,
            prepared,
            validation: None,
        },
        ValidationMode::_Static => {
            let validation = Some(
                prepared
                    .clone()
                    .validate(&context.transaction_validator)
                    .map(|_| ())
                    .map_err(MempoolRejectionReason::ValidationError),
            );
            ParsedNotarizedTransaction {
                model,
                hashes,
                prepared,
                validation,
            }
        }
        ValidationMode::Full => {
            let validation = Some({
                prepared
                    .clone()
                    .validate(&context.transaction_validator)
                    .map_err(MempoolRejectionReason::ValidationError)
                    .and_then(|validated| {
                        let rejection = context
                            .committability_validator
                            .check_for_rejection(
                                &validated.create_executable(),
                                &hashes,
                                SystemTime::now(),
                            )
                            .rejection;
                        match rejection {
                            Some(rejection) => Err(rejection),
                            None => Ok(()),
                        }
                    })
            });
            ParsedNotarizedTransaction {
                model,
                hashes,
                prepared,
                validation,
            }
        }
    })
}

fn to_api_parsed_notarized_transaction(
    context: &ParseContext,
    parsed: ParsedNotarizedTransaction,
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let validation_error = parsed
        .validation
        .and_then(|result| result.err())
        .map(|error| {
            Box::new(models::ParsedNotarizedTransactionAllOfValidationError {
                reason: format!("{error:?}"),
                is_permanent: error.is_permanent_for_payload(),
            })
        });

    match (parsed.model, parsed.prepared) {
        (UserTransaction::V1(model), PreparedUserTransaction::V1(prepared)) => {
            let model = match context.response_mode {
                ResponseMode::Basic => None,
                ResponseMode::Full => Some(Box::new(to_api_notarized_transaction_v1(
                    &context.mapping_context,
                    &model,
                    &parsed.hashes,
                )?)),
            };

            let ledger_hash =
                PreparedLedgerTransactionInner::User(PreparedUserTransaction::V1(prepared))
                    .get_ledger_hash();

            Ok(models::ParsedTransaction::ParsedNotarizedTransaction {
                notarized_transaction: model,
                identifiers: Box::new(to_api_parsed_notarized_transaction_identifiers(
                    context,
                    &parsed.hashes,
                    ledger_hash,
                )?),
                validation_error,
            })
        }
        (UserTransaction::V2(model), PreparedUserTransaction::V2(prepared)) => {
            let model = match context.response_mode {
                ResponseMode::Basic => None,
                ResponseMode::Full => Some(Box::new(to_api_notarized_transaction_v2(
                    &context.mapping_context,
                    &model,
                    &parsed.hashes,
                )?)),
            };

            let ledger_hash =
                PreparedLedgerTransactionInner::User(PreparedUserTransaction::V2(prepared))
                    .get_ledger_hash();

            Ok(models::ParsedTransaction::ParsedNotarizedTransactionV2 {
                notarized_transaction: model,
                identifiers: Box::new(to_api_parsed_notarized_transaction_identifiers(
                    context,
                    &parsed.hashes,
                    ledger_hash,
                )?),
                validation_error,
            })
        }
        (UserTransaction::V1(_), _) | (UserTransaction::V2(_), _) => {
            panic!("Unexpected combination")
        }
    }
}

fn to_api_parsed_notarized_transaction_identifiers(
    context: &ParseContext,
    hashes: &UserTransactionHashes,
    ledger_hash: LedgerTransactionHash,
) -> Result<models::ParsedNotarizedTransactionIdentifiers, MappingError> {
    Ok(models::ParsedNotarizedTransactionIdentifiers {
        intent_hash: to_api_transaction_intent_hash(&hashes.transaction_intent_hash),
        intent_hash_bech32m: to_api_hash_bech32m(
            &context.mapping_context,
            &hashes.transaction_intent_hash,
        )?,
        signed_intent_hash: to_api_signed_transaction_intent_hash(
            &hashes.signed_transaction_intent_hash,
        ),
        signed_intent_hash_bech32m: to_api_hash_bech32m(
            &context.mapping_context,
            &hashes.signed_transaction_intent_hash,
        )?,
        payload_hash: to_api_notarized_transaction_hash(&hashes.notarized_transaction_hash),
        payload_hash_bech32m: to_api_hash_bech32m(
            &context.mapping_context,
            &hashes.notarized_transaction_hash,
        )?,
        ledger_hash: to_api_ledger_hash(&ledger_hash),
        ledger_hash_bech32m: to_api_hash_bech32m(&context.mapping_context, &ledger_hash)?,
    })
}

fn attempt_parsing_as_signed_intent(
    context: &ParseContext,
    bytes: &[u8],
) -> Option<(SignedIntentV1, PreparedSignedIntentV1)> {
    let raw = RawSignedTransactionIntent::from_slice(bytes);
    let signed_intent = SignedIntentV1::from_raw(&raw).ok()?;
    let prepared =
        PreparedSignedIntentV1::prepare(&raw, context.transaction_validator.preparation_settings())
            .ok()?;
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
            &prepared.transaction_intent_hash(),
            &prepared.signed_transaction_intent_hash(),
        )?)),
    };
    Ok(models::ParsedTransaction::ParsedSignedTransactionIntent {
        signed_intent: model,
        identifiers: Box::new(models::ParsedSignedTransactionIntentIdentifiers {
            intent_hash: to_api_transaction_intent_hash(&prepared.transaction_intent_hash()),
            intent_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &prepared.transaction_intent_hash(),
            )?,
            signed_intent_hash: to_api_signed_transaction_intent_hash(
                &prepared.signed_transaction_intent_hash(),
            ),
            signed_intent_hash_bech32m: to_api_hash_bech32m(
                &context.mapping_context,
                &prepared.signed_transaction_intent_hash(),
            )?,
        }),
    })
}

fn attempt_parsing_as_intent(
    context: &ParseContext,
    bytes: &[u8],
) -> Option<(IntentV1, PreparedIntentV1)> {
    let raw = RawTransactionIntent::from_slice(bytes);
    let model = IntentV1::from_raw(&raw).ok()?;
    let prepared =
        PreparedIntentV1::prepare(&raw, context.transaction_validator.preparation_settings())
            .ok()?;
    Some((model, prepared))
}

fn to_api_parsed_intent(
    context: &ParseContext,
    (model, prepared): (IntentV1, PreparedIntentV1),
) -> Result<models::ParsedTransaction, ResponseError<()>> {
    let intent_hash = &prepared.transaction_intent_hash();
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_intent_v1(
            &context.mapping_context,
            &model,
            intent_hash,
        )?)),
    };
    Ok(models::ParsedTransaction::ParsedTransactionIntent {
        intent: model,
        identifiers: Box::new(models::ParsedTransactionIntentIdentifiers {
            intent_hash: to_api_transaction_intent_hash(intent_hash),
            intent_hash_bech32m: to_api_hash_bech32m(&context.mapping_context, intent_hash)?,
        }),
    })
}

fn attempt_parsing_as_ledger_transaction(
    context: &ParseContext,
    bytes: &[u8],
) -> Option<(
    LedgerTransaction,
    PreparedLedgerTransaction,
    RawLedgerTransaction,
)> {
    let raw = RawLedgerTransaction::from_slice(bytes);
    let model = LedgerTransaction::from_raw(&raw).ok()?;
    let prepared = PreparedLedgerTransaction::prepare(
        &raw,
        context.transaction_validator.preparation_settings(),
    )
    .ok()?;
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
    let hashes = prepared.create_hashes();
    let model = match context.response_mode {
        ResponseMode::Basic => None,
        ResponseMode::Full => Some(Box::new(to_api_ledger_transaction(
            &context.mapping_context,
            &raw,
            &model,
            &hashes,
        )?)),
    };

    let user_identifiers = hashes.as_user();

    Ok(models::ParsedTransaction::ParsedLedgerTransaction {
        ledger_transaction: model,
        identifiers: Box::new(models::ParsedLedgerTransactionIdentifiers {
            intent_hash: user_identifiers
                .as_ref()
                .map(|hashes| to_api_transaction_intent_hash(&hashes.transaction_intent_hash)),
            intent_hash_bech32m: user_identifiers
                .as_ref()
                .map(|hashes| {
                    to_api_hash_bech32m(&context.mapping_context, &hashes.transaction_intent_hash)
                })
                .transpose()?,
            signed_intent_hash: user_identifiers.as_ref().map(|hashes| {
                to_api_signed_transaction_intent_hash(&hashes.signed_transaction_intent_hash)
            }),
            signed_intent_hash_bech32m: user_identifiers
                .as_ref()
                .map(|hashes| {
                    to_api_hash_bech32m(
                        &context.mapping_context,
                        &hashes.signed_transaction_intent_hash,
                    )
                })
                .transpose()?,
            payload_hash: user_identifiers.as_ref().map(|hashes| {
                to_api_notarized_transaction_hash(&hashes.notarized_transaction_hash)
            }),
            payload_hash_bech32m: user_identifiers
                .as_ref()
                .map(|hashes| {
                    to_api_hash_bech32m(
                        &context.mapping_context,
                        &hashes.notarized_transaction_hash,
                    )
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
