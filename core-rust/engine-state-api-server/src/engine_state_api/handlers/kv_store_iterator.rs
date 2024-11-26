use crate::prelude::*;

pub(crate) async fn handle_kv_store_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::KeyValueStoreIteratorRequest>,
) -> Result<Json<models::KeyValueStoreIteratorResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);
    let paging_support = HandlerPagingSupport::new_with_serde_filter(
        request.max_page_size,
        request.continuation_token,
        &request.at_ledger_state,
    );

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

    let loader_factory = EngineStateLoaderFactory::new(state.network.clone(), &database)
        .ensure_instantiated(&node_id);

    let meta_loader = loader_factory.create_meta_loader();
    let EntityMeta::KeyValueStore(kv_store_meta) = meta_loader.load_entity_meta(&node_id)? else {
        return Err(ResponseError::new(
            StatusCode::BAD_REQUEST,
            "Given entity is not a Key-Value Store",
        )
        .with_public_details(models::ErrorDetails::RequestedItemInvalidDetails {
            item_type: models::RequestedItemType::Entity,
        }));
    };

    let data_loader = loader_factory.create_data_loader();
    let page = paging_support
        .get_page(|from| data_loader.iter_kv_store_keys(&node_id, &kv_store_meta, from))?;

    let ledger_state = database.at_ledger_state();

    Ok(Json(models::KeyValueStoreIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &ledger_state,
        )?),
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
