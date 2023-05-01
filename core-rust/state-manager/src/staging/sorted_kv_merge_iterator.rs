use std::cmp::Ordering;
use std::iter::Peekable;

/// Merges two ordered (by K) iterators of (K, V) pairs.
/// Preserves the order by interleaving items from both iterators.
/// If the same key is present in both iterators,
/// a pair from the left iterator is returned
/// and a pair from the right iterator is discarded.
pub struct SortedKvMergeIterator<K, V, L, R>
where
    L: Iterator<Item = (K, V)>,
    R: Iterator<Item = L::Item>,
{
    left: Peekable<L>,
    right: Peekable<R>,
}

impl<K, V, L, R> SortedKvMergeIterator<K, V, L, R>
where
    L: Iterator<Item = (K, V)>,
    R: Iterator<Item = L::Item>,
{
    pub fn new(left: L, right: R) -> Self {
        SortedKvMergeIterator {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<K, V, L, R> Iterator for SortedKvMergeIterator<K, V, L, R>
where
    L: Iterator<Item = (K, V)>,
    R: Iterator<Item = L::Item>,
    K: Ord,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<L::Item> {
        match (self.left.peek(), self.right.peek()) {
            (Some((l, _)), Some((r, _))) => match l.cmp(r) {
                Ordering::Less => self.left.next(),
                Ordering::Equal => {
                    let _discarded = self.right.next();
                    self.left.next()
                }
                Ordering::Greater => self.right.next(),
            },
            (Some(_), None) => self.left.next(),
            (None, Some(_)) => self.right.next(),
            (None, None) => None,
        }
    }
}

#[test]
fn test_merge_with_distinct_keys() {
    let left = vec![(1, "a"), (3, "b"), (999, "c")];
    let right = vec![(2, "a"), (998, "b"), (10000, "xyz")];
    let merged: Vec<(i32, &str)> =
        SortedKvMergeIterator::new(left.into_iter(), right.into_iter()).collect();
    assert_eq!(
        merged,
        vec![
            (1, "a"),
            (2, "a"),
            (3, "b"),
            (998, "b"),
            (999, "c"),
            (10000, "xyz")
        ]
    );
}

#[test]
fn test_merge_with_duplicate_keys() {
    let left = vec![
        (vec![0u8], "l0"),
        (vec![1u8, 2u8], "l12"),
        (vec![2u8], "l2"),
        (vec![3u8], "l3"),
    ];
    let right = vec![
        (vec![1u8], "r1"),
        (vec![2u8], "r2"),
        (vec![2u8, 3u8], "r23"),
        (vec![3u8], "r3"),
    ];
    let merged: Vec<(Vec<u8>, &str)> =
        SortedKvMergeIterator::new(left.into_iter(), right.into_iter()).collect();
    assert_eq!(
        merged,
        vec![
            (vec![0u8], "l0"),
            (vec![1u8], "r1"),
            (vec![1u8, 2u8], "l12"),
            (vec![2u8], "l2"),
            (vec![2u8, 3u8], "r23"),
            (vec![3u8], "l3")
        ]
    );
}

#[test]
fn test_no_items_from_right() {
    let left = vec![
        (vec![0u8], "l0"),
        (vec![1u8], "l1"),
        (vec![99u8], "l99"),
        (vec![100u8], "l100"),
    ];
    let right = vec![(vec![99u8], "r99")];
    let merged: Vec<(Vec<u8>, &str)> =
        SortedKvMergeIterator::new(left.clone().into_iter(), right.into_iter()).collect();
    assert_eq!(merged, left);
}
