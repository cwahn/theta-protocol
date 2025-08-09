// Use std::HashMap if std feature is enabled, else but only alloc, use hashbrown

use rustc_hash::FxHasher;

// HashMap

#[cfg(feature = "std")]
pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V, FxHasher>;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V, FxHasher>;

// HashTable

#[cfg(feature = "std")]
pub(crate) type HashSet<T> = std::collections::HashSet<T, FxHasher>;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub(crate) type HashSet<T> = hashbrown::HashSet<T, FxHasher>;
