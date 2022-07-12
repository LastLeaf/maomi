#[derive(Debug)]
pub enum Error {
    /// The operation is invalid before component created
    TreeNotCreated,
    /// A wrong backend tree is visited
    /// 
    /// Generally means some bad operation is done directly in the backend.
    TreeNodeTypeWrong,
    /// A general backend failure
    BackendError {
        msg: String,
        err: Option<Box<dyn std::error::Error>>,
    },
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
            Error::BackendError { msg, err } => {
                if let Some(err) = err {
                    write!(f, "{}: {}", msg, err.to_string())?;
                } else {
                    write!(f, "{}", msg)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}
