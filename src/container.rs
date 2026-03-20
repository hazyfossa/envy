// Env containers

// Raw interface

use std::{collections::HashMap, env as sys, ffi::OsString};

use crate::diff::Diff;

type EnvEntry = (String, OsString);

// TODO: split in two to allow for &-impls, non-view OsEnv
pub trait EnvContainer {
    fn raw_get(&self, key: &str) -> Option<OsString>;
    fn raw_merge(&mut self, diff: impl Diff);
}

// Buf

/// EnvBuf is a thin wrapper around a HashMap
pub struct EnvBuf(HashMap<String, OsString>);

impl EnvBuf {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_values(values: impl IntoIterator<Item = EnvEntry>) -> Self {
        Self(values.into_iter().collect())
    }

    pub fn into_hashmap(self) -> HashMap<String, OsString> {
        self.0
    }
}

impl EnvContainer for EnvBuf {
    fn raw_get(&self, key: &str) -> Option<OsString> {
        // TODO-ref: zerocopy
        self.0.get(key).cloned()
    }

    fn raw_merge(&mut self, diff: impl Diff) {
        self.0.extend(diff.to_env_diff());
    }
}

impl Diff for EnvBuf {
    fn to_env_diff(self) -> impl IntoIterator<Item = (String, OsString)> {
        self.0
    }
}

// System

pub struct OsEnv {
    append_buf: EnvBuf,
}

impl OsEnv {
    /// This creates a new view os the system environment
    ///
    /// It is cheap to call, no allocation is performed until a variable is modified.
    ///
    /// Keep in mind that setting a variable is scoped per view.
    /// For example, in this case:
    /// ```
    /// let a = EnvOs::new_view();
    /// a.set("foo=bar")
    ///
    /// let b = EnvOs::new_view();
    /// let x = b.raw_get("foo")
    /// ```
    /// x will either be None, or what has been in the system's native env.
    ///
    /// This also means that changes to views won't affect the current process env,
    /// eliminating spooky action at a distance.
    ///
    /// If you want to concurrently share an env view across your system,
    /// you can do it much like with any other struct.
    /// A common approach for async is Arc<Mutex<...>>
    pub fn new_view() -> Self {
        Self {
            append_buf: EnvBuf::new(),
        }
    }

    // Get a full copy of env, including both changes from this local view
    // and pre-existing variables.
    //
    // For example, this might be useful when sharing env over a network.
    // When setting env for a subprocess, use to_env_diff instead.
    pub fn dump(self) -> EnvBuf {
        // NOTE: this ignores variables with non-utf8 keys
        let all = sys::vars_os().filter_map(|(key, value)| Some((key.into_string().ok()?, value)));

        let mut buf = self.append_buf;
        buf.0.extend(all);
        buf
    }

    // Make changes from this view visible across the program.
    //
    // See `std::env::set_var` for why this is unsafe.
    pub unsafe fn merge_into_global(self) {
        for (key, value) in self.to_env_diff() {
            unsafe {
                sys::set_var(key, value);
            }
        }
    }
}

impl EnvContainer for OsEnv {
    fn raw_get(&self, key: &str) -> Option<OsString> {
        self.append_buf.raw_get(key).or(sys::var_os(key))
    }

    fn raw_merge(&mut self, diff: impl Diff) {
        self.append_buf.raw_merge(diff);
    }
}

impl Diff for OsEnv {
    fn to_env_diff(self) -> impl IntoIterator<Item = (String, OsString)> {
        self.append_buf.to_env_diff()
    }
}
