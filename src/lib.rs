//!
//! Simple crate to allow dynamic cache implementations
//! to be used as struct or function parameters.
//!
//! The lib is pretty simple, you just need to create a column definition
//! and an instance of [`Engine`] with whatever [`CacheStorage`] you want.
//!
//! There are currently implementations for [`sled`] and [`redis`]
//!
//! ```
//! use omega_cache::{Engine, CacheStorage, noop_engine::NoopEngine, ColumnDefinition, CacheError};
//!
//! struct ShortLivedColumn {}
//!
//! impl ColumnDefinition for ShortLivedColumn {
//!     fn name(&self) -> String {
//!         "short_lived".to_string()
//!     }
//!
//!     fn get_ttl_in_seconds(&self) -> i32 {
//!         1
//!     }
//! }
//!
//! const COLUMN: ShortLivedColumn = ShortLivedColumn {};
//!
//! fn main() -> Result<(), CacheError> {
//!     let cache = Engine::new(NoopEngine::build(String::new(), None));
//!
//!     let key = "your_key";
//!
//!     let _ = cache.try_insert(&COLUMN, &key, &100i32)?;
//!     // cached_variant will be [`Option<None>`] as we are using the NoopEngine
//!     let cached_variant = cache.try_get::<&str, i32>(&COLUMN, &key)?;
//!
//!     assert!(cached_variant.is_none());
//!
//!     Ok(())
//! }
//! ```
//!

pub mod noop_engine;
#[cfg(feature = "redis")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis")))]
pub mod redis_engine;
#[cfg(feature = "sled")]
#[cfg_attr(docsrs, doc(cfg(feature = "sled")))]
pub mod sled_engine;

use std::{any::Any, fmt::Debug};

use bincode::{Decode, Encode};
use noop_engine::NoopEngine;

#[derive(Debug, Clone)]
pub enum CacheError {
    Put(String),
    Get(String),
    Encode(String),
    Decode(String),
    Engine(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Put(key) => write!(f, "Failed to insert value into cache: {key}"),
            CacheError::Get(key) => write!(f, "Failed to get value from cache: {key}"),
            CacheError::Encode(key) => write!(f, "Failed to encode value for cache: {key}"),
            CacheError::Decode(key) => write!(f, "Failed to decode value for cache: {key}"),
            CacheError::Engine(message) => write!(f, "Engine failed: {message}"),
        }
    }
}

impl std::error::Error for CacheError {}

/// Definition of a Cache column
pub trait ColumnDefinition {
    /// Column name
    fn name(&self) -> String;

    /// Column items TTL
    fn get_ttl_in_seconds(&self) -> i32;
}

/// Trait for Cache storage engine
pub trait CacheStorage {
    /// Build new storage
    fn build(path: String, capacity: Option<u64>) -> Box<dyn CacheStorage + Send + Sync>
    where
        Self: Sized;

    /// Insert a value of type V with a key
    /// into the provided column
    /// # Errors
    /// Returns [`CacheError::Put`] if insert fails
    fn try_insert(
        &self,
        c: &dyn ColumnDefinition,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), CacheError>;

    /// # Errors
    /// Returns [`CacheError::Get`] if get fails
    fn try_get(&self, c: &dyn ColumnDefinition, key: &[u8]) -> Result<Option<Vec<u8>>, CacheError>;

    /// # Errors
    /// Returns [`CacheError::Engine`] if drop fails
    fn try_drop_column(&self, c: &dyn ColumnDefinition) -> Result<(), CacheError>;
}

pub struct Engine {
    storage: Box<dyn CacheStorage + Sync + Send>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            storage: Box::new(NoopEngine::default()),
        }
    }
}

impl Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("storage", &(*self.storage).type_id())
            .finish()
    }
}

impl Engine {
    ///
    /// ```
    /// use omega_cache::{Engine, noop_engine::NoopEngine, CacheStorage};
    ///
    /// let engine = Engine::new(NoopEngine::build(String::new(), None));
    /// ```
    ///
    #[must_use]
    pub fn new(storage: Box<dyn CacheStorage + Sync + Send>) -> Engine {
        Engine { storage }
    }

    /// # Errors
    /// Returns [`CacheError::Put`] if insert fails
    /// Returns [`CacheError::Encode`] if type V cannot be encoded to [`Vec<u8>`]
    pub fn try_insert<'a, K: AsRef<[u8]> + 'a, V: Encode + 'a>(
        &'a self,
        c: &dyn ColumnDefinition,
        key: &'a K,
        value: &'a V,
    ) -> Result<(), CacheError> {
        let key_bytes = key.as_ref();
        let value_bytes = bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| CacheError::Encode(e.to_string()))?;

        self.storage.try_insert(c, key_bytes, &value_bytes)
    }

    /// # Errors
    /// Returns [`CacheError::Get`] if get fails.
    /// Returns [`CacheError::Decode`] if get result cannot be decoded to type V from a [`Vec<u8>`]
    pub fn try_get<'a, K: AsRef<[u8]> + 'a, V: Decode<()> + Encode + 'a>(
        &'a self,
        c: &dyn ColumnDefinition,
        key: &'a K,
    ) -> Result<Option<V>, CacheError> {
        let key_bytes = key.as_ref();

        match self.storage.try_get(c, key_bytes)? {
            Some(bytes) => bincode::decode_from_slice(&bytes, bincode::config::standard())
                .map_err(|e| CacheError::Decode(e.to_string()))
                .map(|v| Some(v.0)),
            None => Ok(None),
        }
    }

    /// # Errors
    /// Returns [`CacheError::Engine`] if drop fails
    pub fn try_drop_column(&self, c: &dyn ColumnDefinition) -> Result<(), CacheError> {
        self.storage.try_drop_column(c)
    }
}

#[cfg(test)]
mod test {
    use crate::{ColumnDefinition, Engine};

    struct TestColumn {}

    impl ColumnDefinition for TestColumn {
        fn name(&self) -> String {
            "test_column".to_string()
        }

        fn get_ttl_in_seconds(&self) -> i32 {
            1
        }
    }

    const COLUMN: TestColumn = TestColumn {};

    #[test]
    fn test_default_engine() {
        let engine = Engine::default();

        assert!(engine.try_insert(&COLUMN, &String::new(), &100i32).is_ok());
        assert!(engine.try_get::<&str, i32>(&COLUMN, &"").is_ok());
        assert!(engine.try_drop_column(&COLUMN).is_ok());
    }
}
