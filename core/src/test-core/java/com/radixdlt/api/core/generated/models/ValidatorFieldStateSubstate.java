/*
 * Babylon Core API - RCnet V2
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.core.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.AccessControllerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.AccessRulesModuleFieldOwnerRoleSubstate;
import com.radixdlt.api.core.generated.models.AccessRulesModuleRuleEntrySubstate;
import com.radixdlt.api.core.generated.models.AccountDepositRuleIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.AccountFieldStateSubstate;
import com.radixdlt.api.core.generated.models.AccountVaultIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldConfigSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentProposalStatisticSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentTimeSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentValidatorSetSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldValidatorRewardsSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.EcdsaSecp256k1PublicKey;
import com.radixdlt.api.core.generated.models.EntityReference;
import com.radixdlt.api.core.generated.models.FungibleResourceManagerFieldDivisibilitySubstate;
import com.radixdlt.api.core.generated.models.FungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.FungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.FungibleVaultFieldFrozenStatusSubstate;
import com.radixdlt.api.core.generated.models.GenericKeyValueStoreEntrySubstate;
import com.radixdlt.api.core.generated.models.GenericScryptoComponentFieldStateSubstate;
import com.radixdlt.api.core.generated.models.MetadataModuleEntrySubstate;
import com.radixdlt.api.core.generated.models.MultiResourcePoolFieldStateSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerDataEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldIdTypeSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldMutableFieldsSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultContentsIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultFieldFrozenStatusSubstate;
import com.radixdlt.api.core.generated.models.OneResourcePoolFieldStateSubstate;
import com.radixdlt.api.core.generated.models.PackageAuthTemplateEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintDependenciesEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageCodeEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageFieldRoyaltyAccumulatorSubstate;
import com.radixdlt.api.core.generated.models.PackageRoyaltyEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageSchemaEntrySubstate;
import com.radixdlt.api.core.generated.models.PendingOwnerStakeWithdrawal;
import com.radixdlt.api.core.generated.models.RoyaltyMethodRoyaltyEntrySubstate;
import com.radixdlt.api.core.generated.models.RoyaltyModuleFieldStateSubstate;
import com.radixdlt.api.core.generated.models.Substate;
import com.radixdlt.api.core.generated.models.SubstateKey;
import com.radixdlt.api.core.generated.models.SubstateType;
import com.radixdlt.api.core.generated.models.TransactionTrackerCollectionEntrySubstate;
import com.radixdlt.api.core.generated.models.TransactionTrackerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.TwoResourcePoolFieldStateSubstate;
import com.radixdlt.api.core.generated.models.TypeInfoModuleFieldTypeInfoSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFeeChangeRequest;
import com.radixdlt.api.core.generated.models.ValidatorFieldProtocolUpdateReadinessSignalSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFieldStateSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFieldStateSubstateAllOf;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * ValidatorFieldStateSubstate
 */
