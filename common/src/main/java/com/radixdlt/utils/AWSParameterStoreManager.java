/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

package com.radixdlt.utils;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.security.Security;
import java.util.*;
import java.util.zip.GZIPOutputStream;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import software.amazon.awssdk.core.SdkBytes;
import software.amazon.awssdk.regions.Region;
import software.amazon.awssdk.services.ssm.SsmClient;
import software.amazon.awssdk.services.ssm.model.*;

public class AWSParameterStoreManager {

  private AWSParameterStoreManager() {}

  public static void createParameter(
      String parameterName,
      Object parameterValue,
      String network,
      Region region,
      boolean binaryParameter) {
    removeBouncyCastleSecurityProvider();
    try (SsmClient ssmClient = SsmClient.builder().region(region).build()) {
      createNewParameter(ssmClient, "/" + parameterName, parameterValue, network, binaryParameter);
    }
  }

  public static void createParameter(String parameterName, String parameterValue, String network) {
    createParameter(parameterName, parameterValue, network, getRegion(), false);
  }

  public static void createBinaryParameter(
      String parameterName, SdkBytes parameterValue, String network) {
    createParameter(parameterName, parameterValue, network, getRegion(), true);
  }

  public static String getParameter(String parameterName, Region region) {
    removeBouncyCastleSecurityProvider();
    try (SsmClient ssmClient = SsmClient.builder().region(region).build()) {
      return getValue(ssmClient, "/" + parameterName);
    }
  }

  public static String getParameter(String parameterName) {
    return getParameter(parameterName, getRegion());
  }

  public static boolean awsParameterExists(String parameterName) {
    try {
      getParameter(parameterName);
    } catch (ParameterNotFoundException e) {
      return false;
    } catch (SsmException e) {
      throw e;
    }
    return true;
  }

  public static void updateBinaryParameter(
      String parameterName, SdkBytes parameterValue, Region region) {
    removeBouncyCastleSecurityProvider();
    try (SsmClient ssmClient = SsmClient.builder().region(region).build()) {
      updateParameter(
          ssmClient,
          parameterName,
          Base64.getEncoder().encodeToString(parameterValue.asByteArray()));
    }
  }

  public static void updateParameter(String parameterName, String parameterValue, Region region) {
    removeBouncyCastleSecurityProvider();
    try (SsmClient ssmClient = SsmClient.builder().region(region).build()) {
      updateParameter(ssmClient, parameterName, parameterValue);
    }
  }

  public static void updateBinaryParameter(String parameterName, SdkBytes parameterValue) {
    updateBinaryParameter(parameterName, parameterValue, getRegion());
  }

  public static void updateParameter(String parameterName, String parameterValue) {
    updateParameter(parameterName, parameterValue, getRegion());
  }

  public static void createAWSParameter(
      final Map<String, Object> awsParameter,
      final String parameterName,
      final AWSSecretsOutputOptions awsSecretsOutputOptions,
      boolean compress,
      boolean binaryParameter) {
    ObjectMapper objectMapper = new ObjectMapper();

    try {
      String jsonParameter = objectMapper.writeValueAsString(awsParameter);
      if (compress) {
        byte[] compressedBytes = compressData(jsonParameter);
        createBinaryParameter(
            parameterName,
            SdkBytes.fromByteArray(compressedBytes),
            awsSecretsOutputOptions.getNetworkName());
      } else {
        if (binaryParameter) {
          createBinaryParameter(
              parameterName,
              SdkBytes.fromByteArray((byte[]) awsParameter.get("key")),
              awsSecretsOutputOptions.getNetworkName());
        } else {
          createParameter(parameterName, jsonParameter, awsSecretsOutputOptions.getNetworkName());
        }
      }
    } catch (JsonProcessingException e) {
      handleException(e);
    } catch (SsmException e) {
      handleException(e);
    } catch (IOException e) {
      handleException(e);
    }
  }

