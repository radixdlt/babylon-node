use crate::prelude::*;

// TODO:MESH Might be nice to have a few mini unit tests here to verify
// that extract_amount and to_mesh_api_amount are opposites and can work
// with e.g. a currency at different number of decimals.
pub(crate) fn extract_amount(
    extraction_context: &ExtractionContext,
    amount: &models::Amount,
) -> Result<(ResourceAddress, Decimal), ExtractionError> {
    let address = ResourceAddress::try_from_bech32(
        &extraction_context.address_decoder,
        &amount.currency.symbol,
    )
    .ok_or(ExtractionError::InvalidAmount(amount.clone()))?;

    let scale = if amount.currency.decimals < 0 || amount.currency.decimals > 18 {
        return Err(ExtractionError::InvalidAmount(amount.clone()));
    } else {
        dec!(10)
            .checked_powi(amount.currency.decimals as i64)
            .unwrap()
    };

    let quantity = Decimal::from_str(&amount.value)
        .ok()
        .and_then(|x| x.checked_div(scale))
        .ok_or(ExtractionError::InvalidAmount(amount.clone()))?;

    Ok((address, quantity))
}

pub(crate) fn extract_amount_from_option(
    extraction_context: &ExtractionContext,
    amount: Option<Box<crate::mesh_api::generated::models::Amount>>,
) -> Result<(ResourceAddress, Decimal), ExtractionError> {
    extract_amount(
        extraction_context,
        amount.ok_or(ExtractionError::NotFound)?.borrow(),
    )
}

pub fn to_mesh_api_amount(
    amount: Decimal,
    currency: models::Currency,
) -> Result<models::Amount, MappingError> {
    let value = amount
        / Decimal::TEN
            .checked_powi((Decimal::SCALE as i32 - currency.decimals) as i64)
            .ok_or_else(|| MappingError::IntegerError {
                message: "Integer overflow".to_string(),
            })?;

    Ok(models::Amount::new(value.attos().to_string(), currency))
}
