use crate::prelude::*;
use models::AccountIdentifier;
use models::{Operation, OperationIdentifier};
use radix_engine_interface::blueprints::account::{
    AccountTryDepositOrAbortManifestInput, AccountWithdrawManifestInput,
};
use radix_transactions::manifest::{CallMethod, TakeFromWorktop};
use radix_transactions::validation::TransactionValidator;

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

    let database = state.state_manager.database.snapshot();
    let operations = parse_instructions(
        &instructions,
        &MappingContext::new(&state.network),
        database.deref(),
    )?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionparseresponse for field
    // definitions
    Ok(Json(models::ConstructionParseResponse {
        operations,
        signers: None,
        account_identifier_signers: Some(
            signers
                .into_iter()
                .map(|x| AccountIdentifier {
                    address: state.public_key_to_address_string(x),
                    sub_account: None,
                    metadata: None,
                })
                .collect(),
        ),
        metadata: None,
    }))
}

pub fn parse_instructions(
    instructions: &[InstructionV1],
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
) -> Result<Vec<Operation>, ResponseError> {
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
            }) => {
                let args_bytes = manifest_encode(&args).unwrap();
                match method_name.as_str() {
                    "lock_fee" => (),
                    "withdraw" => {
                        let input = manifest_decode::<AccountWithdrawManifestInput>(&args_bytes)
                            .map_err(|_| {
                                ResponseError::from(ApiError::InvalidWithdrawInstruction)
                                    .with_details("Invalid withdraw instruction")
                            })?;
                        operations.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier {
                                index: operations.len() as i64,
                                network_index: None,
                            }),
                            related_operations: None,
                            _type: "Withdraw".to_owned(),
                            status: None,
                            account: Some(Box::new(to_mesh_api_account_from_address(
                                mapping_context,
                                global_address,
                            )?)),
                            amount: Some(Box::new(to_mesh_api_amount(
                                -input.amount.clone(),
                                to_mesh_api_currency_from_resource_address(
                                    mapping_context,
                                    database,
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
                                )?,
                            )?)),
                            coin_change: None,
                            metadata: None,
                        });
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
                if let InstructionV1::CallMethod(CallMethod {
                    address: DynamicGlobalAddress::Static(global_address),
                    method_name,
                    args,
                }) = instruction
                {
                    if method_name.eq("try_deposit_or_abort") {
                        if let Ok(_input) = manifest_decode::<AccountTryDepositOrAbortManifestInput>(
                            &manifest_encode(&args).unwrap(),
                        ) {
                            operations.push(Operation {
                                operation_identifier: Box::new(OperationIdentifier {
                                    index: operations.len() as i64,
                                    network_index: None,
                                }),
                                related_operations: None,
                                _type: "Deposit".to_owned(),
                                status: None,
                                account: Some(Box::new(to_mesh_api_account_from_address(
                                    mapping_context,
                                    global_address,
                                )?)),
                                amount: Some(Box::new(to_mesh_api_amount(
                                    *amount,
                                    to_mesh_api_currency_from_resource_address(
                                        mapping_context,
                                        database,
                                        resource_address,
                                    )?,
                                )?)),
                                coin_change: None,
                                metadata: None,
                            });
                            continue;
                        }
                    }
                }

                return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                    .with_details(format!("Unrecognized instruction: {:?}", instruction)));
            }
            _ => {
                return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                    .with_details(format!("Unrecognized instruction: {:?}", instruction)));
            }
        }
    }

    Ok(operations)
}
