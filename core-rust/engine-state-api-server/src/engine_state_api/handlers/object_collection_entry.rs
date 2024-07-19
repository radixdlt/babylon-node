use crate::engine_state_api::*;

use crate::engine_prelude::*;

use crate::engine_state_api::factories::EngineStateLoaderFactory;
use state_manager::historical_state::VersionScopingSupport;

pub(crate) async fn handle_object_collection_entry(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectCollectionEntryRequest>,
) -> Result<Json<models::ObjectCollectionEntryResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let module_id = request
        .attached_module_id
        .map(|module_id| extract_attached_module_id(&module_id).into())
        .unwrap_or(ModuleId::Main);
    let collection_input =
        extract_rich_index_input(request.collection_name, request.collection_index)
            .map_err(|err| err.into_response_error("collection_name or collection_index"))?;
    let key = extract_collection_key(
        &extraction_context,
        request.key.expect("not actually optional"),
    )
    .map_err(|err| err.into_response_error("key"))?;

    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let loader_factory = EngineStateLoaderFactory::new(&database).ensure_instantiated(&node_id);

    let meta_loader = loader_factory.create_meta_loader();
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let collection_meta = match collection_input {
        RichIndexInput::Name(name) => module_state_meta.collection_by_name(name),
        RichIndexInput::Index(index) => module_state_meta.collection_by_index(index),
    }?;

    let data_loader = loader_factory.create_data_loader();
    let entry_data =
        data_loader.load_collection_entry(&node_id, module_id, collection_meta, &key)?;

    let ledger_state = database.at_ledger_state();

    Ok(Json(models::ObjectCollectionEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &ledger_state,
        )?),
        content: Box::new(to_api_sbor_data(&mapping_context, entry_data)?),
    }))
}

fn extract_collection_key(
    context: &ExtractionContext,
    key: models::CollectionEntryKey,
) -> Result<RawCollectionKey, ExtractionError> {
    Ok(match key {
        models::CollectionEntryKey::IndexEntryKey { key }
        | models::CollectionEntryKey::KeyValueStoreEntryKey { key } => {
            RawCollectionKey::Unsorted(extract_from_sbor_data(context, *key)?)
        }
        models::CollectionEntryKey::SortedIndexEntryKey {
            sort_prefix_hex,
            key,
        } => RawCollectionKey::Sorted(
            copy_u8_array(&from_hex(sort_prefix_hex)?),
            extract_from_sbor_data(context, *key)?,
        ),
    })
}
