#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Utf8Error;

#[cfg(feature = "std")]
impl Display for Utf8Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("incorrect UTF-8 data")
    }
}

#[cfg(feature = "std")]
impl Error for Utf8Error {}
