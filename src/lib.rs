/*!
# UTF-8 Builder

Build and validate UTF-8 data from chunks. Each chunk doesn't have to be a complete UTF-8 data.

## Motives and Examples

When we want our Rust program to input a UTF-8 data, we can store all data in the memory and use `String::from_utf8(vec)` to validate it and convert it into a `String` instance.

However, it would be better if we perform UTF-8 validation while fetching and storing the data into the memory. In such a way, if the data is not UTF-8, we don't have to waste the memory space and time to store it.

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
*/

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;

use core::cmp::Ordering;

use alloc::string::String;
use alloc::vec::Vec;

pub use error::Utf8Error;

/// A builder for Building and validating UTF-8 data from chunks.
#[derive(Debug, Clone, Default)]
pub struct Utf8Builder {
    buffer: Vec<u8>,
    /// the length for the incomplete character
    sl: u8,
    /// the valid expected length for the incomplete character
    sel: u8,
}

impl Utf8Builder {
    /// Constructs a new, empty `Utf8Builder`.
    #[inline]
    pub const fn new() -> Self {
        Utf8Builder {
            buffer: Vec::new(),
            sl: 0,
            sel: 0,
        }
    }

    /// Constructs a new, empty `with_capacity` with a specific capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Utf8Builder {
            buffer: Vec::with_capacity(capacity),
            sl: 0,
            sel: 0,
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given `Utf8Builder`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// Returns the number of elements in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the builder contains no data.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Utf8Builder {
    /// Returns whether the current data are valid UTF-8
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.sl == 0
    }

    /// Try to get the `String` instance.
    #[inline]
    pub fn finalize(self) -> Result<String, Utf8Error> {
        if self.is_valid() {
            let s = unsafe { String::from_utf8_unchecked(self.buffer) };

            Ok(s)
        } else {
            Err(Utf8Error)
        }
    }
}

impl Utf8Builder {
    /// Pushes a byte.
    pub fn push(&mut self, b: u8) -> Result<(), Utf8Error> {
        if self.sl == 0 {
            let w = utf8_width::get_width(b);

            match w {
                0 => return Err(Utf8Error),
                1 => {
                    self.buffer.push(b);
                }
                _ => {
                    self.buffer.push(b);
                    self.sl = 1;
                    self.sel = w as u8;
                }
            }
        } else if self.sl + 1 == self.sel {
            self.buffer.push(b);

            self.sl = 0;
            // self.sel = 0; // no need
        } else {
            self.buffer.push(b);

            self.sl += 1;
        }

        Ok(())
    }

    /// Pushes a `&str`.
    #[inline]
    pub fn push_str(&mut self, s: &str) -> Result<(), Utf8Error> {
        if self.sl == 0 {
            self.buffer.extend_from_slice(s.as_bytes());

            Ok(())
        } else {
            Err(Utf8Error)
        }
    }

    /// Pushes a char.
    pub fn push_char(&mut self, c: char) -> Result<(), Utf8Error> {
        if self.sl == 0 {
            self.buffer.reserve(4);

            let len = self.buffer.len();

            unsafe {
                self.buffer.set_len(len + 4);
            }

            let c = c.encode_utf8(&mut self.buffer[len..]).len();

            unsafe {
                self.buffer.set_len(len + c);
            }

            Ok(())
        } else {
            Err(Utf8Error)
        }
    }

    /// Pushes a chunk.
    pub fn push_chunk(&mut self, chunk: &[u8]) -> Result<(), Utf8Error> {
        let chunk_size = chunk.len();

        if chunk_size == 0 {
            return Ok(());
        }

        let mut e = if self.sl > 0 {
            let r = (self.sel - self.sl) as usize;

            match r.cmp(&chunk_size) {
                Ordering::Greater => {
                    let sl = self.sl as usize;
                    let nsl = sl + chunk_size;

                    self.buffer.extend_from_slice(chunk);

                    self.sl = nsl as u8;

                    return Ok(());
                }
                Ordering::Equal => {
                    self.buffer.extend_from_slice(chunk);

                    self.sl = 0;
                    // self.sel = 0; // no need

                    return Ok(());
                }
                Ordering::Less => {
                    self.buffer.extend_from_slice(&chunk[..r]);

                    self.sl = 0;
                    // self.sel = 0; // no need

                    r
                }
            }
        } else {
            0usize
        };

        loop {
            let w = utf8_width::get_width(chunk[e]);

            if w == 0 {
                return Err(Utf8Error);
            }

            let r = chunk_size - e;

            if r >= w {
                self.buffer.extend_from_slice(&chunk[e..e + w]);

                e += w;

                if e == chunk_size {
                    break;
                }
            } else {
                self.buffer.extend_from_slice(&chunk[e..]);

                self.sl = r as u8;
                self.sel = w as u8;

                break;
            }
        }

        Ok(())
    }
}

impl From<&str> for Utf8Builder {
    #[inline]
    fn from(s: &str) -> Self {
        Utf8Builder {
            buffer: s.as_bytes().to_vec(),
            sl: 0,
            sel: 0,
        }
    }
}

impl From<String> for Utf8Builder {
    #[inline]
    fn from(s: String) -> Self {
        Utf8Builder {
            buffer: s.into_bytes(),
            sl: 0,
            sel: 0,
        }
    }
}
