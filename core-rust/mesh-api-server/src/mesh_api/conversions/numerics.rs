use crate::prelude::*;

const MAX_API_STATE_VERSION: u64 = 100000000000000;

pub fn to_mesh_api_block_index_from_state_version(
    state_version: StateVersion,
) -> Result<i64, MappingError> {
    let state_version_number = state_version.number();
    if state_version_number > MAX_API_STATE_VERSION {
        return Err(MappingError::IntegerError {
            message: "State version larger than max api state version".to_owned(),
        });
    }
    Ok(state_version_number
        .try_into()
        .expect("State version too large somehow"))
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

pub(crate) fn extract_amount(
    extraction_context: &ExtractionContext,
    amount: &models::Amount,
) -> Result<(ResourceAddress, Decimal), ResponseError> {
    let address = ResourceAddress::try_from_bech32(
        &extraction_context.address_decoder,
        &amount.currency.symbol,
    )
    .ok_or(client_error(
        format!("Invalid resource address: {:?}", amount.currency.symbol),
        false,
    ))?;

    let scale = if amount.currency.decimals < 0 || amount.currency.decimals > 18 {
        return Err(client_error(
            format!("Invalid decimals: {}", amount.currency.decimals),
            false,
        ));
    } else {
        dec!(10)
            .checked_powi(amount.currency.decimals as i64)
            .unwrap()
    };

    let quantity = Decimal::from_str(&amount.value)
        .ok()
        .and_then(|x| x.checked_div(scale))
        .ok_or(client_error(
            format!("Invalid quantity: {:?}", amount.value),
            false,
        ))?;

    Ok((address, quantity))
}

pub(crate) fn extract_amount_from_option(
    extraction_context: &ExtractionContext,
    amount: Option<Box<crate::mesh_api::generated::models::Amount>>,
) -> Result<(ResourceAddress, Decimal), ResponseError> {
    extract_amount(
        extraction_context,
        amount
            .ok_or(client_error("Missing amount", false))?
            .borrow(),
    )
}
