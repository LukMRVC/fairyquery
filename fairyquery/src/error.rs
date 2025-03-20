/// FairyQuery error type
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Input syntax error, mostly for SQL input
    Syntax(usize, String),
    /// Invalid input from user
    InvalidInput(String),
    /// Problem when decoding or encoding data
    Data,
    /// Problem with IO operations
    IO(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Syntax(pos, errstr) => {
                write!(f, "Syntax error near position {pos}: {errstr}")
            }
            Error::Data => write!(f, "Data error"),
            Error::IO(msg) => write!(f, "IO error {msg}"),
            Error::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(err: Error) -> Self {
        Err(err)
    }
}

#[macro_export]
macro_rules! syntax_error {
    ($pos:expr, $($args:tt)*) => {
        crate::error::Error::Syntax($pos, format!($($args)*)).into()
    };
}

macro_rules! impl_from_error {
    ($from:ty, $to:ident) => {
        impl From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$to(err.to_string())
            }
        }
    };
}

impl_from_error!(std::num::ParseIntError, InvalidInput);
impl_from_error!(std::num::ParseFloatError, InvalidInput);
impl_from_error!(std::num::TryFromIntError, InvalidInput);
impl_from_error!(std::string::FromUtf8Error, InvalidInput);
