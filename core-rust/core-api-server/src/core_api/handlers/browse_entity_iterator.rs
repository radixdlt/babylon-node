use crate::core_api::*;

use radix_engine::types::*;

use std::ops::Deref;
use std::time::Duration;

/// A default maximum page size (can be further limited by each request).
const DEFAULT_MAX_PAGE_SIZE: usize = 10000;

/// A maximum wallclock time spent on iteration.
const MAX_ITERATION_DURATION: Duration = Duration::from_millis(100);

/// A *minimum* number of elements which must be reached even if [`MAX_ITERATION_DURATION`] is
/// exceeded. This only prevents unreasonably small (or empty) pages.
const MIN_PAGE_SIZE_DESPITE_MAX_DURATION: usize = 10;

pub(crate) async fn handle_browse_entity_iterator(
    state: State<CoreApiState>,
    Json(request): Json<models::BrowseEntityIteratorRequest>,
) -> Result<Json<models::BrowseEntityIteratorResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

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
    let data_loader = EngineNodeLister::new(database.deref());

    let page = OrderAgnosticPager::get_page(
        wrap_error_free(|from| data_loader.iter_node_ids(from)),
        create_paging_policy(requested_max_page_size),
        continuation_token,
    )
    .expect("FnIterable is error-free");

    let header = read_current_ledger_header(database.deref());

    Ok(models::BrowseEntityIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|node_id| to_api_listed_entity_item(&mapping_context, &node_id))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page
            .continuation_token
            .map(|continuation_token| to_api_sbor_hex_string(&continuation_token))
            .transpose()?,
    })
    .map(Json)
}

fn create_paging_policy(requested_max_page_size: Option<usize>) -> impl PagingPolicy<NodeId> {
    PagingPolicies::until_first_disallowed(
        PagingPolicies::max_page_size(requested_max_page_size.unwrap_or(DEFAULT_MAX_PAGE_SIZE)),
        PagingPolicies::max_duration_but_min_page_size(
            MAX_ITERATION_DURATION,
            MIN_PAGE_SIZE_DESPITE_MAX_DURATION,
        ),
    )
}

fn to_api_listed_entity_item(
    context: &MappingContext,
    node_id: &NodeId,
) -> Result<models::ListedEntityItem, MappingError> {
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    Ok(models::ListedEntityItem {
        entity_type: to_api_entity_type(entity_type),
        system_type: if entity_type.is_internal_kv_store() {
            models::SystemType::KeyValueStore
        } else {
            models::SystemType::Object
        },
        is_global: node_id.is_global(),
        entity_address: to_api_entity_address(context, node_id)?,
    })
}
