use crate::core_api::*;
use radix_engine::types::*;
use state_manager::query::{dump_component_state, VaultData};
use state_manager::store::traits::QueryableProofStore;
use std::ops::Deref;

pub(crate) async fn handle_state_account(
    state: State<CoreApiState>,
    Json(request): Json<models::StateAccountRequest>,
) -> Result<Json<models::StateAccountResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address =
        extract_component_address(&extraction_context, &request.account_address)
            .map_err(|err| err.into_response_error("account_address"))?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error("Only account addresses starting account_ currently work with this endpoint. Try another endpoint instead."));
    }

    let database = state.database.read();
    let type_info = read_optional_substate(
        database.deref(),
        component_address.as_node_id(),
        TYPE_INFO_FIELD_PARTITION,
        &TypeInfoField::TypeInfo.into(),
    )
    .ok_or_else(|| not_found_error("Account not found".to_string()))?;

    let owner_role_substate = read_mandatory_substate(
        database.deref(),
        component_address.as_node_id(),
        ACCESS_RULES_FIELDS_PARTITION,
        &AccessRulesField::OwnerRole.into(),
    )?;

    let state_substate = read_mandatory_main_field_substate(
        database.deref(),
        component_address.as_node_id(),
        &AccountField::Account.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), component_address);

    let vaults = component_dump
        .vaults
        .into_iter()
        .map(|(vault_id, vault_data)| map_to_vault_balance(&mapping_context, vault_id, vault_data))
        .collect::<Result<Vec<_>, MappingError>>()?;

    Ok(models::StateAccountResponse {
        state_version: to_api_state_version(database.max_state_version())?,
        info: Some(to_api_type_info_substate(&mapping_context, &type_info)?),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
        state: Some(to_api_account_state_substate(
            &mapping_context,
            &state_substate,
        )?),
        vaults,
    })
    .map(Json)
}

pub(crate) fn map_to_vault_balance(
    context: &MappingContext,
    vault_id: NodeId,
    vault_data: VaultData,
) -> Result<models::VaultBalance, MappingError> {
    let resource_amount = match vault_data {
        VaultData::NonFungible {
            resource_address,
            amount,
            ids,
        } => to_api_non_fungible_resource_amount(context, &resource_address, &amount, &ids)?,
        VaultData::Fungible {
            resource_address,
            amount,
        } => to_api_fungible_resource_amount(context, &resource_address, &amount)?,
    };
    Ok(models::VaultBalance {
        vault_entity: Box::new(to_api_entity_reference(context, &vault_id)?),
        resource_amount: Some(resource_amount),
    })
}
