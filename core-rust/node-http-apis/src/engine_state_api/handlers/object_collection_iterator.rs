use crate::engine_state_api::*;

use radix_engine::types::*;

use crate::engine_state_api::handlers::HandlerPagingSupport;
use std::ops::Deref;

pub(crate) async fn handle_object_collection_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectCollectionIteratorRequest>,
) -> Result<Json<models::ObjectCollectionIteratorResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);
    let paging_support = HandlerPagingSupport::new(
        request.max_page_size,
        request.continuation_token,
        &Option::<()>::None,
    );

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let module_id = request
        .attached_module_id
        .map(|module_id| extract_api_attached_module_id(&module_id).into())
        .unwrap_or(ModuleId::Main);
    let collection_input =
        extract_api_rich_index_input(request.collection_name, request.collection_index)
            .map_err(|err| err.into_response_error("collection_name or collection_index"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let collection_meta = match collection_input {
        RichIndexInput::Name(name) => module_state_meta.collection_by_name(name),
        RichIndexInput::Index(index) => module_state_meta.collection_by_index(index),
    }?;

    let data_loader = EngineStateDataLoader::new(database.deref());

    let page = paging_support.get_page(|from| {
        data_loader.iter_object_collection_keys(&node_id, module_id, collection_meta, from)
    })?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::ObjectCollectionIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|key| to_api_object_collection_entry_key(&mapping_context, key))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page.continuation_token,
    }))
}

impl HasKey<RawCollectionKey> for ObjectCollectionKey<'_> {
    fn as_key(&self) -> RawCollectionKey {
        match self {
            ObjectCollectionKey::KeyValueStore(sbor_data)
            | ObjectCollectionKey::Index(sbor_data) => {
                RawCollectionKey::Unsorted(sbor_data.to_scrypto_value())
            }
            ObjectCollectionKey::SortedIndex(sorted_prefix, sbor_data) => {
                RawCollectionKey::Sorted(*sorted_prefix, sbor_data.to_scrypto_value())
            }
        }
    }
}

fn to_api_object_collection_entry_key(
    context: &MappingContext,
    key: ObjectCollectionKey,
) -> Result<models::CollectionEntryKey, MappingError> {
    Ok(match key {
        ObjectCollectionKey::KeyValueStore(sbor_data) => {
            models::CollectionEntryKey::KeyValueStoreEntryKey {
                key: Box::new(to_api_sbor_data(context, sbor_data)?),
            }
        }
        ObjectCollectionKey::Index(sbor_data) => models::CollectionEntryKey::IndexEntryKey {
            key: Box::new(to_api_sbor_data(context, sbor_data)?),
        },
        ObjectCollectionKey::SortedIndex(sorted_part, sbor_data) => {
            models::CollectionEntryKey::SortedIndexEntryKey {
                sort_prefix_hex: to_hex(sorted_part),
                key: Box::new(to_api_sbor_data(context, sbor_data)?),
            }
        }
    })
}
