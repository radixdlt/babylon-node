use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::VersionScopingSupport;

pub(crate) async fn handle_kv_store_entry(
    state: State<EngineStateApiState>,
    Json(request): Json<models::KeyValueStoreEntryRequest>,
) -> Result<Json<models::KeyValueStoreEntryResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let key = extract_from_sbor_data(&extraction_context, *request.key)
        .map_err(|err| err.into_response_error("key"))?;
    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let meta_loader = EngineStateMetaLoader::new(&database);
    let EntityMeta::KeyValueStore(kv_store_meta) = meta_loader.load_entity_meta(&node_id)? else {
        return Err(ResponseError::new(
            StatusCode::BAD_REQUEST,
            "Given entity is not a Key-Value Store",
        )
        .with_public_details(models::ErrorDetails::RequestedItemInvalidDetails {
            item_type: models::RequestedItemType::Entity,
        }));
    };

    let data_loader = EngineStateDataLoader::new(&database);
    let entry_data = data_loader.load_kv_store_entry(&node_id, &kv_store_meta, &key)?;

    let ledger_state = database.at_ledger_state();

    Ok(Json(models::KeyValueStoreEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &ledger_state,
        )?),
        content: Box::new(to_api_sbor_data(&mapping_context, entry_data)?),
    }))
}
