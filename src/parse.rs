use std::{error::Error, ffi::OsString, path::PathBuf};

use snafu::Snafu;

pub type Result<T, E = EnvironmentParseError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum EnvironmentParseError {
    #[snafu(display("it contains non-UTF8 encoding"))]
    InvalidEncoding,

    #[snafu(display("the contents of it are invalid"))]
    #[snafu(context(false))]
    InvalidContent { source: Box<dyn Error> },
}

pub struct ParseErrorHatch<T>(pub T);

impl<T> From<ParseErrorHatch<T>> for EnvironmentParseError
where
    T: snafu::Error + 'static,
{
    fn from(value: ParseErrorHatch<T>) -> Self {
        Self::InvalidContent {
            source: value.0.into(),
        }
    }
}

pub trait EnvironmentParse<Repr>: Sized {
    fn env_serialize(self) -> Repr;
    fn env_deserialize(raw: Repr) -> Result<Self>;
}

impl<T: EnvironmentParse<String>> EnvironmentParse<OsString> for T {
    fn env_serialize(self) -> OsString {
        self.env_serialize().into()
    }

    fn env_deserialize(raw: OsString) -> Result<Self> {
        let value = raw
            .into_string()
            .map_err(|_| EnvironmentParseError::InvalidEncoding)?;

        Self::env_deserialize(value)
    }
}

macro_rules! env_parse_raw {
    ($t:ident => $ty:ty) => {
        impl EnvironmentParse<$ty> for $t {
            fn env_serialize(self) -> $ty {
                self.into()
            }

            fn env_deserialize(raw: $ty) -> Result<Self> {
                Ok(Self::from(raw))
            }
        }
    };
}

env_parse_raw!(PathBuf => OsString);
env_parse_raw!(OsString => OsString);
env_parse_raw!(String => String);
