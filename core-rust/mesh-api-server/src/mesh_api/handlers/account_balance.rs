use crate::prelude::*;

pub(crate) async fn handle_account_balance(
    state: State<MeshApiState>,
    Json(request): Json<models::AccountBalanceRequest>,
) -> Result<Json<models::AccountBalanceResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address = extract_radix_account_address_from_account_identifier(
        &extraction_context,
        &request.account_identifier,
    )
    .map_err(|err| err.into_response_error("account_identifier"))?;

    let database = state.state_manager.database.snapshot();
    let state_version = if let Some(block_identifier) = request.block_identifier {
        extract_state_version_from_mesh_api_partial_block_identifier(
            database.deref(),
            &block_identifier,
        )
        .map_err(|err| err.into_response_error("block_identifier"))?
    } else {
        None
    };

    let scoped_database = database.scoped_at(state_version).map_err(|err| {
        ResponseError::from(ApiError::GetStateHistoryError)
            .with_details(format!("Getting state history error: {:?}", err))
    })?;

    let balances = match request.currencies {
        Some(currencies) => {
            let resources = currencies
                .into_iter()
                .filter_map(|c| {
                    // Filter out resources, which were not possible to extract,
                    //  eg. not found because they were not existing at given state version
                    extract_resource_address_from_currency(
                        &extraction_context,
                        &scoped_database,
                        &c,
                    )
                    .ok()
                })
                .collect::<Vec<_>>();

            get_requested_balances(
                &mapping_context,
                &scoped_database,
                &component_address,
                &resources,
            )?
        }
        None => {
            // Check if account is instantiated
            let type_info: Option<TypeInfoSubstate> = read_optional_substate::<TypeInfoSubstate>(
                &scoped_database,
                component_address.as_node_id(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            );

            if type_info.is_some() {
                get_all_balances(&mapping_context, &scoped_database, &component_address)?
            } else {
                // We expect empty balances vector here, but let the `get_requested_balances()`
                // deal with this.
                get_requested_balances(&mapping_context, &scoped_database, &component_address, &[])?
            }
        }
    };

    let ledger_state = scoped_database.at_ledger_state();

    // see https://docs.cdp.coinbase.com/mesh/docs/models#accountbalanceresponse for field
    // definitions
    Ok(Json(models::AccountBalanceResponse {
        block_identifier: Box::new(to_mesh_api_block_identifier_from_ledger_header(
            &ledger_state,
        )?),
        balances,
        metadata: None,
    }))
}
// Method `dump_component_state()` might be slow on large accounts,
// therefore we use it only when user didn't specify which balances
// to get.
fn get_all_balances<'a>(
    mapping_context: &MappingContext,
    database: &VersionScopedDatabase<
        'a,
        impl Deref<Target = <StateManagerDatabase<DirectRocks> as Snapshottable<'a>>::Snapshot>,
    >,
    component_address: &ComponentAddress,
) -> Result<Vec<models::Amount>, MappingError> {
    let component_dump = dump_component_state(database, *component_address);
    component_dump
        .vaults
        .into_iter()
        .filter_map(|(_node_id, vault_data)| match vault_data {
            VaultData::NonFungible { .. } => None,
            VaultData::Fungible {
                resource_address,
                amount,
            } => Some((resource_address, amount)),
        })
        .fold(IndexMap::new(), |mut index, (resource_address, balance)| {
            let sum = index.entry(resource_address).or_insert(Decimal::zero());
            *sum = sum.checked_add(balance).expect("Decimal overflow");

            index
        })
        .into_iter()
        .map(|(resource_address, balance)| {
            let currency = to_mesh_api_currency_from_resource_address(
                &mapping_context,
                database,
                &resource_address,
            )?;
            Ok(to_mesh_api_amount(balance, currency)?)
        })
        .collect::<Result<Vec<_>, MappingError>>()
}

fn get_requested_balances(
    mapping_context: &MappingContext,
    database: &dyn SubstateDatabase,
    component_address: &ComponentAddress,
    resource_addresses: &[ResourceAddress],
) -> Result<Vec<models::Amount>, ResponseError> {
    resource_addresses
        .into_iter()
        .map(|resource_address| {
            let balance = {
                let encoded_key = scrypto_encode(resource_address).expect("Impossible Case!");
                let substate =
                    read_optional_collection_substate::<AccountResourceVaultEntryPayload>(
                        database,
                        component_address.as_node_id(),
                        AccountCollection::ResourceVaultKeyValue.collection_index(),
                        &SubstateKey::Map(encoded_key),
                    );
                match substate {
                    Some(substate) => {
                        let vault = substate
                            .into_value()
                            .ok_or(MappingError::KeyValueStoreEntryUnexpectedlyAbsent)?
                            .fully_update_and_into_latest_version();
                        read_mandatory_main_field_substate::<FungibleVaultBalanceFieldPayload>(
                            database,
                            vault.0.as_node_id(),
                            &FungibleVaultField::Balance.into(),
                        )?
                        .into_payload()
                        .fully_update_and_into_latest_version()
                        .amount()
                    }
                    _ => Decimal::ZERO,
                }
            };

            let currency = to_mesh_api_currency_from_resource_address(
                &mapping_context,
                database,
                &resource_address,
            )?;
            Ok(to_mesh_api_amount(balance, currency)?)
        })
        .collect::<Result<Vec<_>, ResponseError>>()
}
