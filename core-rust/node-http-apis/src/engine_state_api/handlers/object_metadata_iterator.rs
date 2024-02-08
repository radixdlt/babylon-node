use crate::engine_state_api::*;

use radix_engine::system::attached_modules::metadata::MetadataCollection;
use radix_engine::types::*;

use crate::engine_state_api::handlers::HandlerPagingSupport;
use std::ops::Deref;

pub(crate) async fn handle_object_metadata_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectMetadataIteratorRequest>,
) -> Result<Json<models::ObjectMetadataIteratorResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);
    let paging_support = HandlerPagingSupport::new(
        request.max_page_size,
        request.continuation_token,
        &Option::<()>::None,
    );

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let metadata_state_meta =
        meta_loader.load_object_module_state_meta(&node_id, ModuleId::Metadata)?;
    let entries_meta = metadata_state_meta
        .collection_by_index(MetadataCollection::EntryKeyValue.collection_index())?;
    let data_loader = EngineStateDataLoader::new(database.deref());

    let page = paging_support.get_page(|from| {
        data_loader.iter_object_collection_keys(&node_id, ModuleId::Metadata, entries_meta, from)
    })?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::ObjectMetadataIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|key| to_api_metadata_entry_key(&mapping_context, key))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page.continuation_token,
    }))
}

fn to_api_metadata_entry_key(
    _context: &MappingContext,
    key: ObjectCollectionKey,
) -> Result<models::MetadataEntryKey, EngineStateBrowsingError> {
    let ObjectCollectionKey::KeyValueStore(sbor_data) = key else {
        return Err(EngineStateBrowsingError::EngineInvariantBroken(
            "metadata collection must be Key-Value".to_string(),
        ));
    };
    Ok(models::MetadataEntryKey {
        key: scrypto_decode(sbor_data.as_bytes()).map_err(|_err| {
            EngineStateBrowsingError::EngineInvariantBroken(
                "metadata keys must be strings".to_string(),
            )
        })?,
    })
}
