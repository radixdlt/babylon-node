use crate::core_api::models::*;
use crate::core_api::*;
use radix_engine::{
    fee::FeeSummary as EngineFeeSummary,
    ledger::{OutputId, OutputValue},
    state_manager::VirtualSubstateId,
    types::{hash, scrypto_encode, ComponentAddress, PackageAddress, ResourceAddress, SubstateId},
};
use scrypto::address::Bech32Encoder;
use state_manager::{CommittedTransactionStatus, LedgerTransactionReceipt};

pub fn to_api_receipt(
    bech32_encoder: &Bech32Encoder,
    receipt: LedgerTransactionReceipt,
) -> Result<TransactionReceipt, MappingError> {
    let fee_summary = receipt.fee_summary;
    let entity_changes = receipt.entity_changes;

    let (status, output, error_message) = match receipt.status {
        CommittedTransactionStatus::Success(output) => {
            let output_hex: Vec<String> = output.into_iter().map(to_hex).collect();
            (TransactionStatus::Succeeded, Some(output_hex), None)
        }
        CommittedTransactionStatus::Failure(error) => {
            (TransactionStatus::Failed, None, Some(error))
        }
    };

    let state_updates = receipt.state_updates;

    let up_substates = state_updates
        .up_substates
        .into_iter()
        .map(|substate_kv| to_api_up_substate(bech32_encoder, substate_kv))
        .collect::<Result<Vec<_>, _>>()?;

    let down_substates = state_updates
        .down_substates
        .into_iter()
        .map(to_api_down_substate)
        .collect::<Result<Vec<_>, _>>()?;

    let down_virtual_substates = state_updates
        .down_virtual_substates
        .into_iter()
        .map(to_api_down_virtual_substate)
        .collect::<Result<Vec<_>, _>>()?;

    // These should be entity ids, not substate ids
    let new_global_entities = state_updates
        .new_roots
        .into_iter()
        .map(|x| to_api_global_entity_id(bech32_encoder, x))
        .collect::<Result<Vec<_>, _>>()?;

    let api_state_updates = StateUpdates {
        up_substates,
        down_substates,
        down_virtual_substates,
        new_global_entities,
    };

    let api_fee_summary = to_api_fee_summary(fee_summary);

    Ok(TransactionReceipt {
        status,
        fee_summary: Box::new(api_fee_summary),
        state_updates: Box::new(api_state_updates),
        new_package_addresses: entity_changes
            .new_package_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_package_address(&v))
            .collect(),
        new_component_addresses: entity_changes
            .new_component_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_component_address(&v))
            .collect(),
        new_resource_addresses: entity_changes
            .new_resource_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_resource_address(&v))
            .collect(),
        output,
        error_message,
    })
}

pub fn to_api_up_substate(
    bech32_encoder: &Bech32Encoder,
    (substate_id, output_value): (SubstateId, OutputValue),
) -> Result<UpSubstate, MappingError> {
    let substate_bytes = scrypto_encode(&output_value.substate);
    let hash = to_hex(hash(&substate_bytes));
    Ok(UpSubstate {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        version: output_value.version.to_string(),
        substate_bytes: to_hex(substate_bytes),
        substate_data_hash: hash,
        substate_data: Option::Some(to_api_substate(&output_value.substate, bech32_encoder)?),
    })
}

pub fn to_api_down_substate(output_id: OutputId) -> Result<DownSubstate, MappingError> {
    Ok(DownSubstate {
        substate_id: Box::new(to_api_substate_id(output_id.substate_id)?),
        substate_data_hash: to_hex(output_id.substate_hash),
        version: output_id
            .version
            .try_into()
            .map_err(|err| MappingError::Integer {
                message: "Substate version could not be mapped to i32".to_owned(),
                error: err,
            })?,
    })
}

pub fn to_api_down_virtual_substate(
    VirtualSubstateId(root_substate_id, key): VirtualSubstateId,
) -> Result<models::SubstateId, MappingError> {
    to_api_virtual_substate_id(root_substate_id, key)
}

