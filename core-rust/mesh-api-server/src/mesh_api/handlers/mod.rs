use super::{HasKey, Page, ResponseError};
use crate::engine_prelude::*;

mod account_balance;
mod network_list;
mod network_options;
mod network_status;

pub(crate) use account_balance::*;
pub(crate) use network_list::*;
pub(crate) use network_options::*;
pub(crate) use network_status::*;

use crate::mesh_api::{
    extract_from_sbor_hex_string, extract_max_page_size, to_api_sbor_hex_string, ExtractionError,
    MaxItemCountPolicy, NextKeyPager,
};

/// A paging support for handlers.
/// This is technically a convenience facade on top of [`NextKeyPager`], adding HTTP-level handling of
/// continuation token and page size.
pub struct HandlerPagingSupport {
    max_page_size: Option<i32>,
    requested_continuation_token_string: Option<String>,
    requested_filter_hash: Hash,
}

impl HandlerPagingSupport {
    /// A convenience [`Self::new_with_serialized_filter`] adapter for a filter directly coming from
    /// a `serde`-serialized request field.
    pub fn new_with_serde_filter(
        max_page_size: Option<i32>,
        continuation_token: Option<String>,
        filter: &impl serde::Serialize,
    ) -> Self {
        Self::new_with_serialized_filter(
            max_page_size,
            continuation_token,
            serde_json::to_vec(filter).expect("cannot encode serde"),
        )
    }

    /// A convenience [`Self::new_with_serialized_filter`] adapter for a filter constructed by the
    /// endpoint logic (e.g. via composing a few de-facto-filtering request fields).
    pub fn new_with_sbor_filter(
        max_page_size: Option<i32>,
        continuation_token: Option<String>,
        filter: &impl ScryptoEncode,
    ) -> Self {
        Self::new_with_serialized_filter(
            max_page_size,
            continuation_token,
            scrypto_encode(filter).expect("cannot encode SBOR"),
        )
    }

    /// A convenience [`Self::new_with_serialized_filter`] adapter for endpoints without filters.
    pub fn new_without_filter(
        max_page_size: Option<i32>,
        continuation_token: Option<String>,
    ) -> Self {
        Self::new_with_serialized_filter(max_page_size, continuation_token, vec![])
    }

    /// Creates an instance from HTTP-level arguments.
    /// The parsing/validation of `max_page_size` and `continuation_token` is deferred until
    /// [`Self::get_page()`], since only then the required item/key types are known.
    /// The opaque `serialized_filter` is used only to ensure it was not changed from the previous
    /// page - we expect an arbitrary (serializable) structure coming from the request (possibly
    /// empty, if the endpoint does not support filtering).
    pub fn new_with_serialized_filter(
        max_page_size: Option<i32>,
        continuation_token: Option<String>,
        serialized_filter: impl AsRef<[u8]>,
    ) -> Self {
        Self {
            max_page_size,
            requested_continuation_token_string: continuation_token,
            requested_filter_hash: hash(serialized_filter),
        }
    }

    /// Retrieves an appropriate page (i.e. according to the page size and continuation token passed
    /// during construction) from the given collection lister.
    pub fn get_page<K: ScryptoSbor, T: HasKey<K>, I: Iterator<Item = T>, E: Into<ResponseError>>(
        self,
        iterable: impl FnOnce(Option<&K>) -> Result<I, E>,
    ) -> Result<Page<T, String>, ResponseError> {
        let paging_policy = MaxItemCountPolicy::new(
            extract_max_page_size(self.max_page_size)
                .map_err(|error| error.into_response_error("max_page_size"))?,
        );

        let requested_continuation_token = self
            .requested_continuation_token_string
            .as_ref()
            .map(extract_from_sbor_hex_string::<NextKeyAndFilterHash<K>>)
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
