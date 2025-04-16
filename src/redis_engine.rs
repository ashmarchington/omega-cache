use redis::{Commands, SetExpiry, SetOptions};

use crate::{CacheError, CacheStorage};

///
/// Wrapper for [`r2d2::Pool<redis::Client>`]
///
#[derive(Debug)]
pub struct RedisEngine {
    inner: r2d2::Pool<redis::Client>,
}

impl CacheStorage for RedisEngine {
    fn build(path: String, _capacity: Option<u64>) -> Box<dyn CacheStorage + Send + Sync>
    where
        Self: Sized,
    {
        match redis::Client::open(path) {
            Ok(client) => match r2d2::Pool::builder().build(client) {
                Ok(pool) => Box::new(RedisEngine { inner: pool }),
                Err(e) => panic!("Failed to start redis pool: {e}"),
            },
            Err(e) => panic!("Failed to open connection to redis: {e}"),
        }
    }

    fn try_insert(
        &self,
        c: &dyn crate::ColumnDefinition,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), crate::CacheError> {
        match self.inner.get() {
            Ok(mut conn) => {
                let t = std::time::Instant::now();

                let k = [c.name().as_bytes(), ":".as_bytes(), key].concat();
                if let Err(e) = conn.set_options::<&[u8], &[u8], ()>(
                    &k,
                    value,
                    SetOptions::default().with_expiration(SetExpiry::EX(
                        u64::try_from(c.get_ttl_in_seconds())
                            .map_err(|e| CacheError::Put(e.to_string()))?,
                    )),
                ) {
                    Err(CacheError::Put(e.to_string()))
                } else {
                    if cfg!(debug_assertions) {
                        eprintln!(
                            "\x1b[0;34mTime taken for insert:\x1b[0m {}us",
                            t.elapsed().as_micros()
                        );
                    }
                    Ok(())
                }
            }
            Err(e) => Err(CacheError::Engine(e.to_string())),
        }
    }

    fn try_get(
        &self,
        c: &dyn crate::ColumnDefinition,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, crate::CacheError> {
        match self.inner.get() {
            Ok(mut conn) => {
                let t = std::time::Instant::now();
                let k = [c.name().as_bytes(), ":".as_bytes(), key].concat();

                match conn.get::<&[u8], Vec<u8>>(&k) {
                    Ok(bytes) => {
                        if bytes.is_empty() {
                            return Ok(None);
                        }
                        if cfg!(debug_assertions) {
                            eprintln!(
                                "\x1b[0;34mTime taken for get:\x1b[0m {}us",
                                t.elapsed().as_micros()
                            );
                        }

                        Ok(Some(bytes))
                    }
                    Err(e) => Err(CacheError::Get(e.to_string())),
                }
            }
            Err(e) => Err(CacheError::Engine(e.to_string())),
        }
    }

    fn try_drop_column(&self, c: &dyn crate::ColumnDefinition) -> Result<(), crate::CacheError> {
        let mut conn = match self.inner.get() {
            Ok(conn) => conn,
            Err(e) => return Err(CacheError::Engine(e.to_string())),
        };

        let items = match conn.scan_match::<&[u8], Vec<u8>>(format!("{}:*", c.name()).as_ref()) {
            Ok(items) => items.collect::<Vec<Vec<u8>>>(),
            Err(e) => return Err(CacheError::Engine(e.to_string())),
        };

        for i in items {
            if let Err(e) = conn.unlink::<&[u8], ()>(&i) {
                return Err(CacheError::Engine(e.to_string()));
            }
        }

        Ok(())
    }
}
