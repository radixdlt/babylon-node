mod browse_blueprint_info;
mod browse_entity_info;
mod browse_entity_iterator;
mod browse_entity_schema_entry;
mod browse_kv_store_entry;
mod browse_kv_store_iterator;
mod browse_object_collection_entry;
mod browse_object_collection_iterator;
mod browse_object_field;

pub(crate) use browse_blueprint_info::*;
pub(crate) use browse_entity_info::*;
pub(crate) use browse_entity_iterator::*;
pub(crate) use browse_entity_schema_entry::*;
pub(crate) use browse_kv_store_entry::*;
pub(crate) use browse_kv_store_iterator::*;
pub(crate) use browse_object_collection_entry::*;
pub(crate) use browse_object_collection_iterator::*;
pub(crate) use browse_object_field::*;

use crate::browse_api::{PagingPolicies, PagingPolicy};
use std::time::Duration;

/// A default maximum page size (can be further limited by each request).
const DEFAULT_MAX_PAGE_SIZE: usize = 10000;

/// A maximum wallclock time spent on iteration.
const MAX_ITERATION_DURATION: Duration = Duration::from_millis(100);

/// A *minimum* number of elements which must be reached even if [`MAX_ITERATION_DURATION`] is
/// exceeded. This only prevents unreasonably small (or empty) pages.
const MIN_PAGE_SIZE_DESPITE_MAX_DURATION: usize = 10;

/// Creates a default paging policy, which:
/// - returns at most [`DEFAULT_MAX_PAGE_SIZE`] items (or `requested_max_page_size` items, if given);
/// - cuts the iteration short if it takes more than [`MAX_ITERATION_DURATION`], but ensures that
///   at least [`MIN_PAGE_SIZE_DESPITE_MAX_DURATION`] items are returned.
///
/// It is initially used by the Browse sub-API.
pub fn default_paging_policy<T>(requested_max_page_size: Option<usize>) -> impl PagingPolicy<T> {
    PagingPolicies::until_first_disallowed(
        PagingPolicies::max_page_size(requested_max_page_size.unwrap_or(DEFAULT_MAX_PAGE_SIZE)),
        PagingPolicies::max_duration_but_min_page_size(
            MAX_ITERATION_DURATION,
            MIN_PAGE_SIZE_DESPITE_MAX_DURATION,
        ),
    )
}
