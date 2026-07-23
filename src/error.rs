use core::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub struct Utf8Error;

impl Display for Utf8Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("incorrect UTF-8 data")
    }
}

impl Error for Utf8Error {}
