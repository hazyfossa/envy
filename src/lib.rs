pub mod container;
pub mod define;
pub mod diff;
pub mod parse;

use std::ffi::OsString;

use snafu::prelude::*;

use crate::{container::EnvContainer, diff::Diff, parse::EnvironmentParse};

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

pub trait Env: EnvContainer + Diff {
    fn get<T: EnvVar>(&self) -> Result<T, Error> {
        let raw = self.raw_get(T::KEY).context(NoneSnafu { key: T::KEY })?;

        // TODO: zerocopy
        T::env_deserialize(raw.clone())
            .map_err(|e| e.into())
            .context(ParseSnafu { key: T::KEY })
    }

    fn set<T: Diff>(&mut self, e: T) {
        self.raw_merge(e);
    }
}

impl<T> Env for T where T: EnvContainer + Diff {}

// misc

pub trait EnvVecExt: Env + Sized {
    fn to_vec(self) -> Vec<OsString> {
        self.to_env_diff()
            .into_iter()
            .map(|pair| {
                let mut merged = OsString::new();

                merged.push(pair.0);
                merged.push("=");
                merged.push(pair.1);

                merged
            })
            .collect()
    }
}

impl<T: Env> EnvVecExt for T {}
