mod blueprint_info;
mod entity_info;
mod entity_iterator;
mod entity_schema_entry;
mod kv_store_entry;
mod kv_store_iterator;
mod object_collection_entry;
mod object_collection_iterator;
mod object_field;

use super::{HasKey, Page, ResponseError};
use radix_engine_common::prelude::ScryptoSbor;

use crate::engine_state_api::{
    extract_api_max_page_size, extract_api_sbor_hex_string, to_api_sbor_hex_string, FnIterable,
    MaxItemCountPolicy, OrderAgnosticPager, Pager,
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

/// A paging support for handlers.
/// This is technically a convenience facade on top of [`Pager`], adding HTTP-level handling of
/// continuation token and page size.
pub struct HandlerPagingSupport {
    max_page_size: Option<i32>,
    continuation_token_string: Option<String>,
}

impl HandlerPagingSupport {
    /// Creates an instance from raw HTTP-level arguments.
    /// Their parsing/validation is deferred until [`Self::get_page()`].
    /// TODO(filter/sort): move validation here + bring in filter (ensure it matches the continuation token!)
    pub fn new(max_page_size: Option<i32>, continuation_token_string: Option<String>) -> Self {
        Self {
            max_page_size,
            continuation_token_string,
        }
    }

    /// Retrieves an appropriate page (i.e. according to the page size and continuation token passed
    /// during construction) from the given collection lister (see [`FnIterable::wrap()`]).
    pub fn get_page<K: PartialEq + Clone + ScryptoSbor, T: HasKey<K>, I: Iterator<Item = T>, E>(
        self,
        iterable: impl FnOnce(Option<&K>) -> Result<I, E>,
    ) -> Result<HandlerPage<T>, ResponseError>
    where
        ResponseError: From<E>,
    {
        let max_page_size = extract_api_max_page_size(self.max_page_size)
            .map_err(|error| error.into_response_error("max_page_size"))?;

        let continuation_token = self
            .continuation_token_string
            .as_ref()
            .map(extract_api_sbor_hex_string)
            .transpose()
            .map_err(|error| error.into_response_error("continuation_token"))?;

        let Page {
            items,
            continuation_token,
        } = OrderAgnosticPager::get_page(
            FnIterable::wrap(iterable),
            MaxItemCountPolicy::new(max_page_size),
            continuation_token,
        )?;

        let continuation_token_string = continuation_token
            .map(|continuation_token| to_api_sbor_hex_string(&continuation_token))
            .transpose()?;

        Ok(HandlerPage {
            items,
            continuation_token_string,
        })
    }
}

/// A [`Page`] with an already-rendered [`ContinuationToken`].
pub struct HandlerPage<T> {
    /// Items on this page.
    pub items: Vec<T>,
    /// The next continuation token, rendered as a string (only present if there are more pages
    /// after this one).
    pub continuation_token_string: Option<String>,
}
