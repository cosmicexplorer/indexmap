// We *mostly* avoid unsafe code, but `map::core::raw` allows it to use `RawTable` buckets.
#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/indexmap/1/")]
#![no_std]
#![feature(allocator_api)]

//! [`IndexMap`] is a hash table where the iteration order of the key-value
//! pairs is independent of the hash values of the keys.
//!
//! [`IndexSet`] is a corresponding hash set using the same implementation and
//! with similar properties.
//!
//! [`IndexMap`]: map/struct.IndexMap.html
//! [`IndexSet`]: set/struct.IndexSet.html
//!
//!
//! ### Feature Highlights
//!
//! [`IndexMap`] and [`IndexSet`] are drop-in compatible with the std `HashMap`
//! and `HashSet`, but they also have some features of note:
//!
//! - The ordering semantics (see their documentation for details)
//! - Sorting methods and the [`.pop()`][IndexMap::pop] methods.
//! - The [`Equivalent`] trait, which offers more flexible equality definitions
//!   between borrowed and owned versions of keys.
//! - The [`MutableKeys`][map::MutableKeys] trait, which gives opt-in mutable
//!   access to hash map keys.
//!
//! ### Alternate Hashers
//!
//! [`IndexMap`] and [`IndexSet`] have a default hasher type `S = RandomState`,
//! just like the standard `HashMap` and `HashSet`, which is resistant to
//! HashDoS attacks but not the most performant. Type aliases can make it easier
//! to use alternate hashers:
//!
//! ```
//! #![feature(allocator_api)]
//! use fnv::FnvBuildHasher;
//! use fxhash::FxBuildHasher;
//! use indexmap::{IndexMap, IndexSet, alloc_inner::Global};
//!
//! type FnvIndexMap<K, V> = IndexMap<K, V, Global, FnvBuildHasher>;
//! type FnvIndexSet<T> = IndexSet<T, Global, FnvBuildHasher>;
//!
//! type FxIndexMap<K, V> = IndexMap<K, V, Global, FxBuildHasher>;
//! type FxIndexSet<T> = IndexSet<T, Global, FxBuildHasher>;
//!
//! let std: IndexSet<i32> = (0..100).collect();
//! let fnv: FnvIndexSet<i32> = (0..100).collect();
//! let fx: FxIndexSet<i32> = (0..100).collect();
//! assert_eq!(std, fnv);
//! assert_eq!(std, fx);
//! ```
//!
//! ### Rust Version
//!
//! This version of indexmap requires Rust 1.49 or later.
//!
//! The indexmap 1.x release series will use a carefully considered version
//! upgrade policy, where in a later 1.x version, we will raise the minimum
//! required Rust version.
//!
//! ## No Standard Library Targets
//!
//! This crate supports being built without `std`, requiring
//! `alloc` instead. This is enabled automatically when it is detected that
//! `std` is not available. There is no crate feature to enable/disable to
//! trigger this. It can be tested by building for a std-less target.
//!
//! - Creating maps and sets using [`new`][IndexMap::new] and
//! [`with_capacity`][IndexMap::with_capacity] is unavailable without `std`.
//!   Use methods [`IndexMap::default`][def],
//!   [`with_hasher`][IndexMap::with_hasher],
//!   [`with_capacity_and_hasher`][IndexMap::with_capacity_and_hasher] instead.
//!   A no-std compatible hasher will be needed as well, for example
//!   from the crate `twox-hash`.
//! - Macros [`indexmap!`] and [`indexset!`] are unavailable without `std`.
//!
//! [def]: map/struct.IndexMap.html#impl-Default

extern crate alloc;

#[cfg(has_std)]
#[macro_use]
extern crate std;

pub use hashbrown::raw::alloc::{Allocator, Global};

#[cfg(feature = "nightly")]
pub mod alloc_inner {
    pub use super::{Allocator, Global};
    pub use alloc::{
        collections,
        vec::{self, Drain, IntoIter, Vec},
    };
}

#[cfg(not(feature = "nightly"))]
pub mod alloc_inner {
    pub use super::{Allocator, Global};
    pub use alloc::{collections, vec};
    use core::{
        convert::{AsMut, AsRef},
        fmt,
        iter::{FromIterator, IntoIterator, Iterator},
        marker::PhantomData,
        ops::{Deref, DerefMut},
    };

