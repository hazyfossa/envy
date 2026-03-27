pub mod container;
pub mod define;
pub mod diff;
pub mod parse;

use snafu::prelude::*;
use std::ffi::OsString;

use crate::{
    container::{EnvRead, EnvWrite},
    diff::{Diff, unset},
    parse::EnvironmentParse,
};

pub use diff::Unset;

// TODO: zerocopy views

pub trait EnvVar: EnvironmentParse<OsString> {
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

// TODO: proper readonly containers
// TODO: better trait names, explicit Ext traits for disambiguation?
// or simply prefix everything with env...
pub trait UseEnvRead: EnvRead {
    fn get<T: EnvVar>(&self) -> Result<T, Error> {
        let raw = self.raw_get(T::KEY).context(NoneSnafu { key: T::KEY })?;

        // TODO: zerocopy
        T::env_deserialize(raw.clone())
            .map_err(|e| e.into())
            .context(ParseSnafu { key: T::KEY })
    }
}

pub trait UseEnvWrite: EnvWrite {
    fn set<T: EnvVar>(&mut self, e: T) {
        // Set is an alias for merge with length one
        self.raw_merge(e);
    }

    fn apply<T: Diff>(&mut self, e: T) {
        self.raw_merge(e);
    }

    fn pull<T: EnvVar>(&mut self) -> Result<T, Error>
    where
        Self: UseEnvRead,
    {
        let ret = self.get::<T>();
        self.apply(unset::<T>());
        ret
    }
}

impl<T> UseEnvRead for T where T: EnvRead {}
impl<T> UseEnvWrite for T where T: EnvWrite {}
