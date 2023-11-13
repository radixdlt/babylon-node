pub(crate) mod lts;

mod browse_entity_info;
mod browse_entity_iterator;
mod browse_kv_store_iterator;
mod browse_object_collection_iterator;
mod browse_object_field;
mod mempool_list;
mod mempool_transaction;
mod state_access_controller;
mod state_account;
mod state_component;
mod state_consensus_manager;
mod state_non_fungible;
mod state_package;
mod state_resource;
mod state_validator;
mod status_network_configuration;
mod status_network_status;
mod status_scenarios;
mod stream_transactions;
mod transaction_callpreview;
mod transaction_parse;
mod transaction_preview;
mod transaction_receipt;
mod transaction_status;
mod transaction_submit;

pub(crate) use browse_entity_info::*;
pub(crate) use browse_entity_iterator::*;
pub(crate) use browse_kv_store_iterator::*;
pub(crate) use browse_object_collection_iterator::*;
pub(crate) use browse_object_field::*;
pub(crate) use mempool_list::*;
pub(crate) use mempool_transaction::*;
pub(crate) use state_access_controller::*;
pub(crate) use state_account::*;
pub(crate) use state_component::*;
pub(crate) use state_consensus_manager::*;
pub(crate) use state_non_fungible::*;
pub(crate) use state_package::*;
pub(crate) use state_resource::*;
pub(crate) use state_validator::*;
pub(crate) use status_network_configuration::*;
pub(crate) use status_network_status::*;
pub(crate) use status_scenarios::*;
pub(crate) use stream_transactions::*;
pub(crate) use transaction_callpreview::*;
pub(crate) use transaction_parse::*;
pub(crate) use transaction_preview::*;
pub(crate) use transaction_receipt::*;
pub(crate) use transaction_status::*;
pub(crate) use transaction_submit::*;

use crate::core_api::{PagingPolicies, PagingPolicy};
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
