use std::ffi::OsString;

use snafu::{AsErrorSource, Snafu};

pub trait EnvironmentParse<Repr>: Sized {
    type Error: snafu::Error + 'static;

    fn env_serialize(self) -> Repr;
    fn env_deserialize(raw: Repr) -> Result<Self, Self::Error>;
}

// NOTE: the display messages make sense when paired with crate::Error
#[derive(Debug, Snafu)]

pub enum StringParseError<E: AsErrorSource> {
    #[snafu(display("it contains non-UTF8 encoding"))]
    InvalidEncoding,
    #[snafu(context(false))]
    #[snafu(display("the contents of it are invalid"))]
    InvalidContent { source: E },
}

impl<T: EnvironmentParse<String>> EnvironmentParse<OsString> for T {
    type Error = StringParseError<<Self as EnvironmentParse<String>>::Error>;

    fn env_serialize(self) -> OsString {
        self.env_serialize().into()
    }

    fn env_deserialize(raw: OsString) -> Result<Self, Self::Error> {
        let value = raw
            .into_string()
            .map_err(|_| StringParseError::InvalidEncoding)?;

        Ok(Self::env_deserialize(value)?)
    }
}
