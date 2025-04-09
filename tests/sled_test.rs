#[cfg(feature = "sled")]
use omega_cache::{CacheStorage, ColumnDefinition, sled_engine::SledEngine};

#[test]
#[cfg(feature = "sled")]
fn test_sled_insert_and_get() {
    #[derive(bincode::Encode, bincode::Decode, PartialEq, Eq, Debug)]
    struct Data {
        name: String,
    }

    struct Column {}
    impl ColumnDefinition for Column {
        fn name(&self) -> String {
            "test_column".to_string()
        }

        fn get_ttl_in_seconds(&self) -> i32 {
            10
        }
    }

    let c = Column {};
    let d = Data {
        name: "test_data".to_string(),
    };
    let k = "test_key";
    let sled = omega_cache::Engine::new(SledEngine::build("./tmp/sled_test".to_string(), None));
    assert!(sled.try_insert(&c, &k, &d).is_ok());

    match sled.try_get(&c, &k) {
        Ok(data) => {
            assert!(data.is_some());
            assert_eq!(d, data.unwrap());
        }
        Err(e) => panic!("{e}"),
    }
}

#[test]
#[cfg(feature = "sled")]
fn test_sled_insert_and_timeout() {
    #[derive(bincode::Encode, bincode::Decode, PartialEq, Eq, Debug)]
    struct Data {
        name: String,
    }

    struct Column {}
    impl ColumnDefinition for Column {
        fn name(&self) -> String {
            "test_column".to_string()
        }

        fn get_ttl_in_seconds(&self) -> i32 {
            1
        }
    }

    let c = Column {};
    let d = Data {
        name: "test_data".to_string(),
    };
    let k = "test_key";
    let sled = omega_cache::Engine::new(SledEngine::build(
        "./tmp/sled_test_timeout".to_string(),
        None,
    ));
    assert!(sled.try_insert(&c, &k, &d).is_ok());

    std::thread::sleep(std::time::Duration::from_secs(2));

    match sled.try_get::<&str, Data>(&c, &k) {
        Ok(data) => {
            assert!(data.is_none());
        }
        Err(e) => panic!("{e}"),
    }
}

#[test]
#[cfg(feature = "sled")]
fn test_sled_drop() {
    #[derive(bincode::Encode, bincode::Decode, PartialEq, Eq, Debug)]
    struct Data {
        name: String,
    }

    struct Column {}
    impl ColumnDefinition for Column {
        fn name(&self) -> String {
            "test_column".to_string()
        }

        fn get_ttl_in_seconds(&self) -> i32 {
            10
        }
    }

    let c = Column {};
    let d = Data {
        name: "test_data".to_string(),
    };
    let k = "test_key";
    let sled =
        omega_cache::Engine::new(SledEngine::build("./tmp/sled_test_drop".to_string(), None));
    assert!(sled.try_insert(&c, &k, &d).is_ok());
    assert!(sled.try_drop_column(&c).is_ok());

    match sled.try_get::<&str, Data>(&c, &k) {
        Ok(data) => {
            assert!(data.is_none());
        }
        Err(e) => panic!("{e}"),
    }
}
