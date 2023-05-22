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

package com.radixdlt.genesis.olympia.bech32;

import static com.google.common.base.Preconditions.checkArgument;

import java.util.Arrays;
import java.util.Locale;

/**
 * Implementation from
 * https://github.com/bitcoinj/bitcoinj/blob/4dea86a0151e947cd46b2c5f38d15464059f18f5/core/src/main/java/org/bitcoinj/base/Bech32.java
 * Licensed under the Apache License, Version 2.0 (the "License").
 *
 * <p>Implementation of the Bech32 encoding.
 *
 * <p>See <a href="https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki">BIP350</a> and <a
 * href="https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki">BIP173</a> for details.
 */
public final class OlympiaBech32 {
  /** The Bech32 character set for encoding. */
  private static final String CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

  /** The Bech32 character set for decoding. */
  private static final byte[] CHARSET_REV = {
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    15, -1, 10, 17, 21, 20, 26, 30, 7, 5, -1, -1, -1, -1, -1, -1,
    -1, 29, -1, 24, 13, 25, 9, 8, 23, -1, 18, 22, 31, 27, 19, -1,
    1, 0, 3, 16, 11, 28, 12, 14, 6, 4, 2, -1, -1, -1, -1, -1,
    -1, 29, -1, 24, 13, 25, 9, 8, 23, -1, 18, 22, 31, 27, 19, -1,
    1, 0, 3, 16, 11, 28, 12, 14, 6, 4, 2, -1, -1, -1, -1, -1
  };

  public static class Bech32Data {
    public final String hrp;
    public final byte[] data;

    private Bech32Data(final String hrp, final byte[] data) {
      this.hrp = hrp;
      this.data = data;
    }
  }

  /** Find the polynomial with value coefficients mod the generator as 30-bit. */
  private static int polymod(final byte[] values) {
    int c = 1;
    for (byte v_i : values) {
      int c0 = (c >>> 25) & 0xff;
      c = ((c & 0x1ffffff) << 5) ^ (v_i & 0xff);
      if ((c0 & 1) != 0) c ^= 0x3b6a57b2;
      if ((c0 & 2) != 0) c ^= 0x26508e6d;
      if ((c0 & 4) != 0) c ^= 0x1ea119fa;
      if ((c0 & 8) != 0) c ^= 0x3d4233dd;
      if ((c0 & 16) != 0) c ^= 0x2a1462b3;
    }
    return c;
  }

  /** Expand a HRP for use in checksum computation. */
  private static byte[] expandHrp(final String hrp) {
    int hrpLength = hrp.length();
    byte ret[] = new byte[hrpLength * 2 + 1];
    for (int i = 0; i < hrpLength; ++i) {
      int c = hrp.charAt(i) & 0x7f; // Limit to standard 7-bit ASCII
      ret[i] = (byte) ((c >>> 5) & 0x07);
      ret[i + hrpLength + 1] = (byte) (c & 0x1f);
    }
    ret[hrpLength] = 0;
    return ret;
  }

  /** Verify a checksum. */
  private static boolean verifyChecksum(final String hrp, final byte[] values) {
    byte[] hrpExpanded = expandHrp(hrp);
    byte[] combined = new byte[hrpExpanded.length + values.length];
    System.arraycopy(hrpExpanded, 0, combined, 0, hrpExpanded.length);
    System.arraycopy(values, 0, combined, hrpExpanded.length, values.length);
    return polymod(combined) == 1;
  }

  /** Create a checksum. */
  private static byte[] createChecksum(final String hrp, final byte[] values) {
    byte[] hrpExpanded = expandHrp(hrp);
    byte[] enc = new byte[hrpExpanded.length + values.length + 6];
    System.arraycopy(hrpExpanded, 0, enc, 0, hrpExpanded.length);
    System.arraycopy(values, 0, enc, hrpExpanded.length, values.length);
    int mod = polymod(enc) ^ 1;
    byte[] ret = new byte[6];
    for (int i = 0; i < 6; ++i) {
      ret[i] = (byte) ((mod >>> (5 * (5 - i))) & 31);
    }
    return ret;
  }

  /** Encode a Bech32 string. */
  public static String encode(final Bech32Data bech32) {
    return encode(bech32.hrp, bech32.data);
  }

  /** Encode a Bech32 string. */
  public static String encode(String hrp, final byte[] values) {
    checkArgument(hrp.length() >= 1, "Human-readable part is too short");
    checkArgument(hrp.length() <= 83, "Human-readable part is too long");
    hrp = hrp.toLowerCase(Locale.ROOT);
    byte[] checksum = createChecksum(hrp, values);
    byte[] combined = new byte[values.length + checksum.length];
    System.arraycopy(values, 0, combined, 0, values.length);
    System.arraycopy(checksum, 0, combined, values.length, checksum.length);
    StringBuilder sb = new StringBuilder(hrp.length() + 1 + combined.length);
    sb.append(hrp);
    sb.append('1');
    for (byte b : combined) {
      sb.append(CHARSET.charAt(b));
    }
    return sb.toString();
  }

  /** Decode a Bech32 string. */
  public static Bech32Data decode(final String str) throws AddressFormatException {
    boolean lower = false, upper = false;
    if (str.length() < 8)
      throw new AddressFormatException.InvalidDataLength("Input too short: " + str.length());
    if (str.length() > 90)
      throw new AddressFormatException.InvalidDataLength("Input too long: " + str.length());
    for (int i = 0; i < str.length(); ++i) {
      char c = str.charAt(i);
      if (c < 33 || c > 126) throw new AddressFormatException.InvalidCharacter(c, i);
      if (c >= 'a' && c <= 'z') {
        if (upper) throw new AddressFormatException.InvalidCharacter(c, i);
        lower = true;
      }
      if (c >= 'A' && c <= 'Z') {
        if (lower) throw new AddressFormatException.InvalidCharacter(c, i);
        upper = true;
      }
    }
    final int pos = str.lastIndexOf('1');
    if (pos < 1) throw new AddressFormatException.InvalidPrefix("Missing human-readable part");
    final int dataPartLength = str.length() - 1 - pos;
    if (dataPartLength < 6)
      throw new AddressFormatException.InvalidDataLength("Data part too short: " + dataPartLength);
    byte[] values = new byte[dataPartLength];
    for (int i = 0; i < dataPartLength; ++i) {
      char c = str.charAt(i + pos + 1);
      if (CHARSET_REV[c] == -1) throw new AddressFormatException.InvalidCharacter(c, i + pos + 1);
      values[i] = CHARSET_REV[c];
    }
    String hrp = str.substring(0, pos).toLowerCase(Locale.ROOT);
    if (!verifyChecksum(hrp, values)) throw new AddressFormatException.InvalidChecksum();
    return new Bech32Data(hrp, Arrays.copyOfRange(values, 0, values.length - 6));
  }
}
