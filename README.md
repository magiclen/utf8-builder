UTF-8 Builder
====================

[![CI](https://github.com/magiclen/utf8-builder/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/utf8-builder/actions/workflows/ci.yml)

Build and validate UTF-8 data from chunks. Each chunk doesn't have to be a complete UTF-8 data.

## Motives and Examples

When we want our Rust program to input a UTF-8 data, we can store all data in the memory and use `String::from_utf8(vec)` to validate it and convert it into a `String` instance.

However, it would be better if we perform UTF-8 validation while fetching and storing the data into the memory. In such a way, if the data is not UTF-8, we don't have to waste the memory space and time to store all of it.

```rust
use utf8_builder::Utf8Builder;

const TEXT1: &str = "is is English.";
const TEXT2: &str = "這是中文。";

let mut builder = Utf8Builder::new();

builder.push(b'T').unwrap();
builder.push_char('h').unwrap();
builder.push_str(TEXT1).unwrap();
builder.push_chunk(TEXT2.as_bytes()).unwrap();

let result = builder.finalize().unwrap();

assert_eq!(format!("Th{}{}", TEXT1, TEXT2), result);
```

## No Std

Disable the default features to compile this crate without std.

```toml
[dependencies.utf8-builder]
version = "*"
default-features = false
```

## Crates.io

https://crates.io/crates/utf8-builder

## Documentation

https://docs.rs/utf8-builder

## License

[MIT](LICENSE)