#[derive(Debug)]
pub enum Error {
    TreeNotMatchedError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TreeNotMatchedError => {
                write!(f, "Component structure does not match the element tree")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}
