#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UnexpectedVariantIndex { index: u8, ident: &'static str },
    UnexpectedPolymorphicVariantIndex { index: i32, ident: &'static str },
    UnexpectedValueForUnit(u8),
    UnexpectedValueForBool(u8),
    UnexpectedValueForOption(u8),
    Utf8Error(std::str::Utf8Error),
    SameKeyAppearsTwiceInMap,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8Error(e)
    }
}
