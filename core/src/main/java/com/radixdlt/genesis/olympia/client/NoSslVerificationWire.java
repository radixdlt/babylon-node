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

package com.radixdlt.genesis.olympia.client;

import com.jcabi.aspects.Immutable;
import com.jcabi.http.Request;
import com.jcabi.http.Response;
import com.jcabi.http.Wire;
import java.io.IOException;
import java.io.InputStream;
import java.net.Socket;
import java.security.KeyManagementException;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.security.cert.X509Certificate;
import java.util.Collection;
import java.util.Map;
import javax.net.ssl.*;

/**
 * Wire that completely skips SSL verification (including hostname verification).
 *
 * <p>Based on
 * https://github.com/jcabi/jcabi-http/blob/master/src/main/java/com/jcabi/http/wire/TrustedWire.java
 * with modifications.
 *
 * <p>Copyright (c) 2011-2022, jcabi.com All rights reserved.
 *
 * <p>Redistribution and use in source and binary forms, with or without modification, are permitted
 * provided that the following conditions are met: 1) Redistributions of source code must retain the
 * above copyright notice, this list of conditions and the following disclaimer. 2) Redistributions
 * in binary form must reproduce the above copyright notice, this list of conditions and the
 * following disclaimer in the documentation and/or other materials provided with the distribution.
 * 3) Neither the name of the jcabi.com nor the names of its contributors may be used to endorse or
 * promote products derived from this software without specific prior written permission.
 *
 * <p>THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
 * IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
 * FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY
 * WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
@Immutable
public final class NoSslVerificationWire implements Wire {
  private static final TrustManager MANAGER =
      new X509ExtendedTrustManager() {
        @Override
        public void checkClientTrusted(X509Certificate[] chain, String authType, Socket socket) {}

        @Override
        public void checkServerTrusted(X509Certificate[] chain, String authType, Socket socket) {}

        @Override
        public void checkClientTrusted(
            X509Certificate[] chain, String authType, SSLEngine engine) {}

        @Override
        public void checkServerTrusted(
            X509Certificate[] chain, String authType, SSLEngine engine) {}

        @Override
        public java.security.cert.X509Certificate[] getAcceptedIssuers() {
          return null;
        }

        @Override
        public void checkClientTrusted(X509Certificate[] certs, String authType) {}

        @Override
        public void checkServerTrusted(X509Certificate[] certs, String authType) {}
      };

  private final transient Wire origin;

  public NoSslVerificationWire(final Wire wire) {
    this.origin = wire;
  }

  @Override
  public Response send(
      final Request req,
      final String home,
      final String method,
      final Collection<Map.Entry<String, String>> headers,
      final InputStream content,
      final int connect,
      final int read)
      throws IOException {
    synchronized (NoSslVerificationWire.class) {
      final SSLSocketFactory def = HttpsURLConnection.getDefaultSSLSocketFactory();
      try {
        HttpsURLConnection.setDefaultSSLSocketFactory(
            NoSslVerificationWire.context().getSocketFactory());
        return this.origin.send(req, home, method, headers, content, connect, read);
      } finally {
        HttpsURLConnection.setDefaultSSLSocketFactory(def);
      }
    }
  }

  private static SSLContext context() {
    try {
      final SSLContext ctx = SSLContext.getInstance("SSL");
      ctx.init(null, new TrustManager[] {NoSslVerificationWire.MANAGER}, new SecureRandom());
      return ctx;
    } catch (final KeyManagementException | NoSuchAlgorithmException ex) {
      throw new IllegalStateException(ex);
    }
  }
}