    #[derive(Debug)]
    pub struct Drain<'a, T, Arena: Allocator = Global>(pub vec::Drain<'a, T>, PhantomData<Arena>);
    impl<'a, T, Arena: Allocator> From<Drain<'a, T, Arena>> for vec::Drain<'a, T> {
        fn from(value: Drain<'a, T, Arena>) -> Self {
            value.0
        }
    }
    impl<'a, T, Arena: Allocator> From<vec::Drain<'a, T>> for Drain<'a, T, Arena> {
        fn from(value: vec::Drain<'a, T>) -> Self {
            Self(value, PhantomData)
        }
    }
    impl<'a, T, Arena: Allocator> Deref for Drain<'a, T, Arena> {
        type Target = vec::Drain<'a, T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<'a, T, Arena: Allocator> DerefMut for Drain<'a, T, Arena> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<'a, T, Arena: Allocator> Iterator for Drain<'a, T, Arena> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            self.0.next()
        }
    }

    #[derive(Debug, Clone)]
    pub struct IntoIter<T, Arena: Allocator = Global>(pub vec::IntoIter<T>, PhantomData<Arena>);
    impl<T, Arena: Allocator> From<IntoIter<T, Arena>> for vec::IntoIter<T> {
        fn from(value: IntoIter<T, Arena>) -> Self {
            value.0
        }
    }
    impl<T, Arena: Allocator> From<vec::IntoIter<T>> for IntoIter<T, Arena> {
        fn from(value: vec::IntoIter<T>) -> Self {
            let collected: vec::Vec<_> = value.into_iter().collect();
            Self(collected.into_iter(), PhantomData)
        }
    }
    impl<T, Arena: Allocator> From<Drain<'_, T, Arena>> for IntoIter<T, Arena> {
        fn from(value: Drain<'_, T, Arena>) -> Self {
            let collected: vec::Vec<T> = value.0.into_iter().collect();
            Self(collected.into_iter(), PhantomData)
        }
    }
    impl<T, Arena: Allocator> Deref for IntoIter<T, Arena> {
        type Target = vec::IntoIter<T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<T, Arena: Allocator> DerefMut for IntoIter<T, Arena> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<T, Arena: Allocator> Iterator for IntoIter<T, Arena> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            self.0.next()
        }
    }

    #[derive(Clone)]
    pub struct Vec<T, Arena: Allocator = Global>(pub vec::Vec<T>, PhantomData<Arena>);
    impl<T, Arena: Allocator> Vec<T, Arena> {
        pub fn with_capacity_in(capacity: usize, _arena: Arena) -> Self {
            vec::Vec::with_capacity(capacity).into()
        }
        pub fn new_in(_arena: Arena) -> Self {
            vec::Vec::new().into()
        }
    }
    impl<T, Arena: Allocator> From<Vec<T, Arena>> for vec::Vec<T> {
        fn from(value: Vec<T, Arena>) -> Self {
            value.0
        }
    }
    impl<T, Arena: Allocator> From<vec::Vec<T>> for Vec<T, Arena> {
        fn from(value: vec::Vec<T>) -> Self {
            Self(value, PhantomData)
        }
    }
    impl<T, Arena: Allocator> Deref for Vec<T, Arena> {
        type Target = vec::Vec<T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<T, Arena: Allocator> DerefMut for Vec<T, Arena> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<T, Arena: Allocator> AsRef<vec::Vec<T>> for Vec<T, Arena> {
        fn as_ref(&self) -> &vec::Vec<T> {
            &self.0
        }
    }
    impl<T, Arena: Allocator> AsMut<vec::Vec<T>> for Vec<T, Arena> {
        fn as_mut(&mut self) -> &mut vec::Vec<T> {
            &mut self.0
        }
    }
    impl<T> FromIterator<T> for Vec<T, Global> {
        fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
            vec::Vec::from_iter::<I>(iterable).into()
        }
    }
    impl<T: Eq, Arena: Allocator> PartialEq for Vec<T, Arena> {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq(&other.0)
        }
    }
    impl<T: Eq, Arena: Allocator> Eq for Vec<T, Arena> {}
    impl<T: fmt::Debug, Arena: Allocator> fmt::Debug for Vec<T, Arena> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let slice: &[T] = self.0.as_ref();
            write!(f, "{:?}", slice)
        }
    }
}
pub use alloc_inner::Vec;

#[macro_use]
mod macros;
mod equivalent;
mod mutable_keys;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "serde")]
pub mod serde_seq;
mod util;

pub mod map;
pub mod set;

// Placed after `map` and `set` so new `rayon` methods on the types
// are documented after the "normal" methods.
#[cfg(feature = "rayon")]
mod rayon;

#[cfg(feature = "rustc-rayon")]
mod rustc;

pub use crate::equivalent::Equivalent;
pub use crate::map::IndexMap;
pub use crate::set::IndexSet;

// shared private items

/// Hash value newtype. Not larger than usize, since anything larger
/// isn't used for selecting position anyway.
#[derive(Clone, Copy, Debug, PartialEq)]
struct HashValue(usize);

impl HashValue {
    #[inline(always)]
    fn get(self) -> u64 {
        self.0 as u64
    }
}

#[derive(Copy, Debug)]
struct Bucket<K, V> {
    hash: HashValue,
    key: K,
    value: V,
}

impl<K, V> Clone for Bucket<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Bucket {
            hash: self.hash,
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.hash = other.hash;
        self.key.clone_from(&other.key);
        self.value.clone_from(&other.value);
    }
}

impl<K, V> Bucket<K, V> {
    // field accessors -- used for `f` instead of closures in `.map(f)`
    fn key_ref(&self) -> &K {
        &self.key
    }
    fn value_ref(&self) -> &V {
        &self.value
    }
    fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }
    fn key(self) -> K {
        self.key
    }
    fn value(self) -> V {
        self.value
    }
    fn key_value(self) -> (K, V) {
        (self.key, self.value)
    }
    fn refs(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
    fn ref_mut(&mut self) -> (&K, &mut V) {
        (&self.key, &mut self.value)
    }
    fn muts(&mut self) -> (&mut K, &mut V) {
        (&mut self.key, &mut self.value)
    }
}

trait Entries<Arena: alloc_inner::Allocator> {
    type Entry;
    fn into_entries(self) -> Vec<Self::Entry, Arena>;
    fn as_entries(&self) -> &[Self::Entry];
    fn as_entries_mut(&mut self) -> &mut [Self::Entry];
    fn with_entries<F>(&mut self, f: F)
    where
        F: FnOnce(&mut [Self::Entry]);
}
