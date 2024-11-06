use crate::prelude::*;

pub(crate) async fn handle_account_balance(
    state: State<MeshApiState>,
    Json(request): Json<models::AccountBalanceRequest>,
) -> Result<Json<models::AccountBalanceResponse>, ResponseError> {
    // TODO assert sub_network
    assert_matching_network(&request.network_identifier.network, &state.network)?;
    assert_account(&request.account_identifier)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address =
        extract_component_address(&extraction_context, &request.account_identifier.address)
            .map_err(|err| err.into_response_error("account_identifier.address"))?;

    let database = state.state_manager.database.snapshot();

    let header = if let Some(_) = request.block_identifier {
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
        if component_address.as_node_id().is_global_preallocated() {
            return Ok(Json(models::AccountBalanceResponse {
                block_identifier: Box::new(ledger_header_to_block_identifier(&header.into())?),
                balances: vec![],
                metadata: None,
            }));
        } else {
            return Err(ResponseError::from(ApiError::AccountNotFound));
        }
    }

    let component_dump = dump_component_state(database.deref(), component_address);
    let mut resources_set: HashSet<ResourceAddress> = hashset![];
    if let Some(currencies) = request.currencies {
        for currency in currencies.into_iter() {
            let address = extract_resource_address(&extraction_context, &currency.symbol)
                .map_err(|err| err.into_response_error("resource_address"))?;

            let currency_from_address =
                resource_address_to_currency(database.deref(), &currency.symbol, address)?;

            if currency_from_address.decimals != currency.decimals {
                return Err(ResponseError::from(ApiError::InvalidResource)
                    .with_details(format!("currency {} decimals don't match", currency.symbol)));
            }

            resources_set.insert(address);
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
            let symbol = to_api_resource_address(&mapping_context, &fungible_resource_address)?;
            let currency =
                resource_address_to_currency(database.deref(), &symbol, fungible_resource_address)?;
            Ok(models::Amount {
                value: to_api_decimal(&amount),
                currency: Box::new(currency),
                metadata: None,
            })
        })
        .collect::<Result<Vec<_>, MappingError>>()?;

    Ok(Json(models::AccountBalanceResponse {
        block_identifier: Box::new(ledger_header_to_block_identifier(&header.into())?),
        balances,
        metadata: None,
    }))
}
