/// FairyQuery error type
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Input syntax error, mostly for SQL input
    Syntax(usize, String),
    /// Problem when decoding or encoding data
    Data,
    /// Problem with IO operations
    IO(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Syntax(pos, surrounding) => {
                write!(f, "Syntax error near position {pos} in ...{surrounding}...")
            }
            Error::Data => write!(f, "Data error"),
            Error::IO(msg) => write!(f, "IO error {msg}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(err: Error) -> Self {
        Err(err)
    }
}
