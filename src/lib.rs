pub mod container;
pub mod define;
pub mod diff;
pub mod parse;

use snafu::prelude::*;
use std::ffi::OsString;

use crate::{
    container::{EnvContainer, MutableEnvContainer},
    parse::EnvironmentParse,
};

pub use container::OsEnv;
pub use diff::{Diff, Unset, unset};

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

// Ponder: alternatives to "maybe" api (custom result wrapper?)

pub trait Get: EnvContainer {
    fn maybe_get<T: EnvVariable>(&self) -> Result<Option<T>, Error> {
        let raw = match self.raw_get(T::KEY) {
            Some(x) => x,
            None => return Ok(None),
        };

        T::env_deserialize(raw.clone())
            .map_err(|e| e.into())
            .context(ParseSnafu { key: T::KEY })
            .map(Some)
    }

    fn get<T: EnvVariable>(&self) -> Result<T, Error> {
        self.maybe_get()?.ok_or(Error::NoneError { key: T::KEY })
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

    fn maybe_pull<T: EnvVariable>(&mut self) -> Result<Option<T>, Error>
    where
        Self: Get,
    {
        let ret = self.maybe_get::<T>()?;
        self.apply(unset::<T>());
        Ok(ret)
    }

    fn pull<T: EnvVariable>(&mut self) -> Result<T, Error>
    where
        Self: Get,
    {
        let ret = self.get::<T>()?;
        self.apply(unset::<T>());
        Ok(ret)
    }
}

impl<T> Get for T where T: EnvContainer {}
impl<T> Set for T where T: MutableEnvContainer {}

pub trait Env: Get + Diff {}
impl<T> Env for T where T: Get + Diff {}
