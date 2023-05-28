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

package com.radixdlt.rev2;

import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.lang.Functions;
import com.radixdlt.transaction.PreparedNotarizedTransaction;
import com.radixdlt.transaction.TransactionPreparer;
import com.radixdlt.utils.PrivateKeys;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.IntStream;

/*
Note - this builder is stateful, not immutable
*/
public class TransactionBuilder {
  public static final ECKeyPair DEFAULT_NOTARY = generateKeyPair(1);

  public static final AtomicInteger NONCE = new AtomicInteger(0);

  private final NetworkDefinition networkDefinition;
  private long fromEpoch;
  private int numEpochsValidFor;
  private int nonce;
  private ECKeyPair notary;
  private boolean notaryIsSignatory;
  private String manifest;
  private List<byte[]> blobs;
  private List<ECKeyPair> signatories;

  public TransactionBuilder(NetworkDefinition networkDefinition) {
    this.networkDefinition = networkDefinition;
    this.fromEpoch = 0;
    this.numEpochsValidFor = 100;
    this.nonce = NONCE.getAndIncrement();
    this.notary = DEFAULT_NOTARY;
    this.notaryIsSignatory = false;
    this.blobs = List.of();
    this.signatories = List.of();
    // Set a valid manifest
    this.manifest(Manifest.valid());
  }

  public static TransactionBuilder forTests() {
    return new TransactionBuilder(NetworkDefinition.INT_TEST_NET);
  }

  public static TransactionBuilder forNetwork(NetworkDefinition networkDefinition) {
    return new TransactionBuilder(networkDefinition);
  }

  public static ECKeyPair generateKeyPair(int keySource) {
    return PrivateKeys.numeric(keySource).findFirst().orElseThrow();
  }

  public TransactionBuilder manifest(String manifest) {
    this.manifest = manifest;
    return this;
  }

  public TransactionBuilder manifest(
      Functions.Func1<Manifest.Parameters, String> manifestConstructor) {
    this.manifest = manifestConstructor.apply(new Manifest.Parameters(this.networkDefinition));
    return this;
  }

  public TransactionBuilder fromEpoch(long fromEpoch) {
    this.fromEpoch = fromEpoch;
    return this;
  }

  public TransactionBuilder numEpochsValidFor(int numEpochsValidFor) {
    this.numEpochsValidFor = numEpochsValidFor;
    return this;
  }

  public TransactionBuilder nonce(int nonce) {
    this.nonce = nonce;
    return this;
  }

  public TransactionBuilder notary(ECKeyPair notary) {
    this.notary = notary;
    return this;
  }

  public TransactionBuilder notaryIsSignatory(boolean notaryIsSignatory) {
    this.notaryIsSignatory = notaryIsSignatory;
    return this;
  }

  public TransactionBuilder blobs(List<byte[]> blobs) {
    this.blobs = blobs;
    return this;
  }

  public TransactionBuilder signatories(List<ECKeyPair> signatories) {
    this.signatories = signatories;
    return this;
  }

  public TransactionBuilder signatories(int number) {
    this.signatories = createNumericSignatories(number);
    return this;
  }

  public PreparedNotarizedTransaction prepare() {
    var header =
        TransactionHeader.defaults(
            this.networkDefinition,
            this.fromEpoch,
            this.numEpochsValidFor,
            this.nonce,
            this.notary.getPublicKey().toPublicKey(),
            this.notaryIsSignatory);
    var intent =
        TransactionPreparer.prepareIntent(networkDefinition, header, this.manifest, this.blobs);
    var intentSignatures =
        this.signatories.stream()
            .map(ecKeyPair -> ecKeyPair.sign(intent.intentHash()).toSignatureWithPublicKey())
            .toList();
    var signedIntent =
        TransactionPreparer.prepareSignedIntent(intent.intentBytes(), intentSignatures);
    var notarySignature = this.notary.sign(signedIntent.signedIntentHash()).toSignature();
    return TransactionPreparer.prepareNotarizedTransaction(
        signedIntent.signedIntentBytes(), notarySignature);
  }

  public static List<ECKeyPair> createNumericSignatories(int num) {
    return IntStream.rangeClosed(1, num).mapToObj(PrivateKeys::ofNumeric).toList();
  }
}
