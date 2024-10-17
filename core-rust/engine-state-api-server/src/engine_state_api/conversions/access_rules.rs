use crate::prelude::*;

pub fn to_api_access_rule(
    context: &MappingContext,
    access_rule: &AccessRule,
) -> Result<models::AccessRule, MappingError> {
    Ok(match access_rule {
        AccessRule::Protected(access_rule_node) => models::AccessRule::ProtectedAccessRule {
            access_rule: Box::new(to_api_access_rule_node(context, access_rule_node)?),
        },
        AccessRule::AllowAll => models::AccessRule::AllowAllAccessRule {},
        AccessRule::DenyAll => models::AccessRule::DenyAllAccessRule {},
    })
}

pub fn to_api_access_rule_node(
    context: &MappingContext,
    requirement: &CompositeRequirement,
) -> Result<models::AccessRuleNode, MappingError> {
    Ok(match requirement {
        CompositeRequirement::BasicRequirement(requirement) => {
            models::AccessRuleNode::ProofAccessRuleNode {
                proof_rule: Box::new(to_api_proof_rule(context, requirement)?),
            }
        }
        CompositeRequirement::AnyOf(access_rules) => models::AccessRuleNode::AnyOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_access_rule_node(context, ar))
                .collect::<Result<_, _>>()?,
        },
        CompositeRequirement::AllOf(access_rules) => models::AccessRuleNode::AllOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_access_rule_node(context, ar))
                .collect::<Result<_, _>>()?,
        },
    })
}

pub fn to_api_proof_rule(
    context: &MappingContext,
    requirement: &BasicRequirement,
) -> Result<models::ProofRule, MappingError> {
    Ok(match requirement {
        BasicRequirement::Require(resource_or_non_fungible) => {
            models::ProofRule::RequireProofRule {
                requirement: Box::new(to_api_requirement(context, resource_or_non_fungible)?),
            }
        }
        BasicRequirement::AmountOf(amount, resource) => models::ProofRule::AmountOfProofRule {
            amount: to_api_decimal(amount),
            resource: to_api_resource_address(context, resource)?,
        },
        BasicRequirement::AllOf(resource_or_non_fungible_list) => {
            models::ProofRule::AllOfProofRule {
                list: to_api_resource_or_non_fungible_list(context, resource_or_non_fungible_list)?,
            }
        }
        BasicRequirement::AnyOf(resource_or_non_fungible_list) => {
            models::ProofRule::AnyOfProofRule {
                list: to_api_resource_or_non_fungible_list(context, resource_or_non_fungible_list)?,
            }
        }
        BasicRequirement::CountOf(count, resource_or_non_fungible_list) => {
            models::ProofRule::CountOfProofRule {
                count: *count as i32,
                list: to_api_resource_or_non_fungible_list(context, resource_or_non_fungible_list)?,
            }
        }
    })
}

pub fn to_api_resource_or_non_fungible_list(
    context: &MappingContext,
    requirement_list: &[ResourceOrNonFungible],
) -> Result<Vec<models::Requirement>, MappingError> {
    let mut res = Vec::new();
    for resource_or_non_fungible in requirement_list.iter() {
        res.push(to_api_requirement(context, resource_or_non_fungible)?);
    }
    Ok(res)
}

pub fn to_api_requirement(
    context: &MappingContext,
    requirement: &ResourceOrNonFungible,
) -> Result<models::Requirement, MappingError> {
    Ok(match requirement {
        ResourceOrNonFungible::Resource(resource_address) => {
            models::Requirement::ResourceRequirement {
                resource: to_api_resource_address(context, resource_address)?,
            }
        }
        ResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
            models::Requirement::NonFungibleRequirement {
                non_fungible: Box::new(to_api_non_fungible_global_id(
                    context,
                    non_fungible_global_id,
                )?),
            }
        }
    })
}