  public static void updateAWSParameter(
      Map<String, Object> awsParameter,
      String parameterName,
      AWSSecretsOutputOptions awsSecretsOutputOptions,
      boolean compress,
      boolean binaryParameter) {
    ObjectMapper objectMapper = new ObjectMapper();
    if (canBeUpdated(awsSecretsOutputOptions)) {
      System.out.format("Parameter %s exists. And it's going to be replaced %n", parameterName);
      try {
        String jsonParameter = objectMapper.writeValueAsString(awsParameter);
        if (compress) {
          byte[] compressedBytes = compressData(jsonParameter);
          updateBinaryParameter(parameterName, SdkBytes.fromByteArray(compressedBytes));
        } else {
          if (binaryParameter) {
            updateBinaryParameter(
                parameterName, SdkBytes.fromByteArray((byte[]) awsParameter.get("key")));
          } else {
            updateParameter(parameterName, jsonParameter);
          }
        }
      } catch (JsonProcessingException e) {
        handleException(e);
      } catch (SsmException e) {
        handleException(e);
      } catch (IOException e) {
        handleException(e);
      }
    } else {
      System.out.format("Parameter %s exists. It will not be created again %n", parameterName);
    }
  }

  private static void removeBouncyCastleSecurityProvider() {
    Security.removeProvider(BouncyCastleProvider.PROVIDER_NAME);
  }

  private static Region getRegion() {
    return Region.of(Optional.ofNullable(System.getenv("AWS_DEFAULT_REGION")).orElse("eu-west-2"));
  }

  private static void updateParameter(
      SsmClient ssmClient, String parameterName, String parameterValue) {
    PutParameterRequest.Builder parameterRequestBuilder =
        PutParameterRequest.builder()
            .name(parameterName)
            .value(parameterValue)
            .type(ParameterType.SECURE_STRING)
            .overwrite(true);

    ssmClient.putParameter(parameterRequestBuilder.build());
  }

  private static String getValue(SsmClient ssmClient, String parameterName) {
    GetParameterRequest valueRequest =
        GetParameterRequest.builder().name(parameterName).withDecryption(true).build();

    return ssmClient.getParameter(valueRequest).parameter().value();
  }

  private static void createNewParameter(
      SsmClient ssmClient,
      String parameterName,
      Object parameterValue,
      String network,
      boolean binaryParameter) {
    List<Tag> tagList = buildTags(network, parameterName);

    PutParameterRequest.Builder parameterRequestBuilder =
        PutParameterRequest.builder()
            .name(parameterName)
            .description("Validator keys")
            .tags(tagList);

    if (binaryParameter) {
      String encodedValue =
          Base64.getEncoder().encodeToString(((SdkBytes) parameterValue).asByteArray());
      parameterRequestBuilder.value(encodedValue).type(ParameterType.SECURE_STRING);
    } else {
      parameterRequestBuilder.value((String) parameterValue).type(ParameterType.SECURE_STRING);
    }

    ssmClient.putParameter(parameterRequestBuilder.build());
  }

  private static byte[] compressData(String data) throws IOException {
    try (ByteArrayOutputStream bos = new ByteArrayOutputStream(data.length());
        GZIPOutputStream gzip = new GZIPOutputStream(bos)) {
      gzip.write(data.getBytes());
      gzip.finish();
      return bos.toByteArray();
    }
  }

  private static boolean canBeUpdated(final AWSSecretsOutputOptions awsSecretsOutputOptions) {
    return awsSecretsOutputOptions.getRecreateAwsSecrets()
        && !awsSecretsOutputOptions.getNetworkName().equalsIgnoreCase("betanet")
        && !awsSecretsOutputOptions.getNetworkName().equalsIgnoreCase("mainnet");
  }

  private static List<Tag> buildTags(String network, String name) {
    List<Tag> tagList = new ArrayList<>();
    tagList.add(Tag.builder().key("radixdlt:environment-type").value("development").build());
    tagList.add(Tag.builder().key("radixdlt:team").value("devops").build());
    tagList.add(Tag.builder().key("radixdlt:application").value("validator").build());
    tagList.add(Tag.builder().key("radixdlt:name").value(name).build());
    tagList.add(Tag.builder().key("radixdlt:network").value(network).build());
    return tagList;
  }

  private static void handleException(Exception e) {
    System.err.println(e.getMessage());
    System.exit(1);
  }
}
