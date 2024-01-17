use crate::engine_state_api::*;

use radix_engine::types::*;

use crate::engine_state_api::handlers::default_paging_policy;
use std::ops::Deref;

pub(crate) async fn handle_kv_store_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::KeyValueStoreIteratorRequest>,
) -> Result<Json<models::KeyValueStoreIteratorResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;

    let requested_max_page_size = request
        .max_page_size
        .map(extract_api_max_page_size)
        .transpose()
        .map_err(|error| error.into_response_error("max_page_size"))?;
    let continuation_token = request
        .continuation_token
        .as_ref()
        .map(extract_api_sbor_hex_string)
        .transpose()
        .map_err(|error| error.into_response_error("continuation_token"))?;

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

    let page = OrderAgnosticPager::get_page(
        wrap(|from| data_loader.iter_kv_store_keys(&node_id, &kv_store_meta, from)),
        default_paging_policy(requested_max_page_size),
        continuation_token,
    )?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::KeyValueStoreIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|map_key| to_api_key_value_store_map_key(&mapping_context, map_key))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page
            .continuation_token
            .map(|continuation_token| to_api_sbor_hex_string(&continuation_token))
            .transpose()?,
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
        programmatic_json: key.into_programmatic_json(context)?,
    })
}