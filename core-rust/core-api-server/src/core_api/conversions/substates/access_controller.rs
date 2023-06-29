use super::*;
use super::super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_access_controller_substate(
    context: &MappingContext,
    substate: &AccessControllerSubstate,
) -> Result<models::Substate, MappingError> {
    let data = scrypto_encode(substate).unwrap();

    Ok(field_substate!(
        substate,
        AccessControllerFieldState,
        {
            data_struct: Box::new(to_api_data_struct_from_bytes(context, &data)?),
        }
    ))
}