pub const FAKED_SYSTEM_ADDRESS: ComponentAddress = ComponentAddress::System([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
]);

pub fn to_api_global_entity_id(
    bech32_encoder: &Bech32Encoder,
    substate_id: SubstateId,
) -> Result<models::GlobalEntityId, MappingError> {
    let mapped = to_mapped_substate_id(substate_id)?;
    let entity_type = mapped.0;
    let address_bytes = mapped.1;
    let address_bytes_hex = to_hex(&address_bytes);

    let global_address_str = match entity_type {
        entity_type::EntityType::System => bech32_encoder.encode_component_address(
            &ComponentAddress::try_from(address_bytes.as_slice()).unwrap(),
        ),
        entity_type::EntityType::ResourceManager => bech32_encoder
            .encode_resource_address(&ResourceAddress::try_from(address_bytes.as_slice()).unwrap()),
        entity_type::EntityType::Component => bech32_encoder.encode_component_address(
            &ComponentAddress::try_from(address_bytes.as_slice()).unwrap(),
        ),
        entity_type::EntityType::Package => bech32_encoder
            .encode_package_address(&PackageAddress::try_from(address_bytes.as_slice()).unwrap()),
        entity_type::EntityType::Vault => Err(MappingError::InvalidRootEntity {
            message: "Vault".to_owned(),
        })?,
        entity_type::EntityType::KeyValueStore => Err(MappingError::InvalidRootEntity {
            message: "KeyValueStore".to_owned(),
        })?,
    };

    Ok(models::GlobalEntityId {
        entity_type: mapped.0,
        entity_address: address_bytes_hex.clone(),
        global_address_bytes: address_bytes_hex,
        global_address_str,
    })
}

pub fn to_api_substate_id(substate_id: SubstateId) -> Result<models::SubstateId, MappingError> {
    let mapped = to_mapped_substate_id(substate_id)?;

    Ok(models::SubstateId {
        entity_type: mapped.0,
        entity_address: to_hex(mapped.1),
        substate_type: mapped.2,
        substate_key: to_hex(mapped.3),
    })
}

type EntityType = models::EntityType;
type SubstateType = models::SubstateType;
struct MappedSubstateId(EntityType, Vec<u8>, SubstateType, Vec<u8>);

fn to_mapped_substate_id(substate_id: SubstateId) -> Result<MappedSubstateId, MappingError> {
    // It's crucial that we ensure all Entity Addresses are unique
    // It's crucial that we ensure all Substate keys are locally unique
    // NOTE: If you add any transient root spaces here, ensure they're added to to_api_virtual_substate_id
    Ok(match substate_id {
        // SYSTEM SUBSTATES
        SubstateId::System => MappedSubstateId(
            EntityType::System,
            FAKED_SYSTEM_ADDRESS.to_vec(),
            SubstateType::System,
            vec![0],
        ),
        // COMPONENT SUBSTATES
        SubstateId::ComponentInfo(addr) => MappedSubstateId(
            EntityType::Component,
            addr.to_vec(),
            SubstateType::ComponentInfo,
            vec![0],
        ),
        SubstateId::ComponentState(addr) => MappedSubstateId(
            EntityType::Component,
            addr.to_vec(),
            SubstateType::ComponentState,
            vec![1],
        ),
        // PACKAGE SUBSTATES
        SubstateId::Package(addr) => MappedSubstateId(
            EntityType::Package,
            addr.to_vec(),
            SubstateType::Package,
            vec![0],
        ),
        // RESOURCE SUBSTATES
        SubstateId::ResourceManager(addr) => MappedSubstateId(
            EntityType::ResourceManager,
            addr.to_vec(),
            SubstateType::ResourceManager,
            vec![0],
        ),
        SubstateId::NonFungibleSpace(_) => Err(MappingError::VirtualRootSubstatePersisted {
            message: "No state_update known/possible for NonFungibleSpace".to_owned(),
        })?,
        SubstateId::NonFungible(addr, id) => MappedSubstateId(
            EntityType::ResourceManager,
            addr.to_vec(),
            SubstateType::NonFungible,
            prefix(vec![2], id.0),
        ),
        // KEY VALUE STORE SUBSTATES
        SubstateId::KeyValueStoreSpace(_) => Err(MappingError::VirtualRootSubstatePersisted {
            message: "No state_update known/possible for KeyValueStoreSpace".to_owned(),
        })?,
        SubstateId::KeyValueStoreEntry(basic_address, key) => MappedSubstateId(
            EntityType::KeyValueStore,
            basic_address_to_vec(basic_address),
            SubstateType::KeyValueStoreEntry,
            prefix(vec![1], key),
        ),
        // VAULT SUBSTATES
        SubstateId::Vault(basic_address) => MappedSubstateId(
            EntityType::Vault,
            basic_address_to_vec(basic_address),
            SubstateType::Vault,
            vec![0],
        ),
        // TRANSIENT? SUBSTATES
        SubstateId::Bucket(_) => Err(MappingError::TransientSubstatePersisted {
            message: "Proof persisted".to_owned(),
        })?,
        SubstateId::Proof(_) => Err(MappingError::TransientSubstatePersisted {
            message: "Bucket persisted".to_owned(),
        })?,
        SubstateId::Worktop => Err(MappingError::TransientSubstatePersisted {
            message: "Worktop persisted".to_owned(),
        })?,
    })
}

