use radix_engine::types::*;

/// A type holding its key internally.
/// In the paging use-cases, we list items (e.g. `{id: 7, name: John}`), but we identify a point in
/// the listing by some item's key (e.g. `7`).
pub trait HasKey<K> {
    /// Returns the key.
    fn as_key(&self) -> K;
}

/// A policy for deciding how many items should fit on a page.
/// Each page requires a new instance of [`PagingPolicy`].
pub trait PagingPolicy<I> {
    /// Returns `true` if the scoped page allows to add yet another (given) item.
    ///
    /// After returning `false`, further calls to the same policy (even with different items) are
    /// allowed, but pointless - the policy must keep returning `false`.
    ///
    /// A policy also must always allow the first offered element, since a page with 0 items does
    /// not progress paging (and may only confuse the end-user, who sees an empty page followed by
    /// a continuation token).
    fn still_allows(&mut self, item: &I) -> bool;
}

/// A page of items.
pub struct Page<T, C> {
    /// Items on this page.
    pub items: Vec<T>,
    /// The next continuation token (only present if there are more pages after this one).
    pub continuation_token: Option<C>,
}

impl<T, C> Page<T, C> {
    /// Creates a page of the given items; not continued by default.
    fn of(items: Vec<T>) -> Self {
        Self {
            items,
            continuation_token: None,
        }
    }

    /// Adds a continuation token to this page.
    fn continued(self, continuation_token: C) -> Self {
        Self {
            items: self.items,
            continuation_token: Some(continuation_token),
        }
    }
}

/// A simple pager based on a "next item's key" approach.
pub struct NextKeyPager;

impl NextKeyPager {
    /// Collects a page from the given iterator, honoring the given [`PagingPolicy`].
    /// The returned [`Page::continuation_token`] is simply the key to use for starting the iterator
    /// of the next page.
    pub fn get_page<I: HasKey<K>, K>(
        iterator: impl Iterator<Item = I>,
        mut policy: impl PagingPolicy<I>,
    ) -> Page<I, K> {
        let mut items = Vec::new();
        for item in iterator {
            if !policy.still_allows(&item) {
                return Page::of(items).continued(item.as_key());
            }
            items.push(item);
        }
        Page::of(items)
    }
}

/// A [`PagingPolicy`] allowing some maximum item count.
pub struct MaxItemCountPolicy {
    remaining_item_count: usize,
}

impl MaxItemCountPolicy {
    /// Creates a policy allowing at most the given number of items.
    /// Note: the less-specific return type here is deliberate - referencing `<I>` allows for more
    /// convenient type inference.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<I>(max_item_count: usize) -> impl PagingPolicy<I> {
        Self {
            remaining_item_count: max_item_count,
        }
    }
}

impl<I> PagingPolicy<I> for MaxItemCountPolicy {
    fn still_allows(&mut self, _item: &I) -> bool {
        if self.remaining_item_count == 0 {
            return false;
        }
        self.remaining_item_count -= 1;
        true
    }
}
