use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub struct QueryError {
    message: String,
}

impl QueryError {
    pub(crate) fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}
impl Error for QueryError {}

impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.message))
    }
}