pub fn to_api_virtual_substate_id(
    root_substate_id: SubstateId,
    key: Vec<u8>,
) -> Result<models::SubstateId, MappingError> {
    // These should match the ids of the keys
    let sub_id = match root_substate_id {
        // NonFungibleSpace key is downed to create a NonFungible
        SubstateId::NonFungibleSpace(addr) => MappedSubstateId(
            EntityType::ResourceManager,
            addr.to_vec(),
            SubstateType::NonFungible,
            prefix(vec![2], key),
        ),
        // KeyValueStoreSpace key is downed to create a KeyValueStoreEntry
        SubstateId::KeyValueStoreSpace(basic_address) => MappedSubstateId(
            EntityType::KeyValueStore,
            basic_address_to_vec(basic_address),
            SubstateType::KeyValueStoreEntry,
            prefix(vec![1], key),
        ),
        // Assume all other substates are not root spaces
        other => Err(MappingError::VirtualSubstateDownedWithInvalidParent {
            message: format!("{:?}", other),
        })?,
    };
    Ok(models::SubstateId {
        entity_type: sub_id.0,
        entity_address: to_hex(sub_id.1),
        substate_type: sub_id.2,
        substate_key: to_hex(sub_id.3),
    })
}

// NB - see id_allocator.rs - addresses are formed from (tx_hash, index_in_tx_for_exec_mode + offset_for_exec_mode)
// There is a separate exec_mode for the manifest and the standard Application executor
fn basic_address_to_vec(basic_address: (scrypto::crypto::Hash, u32)) -> Vec<u8> {
    // NOTE - this only works because basic_address is of fixed length.
    // If basic_address became variable length, we'd need to do something else (eg sbor encode) to ensure a 1:1 mapping here

    prefix(
        basic_address.0.to_vec(),
        basic_address.1.to_le_bytes().to_vec(),
    )
}

fn prefix(mut prefix: Vec<u8>, mut suffix: Vec<u8>) -> Vec<u8> {
    prefix.append(&mut suffix);
    prefix
}

pub fn to_api_fee_summary(fee_summary: EngineFeeSummary) -> FeeSummary {
    FeeSummary {
        loan_fully_repaid: fee_summary.loan_fully_repaid,
        cost_unit_limit: fee_summary.cost_unit_limit.to_string(),
        cost_unit_consumed: fee_summary.cost_unit_consumed.to_string(),
        cost_unit_price: fee_summary.cost_unit_price.to_string(),
        tip_percentage: fee_summary.tip_percentage.to_string(),
        xrd_burned: fee_summary.burned.to_string(),
        xrd_tipped: fee_summary.tipped.to_string(),
    }
}
