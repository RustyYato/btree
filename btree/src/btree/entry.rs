//! Taken from std, <https://github.com/rust-lang/rust/tree/beae781308e9ddef13074a03faf57ca2fac59a5b/library/alloc/src/collections/btree/map/entry.rs>

use core::fmt::{self, Debug};
use core::marker::PhantomData;
use core::mem;

use Entry::*;

use super::super::borrow::DormantMutRef;
use super::super::node::{Handle, NodeRef, marker};
use super::BTree;
use alloc::alloc::{Allocator, Global};

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`BTreeMap`].
///
/// [`entry`]: BTreeMap::entry
pub enum Entry<'a, K: 'a, V: 'a, A: Allocator + Clone = Global> {
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, A>),

    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, A>),
}

impl<K: Debug, V: Debug, A: Allocator + Clone> Debug for Entry<'_, K, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

/// A view into a vacant entry in a `BTreeMap`.
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, K, V, A: Allocator + Clone = Global> {
    /// `None` for a (empty) map without root
    pub(super) handle: Option<Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>>,
    pub(super) dormant_map: DormantMutRef<'a, BTree<K, V, A>>,

    /// The BTreeMap will outlive this IntoIter so we don't care about drop order for `alloc`.
    pub(super) alloc: A,

    // Be invariant in `K` and `V`
    pub(super) _marker: PhantomData<&'a mut (K, V)>,
}
impl<K, V, A: Allocator + Clone> Debug for VacantEntry<'_, K, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").finish()
    }
}

/// A view into an occupied entry in a `BTreeMap`.
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K, V, A: Allocator + Clone = Global> {
    pub(super) handle: Handle<NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>, marker::KV>,
    pub(super) dormant_map: DormantMutRef<'a, BTree<K, V, A>>,

    /// The BTreeMap will outlive this IntoIter so we don't care about drop order for `alloc`.
    pub(super) alloc: A,

    // Be invariant in `K` and `V`
    pub(super) _marker: PhantomData<&'a mut (K, V)>,
}
impl<K: Debug, V: Debug, A: Allocator + Clone> Debug for OccupiedEntry<'_, K, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}

/// The error returned by [`try_insert`](BTreeMap::try_insert) when the key already exists.
///
/// Contains the occupied entry, key, and the value that was not inserted.
#[non_exhaustive]
pub struct OccupiedError<'a, K: 'a, V: 'a, A: Allocator + Clone = Global> {
    /// The entry in the map that was already occupied.
    pub entry: OccupiedEntry<'a, K, V, A>,
    /// The key which was not inserted, because the entry was already occupied.
    pub key: K,
    /// The value which was not inserted, because the entry was already occupied.
    pub value: V,
}
impl<K: Debug, V: Debug, A: Allocator + Clone> Debug for OccupiedError<'_, K, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedError")
            .field("key", self.entry.key())
            .field("uninserted_key", &self.key)
            .field("old_value", self.entry.get())
            .field("new_value", &self.value)
            .finish()
    }
}

impl<'a, K, V, A: Allocator + Clone> Entry<'a, K, V, A> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// ```
    pub fn or_insert(self, key: K, val: V) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(key, val),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, String> = BTreeMap::new();
    /// let s = "hoho".to_string();
    ///
    /// map.entry("poneyland").or_insert_with(|| s);
    ///
    /// assert_eq!(map["poneyland"], "hoho".to_string());
    /// ```
    pub fn or_insert_with<F: FnOnce() -> (K, V)>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => {
                let (k, v) = default();
                entry.insert(k, v)
            }
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut K, &mut V),
    {
        match self {
            Occupied(mut entry) => {
                let (k, v) = entry.handle.kv_mut();
                f(k, v);
                Occupied(entry)
            }
            Vacant(entry) => Vacant(entry),
        }
    }

    /// Sets the value of the entry, and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, String> = BTreeMap::new();
    /// let entry = map.entry("poneyland").insert_entry("hoho".to_string());
    ///
    /// assert_eq!(entry.key(), &"poneyland");
    /// ```
    #[inline]
    pub fn insert_entry(self, key: K, value: V) -> OccupiedEntry<'a, K, V, A> {
        match self {
            Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Vacant(entry) => entry.insert_entry(key, value),
        }
    }
}

