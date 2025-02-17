use crate::prelude::*;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_state_consensus_manager(
    state: State<CoreApiState>,
    Json(request): Json<models::StateConsensusManagerRequest>,
) -> Result<Json<models::StateConsensusManagerResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let database = state.state_manager.database.snapshot();

    let config_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::Configuration.into(),
    )?;
    let state_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::State.into(),
    )?;
    let current_proposal_statistic_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::CurrentProposalStatistic.into(),
    )?;
    let current_validator_set_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::CurrentValidatorSet.into(),
    )?;
    let current_time_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::ProposerMilliTimestamp.into(),
    )?;
    let current_time_round_to_minutes_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::ProposerMinuteTimestamp.into(),
    )?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::StateConsensusManagerResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &header.into(),
        )?),
        config: Some(to_api_consensus_manager_config_substate(&config_substate)?),
        state: Some(to_api_consensus_manager_state_substate(
            &mapping_context,
            &state_substate,
        )?),
        current_proposal_statistic: Some(to_api_current_proposal_statistic_substate(
            &mapping_context,
            &current_proposal_statistic_substate,
        )?),
        current_validator_set: Some(to_api_current_validator_set_substate(
            &mapping_context,
            &current_validator_set_substate,
        )?),
        current_time: Some(to_api_current_time_substate(&current_time_substate)?),
        current_time_rounded_to_minutes: Some(to_api_current_time_rounded_to_minutes_substate(
            &current_time_round_to_minutes_substate,
        )?),
        current_validator_readiness_signals: request
            .include_readiness_signals
            .filter(|requested| *requested)
            .map(|_| {
                collect_current_validators_by_signalled_protocol_version(
                    database.deref(),
                    current_validator_set_substate,
                )
            })
            .transpose()?
            .map(|current_validators| {
                to_api_current_validator_readiness_signals(&mapping_context, &current_validators)
            })
            .transpose()?,
    }))
}

fn collect_current_validators_by_signalled_protocol_version(
    database: &StateManagerDatabase<impl ReadableRocks>,
    substate: ConsensusManagerCurrentValidatorSetFieldSubstate,
) -> Result<ValidatorsBySignalledProtocolVersion, ResponseError<()>> {
    let mut validators = ValidatorsBySignalledProtocolVersion::default();
    let payload = substate
        .into_payload()
        .fully_update_and_into_latest_version();
    for (index, entry) in payload
        .validator_set
        .validators_by_stake_desc
        .into_iter()
        .enumerate()
    {
        let (validator_address, Validator { stake, .. }) = entry;
        let protocol_version_name = read_mandatory_main_field_substate::<
            ValidatorProtocolUpdateReadinessSignalFieldPayload,
        >(
            database,
            validator_address.as_node_id(),
            &ValidatorField::ProtocolUpdateReadinessSignal.into(),
        )?
        .into_payload()
        .fully_update_and_into_latest_version()
        .protocol_version_name;
        validators.insert(
            ValidatorIndex::try_from(index).expect("validator set size guarantees this"),
            stake,
            protocol_version_name.map(ProtocolVersionName::of_unchecked),
        );
    }
    Ok(validators)
}

#[derive(Default)]
struct ValidatorsBySignalledProtocolVersion {
    total_stake: Decimal,
    versions_to_signalling_validators: IndexMap<Option<ProtocolVersionName>, SignallingValidators>,
}

impl ValidatorsBySignalledProtocolVersion {
    pub fn insert(
        &mut self,
        index: ValidatorIndex,
        stake: Decimal,
        version: Option<ProtocolVersionName>,
    ) {
        self.total_stake = self.total_stake.add_or_panic(stake);
        self.versions_to_signalling_validators
            .entry(version)
            .or_default()
            .insert(index, stake);
    }
}

#[derive(Default)]
struct SignallingValidators {
    total_stake: Decimal,
    indices_and_stakes: Vec<(ValidatorIndex, Decimal)>,
}

impl SignallingValidators {
    pub fn insert(&mut self, index: ValidatorIndex, stake: Decimal) {
        self.total_stake = self.total_stake.add_or_panic(stake);
        self.indices_and_stakes.push((index, stake));
    }
}

fn to_api_current_validator_readiness_signals(
    context: &MappingContext,
    current_validators: &ValidatorsBySignalledProtocolVersion,
) -> Result<Vec<models::ProtocolVersionReadiness>, MappingError> {
    current_validators
        .versions_to_signalling_validators
        .iter()
        .map(|(protocol_version, signalling_validators)| {
            Ok(models::ProtocolVersionReadiness {
                signalled_protocol_version: protocol_version.as_ref().map(|name| name.to_string()),
                total_active_stake_proportion: to_api_decimal(
                    &signalling_validators
                        .total_stake
                        .div_or_panic(current_validators.total_stake),
                ),
                signalling_validators: signalling_validators
                    .indices_and_stakes
                    .iter()
                    .map(|(index, stake)| {
                        to_api_signalling_validator(
                            context,
                            index,
                            &stake.div_or_panic(current_validators.total_stake),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn to_api_signalling_validator(
    _context: &MappingContext,
    index: &ValidatorIndex,
    stake_proportion: &Decimal,
) -> Result<models::SignallingValidator, MappingError> {
    Ok(models::SignallingValidator {
        index: Box::new(to_api_active_validator_index(*index)),
        active_stake_proportion: to_api_decimal(stake_proportion),
    })
}
