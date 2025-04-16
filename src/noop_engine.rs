use crate::{CacheError, CacheStorage, ColumnDefinition};

///
/// Noop engine for testing
///
#[derive(Default, Debug)]
pub struct NoopEngine {}

impl CacheStorage for NoopEngine {
    fn build(_path: String, _capacity: Option<u64>) -> Box<dyn CacheStorage + Send + Sync> {
        Box::new(NoopEngine {})
    }

    fn try_insert(
        &self,
        _c: &dyn ColumnDefinition,
        _key: &[u8],
        _value: &[u8],
    ) -> Result<(), CacheError> {
        Ok(())
    }

    fn try_get(
        &self,
        _c: &dyn ColumnDefinition,
        _key: &[u8],
    ) -> Result<Option<Vec<u8>>, CacheError> {
        Ok(None)
    }

    fn try_drop_column(&self, _c: &dyn ColumnDefinition) -> Result<(), CacheError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{CacheStorage, ColumnDefinition};

    use super::NoopEngine;

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
    fn test_noop() {
        let noop = NoopEngine::default();

        assert!(
            noop.try_insert(&COLUMN, "".as_bytes(), "".as_bytes())
                .is_ok()
        );
        assert!(noop.try_get(&COLUMN, "".as_bytes()).is_ok());
        assert!(noop.try_drop_column(&COLUMN).is_ok());
    }
}
