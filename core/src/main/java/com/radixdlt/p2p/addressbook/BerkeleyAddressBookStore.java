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

package com.radixdlt.p2p.addressbook;

import com.google.common.collect.ImmutableList;
import com.google.inject.Inject;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.serialization.DsonOutput.Output;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.store.NodeStorageLocation;
import com.sleepycat.je.*;
import java.io.File;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.List;
import java.util.Objects;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Persistence for address book entries. */
public final class BerkeleyAddressBookStore implements AddressBookPersistence {
  private static final Logger log = LogManager.getLogger();

  private static final byte[] HIGH_PRIORITY_PEERS_KEY =
      "high_priority_peers".getBytes(StandardCharsets.UTF_8);

  private final Serialization serialization;
  private final Environment dbEnv;
  private final Metrics metrics;
  private Database entriesDb;
  private Database highPriorityPeersDb;

  @Inject
  public BerkeleyAddressBookStore(
      Serialization serialization,
      Metrics metrics,
      @NodeStorageLocation String nodeStorageLocation,
      EnvironmentConfig envConfig) {
    this.serialization = Objects.requireNonNull(serialization);
    this.metrics = Objects.requireNonNull(metrics);

    final var dbHome = new File(nodeStorageLocation, "address_book");
    dbHome.mkdirs();
    this.dbEnv = new Environment(dbHome, envConfig);
    this.open();
  }

  @Override
  public void open() {
    final var config = new DatabaseConfig();
    config.setAllowCreate(true);
    config.setSortedDuplicates(false);

    try {
      this.entriesDb = this.dbEnv.openDatabase(null, "address_book_entries", config);
      this.highPriorityPeersDb =
          this.dbEnv.openDatabase(null, "address_book_high_priority_peers", config);
    } catch (DatabaseException | IllegalArgumentException | IllegalStateException ex) {
      throw new IllegalStateException("while opening database", ex);
    }
  }

  @Override
  public void reset() {
    Transaction transaction = null;

    try {
      transaction =
          this.dbEnv.beginTransaction(null, new TransactionConfig().setReadUncommitted(true));
      this.dbEnv.truncateDatabase(transaction, "address_book_entries", false);
      this.dbEnv.truncateDatabase(transaction, "address_book_high_priority_peers", false);
      transaction.commit();
    } catch (DatabaseNotFoundException dsnfex) {
      if (transaction != null) {
        transaction.abort();
      }
      log.warn(dsnfex.getMessage());
    } catch (Exception ex) {
      if (transaction != null) {
        transaction.abort();
      }
      throw new IllegalStateException("while resetting database", ex);
    }
  }

  @Override
  public void close() {
    if (this.entriesDb != null) {
      this.entriesDb.close();
      this.entriesDb = null;
    }

    if (this.highPriorityPeersDb != null) {
      this.highPriorityPeersDb.close();
      this.highPriorityPeersDb = null;
    }

    dbEnv.close();
  }

  @Override
  public boolean upsertEntry(AddressBookEntry entry) {
    final var start = System.nanoTime();
    try {
      final var key = new DatabaseEntry(entry.getNodeId().getPublicKey().getBytes());
      final var value = new DatabaseEntry(serialization.toDson(entry, Output.PERSIST));

      if (entriesDb.put(null, key, value) == OperationStatus.SUCCESS) {
        addBytesWrite(key.getSize() + value.getSize());
        return true;
      }

      return false;
    } finally {
      addTime(start);
    }
  }

  @Override
  public boolean removeEntry(NodeId nodeId) {
    final var start = System.nanoTime();
    try {
      final var key = new DatabaseEntry(nodeId.getPublicKey().getBytes());

      if (entriesDb.delete(null, key) == OperationStatus.SUCCESS) {
        metrics.berkeleyDb().addressBook().entriesDeleted().inc();
        return true;
      }
      return false;
    } finally {
      addTime(start);
    }
  }

