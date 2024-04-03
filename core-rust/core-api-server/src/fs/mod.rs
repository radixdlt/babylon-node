use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use lru::LruCache;
use radix_engine::types::indexmap::{indexmap, IndexMap};
use radix_engine_common::address::AddressBech32Encoder;
use radix_engine_common::crypto::hash;
use radix_engine_common::network::NetworkDefinition;
use radix_engine_common::types::{EntityType, NodeId, PartitionNumber, SubstateKey};
use radix_engine_queries::typed_substate_layout::{to_typed_substate_key, to_typed_substate_value, TypedSubstateKey};
use radix_engine_store_interface::db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper};
use radix_engine_store_interface::interface::{DbSortKey, SubstateDatabase};
use state_manager::traits::SubstateNodeAncestryRecord;
use state_manager::RocksDBStore;
use utils::hashmap;
use std::cell::RefCell;
use std::cmp;
use std::mem::size_of_val;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, UNIX_EPOCH};

use crate::core_api::{to_api_substate, MappingContext, StateMappingLookups};

const TTL: Duration = Duration::from_secs(1000);

fn mk_file_attr(ino: u64, size: u64, kind: FileType) -> FileAttr {
    let block_size = 512u32;
    let blocks = size.div_ceil(block_size as u64);
    FileAttr {
        ino,
        size,
        blocks,
        atime: UNIX_EPOCH,
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind,
        perm: 0o755,
        nlink: 1,
        uid: 501,
        gid: 20,
        rdev: 0,
        flags: 0,
        blksize: block_size,
    }
}

pub struct FolderFile {
    pub ino: u64,
    pub parent_ino: u64,
    pub children: IndexMap<String, Rc<RefCell<File>>>, // file_name -> file
}

impl FolderFile {
    pub fn new_no_children(ino: u64, parent_ino: u64) -> Self {
        Self {
            ino,
            parent_ino,
            children: indexmap!(),
        }
    }
}

pub struct SubstateContentFile {
    pub ino: u64,
    pub parent_ino: u64,    
    pub node_id: NodeId,
    pub partition_num: PartitionNumber,
    pub db_sort_key: Vec<u8>,
}

pub enum File {
    Folder(FolderFile),
    SubstateContent(SubstateContentFile),
}

impl File {
    pub fn inode(&self) -> u64 {
        match self {
            File::Folder(FolderFile { ino, .. }) => *ino,
            File::SubstateContent(SubstateContentFile { ino, .. }) => *ino,
        }
    }

    pub fn parent_inode(&self) -> u64 {
        match self {
            File::Folder(FolderFile { parent_ino, .. }) => *parent_ino,
            File::SubstateContent(SubstateContentFile { parent_ino, .. }) => *parent_ino,
        }
    }

    pub fn file_type(&self) -> FileType {
        match self {
            File::Folder { .. } => FileType::Directory,
            File::SubstateContent { .. } => FileType::RegularFile,
        }
    }
}

fn get_typed_substate_key(node_id: &NodeId, partition_number: PartitionNumber, db_sort_key: DbSortKey) -> TypedSubstateKey {
    let entity_type = node_id.entity_type().unwrap();
    // TODO: this is wrong, but does the job in most (ar at least some) cases :)
    // Consider:
    // query the blueprint schema of the node and then use that schema info to derive from the partition number what kind of substate keys are under this partition
    std::panic::catch_unwind(|| to_typed_substate_key(entity_type, partition_number, &SubstateKey::Sorted(SpreadPrefixKeyMapper::sorted_from_db_sort_key(&db_sort_key))).unwrap())
        .or(std::panic::catch_unwind(|| to_typed_substate_key(entity_type, partition_number, &SubstateKey::Map(SpreadPrefixKeyMapper::map_from_db_sort_key(&db_sort_key))).unwrap()))
        .or(std::panic::catch_unwind(|| to_typed_substate_key(entity_type, partition_number, &SubstateKey::Field(SpreadPrefixKeyMapper::field_from_db_sort_key(&db_sort_key))).unwrap()))
        .unwrap()
}


struct RadixFS {
    db: RocksDBStore,
    files: HashMap<u64, Rc<RefCell<File>>>,
    content_cache: LruCache<u64, String>,
}

