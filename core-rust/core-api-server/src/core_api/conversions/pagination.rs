use crate::prelude::*;

pub struct SizeRange {
    pub min: usize,
    pub default: usize,
    pub max: usize,
}

// Extracts a valid page size as per the API consistency guidelines.
pub fn extract_valid_size(
    provided: Option<i32>,
    size_range: SizeRange,
) -> Result<usize, ExtractionError> {
    let SizeRange { min, default, max } = size_range;
    let valid_size = match provided {
        Some(provided) => {
            if provided < 0 {
                return Err(ExtractionError::InvalidSize { min, max });
            }
            let provided = provided as usize;
            if provided < min || provided > max {
                return Err(ExtractionError::InvalidSize { min, max });
            } else {
                provided
            }
        }
        None => default,
    };
    Ok(valid_size)
}

/// Extracts a valid optional continuation token as per the API consistency guidelines.
///
/// Whilst normally such methods would not take/return an option, this is an active choice
/// to encourage/simplify following of the API guidelines, where a `continuation_token` is always
/// optional.
pub fn extract_continuation_token<T: ScryptoDecode>(
    continuation_token: Option<String>,
) -> Result<Option<T>, ExtractionError> {
    let Some(continuation_token) = continuation_token else {
        return Ok(None);
    };
    let bytes = from_hex(continuation_token)?;
    let id = scrypto_decode::<T>(&bytes).map_err(ExtractionError::InvalidContinuationToken)?;
    Ok(Some(id))
}

/// Maps a valid optional continuation token as per the API consistency guidelines.
pub fn to_api_continuation_token<T: ScryptoEncode>(
    id_of_start_of_next_page: Option<&T>,
) -> Option<String> {
    id_of_start_of_next_page.map(|id| to_hex(scrypto_encode(id).unwrap()))
}

pub fn optional_max<T: Ord>(value: T, option: Option<T>) -> T {
    match option {
        Some(value_2) => value.max(value_2),
        None => value,
    }
}

/// Returns a page and a continuation token from an iterator.
///
/// This follows the API consistency guidelines, although NOTE that it encodes the continuation token
/// as the id of the item at the *start of the next page*.
pub fn to_api_page<Item, ItemModel, ContinuationToken: ScryptoEncode>(
    iter: &mut dyn Iterator<Item = Item>,
    page_size: usize,
    to_api_item_model: impl Fn(Item) -> Result<ItemModel, MappingError>,
    to_id_for_continuation_token: impl Fn(&Item) -> ContinuationToken,
) -> Result<(Vec<ItemModel>, Option<String>), MappingError> {
    let (page, id_of_start_of_next_page) = iter
        .take(page_size + 1)
        .enumerate()
        .try_fold::<_, _, Result<_, MappingError>>(
            (Vec::with_capacity(page_size), None),
            |(mut page, mut id_of_start_of_next_page), (index, item)| {
                if index < page_size {
                    page.push(to_api_item_model(item)?);
                } else {
                    id_of_start_of_next_page = Some(to_id_for_continuation_token(&item));
                }
                Ok((page, id_of_start_of_next_page))
            },
        )?;
    Ok((
        page,
        to_api_continuation_token(id_of_start_of_next_page.as_ref()),
    ))
}
