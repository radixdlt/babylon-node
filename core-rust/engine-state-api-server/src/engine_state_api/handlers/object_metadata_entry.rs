use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::VersionScopingSupport;

pub(crate) async fn handle_object_metadata_entry(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectMetadataEntryRequest>,
) -> Result<Json<models::ObjectMetadataEntryResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let loader = ObjectMetadataLoader::new(&database);
    let metadata_value = loader.load_entry(&node_id, &MetadataKey::from(request.key))?;

    let ledger_state = database.at_ledger_state();

    Ok(Json(models::ObjectMetadataEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &ledger_state,
        )?),
        content: Some(to_api_metadata_value(&mapping_context, metadata_value)?),
    }))
}

fn to_api_metadata_value(
    context: &MappingContext,
    value: MetadataValue,
) -> Result<models::MetadataValue, MappingError> {
    Ok(match value {
        MetadataValue::String(value) => models::MetadataValue::StringMetadataValue { value },
        MetadataValue::Bool(value) => models::MetadataValue::BoolMetadataValue { value },
        MetadataValue::U8(value) => models::MetadataValue::U8MetadataValue {
            value: to_api_u8_as_i32(value),
        },
        MetadataValue::U32(value) => models::MetadataValue::U32MetadataValue {
            value: to_api_u32_as_i64(value),
        },
        MetadataValue::U64(value) => models::MetadataValue::U64MetadataValue {
            value: to_api_u64_as_string(value),
        },
        MetadataValue::I32(value) => models::MetadataValue::I32MetadataValue {
            value: to_api_i32_as_i64(value),
        },
        MetadataValue::I64(value) => models::MetadataValue::I64MetadataValue {
            value: to_api_i64_as_string(value),
        },
        MetadataValue::Decimal(value) => models::MetadataValue::DecimalMetadataValue {
            value: to_api_decimal(&value),
        },
        MetadataValue::GlobalAddress(value) => models::MetadataValue::GlobalAddressMetadataValue {
            value: to_api_global_address(context, &value)?,
        },
        MetadataValue::PublicKey(value) => models::MetadataValue::PublicKeyMetadataValue {
            value: Box::new(to_api_public_key(&value)),
        },
        MetadataValue::NonFungibleGlobalId(value) => {
            models::MetadataValue::NonFungibleGlobalIdMetadataValue {
                value: Box::new(to_api_non_fungible_global_id(context, &value)?),
            }
        }
        MetadataValue::NonFungibleLocalId(value) => {
            models::MetadataValue::NonFungibleLocalIdMetadataValue {
                value: to_api_non_fungible_local_id(context, &value)?,
            }
        }
        MetadataValue::Instant(value) => models::MetadataValue::InstantMetadataValue {
            value: Box::new(to_api_scrypto_instant(&value)?),
        },
        MetadataValue::Url(value) => models::MetadataValue::UrlMetadataValue {
            value: to_api_url(value),
        },
        MetadataValue::Origin(value) => models::MetadataValue::OriginMetadataValue {
            value: to_api_origin(value),
        },
        MetadataValue::PublicKeyHash(value) => models::MetadataValue::PublicKeyHashMetadataValue {
            value: Box::new(to_api_public_key_hash(value)),
        },
        MetadataValue::StringArray(value) => {
            models::MetadataValue::StringArrayMetadataValue { value }
        }
        MetadataValue::BoolArray(value) => models::MetadataValue::BoolArrayMetadataValue { value },
        MetadataValue::U8Array(value) => models::MetadataValue::U8ArrayMetadataValue {
            value: value.into_iter().map(to_api_u8_as_i32).collect(),
        },
        MetadataValue::U32Array(value) => models::MetadataValue::U32ArrayMetadataValue {
            value: value.into_iter().map(to_api_u32_as_i64).collect(),
        },
        MetadataValue::U64Array(value) => models::MetadataValue::U64ArrayMetadataValue {
            value: value.into_iter().map(to_api_u64_as_string).collect(),
        },
        MetadataValue::I32Array(value) => models::MetadataValue::I32ArrayMetadataValue {
            value: value.into_iter().map(to_api_i32_as_i64).collect(),
        },
        MetadataValue::I64Array(value) => models::MetadataValue::I64ArrayMetadataValue {
            value: value.into_iter().map(to_api_i64_as_string).collect(),
        },
        MetadataValue::DecimalArray(value) => models::MetadataValue::DecimalArrayMetadataValue {
            value: value
                .into_iter()
                .map(|value| to_api_decimal(&value))
                .collect(),
        },
        MetadataValue::GlobalAddressArray(value) => {
            models::MetadataValue::GlobalAddressArrayMetadataValue {
                value: value
                    .into_iter()
                    .map(|value| to_api_global_address(context, &value))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        MetadataValue::PublicKeyArray(value) => {
            models::MetadataValue::PublicKeyArrayMetadataValue {
                value: value
                    .into_iter()
                    .map(|value| to_api_public_key(&value))
                    .collect(),
            }
        }
        MetadataValue::NonFungibleGlobalIdArray(value) => {
            models::MetadataValue::NonFungibleGlobalIdArrayMetadataValue {
                value: value
                    .into_iter()
                    .map(|value| to_api_non_fungible_global_id(context, &value))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        MetadataValue::NonFungibleLocalIdArray(value) => {
            models::MetadataValue::NonFungibleLocalIdArrayMetadataValue {
                value: value
                    .into_iter()
                    .map(|value| to_api_non_fungible_local_id(context, &value))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        MetadataValue::InstantArray(value) => models::MetadataValue::InstantArrayMetadataValue {
            value: value
                .into_iter()
                .map(|value| to_api_scrypto_instant(&value))
                .collect::<Result<Vec<_>, _>>()?,
        },
        MetadataValue::UrlArray(value) => models::MetadataValue::UrlArrayMetadataValue {
            value: value.into_iter().map(to_api_url).collect(),
        },
        MetadataValue::OriginArray(value) => models::MetadataValue::OriginArrayMetadataValue {
            value: value.into_iter().map(to_api_origin).collect(),
        },
        MetadataValue::PublicKeyHashArray(value) => {
            models::MetadataValue::PublicKeyHashArrayMetadataValue {
                value: value.into_iter().map(to_api_public_key_hash).collect(),
            }
        }
    })
}
