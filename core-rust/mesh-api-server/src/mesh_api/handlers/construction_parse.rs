use crate::prelude::*;
use radix_engine_interface::blueprints::account::{
    AccountTryDepositOrAbortManifestInput, AccountWithdrawManifestInput,
};
use radix_transactions::manifest::{CallMethod, TakeFromWorktop};
use radix_transactions::validation::TransactionValidator;

// This method only accepts transactions constructed with the Mesh API,
// which are V1 at the moment.
// Also the number of supported V1 instructions is limited to some basic ones.
// (see `construction_payloads.rs` and parse_instructions() below for more details).
pub(crate) async fn handle_construction_parse(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionParseRequest>,
) -> Result<Json<models::ConstructionParseResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let transaction_bytes = hex::decode(&request.transaction).map_err(|_| {
        ResponseError::from(ApiError::InvalidTransaction)
            .with_details(format!("Invalid transaction hex: {}", &request.transaction))
    })?;

    let (instructions, signers) = if request.signed {
        let transaction =
            NotarizedTransactionV1::from_raw(&RawNotarizedTransaction::from_vec(transaction_bytes))
                .map_err(|_| {
                    ResponseError::from(ApiError::InvalidTransaction)
                        .with_details(format!("Invalid transaction: {}", &request.transaction))
                })?;
        let validated = transaction
            .prepare_and_validate(&TransactionValidator::new_with_latest_config(
                &state.network,
            ))
            .map_err(|e| {
                ResponseError::from(ApiError::InvalidTransaction)
                    .with_details(format!("Invalid transaction: error = {:?}", e))
            })?;

        let instructions = transaction.signed_intent.intent.instructions.0;
        let signers = validated.signer_keys;
        (instructions, signers)
    } else {
        let prepared_intent = PreparedIntentV1::prepare(
            &RawTransactionIntent::from_vec(transaction_bytes),
            &PreparationSettings::latest(),
        )
        .unwrap();
        #[allow(deprecated)]
        let instructions = prepared_intent.instructions.inner.0;
        let signers = index_set_new();
        (instructions, signers)
    };

    let mapping_context = MappingContext::new(&state.network);
    let database = state.state_manager.database.snapshot();
    let operations = parse_instructions(&instructions, &mapping_context, database.deref())?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionparseresponse for field
    // definitions
    Ok(Json(models::ConstructionParseResponse {
        operations,
        signers: None,
        account_identifier_signers: Some(
            signers
                .into_iter()
                .map(|x| -> Result<models::AccountIdentifier, MappingError> {
                    to_api_account_identifier_from_public_key(&mapping_context, x)
                })
                .collect::<Result<Vec<_>, MappingError>>()?,
        ),
        metadata: None,
    }))
}

pub fn parse_instructions(
    instructions: &[InstructionV1],
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
) -> Result<Vec<models::Operation>, ResponseError> {
    let mut operations = Vec::new();
    let mut next_index = 0;
    while next_index < instructions.len() {
        let mut instruction = &instructions[next_index];
        next_index = next_index + 1;
        match instruction {
            InstructionV1::CallMethod(CallMethod {
                address: DynamicGlobalAddress::Static(global_address),
                method_name,
                args,
            }) if global_address.is_account() => {
                let args_bytes = manifest_encode(&args).unwrap();
                match method_name.as_str() {
                    "lock_fee" => (),
                    "withdraw" => {
                        let input = manifest_decode::<AccountWithdrawManifestInput>(&args_bytes)
                            .map_err(|_| {
                                ResponseError::from(ApiError::InvalidWithdrawInstruction)
                                    .with_details("Invalid withdraw instruction")
                            })?;
                        operations.push(to_mesh_api_operation_no_fee(
                            mapping_context,
                            database,
                            operations.len() as i64,
                            None,
                            global_address,
                            &match input.resource_address {
                                ManifestResourceAddress::Static(resource_address) => {
                                    resource_address
                                }
                                ManifestResourceAddress::Named(_) => {
                                    return Err(ResponseError::from(
                                        ApiError::NamedAddressNotSupported,
                                    )
                                    .with_details("Named address is not supported"))
                                }
                            },
                            -input.amount.clone(),
                        )?);
                    }
                    _ => {
                        return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                            .with_details(format!("Unrecognized instruction: {:?}", instruction)));
                    }
                }
            }
            InstructionV1::TakeFromWorktop(TakeFromWorktop {
                resource_address,
                amount,
            }) if next_index < instructions.len() => {
                instruction = &instructions[next_index];
                next_index = next_index + 1;

                match instruction {
                    InstructionV1::CallMethod(CallMethod {
                        address: DynamicGlobalAddress::Static(global_address),
                        method_name,
                        args,
                    }) if method_name.eq("try_deposit_or_abort") && global_address.is_account() => {
                        if let Ok(_input) = manifest_decode::<AccountTryDepositOrAbortManifestInput>(
                            &manifest_encode(&args).unwrap(),
                        ) {
                            operations.push(to_mesh_api_operation_no_fee(
                                mapping_context,
                                database,
                                operations.len() as i64,
                                None,
                                global_address,
                                resource_address,
                                *amount,
                            )?);
                        }
                    }
                    _ => {
                        return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                            .with_details(format!("Unrecognized instruction: {:?}", instruction)));
                    }
                }
            }
            _ => {
                return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                    .with_details(format!("Unrecognized instruction: {:?}", instruction)));
            }
        }
    }

    Ok(operations)
}
