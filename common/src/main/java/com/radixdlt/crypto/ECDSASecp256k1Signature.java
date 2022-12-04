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

package com.radixdlt.crypto;

import static java.util.Objects.requireNonNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.CustomTypeKnownLengthCodec;
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.DsonOutput.Output;
import com.radixdlt.serialization.SerializerConstants;
import com.radixdlt.serialization.SerializerDummy;
import com.radixdlt.serialization.SerializerId2;
import com.radixdlt.utils.Bytes;
import java.io.IOException;
import java.math.BigInteger;
import java.util.Arrays;
import java.util.Objects;
import org.bouncycastle.asn1.ASN1InputStream;
import org.bouncycastle.asn1.ASN1Integer;
import org.bouncycastle.asn1.DLSequence;
import org.bouncycastle.util.encoders.Hex;

/**
 * An <a href="https://en.wikipedia.org/wiki/ Elliptic_Curve_Digital_Signature_Algorithm">ECDSA</a>
 * signature represented as a tuple of {@link BigInteger}s {@code (R, S)}/
 */
@SerializerId2("sig")
public final class ECDSASecp256k1Signature {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        ECDSASecp256k1Signature.class,
        codecs ->
            new CustomTypeKnownLengthCodec<>(
                TypeId.TYPE_CUSTOM_ECDSA_SECP256K1_SIGNATURE,
                COMPRESSED_BYTE_LENGTH,
                ECDSASecp256k1Signature::getConcatRecoveryRSBytes,
                ECDSASecp256k1Signature::decodeFromConcatRecoveryRSBytes));
  }

  public static final int COMPRESSED_BYTE_LENGTH = 65; // 32 + 32 + header byte

  // Placeholder for the serializer ID
  @JsonProperty(SerializerConstants.SERIALIZER_NAME)
  @DsonOutput(Output.ALL)
  private SerializerDummy serializer = SerializerDummy.DUMMY;

  /* The two components of the signature. */
  private final BigInteger r;
  private final BigInteger s;
  private final byte v;

  private static final ECDSASecp256k1Signature ZERO_SIGNATURE =
      new ECDSASecp256k1Signature(BigInteger.ZERO, BigInteger.ZERO, 0);

  private ECDSASecp256k1Signature(BigInteger r, BigInteger s, int v) {
    this.r = requireNonNull(r);
    this.s = requireNonNull(s);
    this.v = ((v & 1) == 0 ? (byte) 0x00 : (byte) 0x01);
  }

  @JsonCreator
  public static ECDSASecp256k1Signature deserialize(
      @JsonProperty(value = "r", required = true) byte[] r,
      @JsonProperty(value = "s", required = true) byte[] s,
      @JsonProperty(value = "v", required = true) int v) {
    return create(new BigInteger(1, requireNonNull(r)), new BigInteger(1, requireNonNull(s)), v);
  }

  /**
   * Constructs a signature with the given components. Does NOT automatically canonicalise the
   * signature.
   */
  public static ECDSASecp256k1Signature create(BigInteger r, BigInteger s, int v) {
    requireNonNull(r);
    requireNonNull(s);

    return new ECDSASecp256k1Signature(r, s, v);
  }

  public static ECDSASecp256k1Signature zeroSignature() {
    return ZERO_SIGNATURE;
  }

  public BigInteger getR() {
    return r;
  }

  public BigInteger getS() {
    return s;
  }

  public byte getV() {
    return v;
  }

  @JsonProperty("r")
  @DsonOutput(Output.ALL)
  private byte[] getJsonR() {
    return Bytes.trimLeadingZeros(r.toByteArray());
  }

  @JsonProperty("s")
  @DsonOutput(Output.ALL)
  private byte[] getJsonS() {
    return Bytes.trimLeadingZeros(s.toByteArray());
  }

  @JsonProperty("v")
  @DsonOutput(Output.ALL)
  private int getJsonV() {
    return v;
  }

  @Override
  public String toString() {
    return toHexString();
  }

  public byte[] getConcatRSBytes() {
    return com.google.common.primitives.Bytes.concat(
        Bytes.bigIntegerToBytes(r, 32), Bytes.bigIntegerToBytes(s, 32));
  }

  public byte[] getConcatRecoveryRSBytes() {
    return com.google.common.primitives.Bytes.concat(
        new byte[] {v}, Bytes.bigIntegerToBytes(r, 32), Bytes.bigIntegerToBytes(s, 32));
  }

  public String toHexString() {
    return Bytes.toHexString(getConcatRSBytes());
  }

  public Signature toSignature() {
    return new Signature.EcdsaSecp256k1(this);
  }

  public SignatureWithPublicKey toSignatureWithPublicKey() {
    return new SignatureWithPublicKey.EcdsaSecp256k1(this);
  }

  @Override
  public boolean equals(Object o) {
    if (o == this) {
      return true;
    }

    return (o instanceof ECDSASecp256k1Signature signature)
        && Objects.equals(r, signature.r)
        && Objects.equals(s, signature.s)
        && Objects.equals(v, signature.v);
  }

  @Override
  public int hashCode() {
    return Objects.hash(r, s, v);
  }

  // WARNING: Never ever use this method to restore recoverable signature! It misses 'v' bit
  // necessary for recovery.
  public static ECDSASecp256k1Signature decodeFromHexDer(String input) {
    return decodeFromDER(Hex.decode(input));
  }

  public static ECDSASecp256k1Signature decodeFromDER(byte[] bytes) {
    try (ASN1InputStream decoder = new ASN1InputStream(bytes)) {
      var seq = (DLSequence) decoder.readObject();
      var r = (ASN1Integer) seq.getObjectAt(0);
      var s = (ASN1Integer) seq.getObjectAt(1);

      return new ECDSASecp256k1Signature(r.getPositiveValue(), s.getPositiveValue(), 0);
    } catch (IOException e) {
      throw new IllegalArgumentException("Failed to read bytes as ASN1 decode bytes", e);
    } catch (ClassCastException e) {
      throw new IllegalArgumentException("Failed to cast to ASN1Integer", e);
    }
  }

  public static ECDSASecp256k1Signature decodeFromConcatRecoveryRSBytes(byte[] bytes) {
    var v = bytes[0];
    var rBytes = Arrays.copyOfRange(bytes, 1, 33);
    var sBytes = Arrays.copyOfRange(bytes, 33, 65);
    return new ECDSASecp256k1Signature(
        Bytes.bytesToBigInteger(rBytes), Bytes.bytesToBigInteger(sBytes), v);
  }
}
