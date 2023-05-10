use radix_engine_store_interface::interface::{DatabaseUpdate, DbSortKey, PartitionEntry};
use std::cmp::Ordering;
use std::iter::Peekable;

pub struct SubstateOverlayIterator<'a> {
    root_db: Peekable<Box<dyn Iterator<Item = PartitionEntry> + 'a>>,
    overlay: Peekable<Box<dyn Iterator<Item = (DbSortKey, DatabaseUpdate)> + 'a>>,
}

impl<'a> SubstateOverlayIterator<'a> {
    pub fn new(
        root_db: Peekable<Box<dyn Iterator<Item = PartitionEntry> + 'a>>,
        overlay: Peekable<Box<dyn Iterator<Item = (DbSortKey, DatabaseUpdate)> + 'a>>,
    ) -> Self {
        Self { root_db, overlay }
    }
}

impl<'a> Iterator for SubstateOverlayIterator<'a> {
    type Item = PartitionEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.root_db.peek(), self.overlay.peek()) {
            (Some((l, _)), Some((r, _))) => match l.cmp(r) {
                Ordering::Less => self.root_db.next(),
                Ordering::Equal => {
                    let _discarded_root_entry = self.root_db.next();
                    let overlay_entry = self.overlay.next().unwrap();
                    match overlay_entry.1 {
                        DatabaseUpdate::Set(value) => Some((overlay_entry.0, value)),
                        DatabaseUpdate::Delete => self.next(),
                    }
                }
                Ordering::Greater => {
                    let next = self.overlay.next().unwrap();
                    match next.1 {
                        DatabaseUpdate::Set(value) => Some((next.0, value)),
                        DatabaseUpdate::Delete => self.next(),
                    }
                }
            },
            (None, Some(_)) => {
                let mut next = self.overlay.next();
                while let Some((k, database_update)) = &next {
                    match database_update {
                        DatabaseUpdate::Set(value) => return Some((k.clone(), value.clone())),
                        DatabaseUpdate::Delete => {
                            next = self.overlay.next();
                        }
                    }
                }
                None
            }
            (Some(_), None) => self.root_db.next(),
            (None, None) => None,
        }
    }
}
