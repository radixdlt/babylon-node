use crate::core_api::*;
use radix_engine::blueprints::account::{AccountCollection, AccountResourceVaultEntryPayload};
use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

use state_manager::LedgerHeader;
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
        == Some(EntityType::GlobalFungibleResourceManager);
    if !is_fungible {
        return Err(client_error(
            "The provided resource address is not a fungible resource.",
        ));
    }

    let account_address = extract_component_address(&extraction_context, &request.account_address)
        .map_err(|err| err.into_response_error("account_address"))?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error(
            "Only addresses starting account_ work with this endpoint.",
        ));
    }

    let database = state.state_manager.database.read_current();

    if !account_address.as_node_id().is_global_virtual() {
        read_optional_substate::<TypeInfoSubstate>(
            database.deref(),
            account_address.as_node_id(),
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )
        .ok_or_else(|| not_found_error("Account not found".to_string()))?;
    }

    let header = read_current_ledger_header(database.deref());

    let type_info: Option<TypeInfoSubstate> = read_optional_substate::<TypeInfoSubstate>(
        database.deref(),
        account_address.as_node_id(),
        TYPE_INFO_FIELD_PARTITION,
        &TypeInfoField::TypeInfo.into(),
    );

    if type_info.is_none() {
        if account_address.as_node_id().is_global_virtual() {
            return response(
                &mapping_context,
                &header,
                &account_address,
                &fungible_resource_address,
                &Decimal::ZERO,
            );
        } else {
            return Err(not_found_error("Account not found".to_string()));
        }
    }

    let balance = {
        let encoded_key = scrypto_encode(&fungible_resource_address).expect("Impossible Case!");
        let substate = read_optional_collection_substate::<AccountResourceVaultEntryPayload>(
            database.deref(),
            account_address.as_node_id(),
            AccountCollection::ResourceVaultKeyValue.collection_index(),
            &SubstateKey::Map(encoded_key),
        );
        match substate {
            Some(substate) => {
                let vault = substate
                    .into_value()
                    .ok_or(MappingError::KeyValueStoreEntryUnexpectedlyAbsent)?
                    .into_latest();
                read_mandatory_main_field_substate::<FungibleVaultBalanceFieldPayload>(
                    database.deref(),
                    vault.0.as_node_id(),
                    &FungibleVaultField::Balance.into(),
                )?
                .into_payload()
                .into_latest()
                .amount()
            }
            _ => Decimal::ZERO,
        }
    };

    response(
        &mapping_context,
        &header,
        &account_address,
        &fungible_resource_address,
        &balance,
    )
}

fn response(
    context: &MappingContext,
    header: &LedgerHeader,
    account_address: &ComponentAddress,
    resource_address: &ResourceAddress,
    amount: &Decimal,
) -> Result<Json<models::LtsStateAccountFungibleResourceBalanceResponse>, ResponseError<()>> {
    Ok(models::LtsStateAccountFungibleResourceBalanceResponse {
        state_version: to_api_state_version(header.state_version)?,
        ledger_header_summary: Box::new(to_api_ledger_header_summary(context, header)?),
        account_address: to_api_component_address(context, account_address)?,
        fungible_resource_balance: Box::new(models::LtsFungibleResourceBalance {
            fungible_resource_address: to_api_resource_address(context, resource_address)?,
            amount: to_api_decimal(amount),
        }),
    })
    .map(Json)
}
