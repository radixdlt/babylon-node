use crate::prelude::*;

pub(crate) async fn handle_account_balance(
    state: State<MeshApiState>,
    Json(request): Json<models::AccountBalanceRequest>,
) -> Result<Json<models::AccountBalanceResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address = extract_account(&extraction_context, &request.account_identifier)
        // TODO:MESH Return something more precise than InvalidRequest
        .map_err(|err| err.into_response_error("account_identifier"))?;

    let database = state.state_manager.database.snapshot();

    let header = if request.block_identifier.is_some() {
        return Err(ResponseError::from(ApiError::InvalidRequest)
            .with_details("Historical balance not supported"));
    } else {
        read_current_ledger_header(database.deref())
    };

    let type_info: Option<TypeInfoSubstate> = read_optional_substate::<TypeInfoSubstate>(
        database.deref(),
        component_address.as_node_id(),
        TYPE_INFO_FIELD_PARTITION,
        &TypeInfoField::TypeInfo.into(),
    );

    if type_info.is_none() {
        return Ok(Json(models::AccountBalanceResponse {
            block_identifier: Box::new(to_mesh_api_block_identifier_from_ledger_header(
                &header.into(),
            )?),
            balances: vec![],
            metadata: None,
        }));
    }

    // TODO:MESH - For performance, we should not use this unless the user provides
    // no request.currencies - as it can be a lot slower on large accounts
    let component_dump = dump_component_state(database.deref(), component_address);
    // TODO:MESH - refactor as per comments:
    // https://github.com/radixdlt/babylon-node/pull/1013#discussion_r1830887100
    // https://github.com/radixdlt/babylon-node/pull/1013#discussion_r1830892623
    let mut resources_set: HashSet<ResourceAddress> = hashset![];
    if let Some(currencies) = request.currencies {
        for currency in currencies.into_iter() {
            let resource_address = extract_resource_address_from_mesh_api_currency(
                &extraction_context,
                database.deref(),
                &currency,
            )
            .map_err(|err| err.into_response_error("resource_address"))?;

            resources_set.insert(resource_address);
        }
    };

    let balances = component_dump
        .vaults
        .into_iter()
        .filter_map(|(_node_id, vault_data)| match vault_data {
            VaultData::NonFungible { .. } => None,
            VaultData::Fungible {
                resource_address,
                amount,
            } => {
                if resources_set.is_empty() || resources_set.get(&resource_address).is_some() {
                    Some((resource_address, amount))
                } else {
                    None
                }
            }
        })
        .fold(
            IndexMap::new(),
            |mut index, (fungible_resource_address, amount)| {
                let sum = index
                    .entry(fungible_resource_address)
                    .or_insert(Decimal::zero());
                *sum = sum.checked_add(amount).expect("Decimal overflow");

                index
            },
        )
        .into_iter()
        .map(|(fungible_resource_address, amount)| {
            let currency = to_mesh_api_currency_from_resource_address(
                &mapping_context,
                database.deref(),
                &fungible_resource_address,
            )?;
            Ok(to_mesh_api_amount(amount, currency)?)
        })
        .collect::<Result<Vec<_>, MappingError>>()?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#accountbalanceresponse for field
    // definitions
    Ok(Json(models::AccountBalanceResponse {
        block_identifier: Box::new(to_mesh_api_block_identifier_from_ledger_header(
            &header.into(),
        )?),
        balances,
        metadata: None,
    }))
}
