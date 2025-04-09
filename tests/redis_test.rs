#[cfg(feature = "redis")]
use omega_cache::{CacheStorage, ColumnDefinition, Engine, redis_engine::RedisEngine};

#[test]
#[cfg(feature = "redis")]
fn test_redis_insert_and_get() {
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
    let redis = Engine::new(RedisEngine::build("redis://127.0.0.1/".to_string(), None));
    assert!(redis.try_insert(&c, &k, &d).is_ok());
    match redis.try_get(&c, &k) {
        Ok(data) => {
            assert!(data.is_some());
            assert_eq!(d, data.unwrap());
        }
        Err(e) => panic!("{e}"),
    }
}

#[test]
#[cfg(feature = "redis")]
fn test_redis_insert_and_timeout() {
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
    let redis = Engine::new(RedisEngine::build("redis://127.0.0.1/".to_string(), None));
    assert!(redis.try_insert(&c, &k, &d).is_ok());
    std::thread::sleep(std::time::Duration::from_secs(2));
    match redis.try_get::<&str, Data>(&c, &k) {
        Ok(data) => {
            assert!(data.is_none());
        }
        Err(e) => panic!("{e}"),
    }
}

#[test]
#[cfg(feature = "redis")]
fn test_redis_drop() {
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
    let redis = Engine::new(RedisEngine::build("redis://127.0.0.1/".to_string(), None));
    assert!(redis.try_insert(&c, &k, &d).is_ok());
    assert!(redis.try_drop_column(&c).is_ok());
    match redis.try_get::<&str, Data>(&c, &k) {
        Ok(data) => {
            assert!(data.is_none());
        }
        Err(e) => panic!("{e}"),
    }
}
