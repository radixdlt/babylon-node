use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_access_controller_substate(
    context: &MappingContext,
    substate: &AccessControllerStateFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        AccessControllerFieldState,
        substate => {
            let data = scrypto_encode(substate).unwrap();
        },
        Value {
            data_struct: Box::new(to_api_data_struct_from_bytes(context, &data)?),
        }
    ))
}
