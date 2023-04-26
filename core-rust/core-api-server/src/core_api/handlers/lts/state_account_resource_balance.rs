use crate::core_api::*;
use radix_engine::{
    system::node_substates::PersistedSubstate,
    types::{
        scrypto_encode, AccountOffset, Address, ComponentAddress, Decimal, KeyValueStoreOffset,
        NodeModuleId, RENodeId, ResourceAddress, SubstateOffset,
    },
};
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

    if let ResourceAddress::NonFungible(_) = fungible_resource_address {
        return Err(client_error(
            "The provided resource address is not a fungible resource.",
        ));
    }

    let component_address =
        extract_component_address(&extraction_context, &request.account_address)
            .map_err(|err| err.into_response_error("account_address"))?;

    let address = Address::Component(component_address);

    let database = state.database.read();

    let account_substate = {
        let substate_offset = SubstateOffset::Account(AccountOffset::Account);
        let loaded_substate = match read_mandatory_substate(
            database.deref(),
            RENodeId::GlobalObject(address),
            NodeModuleId::SELF,
            &substate_offset,
        ) {
            Ok(substate) => substate,
            Err(_) => {
                match component_address {
                    ComponentAddress::Account(_) => {
                        return Err(not_found_error("Account not found"))
                    }
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(_)
                    | ComponentAddress::EddsaEd25519VirtualAccount(_) => {
                        return Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
                            state_version: to_api_state_version(database.max_state_version())?,
                            account_address: to_api_component_address(
                                &mapping_context,
                                &component_address,
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
                };
            }
        };
        let PersistedSubstate::Account(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let vault_substate = {
        let encoded_key = scrypto_encode(&fungible_resource_address).expect("Impossible Case!");
        let kv_store_id = account_substate.vaults.key_value_store_id();

        let node_id = RENodeId::KeyValueStore(kv_store_id);
        let substate_offset =
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));

        let loaded_substate = match read_mandatory_substate(
            database.deref(),
            node_id,
            NodeModuleId::SELF,
            &substate_offset,
        ) {
            Ok(substate) => substate,
            Err(_) => {
                return Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
                    state_version: to_api_state_version(database.max_state_version())?,
                    account_address: to_api_component_address(&mapping_context, &component_address),
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
        let PersistedSubstate::VaultLiquidFungible(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
        state_version: to_api_state_version(database.max_state_version())?,
        account_address: to_api_component_address(&mapping_context, &component_address),
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
