# Omega Cache
Simple caching lib to allow dynamically switching out cache implementations

## Basic Usage

```rust
use omega_cache::{Engine, noop_engine::NoopEngine, ColumnDefinition}

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

const COLUMN: Column = Column {};

fn main() {
    let data = Data {
        name: "Sweet Name".to_string(),
    };

    let key = "some_key";

    let cache = Engine::new(NoopEngine::build(String::new(), None));

    match cache.try_insert(&COLUMN, &key, &data) {
        Ok(()) => {},
        Err(e) => panic!("{e}"),
    };

    match cache.try_get::<&str, Data>(&COLUMN, &key) {
        Ok(data) => println!("{data:#?}"),
        Err(e) => panic!("{e}"),
    };
}
```