impl RadixFS {
    fn inode_to_attr(&self, ino: &u64) -> Option<FileAttr> {
        let file = self.files.get(&ino)?.clone(); // handle me
        let file_borrowed = file.borrow();
        Some(match file_borrowed.deref() {
            File::Folder { .. } => mk_file_attr(*ino, 0, FileType::Directory),
            // TODO: read actual size (and cache the content?)
            File::SubstateContent { .. } => mk_file_attr(*ino, 1*1024*1024, FileType::RegularFile),
        })
    }
}

impl Filesystem for RadixFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let Some(file) = self.files.get(&parent) else {
            reply.error(ENOENT);
            return;
        };
        let file_borrowed = file.borrow();
        let file = file_borrowed.deref();
        let File::Folder(FolderFile { children, .. }) = file else {
            reply.error(ENOENT);
            return;
        };

        for (child_name, child_file) in children {
            let child_borrowed = child_file.borrow();
            let child_file = child_borrowed.deref();
            if Some(child_name.as_str()) == name.to_str() {
                let attr = self.inode_to_attr(&child_file.inode()).unwrap();
                reply.entry(&TTL, &attr, 0);
                return;
            }
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.inode_to_attr(&ino) {
            Some(attr) => reply.attr(&TTL, &attr),
            None => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {

        if let Some(cached) = self.content_cache.get(&ino) {
            reply.data(&cached.as_bytes()[offset as usize..]);
            return;
        }

        let Some(file) = self.files.get(&ino) else {
            reply.error(ENOENT);
            return;
        };
        let file_borrowed = file.borrow();
        let file = file_borrowed.deref();
        let File::SubstateContent(substate_file) = file else {
            reply.error(ENOENT);
            return;
        };

        let partition_key = SpreadPrefixKeyMapper::to_db_partition_key(&substate_file.node_id, substate_file.partition_num);
        let db_sort_key = DbSortKey(substate_file.db_sort_key.clone());
        let substate_data = self.db.get_substate(&partition_key, &db_sort_key).unwrap();

        let typed_substate_key = get_typed_substate_key(
            &substate_file.node_id,
            substate_file.partition_num,
            db_sort_key);

        let typed_substate_value = to_typed_substate_value(&typed_substate_key, &substate_data).unwrap();

        let mapping_context = MappingContext::new(&NetworkDefinition::mainnet());
        let typed_substate = to_api_substate(
            &mapping_context,
            &StateMappingLookups::default(),
            &typed_substate_key,
            &typed_substate_value,
        ).unwrap();

        let json = serde_json::to_string_pretty(&typed_substate).unwrap();

        let content = format!("
node_id: {:?},
partition_number: {:?},
db_sort_key: {},
typed_key: {:?}

{}        
",
        substate_file.node_id,
        substate_file.partition_num,
        hex::encode(&substate_file.db_sort_key),
        typed_substate_key,
        json);

        reply.data(&content.as_bytes()[offset as usize..]);

        self.content_cache.put(ino, content);

    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let Some(file) = self.files.get(&ino) else {
            reply.error(ENOENT);
            return;
        };
        let file_borrowed = file.borrow();
        let file = file_borrowed.deref();
        
        let File::Folder(FolderFile { children, .. }) = file else {
            reply.error(ENOENT);
            return;
        };

        if offset == 0 {
            if reply.add(file.inode(), 1, FileType::Directory, ".") {
                reply.ok();
                return;
            }
            if reply.add(file.parent_inode(), 2, FileType::Directory, "..") {
                reply.ok();
                return;
            }
        } else if offset == 1 {
            if reply.add(file.parent_inode(), 2, FileType::Directory, "..") {
                reply.ok();
                return;
            }
        }

        let offset_in_children = cmp::max(0, offset - 2);

        for (index, (child_name, child_file)) in children.into_iter().skip(offset_in_children as usize).enumerate() {
            let child_borrowed = child_file.borrow();
            let child_file = child_borrowed.deref();
            let next_entry_offset: i64 = offset_in_children + (index as i64) + 2 + 1;
            if reply.add(child_file.inode(), next_entry_offset, child_file.file_type(), child_name) {
                reply.ok();
                return;
            }
        }

        reply.ok();
    }
}

fn mount_radix_fs(db_path: &str, mount_point: &str) {
    let options = vec![
        MountOption::RO,
        MountOption::FSName("hello".to_string()),
        MountOption::AutoUnmount,
        MountOption::AllowRoot
    ];

    let mut files: HashMap<u64, Rc<RefCell<File>>> = hashmap!();

    let mut root = FolderFile::new_no_children(1, 1);
    let mut accounts = FolderFile::new_no_children(2, root.ino);
    let mut packages = FolderFile::new_no_children(3, root.ino);
    let mut resources = FolderFile::new_no_children(4, root.ino);
    let mut validators = FolderFile::new_no_children(5, root.ino);
    let mut components = FolderFile::new_no_children(6, root.ino);
    let mut pools = FolderFile::new_no_children(7, root.ino);
    let mut other = FolderFile::new_no_children(8, root.ino);

    let mut next_free_inode = 9;

    let db = RocksDBStore::new_read_only(
        PathBuf::from(db_path)
    ).unwrap();

    let ancestry_records: BTreeMap<NodeId, SubstateNodeAncestryRecord> =
        db.get_all_ancestry_records_iter()
            .collect();

    let address_encoder = AddressBech32Encoder::new(&NetworkDefinition::mainnet());

    let mut node_inode_preallocation: HashMap<NodeId, u64> = hashmap!();
    let mut node_children_preallocation: HashMap<NodeId, IndexMap<String, Rc<RefCell<File>>>> = hashmap!();

    let mut node_folders: HashMap<NodeId, Rc<RefCell<File>>> = hashmap!();

    for ((db_partition_key, db_sort_key), _) in db.get_all_substates_iter() {
        let (node_id, partition_number) = SpreadPrefixKeyMapper::from_db_partition_key(&db_partition_key);
        let addr = address_encoder.encode(node_id.as_ref()).unwrap();

        let node_folder: Rc<RefCell<File>> = match node_folders.entry(node_id) {
            std::collections::hash_map::Entry::Occupied(o) => o.get().clone(),
            std::collections::hash_map::Entry::Vacant(v) => {
                let inode = node_inode_preallocation.remove(&node_id)
                    .unwrap_or_else(|| {
                        let new_ino = next_free_inode;
                        next_free_inode += 1;
                        new_ino
                    });

                let children = node_children_preallocation.remove(&node_id)
                    .unwrap_or(indexmap!());

                let node_folder = match ancestry_records.get(&node_id) {
                    Some(ancestry_record) => {
                        // An internal entity
                        // its parent may or may not yet exist

                        let parent_node_id = ancestry_record.parent.0;
                        match node_folders.get(&parent_node_id) {
                            Some(parent_folder) => {
                                let parent_folder = parent_folder.clone();
                                let mut parent_folder = parent_folder.borrow_mut();
                                let parent_folder_deref = parent_folder.deref_mut();

                                // parent folder already exists, all good
                                // add self as the child
                                let node_folder = FolderFile {
                                    ino: inode,
                                    parent_ino: parent_folder_deref.inode(),
                                    children,
                                };

                                let node_folder = Rc::new(RefCell::new(File::Folder(node_folder)));

                                let File::Folder(parent_folder_typed) = parent_folder_deref else {
                                    panic!();
                                };

                                parent_folder_typed.children.insert(addr, node_folder.clone());

                                node_folder
                            }
                            None => {
                                // parent folder doesn't exist, preallocate

                                let parent_inode = match node_inode_preallocation.entry(parent_node_id.clone()) {
                                    std::collections::hash_map::Entry::Occupied(o) => o.get().clone(),
                                    std::collections::hash_map::Entry::Vacant(v) => {
                                        let new_ino = next_free_inode;
                                        next_free_inode += 1;
                                        v.insert(new_ino).clone()
                                    }
                                };

                                let node_folder = FolderFile {
                                    ino: inode,
                                    parent_ino: parent_inode,
                                    children,
                                };

                                let node_folder = Rc::new(RefCell::new(File::Folder(node_folder)));

                                match node_children_preallocation.entry(parent_node_id) {
                                    std::collections::hash_map::Entry::Occupied(mut o) => {
                                        o.get_mut().insert(addr, node_folder.clone());
                                    },
                                    std::collections::hash_map::Entry::Vacant(v) => {
                                        v.insert(indexmap!(addr => node_folder.clone()));
                                    }
                                };
                                node_folder
                            }
                        }
                    },
                    None => {
                        // A global entity

                        let parent_folder = match node_id.entity_type().unwrap() {
                            EntityType::GlobalPackage => &mut packages,
                            EntityType::GlobalConsensusManager => &mut other,
                            EntityType::GlobalValidator => &mut validators,
                            EntityType::GlobalTransactionTracker => &mut other,
                            EntityType::GlobalGenericComponent => &mut components,
                            EntityType::GlobalAccount => &mut accounts,
                            EntityType::GlobalIdentity => &mut other,
                            EntityType::GlobalAccessController => &mut other,
                            EntityType::GlobalOneResourcePool => &mut pools,
                            EntityType::GlobalTwoResourcePool => &mut pools,
                            EntityType::GlobalMultiResourcePool => &mut pools,
                            EntityType::GlobalVirtualSecp256k1Account => &mut accounts,
                            EntityType::GlobalVirtualSecp256k1Identity => &mut other,
                            EntityType::GlobalVirtualEd25519Account => &mut accounts,
                            EntityType::GlobalVirtualEd25519Identity => &mut other,
                            EntityType::GlobalFungibleResourceManager => &mut resources,
                            EntityType::InternalFungibleVault => panic!("shouldn't happen"),
                            EntityType::GlobalNonFungibleResourceManager => &mut resources,
                            EntityType::InternalNonFungibleVault => panic!("shouldn't happen"),
                            EntityType::InternalGenericComponent => panic!("shouldn't happen"),
                            EntityType::InternalKeyValueStore => panic!("shouldn't happen"),
                        };

                        let node_folder = FolderFile {
                            ino: inode,
                            parent_ino: parent_folder.ino,
                            children,
                        };

                        let node_folder = Rc::new(RefCell::new(File::Folder(node_folder)));

                        parent_folder.children.insert(addr, node_folder.clone());

                        v.insert(node_folder).clone()
                    }
                };

                files.insert(inode, node_folder.clone());

                node_folder
            }
        };

        let mut node_folder_borrowed = node_folder.borrow_mut();
        let node_folder_deref = node_folder_borrowed.deref_mut();
        let File::Folder(node_folder_typed) = node_folder_deref else {
            panic!();
        };

        let inode = next_free_inode;
        next_free_inode += 1;

        let mut data_to_hash = vec![];
        data_to_hash.extend_from_slice(&node_id.0);
        data_to_hash.push(partition_number.0);
        data_to_hash.extend_from_slice(&db_sort_key.0);
        let file_name = format!("{}_{}", partition_number.0, hex::encode(&hash(&data_to_hash).0[0..8]));

        let substate_file = SubstateContentFile {
            ino: inode,
            parent_ino: node_folder_typed.ino,
            node_id,
            partition_num: partition_number,
            db_sort_key: db_sort_key.0,
        };

        let substate_file = Rc::new(RefCell::new(File::SubstateContent(substate_file)));

        node_folder_typed.children.insert(file_name, substate_file.clone());

        files.insert(inode, substate_file);
    };

    println!("Added {} nodes", next_free_inode);

    let accounts_inode = accounts.ino;
    let packages_inode = packages.ino;
    let resources_inode = resources.ino;
    let validators_inode = validators.ino;
    let components_inode = components.ino;
    let pools_inode = pools.ino;
    let other_inode = other.ino;

    let accounts = Rc::new(RefCell::new(File::Folder(accounts)));
    let packages = Rc::new(RefCell::new(File::Folder(packages)));
    let resources = Rc::new(RefCell::new(File::Folder(resources)));
    let validators = Rc::new(RefCell::new(File::Folder(validators)));
    let components = Rc::new(RefCell::new(File::Folder(components)));
    let pools = Rc::new(RefCell::new(File::Folder(pools)));
    let other = Rc::new(RefCell::new(File::Folder(other)));

    root.children.insert("accounts".to_string(), accounts.clone());
    root.children.insert("packages".to_string(), packages.clone());
    root.children.insert("resources".to_string(), resources.clone());
    root.children.insert("validators".to_string(), validators.clone());
    root.children.insert("components".to_string(), components.clone());
    root.children.insert("pools".to_string(), pools.clone());
    root.children.insert("other".to_string(), other.clone());

    files.insert(accounts_inode, accounts);
    files.insert(packages_inode, packages);
    files.insert(resources_inode, resources);
    files.insert(validators_inode, validators);
    files.insert(components_inode, components);
    files.insert(pools_inode, pools);
    files.insert(other_inode, other);

    files.insert(root.ino, Rc::new(RefCell::new(File::Folder(root))));

    println!("Files size {}", size_of_val(&files));

    fuser::mount2(
        RadixFS {
            db,
            files,
            content_cache: LruCache::new(NonZeroUsize::new(50).unwrap())
        },
        mount_point.to_string(),
        &options
    ).unwrap();
}

#[test]
fn mount_test() {
    mount_radix_fs("./radix-db/state_manager", "./radix-fs-mount")
}
