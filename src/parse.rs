use std::{error::Error, ffi::OsString};

use snafu::{ResultExt, Snafu};

type ErasedError = Box<dyn Error + Send + Sync + 'static>;

pub trait EnvironmentParse<Repr>: Sized {
    type Error: Into<ErasedError>;

    fn env_serialize(self) -> Repr;
    fn env_deserialize(raw: Repr) -> Result<Self, Self::Error>;
}

// NOTE: the display messages make sense when paired with crate::Error
#[derive(Debug, Snafu)]

pub enum StringParseError {
    #[snafu(display("it contains non-UTF8 encoding"))]
    InvalidEncoding,

    #[snafu(display("the contents of it are invalid"))]
    InvalidContent { source: ErasedError },
}

impl<T: EnvironmentParse<String>> EnvironmentParse<OsString> for T {
    type Error = StringParseError;

    fn env_serialize(self) -> OsString {
        self.env_serialize().into()
    }

    fn env_deserialize(raw: OsString) -> Result<Self, Self::Error> {
        let value = raw
            .into_string()
            .map_err(|_| StringParseError::InvalidEncoding)?;

        Ok(Self::env_deserialize(value)
            .map_err(|e| e.into())
            .context(InvalidContentSnafu)?)
    }
}
