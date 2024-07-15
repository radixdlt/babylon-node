use crate::engine_state_api::*;

use crate::engine_prelude::*;

use crate::engine_state_api::handlers::HandlerPagingSupport;

use crate::engine_state_api::factories::EngineStateLoaderFactory;
use state_manager::historical_state::VersionScopingSupport;

pub(crate) async fn handle_object_metadata_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectMetadataIteratorRequest>,
) -> Result<Json<models::ObjectMetadataIteratorResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);
    let paging_support =
        HandlerPagingSupport::new_without_filter(request.max_page_size, request.continuation_token);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;

    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let loader_factory = EngineStateLoaderFactory::new(&database).ensure_instantiated(&node_id);

    let loader = loader_factory.create_object_metadata_loader();
    let page = paging_support.get_page(|from| loader.iter_keys(&node_id, from))?;

    let ledger_state = database.at_ledger_state();

    Ok(Json(models::ObjectMetadataIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &ledger_state,
        )?),
        page: page
            .items
            .into_iter()
            .map(to_api_metadata_entry_key)
            .collect(),
        continuation_token: page.continuation_token,
    }))
}

fn to_api_metadata_entry_key(key: MetadataKey) -> models::MetadataEntryKey {
    models::MetadataEntryKey { key: key.string }
}

impl HasKey<MetadataKey> for MetadataKey {
    fn as_key(&self) -> MetadataKey {
        self.clone()
    }
}