impl<'a, K, V, A: Allocator + Clone> VacantEntry<'a, K, V, A> {
    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, u32> = BTreeMap::new();
    ///
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     o.insert(37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    pub fn insert(self, key: K, value: V) -> &'a mut V {
        self.insert_entry(key, value).into_mut()
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, u32> = BTreeMap::new();
    ///
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     let entry = o.insert_entry(37);
    ///     assert_eq!(entry.get(), &37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    pub fn insert_entry(mut self, key: K, value: V) -> OccupiedEntry<'a, K, V, A> {
        let handle = match self.handle {
            None => {
                // SAFETY: There is no tree yet so no reference to it exists.
                let map = unsafe { self.dormant_map.reborrow() };
                let root = map
                    .root
                    .insert(NodeRef::new_leaf(self.alloc.clone()).forget_type());
                // SAFETY: We *just* created the root as a leaf, and we're
                // stacking the new handle on the original borrow lifetime.
                unsafe {
                    let mut leaf = root.borrow_mut().cast_to_leaf_unchecked();
                    leaf.push_with_handle(key, value)
                }
            }
            Some(handle) => handle.insert_recursing(key, value, self.alloc.clone(), |ins| {
                drop(ins.left);
                // SAFETY: Pushing a new root node doesn't invalidate
                // handles to existing nodes.
                let map = unsafe { self.dormant_map.reborrow() };
                let root = map.root.as_mut().unwrap(); // same as ins.left
                root.push_internal_level(self.alloc.clone())
                    .push(ins.kv.0, ins.kv.1, ins.right)
            }),
        };

        // SAFETY: modifying the length doesn't invalidate handles to existing nodes.
        unsafe { self.dormant_map.reborrow().length += 1 };

        OccupiedEntry {
            handle: handle.forget_node_type(),
            dormant_map: self.dormant_map,
            alloc: self.alloc,
            _marker: PhantomData,
        }
    }
}

impl<'a, K, V, A: Allocator + Clone> OccupiedEntry<'a, K, V, A> {
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[must_use]
    pub fn key(&self) -> &K {
        self.handle.reborrow().into_kv().0
    }

    /// Converts the entry into a reference to its key.
    pub(crate) fn into_key(self) -> &'a K {
        self.handle.into_kv_mut().0
    }

    /// Take ownership of the key and value from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     // We delete the entry from the map.
    ///     o.remove_entry();
    /// }
    ///
    /// // If now try to get the value, it will panic:
    /// // println!("{}", map["poneyland"]);
    /// ```
    pub fn remove_entry(self) -> (K, V) {
        self.remove_kv()
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    #[must_use]
    pub fn get(&self) -> &V {
        self.handle.reborrow().into_kv().1
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` that may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: OccupiedEntry::into_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     *o.get_mut() += 10;
    ///     assert_eq!(*o.get(), 22);
    ///
    ///     // We can use the same Entry multiple times.
    ///     *o.get_mut() += 2;
    /// }
    /// assert_eq!(map["poneyland"], 24);
    /// ```
    pub fn get_mut(&mut self) -> &mut V {
        self.handle.kv_mut().1
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: OccupiedEntry::get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     *o.into_mut() += 10;
    /// }
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_mut(self) -> &'a mut V {
        self.handle.into_val_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key,
    /// and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     assert_eq!(o.insert(15), 12);
    /// }
    /// assert_eq!(map["poneyland"], 15);
    /// ```
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::collections::BTreeMap;
    /// use core::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    /// // If we try to get "poneyland"'s value, it'll panic:
    /// // println!("{}", map["poneyland"]);
    /// ```
    pub fn remove(self) -> V {
        self.remove_kv().1
    }

    // Body of `remove_entry`, probably separate because the name reflects the returned pair.
    pub(super) fn remove_kv(self) -> (K, V) {
        let mut emptied_internal_root = false;
        let (old_kv, _) = self
            .handle
            .remove_kv_tracking(|| emptied_internal_root = true, self.alloc.clone());
        // SAFETY: we consumed the intermediate root borrow, `self.handle`.
        let map = unsafe { self.dormant_map.awaken() };
        map.length -= 1;
        if emptied_internal_root {
            let root = map.root.as_mut().unwrap();
            root.pop_internal_level(self.alloc);
        }
        old_kv
    }
}
