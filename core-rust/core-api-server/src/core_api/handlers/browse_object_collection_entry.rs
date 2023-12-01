use crate::core_api::*;

use radix_engine::types::*;

use crate::core_api::handlers::RawCollectionKey;
use std::ops::Deref;

pub(crate) async fn handle_object_collection_entry(
    state: State<CoreApiState>,
    Json(request): Json<models::BrowseObjectCollectionEntryRequest>,
) -> Result<Json<models::BrowseObjectCollectionEntryResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let module_id = request
        .module_id
        .map(|module_id| extract_api_module_id(&module_id))
        .unwrap_or(ModuleId::Main);
    let collection_input =
        extract_api_rich_index_input(request.collection_name, request.collection_index)
            .map_err(|err| err.into_response_error("collection_name or collection_index"))?;
    let key = extract_api_collection_key(
        &extraction_context,
        request.key.expect("not actually optional"),
    )
    .map_err(|err| err.into_response_error("key"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let collection_meta = match collection_input {
        RichIndexInput::Name(name) => module_state_meta.collection_by_name(name),
        RichIndexInput::Index(index) => module_state_meta.collection_by_index(index),
    }?;

    let data_loader = EngineStateDataLoader::new(database.deref());

    let entry_data =
        data_loader.load_collection_entry(&node_id, module_id, collection_meta, &key)?;
    let programmatic_json = entry_data.into_programmatic_json(&mapping_context)?;

    let header = read_current_ledger_header(database.deref());

    Ok(models::BrowseObjectCollectionEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(models::BrowseObjectCollectionEntryResponseContent { programmatic_json }),
    })
    .map(Json)
}

fn extract_api_collection_key(
    context: &ExtractionContext,
    key: models::CollectionEntryKey,
) -> Result<RawCollectionKey, ExtractionError> {
    let decoder = ProgrammaticJsonDecoder::new(context);
    Ok(match key {
        models::CollectionEntryKey::IndexEntryKey { programmatic_json }
        | models::CollectionEntryKey::KeyValueStoreEntryKey { programmatic_json } => {
            RawCollectionKey::Unsorted(decoder.decode(programmatic_json)?)
        }
        models::CollectionEntryKey::SortedIndexEntryKey {
            sort_prefix_hex,
            programmatic_json,
        } => RawCollectionKey::Sorted(
            copy_u8_array(&from_hex(sort_prefix_hex)?),
            decoder.decode(programmatic_json)?,
        ),
    })
}
