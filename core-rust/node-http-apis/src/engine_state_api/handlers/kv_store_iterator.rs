use crate::engine_state_api::*;

use crate::engine_prelude::*;

use crate::engine_state_api::handlers::HandlerPagingSupport;
use std::ops::Deref;

pub(crate) async fn handle_kv_store_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::KeyValueStoreIteratorRequest>,
) -> Result<Json<models::KeyValueStoreIteratorResponse>, ResponseError> {
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

    let database = state.state_manager.database.snapshot();

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

    let page = paging_support
        .get_page(|from| data_loader.iter_kv_store_keys(&node_id, &kv_store_meta, from))?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::KeyValueStoreIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|map_key| to_api_key_value_store_map_key(&mapping_context, map_key))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page.continuation_token,
    }))
}

impl HasKey<MapKey> for SborData<'_> {
    fn as_key(&self) -> MapKey {
        self.as_bytes().to_vec()
    }
}

fn to_api_key_value_store_map_key(
    context: &MappingContext,
    key: SborData,
) -> Result<models::KeyValueStoreMapKey, MappingError> {
    Ok(models::KeyValueStoreMapKey {
        key: Box::new(to_api_sbor_data(context, key)?),
    })
}