@JsonPropertyOrder({
  ValidatorFieldStateSubstate.JSON_PROPERTY_SORTED_KEY,
  ValidatorFieldStateSubstate.JSON_PROPERTY_PUBLIC_KEY,
  ValidatorFieldStateSubstate.JSON_PROPERTY_IS_REGISTERED,
  ValidatorFieldStateSubstate.JSON_PROPERTY_ACCEPTS_DELEGATED_STAKE,
  ValidatorFieldStateSubstate.JSON_PROPERTY_VALIDATOR_FEE_FACTOR,
  ValidatorFieldStateSubstate.JSON_PROPERTY_VALIDATOR_FEE_CHANGE_REQUEST,
  ValidatorFieldStateSubstate.JSON_PROPERTY_STAKE_UNIT_RESOURCE_ADDRESS,
  ValidatorFieldStateSubstate.JSON_PROPERTY_STAKE_XRD_VAULT,
  ValidatorFieldStateSubstate.JSON_PROPERTY_UNSTAKE_CLAIM_TOKEN_RESOURCE_ADDRESS,
  ValidatorFieldStateSubstate.JSON_PROPERTY_PENDING_XRD_WITHDRAW_VAULT,
  ValidatorFieldStateSubstate.JSON_PROPERTY_LOCKED_OWNER_STAKE_UNIT_VAULT,
  ValidatorFieldStateSubstate.JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_UNLOCK_VAULT,
  ValidatorFieldStateSubstate.JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_WITHDRAWALS,
  ValidatorFieldStateSubstate.JSON_PROPERTY_ALREADY_UNLOCKED_OWNER_STAKE_UNIT_AMOUNT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "substate_type", // ignore manually set substate_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the substate_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "substate_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AccessControllerFieldStateSubstate.class, name = "AccessControllerFieldState"),
  @JsonSubTypes.Type(value = AccessRulesModuleFieldOwnerRoleSubstate.class, name = "AccessRulesModuleFieldOwnerRole"),
  @JsonSubTypes.Type(value = AccessRulesModuleRuleEntrySubstate.class, name = "AccessRulesModuleRuleEntry"),
  @JsonSubTypes.Type(value = AccountDepositRuleIndexEntrySubstate.class, name = "AccountDepositRuleIndexEntry"),
  @JsonSubTypes.Type(value = AccountFieldStateSubstate.class, name = "AccountFieldState"),
  @JsonSubTypes.Type(value = AccountVaultIndexEntrySubstate.class, name = "AccountVaultIndexEntry"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldConfigSubstate.class, name = "ConsensusManagerFieldConfig"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentProposalStatisticSubstate.class, name = "ConsensusManagerFieldCurrentProposalStatistic"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeSubstate.class, name = "ConsensusManagerFieldCurrentTime"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class, name = "ConsensusManagerFieldCurrentTimeRoundedToMinutes"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentValidatorSetSubstate.class, name = "ConsensusManagerFieldCurrentValidatorSet"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldStateSubstate.class, name = "ConsensusManagerFieldState"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldValidatorRewardsSubstate.class, name = "ConsensusManagerFieldValidatorRewards"),
  @JsonSubTypes.Type(value = ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class, name = "ConsensusManagerRegisteredValidatorsByStakeIndexEntry"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldDivisibilitySubstate.class, name = "FungibleResourceManagerFieldDivisibility"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldTotalSupplySubstate.class, name = "FungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = FungibleVaultFieldBalanceSubstate.class, name = "FungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = FungibleVaultFieldFrozenStatusSubstate.class, name = "FungibleVaultFieldFrozenStatus"),
  @JsonSubTypes.Type(value = GenericKeyValueStoreEntrySubstate.class, name = "GenericKeyValueStoreEntry"),
  @JsonSubTypes.Type(value = GenericScryptoComponentFieldStateSubstate.class, name = "GenericScryptoComponentFieldState"),
  @JsonSubTypes.Type(value = MetadataModuleEntrySubstate.class, name = "MetadataModuleEntry"),
  @JsonSubTypes.Type(value = MultiResourcePoolFieldStateSubstate.class, name = "MultiResourcePoolFieldState"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerDataEntrySubstate.class, name = "NonFungibleResourceManagerDataEntry"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldIdTypeSubstate.class, name = "NonFungibleResourceManagerFieldIdType"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldMutableFieldsSubstate.class, name = "NonFungibleResourceManagerFieldMutableFields"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldTotalSupplySubstate.class, name = "NonFungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = NonFungibleVaultContentsIndexEntrySubstate.class, name = "NonFungibleVaultContentsIndexEntry"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldBalanceSubstate.class, name = "NonFungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldFrozenStatusSubstate.class, name = "NonFungibleVaultFieldFrozenStatus"),
  @JsonSubTypes.Type(value = OneResourcePoolFieldStateSubstate.class, name = "OneResourcePoolFieldState"),
  @JsonSubTypes.Type(value = PackageAuthTemplateEntrySubstate.class, name = "PackageAuthTemplateEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintDependenciesEntrySubstate.class, name = "PackageBlueprintDependenciesEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintEntrySubstate.class, name = "PackageBlueprintEntry"),
  @JsonSubTypes.Type(value = PackageCodeEntrySubstate.class, name = "PackageCodeEntry"),
  @JsonSubTypes.Type(value = PackageFieldRoyaltyAccumulatorSubstate.class, name = "PackageFieldRoyaltyAccumulator"),
  @JsonSubTypes.Type(value = PackageRoyaltyEntrySubstate.class, name = "PackageRoyaltyEntry"),
  @JsonSubTypes.Type(value = PackageSchemaEntrySubstate.class, name = "PackageSchemaEntry"),
  @JsonSubTypes.Type(value = RoyaltyMethodRoyaltyEntrySubstate.class, name = "RoyaltyMethodRoyaltyEntry"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldStateSubstate.class, name = "RoyaltyModuleFieldState"),
  @JsonSubTypes.Type(value = TransactionTrackerCollectionEntrySubstate.class, name = "TransactionTrackerCollectionEntry"),
  @JsonSubTypes.Type(value = TransactionTrackerFieldStateSubstate.class, name = "TransactionTrackerFieldState"),
  @JsonSubTypes.Type(value = TwoResourcePoolFieldStateSubstate.class, name = "TwoResourcePoolFieldState"),
  @JsonSubTypes.Type(value = TypeInfoModuleFieldTypeInfoSubstate.class, name = "TypeInfoModuleFieldTypeInfo"),
  @JsonSubTypes.Type(value = ValidatorFieldProtocolUpdateReadinessSignalSubstate.class, name = "ValidatorFieldProtocolUpdateReadinessSignal"),
  @JsonSubTypes.Type(value = ValidatorFieldStateSubstate.class, name = "ValidatorFieldState"),
})

