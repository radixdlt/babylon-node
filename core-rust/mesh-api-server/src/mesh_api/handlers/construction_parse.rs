use crate::prelude::*;
use models::{AccountIdentifier, Amount, Currency, Operation, OperationIdentifier};
use radix_engine_interface::blueprints::account::{
    AccountLockFeeManifestInput, AccountTryDepositOrAbortManifestInput,
    AccountWithdrawManifestInput,
};
use radix_transactions::manifest::{CallMethod, TakeFromWorktop};
use radix_transactions::validation::TransactionValidator;

pub(crate) async fn handle_construction_parse(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionParseRequest>,
) -> Result<Json<models::ConstructionParseResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let transaction_bytes = hex::decode(&request.transaction).map_err(|_| {
        client_error(
            format!("Invalid transaction hex: {}", &request.transaction),
            false,
        )
    })?;
    let (instructions, signers) = if request.signed {
        let transaction =
            NotarizedTransactionV1::from_raw(&RawNotarizedTransaction::from_vec(transaction_bytes))
                .map_err(|_| {
                    client_error(
                        format!("Invalid transaction: {}", &request.transaction),
                        false,
                    )
                })?;
        let validated = transaction
            .prepare_and_validate(&TransactionValidator::new_with_latest_config(
                &state.network,
            ))
            .map_err(|e| client_error(format!("Invalid transaction: error = {:?}", e), false))?;

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

    let mut operations = Vec::new();
    let index = 0;
    while index < instructions.len() {
        let mut instruction = &instructions[index];
        match instruction {
            InstructionV1::CallMethod(CallMethod {
                address: DynamicGlobalAddress::Static(global_address),
                method_name,
                args,
            }) => {
                let args_bytes = manifest_encode(&args).unwrap();
                match method_name.as_str() {
                    "lock_fee" => {
                        let input = manifest_decode::<AccountLockFeeManifestInput>(&args_bytes)
                            .map_err(|_| client_error("Invalid lock_fee instruction", false))?;
                        operations.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier {
                                index: operations.len() as i64,
                                network_index: None,
                            }),
                            related_operations: None,
                            _type: "LockFee".to_owned(),
                            status: None,
                            account: Some(Box::new(AccountIdentifier {
                                address: state
                                    .address_encoder()
                                    .encode(global_address.as_bytes())
                                    .unwrap(),
                                sub_account: None,
                                metadata: None,
                            })),
                            amount: Some(Box::new(Amount {
                                value: input.amount.to_string(), // TODO: fix decimal
                                currency: Box::new(Currency {
                                    symbol: "XRD".to_owned(),
                                    decimals: 18,
                                    metadata: None,
                                }),
                                metadata: None,
                            })),
                            coin_change: None,
                            metadata: None,
                        });
                    }
                    "withdraw" => {
                        let input = manifest_decode::<AccountWithdrawManifestInput>(&args_bytes)
                            .map_err(|_| client_error("Invalid withdraw instruction", false))?;
                        operations.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier {
                                index: operations.len() as i64,
                                network_index: None,
                            }),
                            related_operations: None,
                            _type: "Withdraw".to_owned(),
                            status: None,
                            account: Some(Box::new(AccountIdentifier {
                                address: state
                                    .address_encoder()
                                    .encode(global_address.as_bytes())
                                    .unwrap(),
                                sub_account: None,
                                metadata: None,
                            })),
                            amount: Some(Box::new(Amount {
                                value: input.amount.to_string(), // TODO: fix decimal
                                currency: Box::new(Currency {
                                    symbol: "XRD".to_owned(), // TODO: fix resource
                                    decimals: 18,
                                    metadata: None,
                                }),
                                metadata: None,
                            })),
                            coin_change: None,
                            metadata: None,
                        });
                    }
                    _ => {
                        return Err(client_error(
                            format!("Unrecognized instruction: {:?}", instruction),
                            false,
                        ));
                    }
                }
            }
            InstructionV1::TakeFromWorktop(TakeFromWorktop {
                resource_address: _,
                amount,
            }) if index < instructions.len() - 1 => {
                instruction = &instructions[index];
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
                                _type: "Withdraw".to_owned(),
                                status: None,
                                account: Some(Box::new(AccountIdentifier {
                                    address: state
                                        .address_encoder()
                                        .encode(global_address.as_bytes())
                                        .unwrap(),
                                    sub_account: None,
                                    metadata: None,
                                })),
                                amount: Some(Box::new(Amount {
                                    value: amount.to_string(), // TODO: fix decimal
                                    currency: Box::new(Currency {
                                        symbol: "XRD".to_owned(), // TODO: fix resource
                                        decimals: 18,
                                        metadata: None,
                                    }),
                                    metadata: None,
                                })),
                                coin_change: None,
                                metadata: None,
                            });
                            continue;
                        }
                    }
                }

                return Err(client_error(
                    format!("Unrecognized instruction: {:?}", instruction),
                    false,
                ));
            }
            _ => {
                return Err(client_error(
                    format!("Unrecognized instruction: {:?}", instruction),
                    false,
                ));
            }
        }
    }

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionparseresponse for field
    // definitions
    Ok(Json(models::ConstructionParseResponse {
        operations,
        signers: None,
        account_identifier_signers: Some(
            signers
                .into_iter()
                .map(|x| AccountIdentifier {
                    address: state.public_key_to_address(x),
                    sub_account: None,
                    metadata: None,
                })
                .collect(),
        ),
        metadata: None,
    }))
}
