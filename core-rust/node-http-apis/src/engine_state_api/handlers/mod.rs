mod blueprint_info;
mod entity_info;
mod entity_iterator;
mod entity_schema_entry;
mod kv_store_entry;
mod kv_store_iterator;
mod object_collection_entry;
mod object_collection_iterator;
mod object_field;
mod object_metadata_iterator;

use super::{HasKey, Page, ResponseError};
use radix_engine_common::crypto::Hash;
use radix_engine_common::prelude::{hash, ScryptoSbor};

use crate::engine_state_api::{
    extract_api_max_page_size, extract_api_sbor_hex_string, to_api_sbor_hex_string,
    ExtractionError, MaxItemCountPolicy, NextKeyPager,
};
pub(crate) use blueprint_info::*;
pub(crate) use entity_info::*;
pub(crate) use entity_iterator::*;
pub(crate) use entity_schema_entry::*;
pub(crate) use kv_store_entry::*;
pub(crate) use kv_store_iterator::*;
pub(crate) use object_collection_entry::*;
pub(crate) use object_collection_iterator::*;
pub(crate) use object_field::*;
pub(crate) use object_metadata_iterator::*;

/// A paging support for handlers.
/// This is technically a convenience facade on top of [`NextKeyPager`], adding HTTP-level handling of
/// continuation token and page size.
pub struct HandlerPagingSupport {
    max_page_size: Option<i32>,
    requested_continuation_token_string: Option<String>,
    requested_filter_hash: Hash,
}

impl HandlerPagingSupport {
    /// Creates an instance from raw HTTP-level arguments.
    /// Their parsing/validation of `max_page_size` and `continuation_token` is deferred until
    /// [`Self::get_page()`], since only then the required item/key types are known.
    /// The `filter` is used only to ensure it was not changed from previous page - we expect an
    /// arbitrary (serializable) structure coming from the request (may be `None` if the endpoint
    /// does not support filtering).
    pub fn new<F: serde::Serialize>(
        max_page_size: Option<i32>,
        continuation_token: Option<String>,
        filter: &F,
    ) -> Self {
        Self {
            max_page_size,
            requested_continuation_token_string: continuation_token,
            requested_filter_hash: hash(serde_json::to_vec(filter).expect("it was serialized")),
        }
    }

    /// Retrieves an appropriate page (i.e. according to the page size and continuation token passed
    /// during construction) from the given collection lister.
    pub fn get_page<K: ScryptoSbor, T: HasKey<K>, I: Iterator<Item = T>, E: Into<ResponseError>>(
        self,
        iterable: impl FnOnce(Option<&K>) -> Result<I, E>,
    ) -> Result<Page<T, String>, ResponseError> {
        let paging_policy = MaxItemCountPolicy::new(
            extract_api_max_page_size(self.max_page_size)
                .map_err(|error| error.into_response_error("max_page_size"))?,
        );

        let requested_continuation_token = self
            .requested_continuation_token_string
            .as_ref()
            .map(extract_api_sbor_hex_string::<NextKeyAndFilterHash<K>>)
            .transpose()
            .map_err(|error| error.into_response_error("continuation_token"))?;

        let requested_next_key = requested_continuation_token
            .map(|continuation_token| {
                let NextKeyAndFilterHash {
                    next_key,
                    filter_hash,
                } = continuation_token;
                if self.requested_filter_hash == filter_hash {
                    Ok(next_key)
                } else {
                    Err(ExtractionError::DifferentFilterAcrossPages.into_response_error("filter"))
                }
            })
            .transpose()?;

        let iterator = iterable(requested_next_key.as_ref()).map_err(|error| error.into())?;
        let page = NextKeyPager::get_page(iterator, paging_policy);

        let continuation_token_string = page
            .continuation_token
            .map(|next_key| {
                to_api_sbor_hex_string(&NextKeyAndFilterHash {
                    next_key,
                    filter_hash: self.requested_filter_hash,
                })
            })
            .transpose()?;

        Ok(Page {
            items: page.items,
            continuation_token: continuation_token_string,
        })
    }
}

/// An internal "continuation token" structure.
#[derive(ScryptoSbor)]
struct NextKeyAndFilterHash<K> {
    next_key: K,
    filter_hash: Hash,
}
