use crate::errors::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;