public class ValidatorFieldStateSubstate extends Substate {
  public static final String JSON_PROPERTY_SORTED_KEY = "sorted_key";
  private SubstateKey sortedKey;

  public static final String JSON_PROPERTY_PUBLIC_KEY = "public_key";
  private EcdsaSecp256k1PublicKey publicKey;

  public static final String JSON_PROPERTY_IS_REGISTERED = "is_registered";
  private Boolean isRegistered;

  public static final String JSON_PROPERTY_ACCEPTS_DELEGATED_STAKE = "accepts_delegated_stake";
  private Boolean acceptsDelegatedStake;

  public static final String JSON_PROPERTY_VALIDATOR_FEE_FACTOR = "validator_fee_factor";
  private String validatorFeeFactor;

  public static final String JSON_PROPERTY_VALIDATOR_FEE_CHANGE_REQUEST = "validator_fee_change_request";
  private ValidatorFeeChangeRequest validatorFeeChangeRequest;

  public static final String JSON_PROPERTY_STAKE_UNIT_RESOURCE_ADDRESS = "stake_unit_resource_address";
  private String stakeUnitResourceAddress;

  public static final String JSON_PROPERTY_STAKE_XRD_VAULT = "stake_xrd_vault";
  private EntityReference stakeXrdVault;

  public static final String JSON_PROPERTY_UNSTAKE_CLAIM_TOKEN_RESOURCE_ADDRESS = "unstake_claim_token_resource_address";
  private String unstakeClaimTokenResourceAddress;

  public static final String JSON_PROPERTY_PENDING_XRD_WITHDRAW_VAULT = "pending_xrd_withdraw_vault";
  private EntityReference pendingXrdWithdrawVault;

  public static final String JSON_PROPERTY_LOCKED_OWNER_STAKE_UNIT_VAULT = "locked_owner_stake_unit_vault";
  private EntityReference lockedOwnerStakeUnitVault;

  public static final String JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_UNLOCK_VAULT = "pending_owner_stake_unit_unlock_vault";
  private EntityReference pendingOwnerStakeUnitUnlockVault;

  public static final String JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_WITHDRAWALS = "pending_owner_stake_unit_withdrawals";
  private List<PendingOwnerStakeWithdrawal> pendingOwnerStakeUnitWithdrawals = new ArrayList<>();

