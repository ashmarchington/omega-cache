//!
//! Simple crate to allow dynamic cache implementations
//! to be used as struct or function parameters.
//!

pub mod noop_engine;
#[cfg(feature = "redis")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis")))]
pub mod redis_engine;
#[cfg(feature = "sled")]
#[cfg_attr(docsrs, doc(cfg(feature = "sled")))]
pub mod sled_engine;

use bincode::{Decode, Encode};

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
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), CacheError>;

    /// # Errors
    /// Returns [`CacheError::Get`] if get fails
    fn try_get(
        &self,
        c: &dyn ColumnDefinition,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, CacheError>;

    /// # Errors
    /// Returns [`CacheError::Engine`] if drop fails
    fn try_drop_column(&self, c: &dyn ColumnDefinition) -> Result<(), CacheError>;
}

pub struct Engine {
    storage: Box<dyn CacheStorage + Sync + Send>,
}

impl Engine {
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

        self.storage.try_insert(c, key_bytes.to_vec(), value_bytes)
    }

    /// # Errors
    /// Returns [`CacheError::Get`] if get fails.
    /// Returns [`CacheError::Decode`] if get result cannot be decoded to type V from a [`Vec<u8>`]
    pub fn try_get<'a, K: AsRef<[u8]> + 'a, V: Decode + Encode + 'a>(
        &'a self,
        c: &dyn ColumnDefinition,
        key: &'a K,
    ) -> Result<Option<V>, CacheError> {
        let key_bytes = key.as_ref();

        match self.storage.try_get(c, key_bytes.to_vec())? {
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
