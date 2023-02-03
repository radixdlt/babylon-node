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

use bit_vec::BitVec;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::{ScryptoDecoder, ScryptoEncoder};
use radix_engine_stores::hash_tree::tree_store::{
    Nibble, NibblePath, NodeKey, TreeChildEntry, TreeInternalNode, TreeLeafNode, TreeNode,
};
use sbor::{Decoder, Encoder};
use std::io::{Cursor, Read, Write};
use std::iter;

const NULL_TYPE: u8 = 0;
const LEAF_TYPE: u8 = 1;
const INTERNAL_TYPE: u8 = 2;

pub fn encode_key(key: &NodeKey) -> Vec<u8> {
    let mut writer = Vec::new();
    writer.write_u64::<BigEndian>(key.version()).unwrap();
    write_nibble_path(&mut writer, key.nibble_path());
    writer
}

pub fn encode_node(node: &TreeNode) -> Vec<u8> {
    let mut writer = Vec::new();
    match node {
        TreeNode::Internal(internal) => {
            writer.write_u8(INTERNAL_TYPE).unwrap();
            let mut present_nibbles = NibbleSet::new();
            let mut leaf_nibbles = NibbleSet::new();
            for child in &internal.children {
                let nibble = child.nibble;
                present_nibbles.insert(nibble);
                if child.is_leaf {
                    leaf_nibbles.insert(nibble);
                }
            }
            writer
                .write_u16::<BigEndian>(u16::from(present_nibbles))
                .unwrap();
            writer
                .write_u16::<BigEndian>(u16::from(leaf_nibbles))
                .unwrap();
            for child in &internal.children {
                writer.write_u64::<BigEndian>(child.version).unwrap();
                write_hash(&mut writer, &child.hash);
            }
        }
        TreeNode::Leaf(leaf) => {
            writer.write_u8(LEAF_TYPE).unwrap();
            write_nibble_path(&mut writer, &leaf.key_suffix);
            write_hash(&mut writer, &leaf.value_hash);
            ScryptoEncoder::new(&mut writer)
                .encode(&leaf.substate_id)
                .unwrap();
        }
        TreeNode::Null => {
            writer.write_u8(NULL_TYPE).unwrap();
        }
    };
    writer
}

pub fn decode_tree_node(buffer: &[u8]) -> TreeNode {
    let mut reader = Cursor::new(buffer);
    let type_discriminator = reader.read_u8().unwrap();
    match type_discriminator {
        INTERNAL_TYPE => {
            let present_nibbles = NibbleSet::from(reader.read_u16::<BigEndian>().unwrap());
            let leaf_nibbles = NibbleSet::from(reader.read_u16::<BigEndian>().unwrap());
            let children = present_nibbles
                .to_vec()
                .iter()
                .map(|nibble| {
                    let version = reader.read_u64::<BigEndian>().unwrap();
                    let hash = read_hash(&mut reader);
                    TreeChildEntry {
                        nibble: *nibble,
                        version,
                        hash,
                        is_leaf: leaf_nibbles.contains(nibble),
                    }
                })
                .collect();
            TreeNode::Internal(TreeInternalNode { children })
        }
        LEAF_TYPE => {
            let key_suffix = read_nibble_path(&mut reader);
            let value_hash = read_hash(&mut reader);
            let pos = reader.position() as usize;
            let mut scrypto_decoder = ScryptoDecoder::new(&reader.into_inner()[pos..]);
            let substate_id = scrypto_decoder.decode().unwrap();
            scrypto_decoder.check_end().unwrap();
            TreeNode::Leaf(TreeLeafNode {
                key_suffix,
                substate_id,
                value_hash,
            })
        }
        NULL_TYPE => TreeNode::Null,
        unknown => panic!("unknown node type discriminator: {}", unknown),
    }
}

fn write_nibble_path(writer: &mut Vec<u8>, nibble_path: &NibblePath) {
    writer
        .write_u8(u8::try_from(nibble_path.num_nibbles()).unwrap())
        .unwrap();
    writer.write_all(nibble_path.bytes()).unwrap();
}

fn read_nibble_path(reader: &mut Cursor<&[u8]>) -> NibblePath {
    let num_nibbles = reader.read_u8().unwrap() as usize;
    let mut bytes = vec![0u8; (num_nibbles + 1) / 2];
    reader.read_exact(&mut bytes[..]).unwrap();
    if num_nibbles % 2 == 0 {
        NibblePath::new_even(bytes)
    } else {
        NibblePath::new_odd(bytes)
    }
}

fn write_hash(writer: &mut Vec<u8>, hash: &Hash) {
    writer.write_all(&hash.0).unwrap();
}

fn read_hash(reader: &mut Cursor<&[u8]>) -> Hash {
    let mut bytes = [0u8; Hash::LENGTH];
    reader.read_exact(&mut bytes).unwrap();
    Hash(bytes)
}

struct NibbleSet(BitVec<u16>);

impl NibbleSet {
    pub fn new() -> Self {
        Self(BitVec::from_iter(iter::repeat(false).take(16)))
    }

    pub fn insert(&mut self, nibble: Nibble) {
        self.0.set(u8::from(nibble) as usize, true);
    }

    pub fn contains(&self, nibble: &Nibble) -> bool {
        self.0.get(u8::from(*nibble) as usize).unwrap()
    }

    pub fn to_vec(&self) -> Vec<Nibble> {
        self.0
            .iter()
            .enumerate()
            .filter(|entry| entry.1)
            .map(|entry| Nibble::from(entry.0 as u8))
            .collect()
    }
}

impl From<NibbleSet> for u16 {
    fn from(nibble_set: NibbleSet) -> Self {
        nibble_set.0.storage()[0]
    }
}

impl From<u16> for NibbleSet {
    fn from(bits: u16) -> Self {
        let mut result = NibbleSet(BitVec::default());
        unsafe { result.0.storage_mut().push(bits) }
        result
    }
}
