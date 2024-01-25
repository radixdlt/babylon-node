use crate::engine_state_api::*;

use radix_engine::types::*;

use std::ops::Deref;

pub(crate) async fn handle_kv_store_entry(
    state: State<EngineStateApiState>,
    Json(request): Json<models::KeyValueStoreEntryRequest>,
) -> Result<Json<models::KeyValueStoreEntryResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let key = extract_api_sbor_data(&extraction_context, *request.key)
        .map_err(|err| err.into_response_error("key"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let EntityMeta::KeyValueStore(kv_store_meta) = meta_loader.load_entity_meta(&node_id)? else {
        return Err(ResponseError::new(
            StatusCode::BAD_REQUEST,
            "Given entity is not a Key-Value Store",
        )
        .with_public_details(models::ErrorDetails::RequestedItemInvalidDetails {
            item_type: models::RequestedItemType::Entity,
        }));
    };

    let data_loader = EngineStateDataLoader::new(database.deref());
    let entry_data = data_loader.load_kv_store_entry(&node_id, &kv_store_meta, &key)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::KeyValueStoreEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(to_api_sbor_data(&mapping_context, entry_data)?),
    }))
}
