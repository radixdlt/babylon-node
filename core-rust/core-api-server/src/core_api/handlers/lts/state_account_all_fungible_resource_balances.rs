use crate::core_api::*;
use radix_engine::types::{Decimal, IndexMap};
use state_manager::query::{dump_component_state, VaultData};
use state_manager::store::traits::QueryableProofStore;
use std::ops::Deref;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_lts_state_account_all_fungible_resource_balances(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStateAccountAllFungibleResourceBalancesRequest>,
) -> Result<Json<models::LtsStateAccountAllFungibleResourceBalancesResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error(
            "Only component addresses starting with account_ currently work with this endpoint.",
        ));
    }

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address =
        extract_component_address(&extraction_context, &request.account_address)
            .map_err(|err| err.into_response_error("account_address"))?;

    let database = state.database.read();

    let component_dump = dump_component_state(database.deref(), component_address);

    let fungible_resource_balances = component_dump
        .vaults
        .into_iter()
        .filter_map(|(_node_id, vault_data)| match vault_data {
            VaultData::NonFungible { .. } => None,
            VaultData::Fungible {
                resource_address,
                amount,
            } => Some((resource_address, amount)),
        })
        .fold(
            IndexMap::new(),
            |mut index, (fungible_resource_address, amount)| {
                {
                    *index
                        .entry(fungible_resource_address)
                        .or_insert(Decimal::zero()) += amount;
                }
                index
            },
        )
        .into_iter()
        .map(
            |(fungible_resource_address, amount)| models::LtsFungibleResourceBalance {
                fungible_resource_address: to_api_resource_address(
                    &mapping_context,
                    &fungible_resource_address,
                ),
                amount: to_api_decimal(&amount),
            },
        )
        .collect();

    Ok(models::LtsStateAccountAllFungibleResourceBalancesResponse {
        state_version: to_api_state_version(database.max_state_version())?,
        account_address: to_api_component_address(&mapping_context, &component_address),
        fungible_resource_balances,
    })
    .map(Json)
}
