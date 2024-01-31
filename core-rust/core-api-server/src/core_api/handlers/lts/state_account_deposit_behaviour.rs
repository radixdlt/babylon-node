use crate::core_api::*;
use radix_engine::blueprints::account::{
    AccountAuthorizedDepositorEntryPayload, AccountCollection, AccountDepositRuleFieldSubstate,
    AccountField, AccountResourcePreference, AccountResourcePreferenceEntryPayload,
    AccountResourcePreferenceV1, AccountResourceVaultEntryPayload,
};
use radix_engine::types::*;

use state_manager::LedgerHeader;
use std::ops::Deref;

use node_common::utils::IsAccountExt;
use radix_engine_interface::blueprints::account::{DefaultDepositRule, ResourcePreference};

/// Maximum number of resource addresses allowed in the request.
/// Must be aligned with the `maxItems` in the API documentation.
const MAX_RESOURCES: usize = 20;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_state_account_deposit_behaviour(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStateAccountDepositBehaviourRequest>,
) -> Result<Json<models::LtsStateAccountDepositBehaviourResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    // We expect an address of a global account:
    let account_address = extract_global_address(&extraction_context, &request.account_address)
        .map_err(|err| err.into_response_error("account_address"))?;
    if !account_address.is_account() {
        return Err(client_error("Not an account address"));
    }

    // We expect at most `MAX_RESOURCES` resource addresses:
    let requested_resource_addresses = request.resource_addresses;
    let resource_addresses = requested_resource_addresses
        .iter()
        .flatten()
        .map(|string| extract_resource_address(&extraction_context, string))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.into_response_error("resource_addresses"))?;
    if resource_addresses.len() > MAX_RESOURCES {
        return Err(client_error("Too many resource addresses"));
    }

    // Parse the optional presented depositor badge:
    let badge = request
        .badge
        .map(|badge| extract_resource_or_non_fungible(&extraction_context, badge.as_ref()))
        .transpose()
        .map_err(|err| err.into_response_error("badge"))?;

    // If the above checks were al fine, open database (and capture the "at state" information):
    let database = state.state_manager.database.snapshot();
    let header = read_current_ledger_header(database.deref());

    // Read out the field that must exist for non-virtual addresses:
    let default_deposit_rule = read_optional_substate::<AccountDepositRuleFieldSubstate>(
        database.deref(),
        account_address.as_node_id(),
        AccountPartitionOffset::Field.as_main_partition(),
        &AccountField::DepositRule.into(),
    )
    .map(|substate| substate.into_payload().into_latest().default_deposit_rule);

    // If it does not exist, then either it is an empty virtual account, or a bad account address:
    let Some(default_deposit_rule) = default_deposit_rule else {
        return if account_address.as_node_id().is_global_virtual() {
            Ok(empty_virtual_account_response(
                &mapping_context,
                &header,
                badge,
                requested_resource_addresses.map(|requested_resource_addresses| {
                    requested_resource_addresses
                        .into_iter()
                        .zip(resource_addresses)
                        .collect()
                }),
            )?)
        } else {
            Err(not_found_error("Account not found".to_string()))
        };
    };

    // Read out the badge status (`None` when not provided, else `Some<is on AD list?>`):
    let is_badge_authorized_depositor = badge.map(|badge| {
        read_optional_collection_substate_value::<AccountAuthorizedDepositorEntryPayload>(
            database.deref(),
            account_address.as_node_id(),
            AccountCollection::AuthorizedDepositorKeyValue.collection_index(),
            &SubstateKey::Map(scrypto_encode(&badge).unwrap()),
        )
        .is_some()
    });

    let resource_specific_behaviours = resource_addresses
        .iter()
        .map(|resource_address| {
            // Gather inputs to the deposit rules:
            let resource_address_substate_key =
                SubstateKey::Map(scrypto_encode(resource_address).unwrap());
            let resource_preference =
                read_optional_collection_substate_value::<AccountResourcePreferenceEntryPayload>(
                    database.deref(),
                    account_address.as_node_id(),
                    AccountCollection::ResourcePreferenceKeyValue.collection_index(),
                    &resource_address_substate_key,
                )
                .map(|payload| payload.into_latest());
            let vault_exists =
                read_optional_collection_substate_value::<AccountResourceVaultEntryPayload>(
                    database.deref(),
                    account_address.as_node_id(),
                    AccountCollection::ResourceVaultKeyValue.collection_index(),
                    &resource_address_substate_key,
                )
                .is_some();
            let is_xrd = resource_address == &XRD;

            // Compose a response containing the inputs and the resolution:
            let allows_try_deposit = allows_deposit(
                &default_deposit_rule,
                &is_badge_authorized_depositor,
                &resource_preference,
                vault_exists,
                is_xrd,
            );
            Ok(models::ResourceSpecificDepositBehaviour {
                resource_preference: resource_preference.map(to_api_resource_preference),
                vault_exists,
                is_xrd,
                allows_try_deposit,
            })
        })
        .collect::<Result<Vec<_>, MappingError>>()?;

    response(
        &mapping_context,
        &header,
        &default_deposit_rule,
        is_badge_authorized_depositor,
        requested_resource_addresses.map(|resource_addresses| {
            resource_addresses
                .into_iter()
                .zip(resource_specific_behaviours)
                .collect::<IndexMap<_, _>>()
        }),
    )
}

