/*!
# UTF-8 Builder

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

assert_eq!(format!("Th{TEXT1}{TEXT2}"), result);
```
*/

#![no_std]

extern crate alloc;

mod error;

use alloc::{string::String, vec::Vec};
use core::{fmt, str::from_utf8};

pub use error::Utf8Error;

/// A builder for building and validating UTF-8 data from chunks.
#[derive(Debug, Clone, Default)]
pub struct Utf8Builder {
    buffer: Vec<u8>,
    /// the length for the incomplete character
    sl:     u8,
    /// the valid expected length for the incomplete character
    sel:    u8,
}

impl Utf8Builder {
    /// Constructs a new, empty `Utf8Builder`.
    #[inline]
    pub const fn new() -> Self {
        Utf8Builder {
            buffer: Vec::new(), sl: 0, sel: 0
        }
    }

    /// Constructs a new, empty `Utf8Builder` with a specific capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Utf8Builder {
            buffer: Vec::with_capacity(capacity), sl: 0, sel: 0
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given `Utf8Builder`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// Clears the builder, removing all data but keeping the allocated capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.sl = 0;
    }

    /// Returns the number of bytes in the buffer, including the bytes of the pending incomplete character.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the builder contains no data.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the current data as a byte slice, including the bytes of the pending incomplete character.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

impl Utf8Builder {
    /// Returns `true` if there is no pending incomplete character.
    ///
    /// Invalid bytes are never stored, so the data in the buffer is always valid UTF-8 when this returns `true`.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.sl == 0
    }

    /// Consumes the builder and returns the built `String`.
    ///
    /// Returns an error if the data ends with an incomplete character.
    /// The data is dropped in that case, so use `into_bytes` instead if the raw data is still needed.
    #[inline]
    pub fn finalize(self) -> Result<String, Utf8Error> {
        if self.is_valid() {
            let s = unsafe { String::from_utf8_unchecked(self.buffer) };

            Ok(s)
        } else {
            Err(Utf8Error)
        }
    }

    /// Consumes the builder and returns the raw data, even if it ends with an incomplete character.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}

/// Checks whether `b` can be the `index`-th (1-based) continuation byte of a character starting with `lead`, according to RFC 3629.
#[inline]
const fn is_valid_continuation(lead: u8, index: u8, b: u8) -> bool {
    if index == 1 {
        match lead {
            0xE0 => matches!(b, 0xA0..=0xBF),
            0xED => matches!(b, 0x80..=0x9F),
            0xF0 => matches!(b, 0x90..=0xBF),
            0xF4 => matches!(b, 0x80..=0x8F),
            _ => matches!(b, 0x80..=0xBF),
        }
    } else {
        matches!(b, 0x80..=0xBF)
    }
}

impl Utf8Builder {
    /// Pushes a byte.
    ///
    /// Returns an error if the byte is not allowed at the current position.
    /// Nothing is stored in that case, so a correct byte can still be pushed afterwards.
    #[inline]
    pub fn push(&mut self, b: u8) -> Result<(), Utf8Error> {
        if self.sl == 0 {
            let w = utf8_width::get_width(b);

            match w {
                0 => return Err(Utf8Error),
                1 => {
                    self.buffer.push(b);
                },
                _ => {
                    self.buffer.push(b);
                    self.sl = 1;
                    self.sel = w as u8;
                },
            }
        } else {
            // the lead byte of the pending incomplete character is still in the buffer
            let lead = self.buffer[self.buffer.len() - self.sl as usize];

            if !is_valid_continuation(lead, self.sl, b) {
                return Err(Utf8Error);
            }

            self.buffer.push(b);

            if self.sl + 1 == self.sel {
                self.sl = 0;
                // self.sel = 0; // no need
            } else {
                self.sl += 1;
            }
        }

        Ok(())
    }

    /// Pushes a `&str`.
    ///
    /// Returns an error if there is a pending incomplete character.
    /// The builder state is unchanged in that case.
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
    ///
    /// Returns an error if there is a pending incomplete character.
    /// The builder state is unchanged in that case.
    #[inline]
    pub fn push_char(&mut self, c: char) -> Result<(), Utf8Error> {
        if self.sl == 0 {
            let mut tmp = [0u8; 4];

            self.buffer.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());

            Ok(())
        } else {
            Err(Utf8Error)
        }
    }

    /// Pushes a chunk.
    ///
    /// The chunk does not have to be complete UTF-8 data, because a character can be split across chunks.
    /// Returns an error if the chunk contains invalid data.
    /// In that case, the leading valid part of the chunk is still stored, so the builder remains usable.
    /// However, if the error happens while completing a pending incomplete character, nothing is stored.
    #[inline]
    pub fn push_chunk(&mut self, chunk: &[u8]) -> Result<(), Utf8Error> {
        let chunk_size = chunk.len();

        if chunk_size == 0 {
            return Ok(());
        }

        let e = if self.sl > 0 {
            // the lead byte of the pending incomplete character is still in the buffer
            let lead = self.buffer[self.buffer.len() - self.sl as usize];

            let r = (self.sel - self.sl) as usize;
            let take = r.min(chunk_size);

            for (i, &b) in chunk[..take].iter().enumerate() {
                if !is_valid_continuation(lead, self.sl + i as u8, b) {
                    return Err(Utf8Error);
                }
            }

            self.buffer.extend_from_slice(&chunk[..take]);

            if take < r {
                self.sl += take as u8;

                return Ok(());
            }

            self.sl = 0;
            // self.sel = 0; // no need

            if take == chunk_size {
                return Ok(());
            }

            take
        } else {
            0usize
        };

        let rest = &chunk[e..];

        match from_utf8(rest) {
            Ok(s) => {
                self.buffer.extend_from_slice(s.as_bytes());
            },
            Err(error) => {
                let valid_up_to = error.valid_up_to();

                if error.error_len().is_some() {
                    // still keep the leading valid characters so the builder remains usable
                    self.buffer.extend_from_slice(&rest[..valid_up_to]);

                    return Err(Utf8Error);
                }

                // the chunk ends with an incomplete character which is guaranteed to be a valid prefix
                self.buffer.extend_from_slice(rest);

                self.sl = (rest.len() - valid_up_to) as u8;
                self.sel = utf8_width::get_width(rest[valid_up_to]) as u8;
            },
        }

        Ok(())
    }
}

/// Writing fails if there is a pending incomplete character.
impl fmt::Write for Utf8Builder {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push_char(c).map_err(|_| fmt::Error)
    }
}

impl From<&str> for Utf8Builder {
    #[inline]
    fn from(s: &str) -> Self {
        Utf8Builder {
            buffer: s.as_bytes().to_vec(), sl: 0, sel: 0
        }
    }
}

impl From<String> for Utf8Builder {
    #[inline]
    fn from(s: String) -> Self {
        Utf8Builder {
            buffer: s.into_bytes(), sl: 0, sel: 0
        }
    }
}
