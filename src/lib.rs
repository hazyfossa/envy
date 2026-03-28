pub mod container;
pub mod define;
pub mod diff;
pub mod parse;

use snafu::prelude::*;
use std::ffi::OsString;

use crate::{
    container::{EnvContainer, MutableEnvContainer},
    diff::{Diff, unset},
    parse::EnvironmentParse,
};

pub use container::OsEnv;
pub use diff::Unset;

// TODO: zerocopy views

pub trait EnvVariable: EnvironmentParse<OsString> {
    const KEY: &str;
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The variable {key} exists, but"))]
    ParseError {
        key: &'static str,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[snafu(display("The variable {key} does not exist"))]
    NoneError { key: &'static str },
}

pub trait Get: EnvContainer {
    fn get<T: EnvVariable>(&self) -> Result<T, Error> {
        let raw = self.raw_get(T::KEY).context(NoneSnafu { key: T::KEY })?;

        // TODO: zerocopy
        T::env_deserialize(raw.clone())
            .map_err(|e| e.into())
            .context(ParseSnafu { key: T::KEY })
    }
}

pub trait Set: MutableEnvContainer {
    fn set<T: EnvVariable>(&mut self, e: T) {
        // Set is an alias for merge with length one
        self.raw_merge(e);
    }

    fn apply<T: Diff>(&mut self, e: T) {
        self.raw_merge(e);
    }

    fn pull<T: EnvVariable>(&mut self) -> Result<T, Error>
    where
        Self: Get,
    {
        let ret = self.get::<T>();
        self.apply(unset::<T>());
        ret
    }
}

impl<T> Get for T where T: EnvContainer {}
impl<T> Set for T where T: MutableEnvContainer {}
