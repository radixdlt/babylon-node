use crate::core_api::*;
use radix_engine::types::{Decimal, ResourceAddress};
use state_manager::{
    jni::state_manager::ActualStateManager,
    query::{dump_component_state, VaultData},
    store::traits::QueryableProofStore,
};

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_rc_state_account_fungible_resource_balance(
    state: State<CoreApiState>,
    request: Json<models::LtsStateAccountFungibleResourceBalanceRequest>,
) -> Result<Json<models::LtsStateAccountFungibleResourceBalanceResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_rc_state_account_fungible_resource_balance_internal,
    )
}

fn handle_rc_state_account_fungible_resource_balance_internal(
    state_manager: &ActualStateManager,
    request: models::LtsStateAccountFungibleResourceBalanceRequest,
) -> Result<models::LtsStateAccountFungibleResourceBalanceResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error(
            "Only component addresses starting with account_ currently work with this endpoint.",
        ));
    }

    let mapping_context = MappingContext::new(&state_manager.network);
    let extraction_context = ExtractionContext::new(&state_manager.network);

    let fungible_resource_address =
        extract_resource_address(&extraction_context, &request.fungible_resource_address)
            .map_err(|err| err.into_response_error("fungible_resource_address"))?;

    if let ResourceAddress::NonFungible(_) = fungible_resource_address {
        return Err(client_error(
            "The provided resource address is not a fungible resource.",
        ));
    }

    let component_address =
        extract_component_address(&extraction_context, &request.account_address)
            .map_err(|err| err.into_response_error("account_address"))?;

    let component_dump = dump_component_state(state_manager.store(), component_address)
        .map_err(|err| server_error(format!("Error traversing component state: {err:?}")))?;

    let fungible_resource_balance_amount = component_dump
        .vaults
        .into_iter()
        .filter_map(|vault| match vault {
            VaultData::NonFungible {
                resource_address: _,
                ids: _,
            } => None,
            VaultData::Fungible {
                resource_address,
                amount,
            } => {
                if resource_address != fungible_resource_address {
                    return None;
                }
                Some(amount)
            }
        })
        .fold(Decimal::zero(), |total, amount| total + amount);

    Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
        state_version: to_api_state_version(state_manager.store().max_state_version())?,
        account_address: to_api_component_address(&mapping_context, &component_address),
        fungible_resource_balance: Box::new(models::LtsFungibleResourceBalance {
            fungible_resource_address: to_api_resource_address(
                &mapping_context,
                &fungible_resource_address,
            ),
            amount: to_api_decimal(&fungible_resource_balance_amount),
        }),
    })
}
