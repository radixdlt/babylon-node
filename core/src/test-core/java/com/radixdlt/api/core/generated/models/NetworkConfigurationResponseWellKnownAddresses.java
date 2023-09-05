/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Key addresses for this network.
 */
@ApiModel(description = "Key addresses for this network.")
@JsonPropertyOrder({
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_XRD,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_SECP256K1_SIGNATURE_VIRTUAL_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_ED25519_SIGNATURE_VIRTUAL_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_GLOBAL_CALLER_VIRTUAL_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_SYSTEM_TRANSACTION_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_PACKAGE_OWNER_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_VALIDATOR_OWNER_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_ACCOUNT_OWNER_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_IDENTITY_OWNER_BADGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_PACKAGE_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_RESOURCE_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_ACCOUNT_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_IDENTITY_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_CONSENSUS_MANAGER_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_ACCESS_CONTROLLER_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_TRANSACTION_PROCESSOR_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_METADATA_MODULE_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_ROYALTY_MODULE_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_ROLE_ASSIGNMENT_MODULE_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_GENESIS_HELPER_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_FAUCET_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_POOL_PACKAGE,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_CONSENSUS_MANAGER,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_GENESIS_HELPER,
  NetworkConfigurationResponseWellKnownAddresses.JSON_PROPERTY_FAUCET
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NetworkConfigurationResponseWellKnownAddresses {
  public static final String JSON_PROPERTY_XRD = "xrd";
  private String xrd;

  public static final String JSON_PROPERTY_SECP256K1_SIGNATURE_VIRTUAL_BADGE = "secp256k1_signature_virtual_badge";
  private String secp256k1SignatureVirtualBadge;

  public static final String JSON_PROPERTY_ED25519_SIGNATURE_VIRTUAL_BADGE = "ed25519_signature_virtual_badge";
  private String ed25519SignatureVirtualBadge;

  public static final String JSON_PROPERTY_PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE = "package_of_direct_caller_virtual_badge";
  private String packageOfDirectCallerVirtualBadge;

  public static final String JSON_PROPERTY_GLOBAL_CALLER_VIRTUAL_BADGE = "global_caller_virtual_badge";
  private String globalCallerVirtualBadge;

  public static final String JSON_PROPERTY_SYSTEM_TRANSACTION_BADGE = "system_transaction_badge";
  private String systemTransactionBadge;

  public static final String JSON_PROPERTY_PACKAGE_OWNER_BADGE = "package_owner_badge";
  private String packageOwnerBadge;

  public static final String JSON_PROPERTY_VALIDATOR_OWNER_BADGE = "validator_owner_badge";
  private String validatorOwnerBadge;

  public static final String JSON_PROPERTY_ACCOUNT_OWNER_BADGE = "account_owner_badge";
  private String accountOwnerBadge;

  public static final String JSON_PROPERTY_IDENTITY_OWNER_BADGE = "identity_owner_badge";
  private String identityOwnerBadge;

  public static final String JSON_PROPERTY_PACKAGE_PACKAGE = "package_package";
  private String packagePackage;

  public static final String JSON_PROPERTY_RESOURCE_PACKAGE = "resource_package";
  private String resourcePackage;

  public static final String JSON_PROPERTY_ACCOUNT_PACKAGE = "account_package";
  private String accountPackage;

  public static final String JSON_PROPERTY_IDENTITY_PACKAGE = "identity_package";
  private String identityPackage;

  public static final String JSON_PROPERTY_CONSENSUS_MANAGER_PACKAGE = "consensus_manager_package";
  private String consensusManagerPackage;

  public static final String JSON_PROPERTY_ACCESS_CONTROLLER_PACKAGE = "access_controller_package";
  private String accessControllerPackage;

  public static final String JSON_PROPERTY_TRANSACTION_PROCESSOR_PACKAGE = "transaction_processor_package";
  private String transactionProcessorPackage;

  public static final String JSON_PROPERTY_METADATA_MODULE_PACKAGE = "metadata_module_package";
  private String metadataModulePackage;

  public static final String JSON_PROPERTY_ROYALTY_MODULE_PACKAGE = "royalty_module_package";
  private String royaltyModulePackage;

  public static final String JSON_PROPERTY_ROLE_ASSIGNMENT_MODULE_PACKAGE = "role_assignment_module_package";
  private String roleAssignmentModulePackage;

  public static final String JSON_PROPERTY_GENESIS_HELPER_PACKAGE = "genesis_helper_package";
  private String genesisHelperPackage;

  public static final String JSON_PROPERTY_FAUCET_PACKAGE = "faucet_package";
  private String faucetPackage;

  public static final String JSON_PROPERTY_POOL_PACKAGE = "pool_package";
  private String poolPackage;

  public static final String JSON_PROPERTY_CONSENSUS_MANAGER = "consensus_manager";
  private String consensusManager;

  public static final String JSON_PROPERTY_GENESIS_HELPER = "genesis_helper";
  private String genesisHelper;

  public static final String JSON_PROPERTY_FAUCET = "faucet";
  private String faucet;

  public NetworkConfigurationResponseWellKnownAddresses() { 
  }

  public NetworkConfigurationResponseWellKnownAddresses xrd(String xrd) {
    this.xrd = xrd;
    return this;
  }

   /**
   * Get xrd
   * @return xrd
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_XRD)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrd() {
    return xrd;
  }


  @JsonProperty(JSON_PROPERTY_XRD)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrd(String xrd) {
    this.xrd = xrd;
  }


  public NetworkConfigurationResponseWellKnownAddresses secp256k1SignatureVirtualBadge(String secp256k1SignatureVirtualBadge) {
    this.secp256k1SignatureVirtualBadge = secp256k1SignatureVirtualBadge;
    return this;
  }

   /**
   * Get secp256k1SignatureVirtualBadge
   * @return secp256k1SignatureVirtualBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_SECP256K1_SIGNATURE_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSecp256k1SignatureVirtualBadge() {
    return secp256k1SignatureVirtualBadge;
  }


  @JsonProperty(JSON_PROPERTY_SECP256K1_SIGNATURE_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSecp256k1SignatureVirtualBadge(String secp256k1SignatureVirtualBadge) {
    this.secp256k1SignatureVirtualBadge = secp256k1SignatureVirtualBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses ed25519SignatureVirtualBadge(String ed25519SignatureVirtualBadge) {
    this.ed25519SignatureVirtualBadge = ed25519SignatureVirtualBadge;
    return this;
  }

   /**
   * Get ed25519SignatureVirtualBadge
   * @return ed25519SignatureVirtualBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ED25519_SIGNATURE_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getEd25519SignatureVirtualBadge() {
    return ed25519SignatureVirtualBadge;
  }


  @JsonProperty(JSON_PROPERTY_ED25519_SIGNATURE_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEd25519SignatureVirtualBadge(String ed25519SignatureVirtualBadge) {
    this.ed25519SignatureVirtualBadge = ed25519SignatureVirtualBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses packageOfDirectCallerVirtualBadge(String packageOfDirectCallerVirtualBadge) {
    this.packageOfDirectCallerVirtualBadge = packageOfDirectCallerVirtualBadge;
    return this;
  }

   /**
   * Get packageOfDirectCallerVirtualBadge
   * @return packageOfDirectCallerVirtualBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPackageOfDirectCallerVirtualBadge() {
    return packageOfDirectCallerVirtualBadge;
  }


  @JsonProperty(JSON_PROPERTY_PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPackageOfDirectCallerVirtualBadge(String packageOfDirectCallerVirtualBadge) {
    this.packageOfDirectCallerVirtualBadge = packageOfDirectCallerVirtualBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses globalCallerVirtualBadge(String globalCallerVirtualBadge) {
    this.globalCallerVirtualBadge = globalCallerVirtualBadge;
    return this;
  }

   /**
   * Get globalCallerVirtualBadge
   * @return globalCallerVirtualBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_GLOBAL_CALLER_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getGlobalCallerVirtualBadge() {
    return globalCallerVirtualBadge;
  }


  @JsonProperty(JSON_PROPERTY_GLOBAL_CALLER_VIRTUAL_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGlobalCallerVirtualBadge(String globalCallerVirtualBadge) {
    this.globalCallerVirtualBadge = globalCallerVirtualBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses systemTransactionBadge(String systemTransactionBadge) {
    this.systemTransactionBadge = systemTransactionBadge;
    return this;
  }

   /**
   * Get systemTransactionBadge
   * @return systemTransactionBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_SYSTEM_TRANSACTION_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSystemTransactionBadge() {
    return systemTransactionBadge;
  }


  @JsonProperty(JSON_PROPERTY_SYSTEM_TRANSACTION_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSystemTransactionBadge(String systemTransactionBadge) {
    this.systemTransactionBadge = systemTransactionBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses packageOwnerBadge(String packageOwnerBadge) {
    this.packageOwnerBadge = packageOwnerBadge;
    return this;
  }

   /**
   * Get packageOwnerBadge
   * @return packageOwnerBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PACKAGE_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPackageOwnerBadge() {
    return packageOwnerBadge;
  }


  @JsonProperty(JSON_PROPERTY_PACKAGE_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPackageOwnerBadge(String packageOwnerBadge) {
    this.packageOwnerBadge = packageOwnerBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses validatorOwnerBadge(String validatorOwnerBadge) {
    this.validatorOwnerBadge = validatorOwnerBadge;
    return this;
  }

   /**
   * Get validatorOwnerBadge
   * @return validatorOwnerBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VALIDATOR_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getValidatorOwnerBadge() {
    return validatorOwnerBadge;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATOR_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValidatorOwnerBadge(String validatorOwnerBadge) {
    this.validatorOwnerBadge = validatorOwnerBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses accountOwnerBadge(String accountOwnerBadge) {
    this.accountOwnerBadge = accountOwnerBadge;
    return this;
  }

   /**
   * Get accountOwnerBadge
   * @return accountOwnerBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ACCOUNT_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAccountOwnerBadge() {
    return accountOwnerBadge;
  }


  @JsonProperty(JSON_PROPERTY_ACCOUNT_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAccountOwnerBadge(String accountOwnerBadge) {
    this.accountOwnerBadge = accountOwnerBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses identityOwnerBadge(String identityOwnerBadge) {
    this.identityOwnerBadge = identityOwnerBadge;
    return this;
  }

   /**
   * Get identityOwnerBadge
   * @return identityOwnerBadge
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IDENTITY_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getIdentityOwnerBadge() {
    return identityOwnerBadge;
  }


  @JsonProperty(JSON_PROPERTY_IDENTITY_OWNER_BADGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIdentityOwnerBadge(String identityOwnerBadge) {
    this.identityOwnerBadge = identityOwnerBadge;
  }


  public NetworkConfigurationResponseWellKnownAddresses packagePackage(String packagePackage) {
    this.packagePackage = packagePackage;
    return this;
  }

   /**
   * Get packagePackage
   * @return packagePackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PACKAGE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPackagePackage() {
    return packagePackage;
  }


  @JsonProperty(JSON_PROPERTY_PACKAGE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPackagePackage(String packagePackage) {
    this.packagePackage = packagePackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses resourcePackage(String resourcePackage) {
    this.resourcePackage = resourcePackage;
    return this;
  }

   /**
   * Get resourcePackage
   * @return resourcePackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_RESOURCE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getResourcePackage() {
    return resourcePackage;
  }


  @JsonProperty(JSON_PROPERTY_RESOURCE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setResourcePackage(String resourcePackage) {
    this.resourcePackage = resourcePackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses accountPackage(String accountPackage) {
    this.accountPackage = accountPackage;
    return this;
  }

   /**
   * Get accountPackage
   * @return accountPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ACCOUNT_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAccountPackage() {
    return accountPackage;
  }


  @JsonProperty(JSON_PROPERTY_ACCOUNT_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAccountPackage(String accountPackage) {
    this.accountPackage = accountPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses identityPackage(String identityPackage) {
    this.identityPackage = identityPackage;
    return this;
  }

   /**
   * Get identityPackage
   * @return identityPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IDENTITY_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getIdentityPackage() {
    return identityPackage;
  }


  @JsonProperty(JSON_PROPERTY_IDENTITY_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIdentityPackage(String identityPackage) {
    this.identityPackage = identityPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses consensusManagerPackage(String consensusManagerPackage) {
    this.consensusManagerPackage = consensusManagerPackage;
    return this;
  }

   /**
   * Get consensusManagerPackage
   * @return consensusManagerPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CONSENSUS_MANAGER_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getConsensusManagerPackage() {
    return consensusManagerPackage;
  }


  @JsonProperty(JSON_PROPERTY_CONSENSUS_MANAGER_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setConsensusManagerPackage(String consensusManagerPackage) {
    this.consensusManagerPackage = consensusManagerPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses accessControllerPackage(String accessControllerPackage) {
    this.accessControllerPackage = accessControllerPackage;
    return this;
  }

   /**
   * Get accessControllerPackage
   * @return accessControllerPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ACCESS_CONTROLLER_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAccessControllerPackage() {
    return accessControllerPackage;
  }


  @JsonProperty(JSON_PROPERTY_ACCESS_CONTROLLER_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAccessControllerPackage(String accessControllerPackage) {
    this.accessControllerPackage = accessControllerPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses transactionProcessorPackage(String transactionProcessorPackage) {
    this.transactionProcessorPackage = transactionProcessorPackage;
    return this;
  }

   /**
   * Get transactionProcessorPackage
   * @return transactionProcessorPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_PROCESSOR_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getTransactionProcessorPackage() {
    return transactionProcessorPackage;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_PROCESSOR_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactionProcessorPackage(String transactionProcessorPackage) {
    this.transactionProcessorPackage = transactionProcessorPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses metadataModulePackage(String metadataModulePackage) {
    this.metadataModulePackage = metadataModulePackage;
    return this;
  }

   /**
   * Get metadataModulePackage
   * @return metadataModulePackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_METADATA_MODULE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getMetadataModulePackage() {
    return metadataModulePackage;
  }


  @JsonProperty(JSON_PROPERTY_METADATA_MODULE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMetadataModulePackage(String metadataModulePackage) {
    this.metadataModulePackage = metadataModulePackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses royaltyModulePackage(String royaltyModulePackage) {
    this.royaltyModulePackage = royaltyModulePackage;
    return this;
  }

   /**
   * Get royaltyModulePackage
   * @return royaltyModulePackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ROYALTY_MODULE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getRoyaltyModulePackage() {
    return royaltyModulePackage;
  }


  @JsonProperty(JSON_PROPERTY_ROYALTY_MODULE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoyaltyModulePackage(String royaltyModulePackage) {
    this.royaltyModulePackage = royaltyModulePackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses roleAssignmentModulePackage(String roleAssignmentModulePackage) {
    this.roleAssignmentModulePackage = roleAssignmentModulePackage;
    return this;
  }

   /**
   * Get roleAssignmentModulePackage
   * @return roleAssignmentModulePackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ROLE_ASSIGNMENT_MODULE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getRoleAssignmentModulePackage() {
    return roleAssignmentModulePackage;
  }


  @JsonProperty(JSON_PROPERTY_ROLE_ASSIGNMENT_MODULE_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoleAssignmentModulePackage(String roleAssignmentModulePackage) {
    this.roleAssignmentModulePackage = roleAssignmentModulePackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses genesisHelperPackage(String genesisHelperPackage) {
    this.genesisHelperPackage = genesisHelperPackage;
    return this;
  }

   /**
   * Get genesisHelperPackage
   * @return genesisHelperPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_GENESIS_HELPER_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getGenesisHelperPackage() {
    return genesisHelperPackage;
  }


  @JsonProperty(JSON_PROPERTY_GENESIS_HELPER_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGenesisHelperPackage(String genesisHelperPackage) {
    this.genesisHelperPackage = genesisHelperPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses faucetPackage(String faucetPackage) {
    this.faucetPackage = faucetPackage;
    return this;
  }

   /**
   * Get faucetPackage
   * @return faucetPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_FAUCET_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getFaucetPackage() {
    return faucetPackage;
  }


  @JsonProperty(JSON_PROPERTY_FAUCET_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFaucetPackage(String faucetPackage) {
    this.faucetPackage = faucetPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses poolPackage(String poolPackage) {
    this.poolPackage = poolPackage;
    return this;
  }

   /**
   * Get poolPackage
   * @return poolPackage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_POOL_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPoolPackage() {
    return poolPackage;
  }


  @JsonProperty(JSON_PROPERTY_POOL_PACKAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPoolPackage(String poolPackage) {
    this.poolPackage = poolPackage;
  }


  public NetworkConfigurationResponseWellKnownAddresses consensusManager(String consensusManager) {
    this.consensusManager = consensusManager;
    return this;
  }

   /**
   * Get consensusManager
   * @return consensusManager
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CONSENSUS_MANAGER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getConsensusManager() {
    return consensusManager;
  }


  @JsonProperty(JSON_PROPERTY_CONSENSUS_MANAGER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setConsensusManager(String consensusManager) {
    this.consensusManager = consensusManager;
  }


  public NetworkConfigurationResponseWellKnownAddresses genesisHelper(String genesisHelper) {
    this.genesisHelper = genesisHelper;
    return this;
  }

   /**
   * Get genesisHelper
   * @return genesisHelper
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_GENESIS_HELPER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getGenesisHelper() {
    return genesisHelper;
  }


  @JsonProperty(JSON_PROPERTY_GENESIS_HELPER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGenesisHelper(String genesisHelper) {
    this.genesisHelper = genesisHelper;
  }


  public NetworkConfigurationResponseWellKnownAddresses faucet(String faucet) {
    this.faucet = faucet;
    return this;
  }

   /**
   * Get faucet
   * @return faucet
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_FAUCET)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getFaucet() {
    return faucet;
  }


  @JsonProperty(JSON_PROPERTY_FAUCET)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFaucet(String faucet) {
    this.faucet = faucet;
  }


  /**
   * Return true if this NetworkConfigurationResponse_well_known_addresses object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NetworkConfigurationResponseWellKnownAddresses networkConfigurationResponseWellKnownAddresses = (NetworkConfigurationResponseWellKnownAddresses) o;
    return Objects.equals(this.xrd, networkConfigurationResponseWellKnownAddresses.xrd) &&
        Objects.equals(this.secp256k1SignatureVirtualBadge, networkConfigurationResponseWellKnownAddresses.secp256k1SignatureVirtualBadge) &&
        Objects.equals(this.ed25519SignatureVirtualBadge, networkConfigurationResponseWellKnownAddresses.ed25519SignatureVirtualBadge) &&
        Objects.equals(this.packageOfDirectCallerVirtualBadge, networkConfigurationResponseWellKnownAddresses.packageOfDirectCallerVirtualBadge) &&
        Objects.equals(this.globalCallerVirtualBadge, networkConfigurationResponseWellKnownAddresses.globalCallerVirtualBadge) &&
        Objects.equals(this.systemTransactionBadge, networkConfigurationResponseWellKnownAddresses.systemTransactionBadge) &&
        Objects.equals(this.packageOwnerBadge, networkConfigurationResponseWellKnownAddresses.packageOwnerBadge) &&
        Objects.equals(this.validatorOwnerBadge, networkConfigurationResponseWellKnownAddresses.validatorOwnerBadge) &&
        Objects.equals(this.accountOwnerBadge, networkConfigurationResponseWellKnownAddresses.accountOwnerBadge) &&
        Objects.equals(this.identityOwnerBadge, networkConfigurationResponseWellKnownAddresses.identityOwnerBadge) &&
        Objects.equals(this.packagePackage, networkConfigurationResponseWellKnownAddresses.packagePackage) &&
        Objects.equals(this.resourcePackage, networkConfigurationResponseWellKnownAddresses.resourcePackage) &&
        Objects.equals(this.accountPackage, networkConfigurationResponseWellKnownAddresses.accountPackage) &&
        Objects.equals(this.identityPackage, networkConfigurationResponseWellKnownAddresses.identityPackage) &&
        Objects.equals(this.consensusManagerPackage, networkConfigurationResponseWellKnownAddresses.consensusManagerPackage) &&
        Objects.equals(this.accessControllerPackage, networkConfigurationResponseWellKnownAddresses.accessControllerPackage) &&
        Objects.equals(this.transactionProcessorPackage, networkConfigurationResponseWellKnownAddresses.transactionProcessorPackage) &&
        Objects.equals(this.metadataModulePackage, networkConfigurationResponseWellKnownAddresses.metadataModulePackage) &&
        Objects.equals(this.royaltyModulePackage, networkConfigurationResponseWellKnownAddresses.royaltyModulePackage) &&
        Objects.equals(this.roleAssignmentModulePackage, networkConfigurationResponseWellKnownAddresses.roleAssignmentModulePackage) &&
        Objects.equals(this.genesisHelperPackage, networkConfigurationResponseWellKnownAddresses.genesisHelperPackage) &&
        Objects.equals(this.faucetPackage, networkConfigurationResponseWellKnownAddresses.faucetPackage) &&
        Objects.equals(this.poolPackage, networkConfigurationResponseWellKnownAddresses.poolPackage) &&
        Objects.equals(this.consensusManager, networkConfigurationResponseWellKnownAddresses.consensusManager) &&
        Objects.equals(this.genesisHelper, networkConfigurationResponseWellKnownAddresses.genesisHelper) &&
        Objects.equals(this.faucet, networkConfigurationResponseWellKnownAddresses.faucet);
  }

  @Override
  public int hashCode() {
    return Objects.hash(xrd, secp256k1SignatureVirtualBadge, ed25519SignatureVirtualBadge, packageOfDirectCallerVirtualBadge, globalCallerVirtualBadge, systemTransactionBadge, packageOwnerBadge, validatorOwnerBadge, accountOwnerBadge, identityOwnerBadge, packagePackage, resourcePackage, accountPackage, identityPackage, consensusManagerPackage, accessControllerPackage, transactionProcessorPackage, metadataModulePackage, royaltyModulePackage, roleAssignmentModulePackage, genesisHelperPackage, faucetPackage, poolPackage, consensusManager, genesisHelper, faucet);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NetworkConfigurationResponseWellKnownAddresses {\n");
    sb.append("    xrd: ").append(toIndentedString(xrd)).append("\n");
    sb.append("    secp256k1SignatureVirtualBadge: ").append(toIndentedString(secp256k1SignatureVirtualBadge)).append("\n");
    sb.append("    ed25519SignatureVirtualBadge: ").append(toIndentedString(ed25519SignatureVirtualBadge)).append("\n");
    sb.append("    packageOfDirectCallerVirtualBadge: ").append(toIndentedString(packageOfDirectCallerVirtualBadge)).append("\n");
    sb.append("    globalCallerVirtualBadge: ").append(toIndentedString(globalCallerVirtualBadge)).append("\n");
    sb.append("    systemTransactionBadge: ").append(toIndentedString(systemTransactionBadge)).append("\n");
    sb.append("    packageOwnerBadge: ").append(toIndentedString(packageOwnerBadge)).append("\n");
    sb.append("    validatorOwnerBadge: ").append(toIndentedString(validatorOwnerBadge)).append("\n");
    sb.append("    accountOwnerBadge: ").append(toIndentedString(accountOwnerBadge)).append("\n");
    sb.append("    identityOwnerBadge: ").append(toIndentedString(identityOwnerBadge)).append("\n");
    sb.append("    packagePackage: ").append(toIndentedString(packagePackage)).append("\n");
    sb.append("    resourcePackage: ").append(toIndentedString(resourcePackage)).append("\n");
    sb.append("    accountPackage: ").append(toIndentedString(accountPackage)).append("\n");
    sb.append("    identityPackage: ").append(toIndentedString(identityPackage)).append("\n");
    sb.append("    consensusManagerPackage: ").append(toIndentedString(consensusManagerPackage)).append("\n");
    sb.append("    accessControllerPackage: ").append(toIndentedString(accessControllerPackage)).append("\n");
    sb.append("    transactionProcessorPackage: ").append(toIndentedString(transactionProcessorPackage)).append("\n");
    sb.append("    metadataModulePackage: ").append(toIndentedString(metadataModulePackage)).append("\n");
    sb.append("    royaltyModulePackage: ").append(toIndentedString(royaltyModulePackage)).append("\n");
    sb.append("    roleAssignmentModulePackage: ").append(toIndentedString(roleAssignmentModulePackage)).append("\n");
    sb.append("    genesisHelperPackage: ").append(toIndentedString(genesisHelperPackage)).append("\n");
    sb.append("    faucetPackage: ").append(toIndentedString(faucetPackage)).append("\n");
    sb.append("    poolPackage: ").append(toIndentedString(poolPackage)).append("\n");
    sb.append("    consensusManager: ").append(toIndentedString(consensusManager)).append("\n");
    sb.append("    genesisHelper: ").append(toIndentedString(genesisHelper)).append("\n");
    sb.append("    faucet: ").append(toIndentedString(faucet)).append("\n");
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

}

