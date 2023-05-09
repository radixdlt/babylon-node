use crate::core_api::*;
use radix_engine::types::blueprints::resource::LiquidFungibleResource;
use radix_engine::types::{scrypto_encode, Decimal};
use radix_engine_common::data::scrypto::model::Own;
use radix_engine_common::types::{EntityType, SubstateKey};
use radix_engine_interface::types::{FungibleVaultOffset, OBJECT_BASE_MODULE};
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

    let encoded_resource_address =
        scrypto_encode(&fungible_resource_address).expect("Impossible Case!");
    let substate_key = SubstateKey::Map(encoded_resource_address);

    let vault_reference_opt: Option<Option<Own>> = read_optional_substate(
        database.deref(),
        account_address.as_node_id(),
        OBJECT_BASE_MODULE,
        &substate_key,
    );

    let amount = match vault_reference_opt {
        Some(Some(vault_reference)) => {
            let liquid_fungible_resource_opt: Option<LiquidFungibleResource> =
                read_optional_substate(
                    database.deref(),
                    vault_reference.as_node_id(),
                    OBJECT_BASE_MODULE,
                    &FungibleVaultOffset::LiquidFungible.into(),
                );
            liquid_fungible_resource_opt
                .map(|r| r.amount())
                .unwrap_or(Decimal::zero())
        }
        _ => Decimal::zero(),
    };

    Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
        state_version: to_api_state_version(database.max_state_version())?,
        account_address: to_api_global_address(&mapping_context, &account_address),
        fungible_resource_balance: Box::new(models::LtsFungibleResourceBalance {
            fungible_resource_address: to_api_resource_address(
                &mapping_context,
                &fungible_resource_address,
            ),
            amount: to_api_decimal(&amount),
        }),
    })
    .map(Json)
}
