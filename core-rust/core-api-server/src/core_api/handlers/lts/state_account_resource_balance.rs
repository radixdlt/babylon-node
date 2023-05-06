use crate::core_api::*;
use radix_engine::types::blueprints::resource::LiquidFungibleResource;
use radix_engine::types::{scrypto_encode, AccountOffset, Decimal};
use radix_engine_common::types::{EntityType, SubstateKey};
use radix_engine_interface::types::SysModuleId;
use radix_engine_queries::typed_substate_layout::AccountSubstate;
use state_manager::store::traits::QueryableProofStore;
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_state_account_fungible_resource_balance(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStateAccountFungibleResourceBalanceRequest>,
) -> Result<Json<models::LtsStateAccountFungibleResourceBalanceResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error(
            "Only component addresses starting with account_ work with this endpoint.",
        ));
    }

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let fungible_resource_address =
        extract_resource_address(&extraction_context, &request.resource_address)
            .map_err(|err| err.into_response_error("resource_address"))?;

    let is_fungible = fungible_resource_address.as_node_id().entity_type()
        == Some(EntityType::GlobalFungibleResource);
    if !is_fungible {
        return Err(client_error(
            "The provided resource address is not a fungible resource.",
        ));
    }

    let account_address = extract_global_address(&extraction_context, &request.account_address)
        .map_err(|err| err.into_response_error("account_address"))?;

    let database = state.database.read();

    let account_substate: AccountSubstate = match read_mandatory_substate(
        database.deref(),
        account_address.as_node_id(),
        SysModuleId::Object.into(),
        &AccountOffset::Account.into(),
    ) {
        Ok(substate) => substate,
        Err(_) => {
            match account_address.as_node_id().entity_type() {
                Some(entity_type) => match entity_type {
                    EntityType::GlobalAccount => {
                        return Err(not_found_error("Account not found"));
                    }
                    EntityType::GlobalVirtualEcdsaAccount
                    | EntityType::GlobalVirtualEddsaAccount => {
                        return Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
                            state_version: to_api_state_version(database.max_state_version())?,
                            account_address: to_api_global_address(
                                &mapping_context,
                                &account_address,
                            ),
                            fungible_resource_balance: Box::new(
                                models::LtsFungibleResourceBalance {
                                    fungible_resource_address: to_api_resource_address(
                                        &mapping_context,
                                        &fungible_resource_address,
                                    ),
                                    amount: to_api_decimal(&Decimal::ZERO),
                                },
                            ),
                        })
                        .map(Json)
                    }
                    _ => {
                        return Err(client_error(
                            "Provided address is not an Account address.".to_string(),
                        ));
                    }
                },
                None => {
                    return Err(client_error(
                        "Provided address is not an Account address.".to_string(),
                    ));
                }
            };
        }
    };

    let vault_substate = {
        let kv_store_id = account_substate.vaults.as_node_id();
        let encoded_key = scrypto_encode(&fungible_resource_address).expect("Impossible Case!");
        let substate_key = SubstateKey::Map(encoded_key);

        let loaded_substate: LiquidFungibleResource = match read_mandatory_substate(
            database.deref(),
            kv_store_id,
            SysModuleId::Object.into(),
            &substate_key,
        ) {
            Ok(substate) => substate,
            Err(_) => {
                return Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
                    state_version: to_api_state_version(database.max_state_version())?,
                    account_address: to_api_global_address(&mapping_context, &account_address),
                    fungible_resource_balance: Box::new(models::LtsFungibleResourceBalance {
                        fungible_resource_address: to_api_resource_address(
                            &mapping_context,
                            &fungible_resource_address,
                        ),
                        amount: to_api_decimal(&Decimal::zero()),
                    }),
                })
                .map(Json)
            }
        };
        loaded_substate
    };

    Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
        state_version: to_api_state_version(database.max_state_version())?,
        account_address: to_api_global_address(&mapping_context, &account_address),
        fungible_resource_balance: Box::new(models::LtsFungibleResourceBalance {
            fungible_resource_address: to_api_resource_address(
                &mapping_context,
                &fungible_resource_address,
            ),
            amount: to_api_decimal(&vault_substate.amount()),
        }),
    })
    .map(Json)
}
