#[derive(Debug)]
pub enum Error {
    TreeNotCreated,
    TreeNodeTypeWrong,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TreeNotCreated => {
                write!(f, "Component template has not been initialized")?;
            }
            Error::TreeNodeTypeWrong => {
                write!(f, "The node type in backend element tree is incorrect")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}