  @Override
  public ImmutableList<AddressBookEntry> getAllEntries() {
    final var start = System.nanoTime();
    try {
      try (var cursor = this.entriesDb.openCursor(null, null)) {
        final var key = new DatabaseEntry();
        final var value = new DatabaseEntry();

        final var builder = ImmutableList.<AddressBookEntry>builder();
        while (cursor.getNext(key, value, LockMode.DEFAULT) == OperationStatus.SUCCESS) {
          addBytesRead(key.getSize() + value.getSize());
          final var entry = serialization.fromDson(value.getData(), AddressBookEntry.class);
          builder.add(entry);
        }

        return builder.build();
      } catch (IOException ex) {
        throw new UncheckedIOException("Error while loading database", ex);
      }
    } finally {
      addTime(start);
    }
  }

  @Override
  public void storeHighPriorityPeers(List<NodeId> ids) {
    final var start = System.nanoTime();

    final var idsBytes = new byte[ids.size() * ECDSASecp256k1PublicKey.COMPRESSED_BYTES];
    for (int i = 0; i < ids.size(); i++) {
      System.arraycopy(
          ids.get(i).getPubKey(),
          0,
          idsBytes,
          i * ECDSASecp256k1PublicKey.COMPRESSED_BYTES,
          ECDSASecp256k1PublicKey.COMPRESSED_BYTES);
    }

    try {
      final var key = new DatabaseEntry(HIGH_PRIORITY_PEERS_KEY);
      final var value = new DatabaseEntry(idsBytes);

      if (highPriorityPeersDb.put(null, key, value) == OperationStatus.SUCCESS) {
        addBytesWrite(key.getSize() + value.getSize());
      } else {
        throw new BerkeleyAddressBookStoreException("Couldn't save high priority peers");
      }
    } finally {
      addTime(start);
    }
  }

  @Override
  public List<NodeId> getHighPriorityPeers() {
    final var start = System.nanoTime();
    try (com.sleepycat.je.Cursor cursor = this.highPriorityPeersDb.openCursor(null, null)) {
      final var pKey = new DatabaseEntry(HIGH_PRIORITY_PEERS_KEY);
      final var value = new DatabaseEntry();
      final var status = cursor.getSearchKey(pKey, value, LockMode.DEFAULT);
      if (status == OperationStatus.SUCCESS) {
        addBytesRead(pKey.getSize() + value.getSize());
        final var builder = ImmutableList.<NodeId>builder();
        int i = 0;
        final var maxIdx = value.getData().length - ECDSASecp256k1PublicKey.COMPRESSED_BYTES;
        while (i <= maxIdx) {
          final var nextIdBytes = new byte[ECDSASecp256k1PublicKey.COMPRESSED_BYTES];
          System.arraycopy(
              value.getData(), i, nextIdBytes, 0, ECDSASecp256k1PublicKey.COMPRESSED_BYTES);
          i += ECDSASecp256k1PublicKey.COMPRESSED_BYTES;
          builder.add(NodeId.fromPublicKey(ECDSASecp256k1PublicKey.fromBytes(nextIdBytes)));
        }
        return builder.build();
      } else if (status == OperationStatus.NOTFOUND) {
        return List.of();
      } else {
        throw new BerkeleyAddressBookStoreException("Couldn't read high priority peers");
      }
    } catch (PublicKeyException e) {
      throw new BerkeleyAddressBookStoreException("Couldn't read high priority peers", e);
    } finally {
      addTime(start);
    }
  }

  private void addTime(long start) {
    final var elapsed = Duration.ofNanos(System.nanoTime() - start);
    this.metrics.berkeleyDb().addressBook().interact().observe(elapsed);
  }

  private void addBytesRead(int bytesRead) {
    this.metrics.berkeleyDb().addressBook().bytesRead().inc(bytesRead);
  }

  private void addBytesWrite(int bytesWrite) {
    this.metrics.berkeleyDb().addressBook().bytesWritten().inc(bytesWrite);
  }

  public static final class BerkeleyAddressBookStoreException extends RuntimeException {
    public BerkeleyAddressBookStoreException(String message) {
      super(message);
    }

    public BerkeleyAddressBookStoreException(String message, Throwable cause) {
      super(message, cause);
    }
  }
}
