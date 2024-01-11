use crate::core_api::*;

use radix_engine::types::*;

use std::ops::Deref;

pub(crate) async fn handle_browse_kv_store_entry(
    state: State<CoreApiState>,
    Json(request): Json<models::BrowseKeyValueStoreEntryRequest>,
) -> Result<Json<models::BrowseKeyValueStoreEntryResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let key = ProgrammaticJsonDecoder::new(&extraction_context)
        .decode(request.key.programmatic_json)
        .map_err(|err| err.into_response_error("key"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let EntityMeta::KeyValueStore(kv_store_meta) = meta_loader.load_entity_meta(&node_id)? else {
        return Err(client_error("given entity is not a Key-Value Store"));
    };

    let data_loader = EngineStateDataLoader::new(database.deref());
    let entry_data = data_loader.load_kv_store_entry(&node_id, &kv_store_meta, &key)?;
    let programmatic_json = entry_data.into_programmatic_json(&mapping_context)?;

    let header = read_current_ledger_header(database.deref());

    Ok(models::BrowseKeyValueStoreEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(models::BrowseKeyValueStoreEntryResponseContent { programmatic_json }),
    })
    .map(Json)
}
