use crate::{CacheError, CacheStorage, ColumnDefinition};

///
/// Noop engine for testing
///
pub struct NoopEngine {}

impl CacheStorage for NoopEngine {
    fn build(_path: String, _capacity: Option<u64>) -> Box<dyn CacheStorage + Send + Sync> {
        Box::new(NoopEngine {})
    }

    fn try_insert(
        &self,
        _c: &dyn ColumnDefinition,
        _key: Vec<u8>,
        _value: Vec<u8>,
    ) -> Result<(), CacheError> {
        Ok(())
    }

    fn try_get(
        &self,
        _c: &dyn ColumnDefinition,
        _key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, CacheError> {
        Ok(None)
    }

    fn try_drop_column(&self, _c: &dyn ColumnDefinition) -> Result<(), CacheError> {
        Ok(())
    }
}