  public static final String JSON_PROPERTY_ALREADY_UNLOCKED_OWNER_STAKE_UNIT_AMOUNT = "already_unlocked_owner_stake_unit_amount";
  private String alreadyUnlockedOwnerStakeUnitAmount;

  public ValidatorFieldStateSubstate() { 
  }

  public ValidatorFieldStateSubstate sortedKey(SubstateKey sortedKey) {
    this.sortedKey = sortedKey;
    return this;
  }

   /**
   * Get sortedKey
   * @return sortedKey
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_SORTED_KEY)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public SubstateKey getSortedKey() {
    return sortedKey;
  }


  @JsonProperty(JSON_PROPERTY_SORTED_KEY)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSortedKey(SubstateKey sortedKey) {
    this.sortedKey = sortedKey;
  }


  public ValidatorFieldStateSubstate publicKey(EcdsaSecp256k1PublicKey publicKey) {
    this.publicKey = publicKey;
    return this;
  }

   /**
   * Get publicKey
   * @return publicKey
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EcdsaSecp256k1PublicKey getPublicKey() {
    return publicKey;
  }


  @JsonProperty(JSON_PROPERTY_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPublicKey(EcdsaSecp256k1PublicKey publicKey) {
    this.publicKey = publicKey;
  }


  public ValidatorFieldStateSubstate isRegistered(Boolean isRegistered) {
    this.isRegistered = isRegistered;
    return this;
  }

   /**
   * Get isRegistered
   * @return isRegistered
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IS_REGISTERED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsRegistered() {
    return isRegistered;
  }


  @JsonProperty(JSON_PROPERTY_IS_REGISTERED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsRegistered(Boolean isRegistered) {
    this.isRegistered = isRegistered;
  }


  public ValidatorFieldStateSubstate acceptsDelegatedStake(Boolean acceptsDelegatedStake) {
    this.acceptsDelegatedStake = acceptsDelegatedStake;
    return this;
  }

   /**
   * Get acceptsDelegatedStake
   * @return acceptsDelegatedStake
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ACCEPTS_DELEGATED_STAKE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getAcceptsDelegatedStake() {
    return acceptsDelegatedStake;
  }


  @JsonProperty(JSON_PROPERTY_ACCEPTS_DELEGATED_STAKE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAcceptsDelegatedStake(Boolean acceptsDelegatedStake) {
    this.acceptsDelegatedStake = acceptsDelegatedStake;
  }


  public ValidatorFieldStateSubstate validatorFeeFactor(String validatorFeeFactor) {
    this.validatorFeeFactor = validatorFeeFactor;
    return this;
  }

   /**
   * A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(256 - 1) &lt;&#x3D; m &lt; 2^(256 - 1)&#x60;. 
   * @return validatorFeeFactor
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_VALIDATOR_FEE_FACTOR)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getValidatorFeeFactor() {
    return validatorFeeFactor;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATOR_FEE_FACTOR)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValidatorFeeFactor(String validatorFeeFactor) {
    this.validatorFeeFactor = validatorFeeFactor;
  }


  public ValidatorFieldStateSubstate validatorFeeChangeRequest(ValidatorFeeChangeRequest validatorFeeChangeRequest) {
    this.validatorFeeChangeRequest = validatorFeeChangeRequest;
    return this;
  }

   /**
   * Get validatorFeeChangeRequest
   * @return validatorFeeChangeRequest
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_VALIDATOR_FEE_CHANGE_REQUEST)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ValidatorFeeChangeRequest getValidatorFeeChangeRequest() {
    return validatorFeeChangeRequest;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATOR_FEE_CHANGE_REQUEST)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setValidatorFeeChangeRequest(ValidatorFeeChangeRequest validatorFeeChangeRequest) {
    this.validatorFeeChangeRequest = validatorFeeChangeRequest;
  }


  public ValidatorFieldStateSubstate stakeUnitResourceAddress(String stakeUnitResourceAddress) {
    this.stakeUnitResourceAddress = stakeUnitResourceAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the resource address
   * @return stakeUnitResourceAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the resource address")
  @JsonProperty(JSON_PROPERTY_STAKE_UNIT_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getStakeUnitResourceAddress() {
    return stakeUnitResourceAddress;
  }


  @JsonProperty(JSON_PROPERTY_STAKE_UNIT_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStakeUnitResourceAddress(String stakeUnitResourceAddress) {
    this.stakeUnitResourceAddress = stakeUnitResourceAddress;
  }


  public ValidatorFieldStateSubstate stakeXrdVault(EntityReference stakeXrdVault) {
    this.stakeXrdVault = stakeXrdVault;
    return this;
  }

   /**
   * Get stakeXrdVault
   * @return stakeXrdVault
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_STAKE_XRD_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getStakeXrdVault() {
    return stakeXrdVault;
  }


  @JsonProperty(JSON_PROPERTY_STAKE_XRD_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStakeXrdVault(EntityReference stakeXrdVault) {
    this.stakeXrdVault = stakeXrdVault;
  }


  public ValidatorFieldStateSubstate unstakeClaimTokenResourceAddress(String unstakeClaimTokenResourceAddress) {
    this.unstakeClaimTokenResourceAddress = unstakeClaimTokenResourceAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the resource address
   * @return unstakeClaimTokenResourceAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the resource address")
  @JsonProperty(JSON_PROPERTY_UNSTAKE_CLAIM_TOKEN_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getUnstakeClaimTokenResourceAddress() {
    return unstakeClaimTokenResourceAddress;
  }


  @JsonProperty(JSON_PROPERTY_UNSTAKE_CLAIM_TOKEN_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUnstakeClaimTokenResourceAddress(String unstakeClaimTokenResourceAddress) {
    this.unstakeClaimTokenResourceAddress = unstakeClaimTokenResourceAddress;
  }


  public ValidatorFieldStateSubstate pendingXrdWithdrawVault(EntityReference pendingXrdWithdrawVault) {
    this.pendingXrdWithdrawVault = pendingXrdWithdrawVault;
    return this;
  }

   /**
   * Get pendingXrdWithdrawVault
   * @return pendingXrdWithdrawVault
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PENDING_XRD_WITHDRAW_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getPendingXrdWithdrawVault() {
    return pendingXrdWithdrawVault;
  }


  @JsonProperty(JSON_PROPERTY_PENDING_XRD_WITHDRAW_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPendingXrdWithdrawVault(EntityReference pendingXrdWithdrawVault) {
    this.pendingXrdWithdrawVault = pendingXrdWithdrawVault;
  }


  public ValidatorFieldStateSubstate lockedOwnerStakeUnitVault(EntityReference lockedOwnerStakeUnitVault) {
    this.lockedOwnerStakeUnitVault = lockedOwnerStakeUnitVault;
    return this;
  }

   /**
   * Get lockedOwnerStakeUnitVault
   * @return lockedOwnerStakeUnitVault
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LOCKED_OWNER_STAKE_UNIT_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getLockedOwnerStakeUnitVault() {
    return lockedOwnerStakeUnitVault;
  }


  @JsonProperty(JSON_PROPERTY_LOCKED_OWNER_STAKE_UNIT_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLockedOwnerStakeUnitVault(EntityReference lockedOwnerStakeUnitVault) {
    this.lockedOwnerStakeUnitVault = lockedOwnerStakeUnitVault;
  }


  public ValidatorFieldStateSubstate pendingOwnerStakeUnitUnlockVault(EntityReference pendingOwnerStakeUnitUnlockVault) {
    this.pendingOwnerStakeUnitUnlockVault = pendingOwnerStakeUnitUnlockVault;
    return this;
  }

   /**
   * Get pendingOwnerStakeUnitUnlockVault
   * @return pendingOwnerStakeUnitUnlockVault
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_UNLOCK_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getPendingOwnerStakeUnitUnlockVault() {
    return pendingOwnerStakeUnitUnlockVault;
  }


  @JsonProperty(JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_UNLOCK_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPendingOwnerStakeUnitUnlockVault(EntityReference pendingOwnerStakeUnitUnlockVault) {
    this.pendingOwnerStakeUnitUnlockVault = pendingOwnerStakeUnitUnlockVault;
  }


  public ValidatorFieldStateSubstate pendingOwnerStakeUnitWithdrawals(List<PendingOwnerStakeWithdrawal> pendingOwnerStakeUnitWithdrawals) {
    this.pendingOwnerStakeUnitWithdrawals = pendingOwnerStakeUnitWithdrawals;
    return this;
  }

  public ValidatorFieldStateSubstate addPendingOwnerStakeUnitWithdrawalsItem(PendingOwnerStakeWithdrawal pendingOwnerStakeUnitWithdrawalsItem) {
    this.pendingOwnerStakeUnitWithdrawals.add(pendingOwnerStakeUnitWithdrawalsItem);
    return this;
  }

   /**
   * Get pendingOwnerStakeUnitWithdrawals
   * @return pendingOwnerStakeUnitWithdrawals
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_WITHDRAWALS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<PendingOwnerStakeWithdrawal> getPendingOwnerStakeUnitWithdrawals() {
    return pendingOwnerStakeUnitWithdrawals;
  }


  @JsonProperty(JSON_PROPERTY_PENDING_OWNER_STAKE_UNIT_WITHDRAWALS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPendingOwnerStakeUnitWithdrawals(List<PendingOwnerStakeWithdrawal> pendingOwnerStakeUnitWithdrawals) {
    this.pendingOwnerStakeUnitWithdrawals = pendingOwnerStakeUnitWithdrawals;
  }


  public ValidatorFieldStateSubstate alreadyUnlockedOwnerStakeUnitAmount(String alreadyUnlockedOwnerStakeUnitAmount) {
    this.alreadyUnlockedOwnerStakeUnitAmount = alreadyUnlockedOwnerStakeUnitAmount;
    return this;
  }

   /**
   * A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(256 - 1) &lt;&#x3D; m &lt; 2^(256 - 1)&#x60;. 
   * @return alreadyUnlockedOwnerStakeUnitAmount
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_ALREADY_UNLOCKED_OWNER_STAKE_UNIT_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAlreadyUnlockedOwnerStakeUnitAmount() {
    return alreadyUnlockedOwnerStakeUnitAmount;
  }


  @JsonProperty(JSON_PROPERTY_ALREADY_UNLOCKED_OWNER_STAKE_UNIT_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAlreadyUnlockedOwnerStakeUnitAmount(String alreadyUnlockedOwnerStakeUnitAmount) {
    this.alreadyUnlockedOwnerStakeUnitAmount = alreadyUnlockedOwnerStakeUnitAmount;
  }


  /**
   * Return true if this ValidatorFieldStateSubstate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ValidatorFieldStateSubstate validatorFieldStateSubstate = (ValidatorFieldStateSubstate) o;
    return Objects.equals(this.sortedKey, validatorFieldStateSubstate.sortedKey) &&
        Objects.equals(this.publicKey, validatorFieldStateSubstate.publicKey) &&
        Objects.equals(this.isRegistered, validatorFieldStateSubstate.isRegistered) &&
        Objects.equals(this.acceptsDelegatedStake, validatorFieldStateSubstate.acceptsDelegatedStake) &&
        Objects.equals(this.validatorFeeFactor, validatorFieldStateSubstate.validatorFeeFactor) &&
        Objects.equals(this.validatorFeeChangeRequest, validatorFieldStateSubstate.validatorFeeChangeRequest) &&
        Objects.equals(this.stakeUnitResourceAddress, validatorFieldStateSubstate.stakeUnitResourceAddress) &&
        Objects.equals(this.stakeXrdVault, validatorFieldStateSubstate.stakeXrdVault) &&
        Objects.equals(this.unstakeClaimTokenResourceAddress, validatorFieldStateSubstate.unstakeClaimTokenResourceAddress) &&
        Objects.equals(this.pendingXrdWithdrawVault, validatorFieldStateSubstate.pendingXrdWithdrawVault) &&
        Objects.equals(this.lockedOwnerStakeUnitVault, validatorFieldStateSubstate.lockedOwnerStakeUnitVault) &&
        Objects.equals(this.pendingOwnerStakeUnitUnlockVault, validatorFieldStateSubstate.pendingOwnerStakeUnitUnlockVault) &&
        Objects.equals(this.pendingOwnerStakeUnitWithdrawals, validatorFieldStateSubstate.pendingOwnerStakeUnitWithdrawals) &&
        Objects.equals(this.alreadyUnlockedOwnerStakeUnitAmount, validatorFieldStateSubstate.alreadyUnlockedOwnerStakeUnitAmount) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(sortedKey, publicKey, isRegistered, acceptsDelegatedStake, validatorFeeFactor, validatorFeeChangeRequest, stakeUnitResourceAddress, stakeXrdVault, unstakeClaimTokenResourceAddress, pendingXrdWithdrawVault, lockedOwnerStakeUnitVault, pendingOwnerStakeUnitUnlockVault, pendingOwnerStakeUnitWithdrawals, alreadyUnlockedOwnerStakeUnitAmount, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ValidatorFieldStateSubstate {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    sortedKey: ").append(toIndentedString(sortedKey)).append("\n");
    sb.append("    publicKey: ").append(toIndentedString(publicKey)).append("\n");
    sb.append("    isRegistered: ").append(toIndentedString(isRegistered)).append("\n");
    sb.append("    acceptsDelegatedStake: ").append(toIndentedString(acceptsDelegatedStake)).append("\n");
    sb.append("    validatorFeeFactor: ").append(toIndentedString(validatorFeeFactor)).append("\n");
    sb.append("    validatorFeeChangeRequest: ").append(toIndentedString(validatorFeeChangeRequest)).append("\n");
    sb.append("    stakeUnitResourceAddress: ").append(toIndentedString(stakeUnitResourceAddress)).append("\n");
    sb.append("    stakeXrdVault: ").append(toIndentedString(stakeXrdVault)).append("\n");
    sb.append("    unstakeClaimTokenResourceAddress: ").append(toIndentedString(unstakeClaimTokenResourceAddress)).append("\n");
    sb.append("    pendingXrdWithdrawVault: ").append(toIndentedString(pendingXrdWithdrawVault)).append("\n");
    sb.append("    lockedOwnerStakeUnitVault: ").append(toIndentedString(lockedOwnerStakeUnitVault)).append("\n");
    sb.append("    pendingOwnerStakeUnitUnlockVault: ").append(toIndentedString(pendingOwnerStakeUnitUnlockVault)).append("\n");
    sb.append("    pendingOwnerStakeUnitWithdrawals: ").append(toIndentedString(pendingOwnerStakeUnitWithdrawals)).append("\n");
    sb.append("    alreadyUnlockedOwnerStakeUnitAmount: ").append(toIndentedString(alreadyUnlockedOwnerStakeUnitAmount)).append("\n");
    sb.append("}");
    return sb.toString();
  }

  /**
   * Convert the given object to string with each line indented by 4 spaces
   * (except the first line).
   */
  private String toIndentedString(Object o) {
    if (o == null) {
      return "null";
    }
    return o.toString().replace("\n", "\n    ");
  }

static {
  // Initialize and register the discriminator mappings.
  Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
  mappings.put("AccessControllerFieldState", AccessControllerFieldStateSubstate.class);
  mappings.put("AccessRulesModuleFieldOwnerRole", AccessRulesModuleFieldOwnerRoleSubstate.class);
  mappings.put("AccessRulesModuleRuleEntry", AccessRulesModuleRuleEntrySubstate.class);
  mappings.put("AccountDepositRuleIndexEntry", AccountDepositRuleIndexEntrySubstate.class);
  mappings.put("AccountFieldState", AccountFieldStateSubstate.class);
  mappings.put("AccountVaultIndexEntry", AccountVaultIndexEntrySubstate.class);
  mappings.put("ConsensusManagerFieldConfig", ConsensusManagerFieldConfigSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentProposalStatistic", ConsensusManagerFieldCurrentProposalStatisticSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTime", ConsensusManagerFieldCurrentTimeSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTimeRoundedToMinutes", ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentValidatorSet", ConsensusManagerFieldCurrentValidatorSetSubstate.class);
  mappings.put("ConsensusManagerFieldState", ConsensusManagerFieldStateSubstate.class);
  mappings.put("ConsensusManagerFieldValidatorRewards", ConsensusManagerFieldValidatorRewardsSubstate.class);
  mappings.put("ConsensusManagerRegisteredValidatorsByStakeIndexEntry", ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class);
  mappings.put("FungibleResourceManagerFieldDivisibility", FungibleResourceManagerFieldDivisibilitySubstate.class);
  mappings.put("FungibleResourceManagerFieldTotalSupply", FungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("FungibleVaultFieldBalance", FungibleVaultFieldBalanceSubstate.class);
  mappings.put("FungibleVaultFieldFrozenStatus", FungibleVaultFieldFrozenStatusSubstate.class);
  mappings.put("GenericKeyValueStoreEntry", GenericKeyValueStoreEntrySubstate.class);
  mappings.put("GenericScryptoComponentFieldState", GenericScryptoComponentFieldStateSubstate.class);
  mappings.put("MetadataModuleEntry", MetadataModuleEntrySubstate.class);
  mappings.put("MultiResourcePoolFieldState", MultiResourcePoolFieldStateSubstate.class);
  mappings.put("NonFungibleResourceManagerDataEntry", NonFungibleResourceManagerDataEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerFieldIdType", NonFungibleResourceManagerFieldIdTypeSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldMutableFields", NonFungibleResourceManagerFieldMutableFieldsSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldTotalSupply", NonFungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("NonFungibleVaultContentsIndexEntry", NonFungibleVaultContentsIndexEntrySubstate.class);
  mappings.put("NonFungibleVaultFieldBalance", NonFungibleVaultFieldBalanceSubstate.class);
  mappings.put("NonFungibleVaultFieldFrozenStatus", NonFungibleVaultFieldFrozenStatusSubstate.class);
  mappings.put("OneResourcePoolFieldState", OneResourcePoolFieldStateSubstate.class);
  mappings.put("PackageAuthTemplateEntry", PackageAuthTemplateEntrySubstate.class);
  mappings.put("PackageBlueprintDependenciesEntry", PackageBlueprintDependenciesEntrySubstate.class);
  mappings.put("PackageBlueprintEntry", PackageBlueprintEntrySubstate.class);
  mappings.put("PackageCodeEntry", PackageCodeEntrySubstate.class);
  mappings.put("PackageFieldRoyaltyAccumulator", PackageFieldRoyaltyAccumulatorSubstate.class);
  mappings.put("PackageRoyaltyEntry", PackageRoyaltyEntrySubstate.class);
  mappings.put("PackageSchemaEntry", PackageSchemaEntrySubstate.class);
  mappings.put("RoyaltyMethodRoyaltyEntry", RoyaltyMethodRoyaltyEntrySubstate.class);
  mappings.put("RoyaltyModuleFieldState", RoyaltyModuleFieldStateSubstate.class);
  mappings.put("TransactionTrackerCollectionEntry", TransactionTrackerCollectionEntrySubstate.class);
  mappings.put("TransactionTrackerFieldState", TransactionTrackerFieldStateSubstate.class);
  mappings.put("TwoResourcePoolFieldState", TwoResourcePoolFieldStateSubstate.class);
  mappings.put("TypeInfoModuleFieldTypeInfo", TypeInfoModuleFieldTypeInfoSubstate.class);
  mappings.put("ValidatorFieldProtocolUpdateReadinessSignal", ValidatorFieldProtocolUpdateReadinessSignalSubstate.class);
  mappings.put("ValidatorFieldState", ValidatorFieldStateSubstate.class);
  mappings.put("ValidatorFieldStateSubstate", ValidatorFieldStateSubstate.class);
  JSON.registerDiscriminator(ValidatorFieldStateSubstate.class, "substate_type", mappings);
}
}