fn to_api_resource_preference(preference: ResourcePreference) -> models::ResourcePreference {
    match preference {
        AccountResourcePreferenceV1::Allowed => models::ResourcePreference::Allowed,
        AccountResourcePreferenceV1::Disallowed => models::ResourcePreference::Disallowed,
    }
}

fn empty_virtual_account_response(
    context: &MappingContext,
    header: &LedgerHeader,
    badge: Option<ResourceOrNonFungible>,
    requested_resource_addresses: Option<Vec<(String, ResourceAddress)>>,
) -> Result<Json<models::LtsStateAccountDepositBehaviourResponse>, ResponseError<()>> {
    response(
        context,
        header,
        &DefaultDepositRule::Accept, // that's how Engine creates new accounts
        badge.map(|_| false),        // regardless what badge - it is not on the AD list
        requested_resource_addresses.map(|requested_resource_addresses| {
            requested_resource_addresses
                .into_iter()
                .map(|(requested_resource_address, resource_address)| {
                    (
                        requested_resource_address,
                        empty_virtual_account_resource_specific_behaviour(resource_address),
                    )
                })
                .collect()
        }),
    )
}

fn empty_virtual_account_resource_specific_behaviour(
    resource_address: ResourceAddress,
) -> models::ResourceSpecificDepositBehaviour {
    models::ResourceSpecificDepositBehaviour {
        resource_preference: None,
        vault_exists: false,
        is_xrd: resource_address == XRD,
        allows_try_deposit: true,
    }
}

fn response(
    context: &MappingContext,
    header: &LedgerHeader,
    default_deposit_rule: &DefaultDepositRule,
    is_badge_authorized_depositor: Option<bool>,
    resource_specific_behaviours: Option<
        IndexMap<String, models::ResourceSpecificDepositBehaviour>,
    >,
) -> Result<Json<models::LtsStateAccountDepositBehaviourResponse>, ResponseError<()>> {
    Ok(Json(models::LtsStateAccountDepositBehaviourResponse {
        state_version: to_api_state_version(header.state_version)?,
        ledger_header_summary: Box::new(to_api_ledger_header_summary(context, header)?),
        default_deposit_rule: match default_deposit_rule {
            DefaultDepositRule::Accept => models::DefaultDepositRule::Accept,
            DefaultDepositRule::Reject => models::DefaultDepositRule::Reject,
            DefaultDepositRule::AllowExisting => models::DefaultDepositRule::AllowExisting,
        },
        is_badge_authorized_depositor,
        resource_specific_behaviours,
    }))
}

/// Resolves whether the deposit is allowed, based on raw inputs.
/// See https://docs-babylon.radixdlt.com/main/scrypto/native-blueprints/accounts.html for a
/// flow chart.
fn allows_deposit(
    default_deposit_rule: &DefaultDepositRule,
    is_badge_authorized_depositor: &Option<bool>,
    resource_preference: &Option<AccountResourcePreference>,
    vault_exists: bool,
    is_xrd: bool,
) -> bool {
    // A presented, valid authorized depositor badge can override all other account settings:
    if is_badge_authorized_depositor == &Some(true) {
        return true;
    }
    // If not presented (or not in the AD list), then a resource preference decides:
    if let Some(resource_preference) = resource_preference {
        return match resource_preference {
            AccountResourcePreference::Allowed => true,
            AccountResourcePreference::Disallowed => false,
        };
    }
    // If the preference is not configured for the given resource, then the default rule decides:
    match default_deposit_rule {
        DefaultDepositRule::Accept => true,
        DefaultDepositRule::Reject => false,
        // The extra case of the default rule, allowing existing vaults *or* XRD:
        DefaultDepositRule::AllowExisting => vault_exists || is_xrd,
    }
}
