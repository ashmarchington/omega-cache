use crate::{CacheError, CacheStorage, ColumnDefinition};

/// A cache item.
///
/// Holds the timestamp of the item and data.
/// Timestamp is used to check that the item is within it's TTL
/// based on the [`ColumnDefinition`] used when inserting
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
struct Item<T: bincode::Encode> {
    /// Time in seconds this item was added to the cache
    time: u64,
    /// The data held by this item
    data: T,
}

///
/// Wrapper for ``sled::Db``
///
pub struct SledEngine {
    inner: sled::Db,
}

impl CacheStorage for SledEngine {
    fn build(path: String, capacity: Option<u64>) -> Box<dyn CacheStorage + Send + Sync> {
        match sled::Config::default()
            .mode(sled::Mode::HighThroughput)
            .path(path)
            .cache_capacity(capacity.unwrap_or(1024 * 1024 * 1024))
            .use_compression(true)
            .compression_factor(5)
            .open()
        {
            Ok(db) => Box::new(SledEngine { inner: db }),
            Err(e) => panic!("Failed to open cache: {e}"),
        }
    }

    fn try_insert(
        &self,
        c: &dyn ColumnDefinition,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), crate::CacheError> {
        let t = std::time::Instant::now();

        let item = Item {
            time: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_err(|e| CacheError::Put(e.to_string()))?
                .as_secs(),
            data: value,
        };

        match bincode::encode_to_vec(item, bincode::config::standard()) {
            Ok(bytes) => match self
                .inner
                .open_tree(c.name())
                .map_err(|e| CacheError::Engine(e.to_string()))?
                .insert(key, bytes)
            {
                Ok(_) => {
                    if cfg!(debug_assertions) {
                        eprintln!(
                            "\x1b[0;34mTime taken for insert:\x1b[0m {}us",
                            t.elapsed().as_micros()
                        );
                    }

                    Ok(())
                }
                Err(e) => Err(CacheError::Put(e.to_string())),
            },
            Err(e) => Err(CacheError::Encode(e.to_string())),
        }
    }

    fn try_get(
        &self,
        c: &dyn ColumnDefinition,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, crate::CacheError> {
        let t = std::time::Instant::now();

        let key_bytes = key.as_slice();

        match self
            .inner
            .open_tree(c.name())
            .map_err(|e| CacheError::Engine(e.to_string()))?
            .get(key_bytes)
        {
            Ok(Some(bytes)) => {
                match bincode::decode_from_slice::<Item<Vec<u8>>, _>(
                    &bytes,
                    bincode::config::standard(),
                ) {
                    Ok(value) => {
                        if cfg!(debug_assertions) {
                            eprintln!(
                                "\x1b[0;34mTime taken for get:\x1b[0m {}us",
                                t.elapsed().as_micros()
                            );
                        }

                        if (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_err(|e| CacheError::Get(e.to_string()))?
                            .as_secs()
                            - value.0.time)
                            > u64::try_from(c.get_ttl_in_seconds())
                                .map_err(|e| CacheError::Get(e.to_string()))?
                        {
                            self.inner
                                .open_tree(c.name())
                                .map_err(|e| CacheError::Engine(e.to_string()))?
                                .remove(key_bytes)
                                .expect("Failed to remove outdated cache item");

                            return Ok(None);
                        }

                        Ok(Some(value.0.data))
                    }
                    Err(e) => Err(CacheError::Get(e.to_string())),
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(CacheError::Get(e.to_string())),
        }
    }

    fn try_drop_column(&self, c: &dyn ColumnDefinition) -> Result<(), CacheError> {
        if let Err(e) = self.inner.drop_tree(c.name()) {
            return Err(CacheError::Engine(e.to_string()));
        }

        match self.inner.open_tree(c.name()) {
            Ok(_) => Ok(()),
            Err(e) => Err(CacheError::Engine(e.to_string())),
        }
    }
}
