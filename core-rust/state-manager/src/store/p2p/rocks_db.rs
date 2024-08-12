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

use std::path::PathBuf;

use rocksdb::{
    ColumnFamilyDescriptor, DB, Options,
};

use crate::engine_prelude::*;
use crate::p2p::address_book_components::AddressBookNodeId;
use crate::p2p::column_families::{AddressBookCf, HighPriorityPeersCf, MigrationStatusCf, SafetyStoreCf};
use crate::p2p::traits::node::{AddressBookStore, HighPriorityPeersStore, MigrationStore, SafetyStateStore};
use crate::store::common::rocks_db::*;
use crate::store::p2p::migration::{MigrationId, MigrationStatus};
use crate::store::common::typed_cf_api::*;

/// A listing of all column family names used by the Node.
///
/// This is directly needed to initialize the column families within the DB, but is also a nice
/// place to link to all of them (please see the documentation of each CF to learn about its
/// business purpose and DB schema) and to put the important general notes regarding all of them
/// (see below).
///
/// **Note on the key encoding used throughout all column families:**
/// We often rely on the RocksDB's unsurprising ability to efficiently list entries sorted
/// lexicographically by key. For this reason, our byte-level encoding of certain keys (e.g.
/// [`StateVersion`]) needs to reflect the business-level ordering of the represented concept (i.e.
/// since state versions grow, the "last" state version must have a lexicographically greatest key,
/// which means that we need to use a constant-length big-endian integer encoding).
///
/// **Note on the name strings:**
/// The `NAME` constants defined by `*Cf` structs (and referenced below) are used as database column
/// family names. Any change would effectively mean a ledger wipe. For this reason, we choose to
/// define them manually (rather than using the `Into<String>`, which is refactor-sensitive).

const ALL_NODE_COLUMN_FAMILIES: [&str; 4] = [
    MigrationStatusCf::DEFAULT_NAME,
    AddressBookCf::DEFAULT_NAME,
    SafetyStoreCf::DEFAULT_NAME,
    HighPriorityPeersCf::DEFAULT_NAME,
];

pub type ActualNodeDatabase = NodeDatabase<DirectRocks>;

/// A RocksDB-backed persistence layer for node-specific address book and safety store.
pub struct NodeDatabase<R> {
    /// Underlying RocksDB instance.
    rocks: R,
}

impl ActualNodeDatabase {
    pub fn new(
        root_path: PathBuf,
    ) -> ActualNodeDatabase {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_NODE_COLUMN_FAMILIES
            .iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&db_opts, root_path.as_path(), column_families).unwrap();

        NodeDatabase {
            rocks: DirectRocks { db },
        }
    }
}

impl<R: WriteableRocks> NodeDatabase<R> {
    fn open_rw_context(&self) -> TypedDbContext<R, BufferedWriteSupport<R>> {
        TypedDbContext::new(&self.rocks, BufferedWriteSupport::new(&self.rocks))
    }
}

impl<R: WriteableRocks> AddressBookStore for NodeDatabase<R> {
    fn remove_one(&self, node_id: &AddressBookNodeId) -> bool {
        let binding = self.open_rw_context();
        let context = binding.cf(AddressBookCf);

        if context.get(node_id).is_some() {
            context.delete(node_id);
        }

        true
    }

    fn upsert_one(&self, node_id: &AddressBookNodeId, entry: &[u8]) -> bool {
        let binding = self.open_rw_context();
        let context = binding.cf(AddressBookCf);

        context.put(node_id, &entry.to_vec());

        true
    }

    fn reset(&self) {
        self.open_rw_context().cf(AddressBookCf).delete_all();
    }

    fn get_all(&self) -> Vec<Vec<u8>> {
        self.open_rw_context()
            .cf(AddressBookCf)
            .get_all()
    }
}

impl<R: WriteableRocks> HighPriorityPeersStore for NodeDatabase<R> {
    fn upsert_all_peers(&self, peers: &[u8]) {
        self.open_rw_context().cf(HighPriorityPeersCf).put(&(), &peers.to_vec());
    }

    fn get_all_peers(&self) -> Option<Vec<u8>> {
        self.open_rw_context().cf(HighPriorityPeersCf)
            .get(&())
    }

    fn reset_high_priority_peers(&self) {
        self.open_rw_context().cf(HighPriorityPeersCf).delete(&());
    }
}

impl<R: WriteableRocks> SafetyStateStore for NodeDatabase<R> {
    fn upsert_safety_state(&self, safety_state: &[u8]) {
        self.open_rw_context().cf(SafetyStoreCf).put(&(), &safety_state.to_vec());
    }

    fn get_safety_state(&self) -> Option<Vec<u8>> {
        self.open_rw_context().cf(SafetyStoreCf)
            .get(&())
    }
}

impl<R: WriteableRocks> MigrationStore for NodeDatabase<R> {
    fn is_migration_done(&self, store_id: MigrationId) -> bool {
        self.open_rw_context().cf(MigrationStatusCf)
            .get(&store_id).is_some()
    }

    fn migration_done(&self, store_id: MigrationId) {
        self.open_rw_context().cf(MigrationStatusCf)
            .put(&store_id, &MigrationStatus::Completed)
    }
}
