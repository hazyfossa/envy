// Env containers

// Raw interface

use std::{collections::HashMap, env as sys, ffi::OsString, vec};

use crate::diff::{Diff, Entry};

pub trait EnvContainer {
    fn raw_get(&self, key: &str) -> Option<OsString>;
}

pub trait MutableEnvContainer {
    fn raw_merge(&mut self, diff: impl Diff);
}

// Buf

/// EnvBuf is a thin wrapper around a HashMap
pub struct EnvBuf(HashMap<String, Option<OsString>>);

impl EnvBuf {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_entries(entries: impl IntoIterator<Item = Entry>) -> Self {
        let tuples = entries.into_iter().map(Entry::to_tuple);
        Self(HashMap::from_iter(tuples))
    }

    pub fn from_diff(diff: impl Diff) -> Self {
        Self::from_entries(diff.to_env_diff())
    }

    pub fn into_diff(self) -> EnvBufDiff {
        EnvBufDiff { inner: self }
    }

    pub fn as_hashmap(&self) -> HashMap<&String, &OsString> {
        self.0
            .iter()
            .filter_map(|(k, v)| Some((k, v.as_ref()?)))
            .collect()
    }
}

impl EnvContainer for EnvBuf {
    fn raw_get(&self, key: &str) -> Option<OsString> {
        // TODO-ref: zerocopy
        Some(self.0.get(key)?.as_ref()?.clone())
    }
}

impl MutableEnvContainer for EnvBuf {
    fn raw_merge(&mut self, diff: impl Diff) {
        let entries = diff.to_env_diff().into_iter().map(Entry::to_tuple);
        self.0.extend(entries);
    }
}

impl<T: Diff> From<T> for EnvBuf {
    fn from(value: T) -> Self {
        Self::from_diff(value)
    }
}

// This is required, since the `From` impl above restricts us
// from implementing diff directly
pub struct EnvBufDiff {
    inner: EnvBuf,
}

impl Diff for EnvBufDiff {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        self.inner.0.into_iter().map(|(key, value)| match value {
            Some(value) => Entry::Set { key, value },
            None => Entry::Unset { key },
        })
    }
}

// This is sugar for buf.into_diff().to_env_diff()
impl IntoIterator for EnvBufDiff {
    // TODO: would a full `Map<...> declaration here be more performant?`
    type IntoIter = vec::IntoIter<Self::Item>;
    type Item = Entry;

    fn into_iter(self) -> Self::IntoIter {
        self.to_env_diff()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
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
        let all =
            sys::vars_os().filter_map(|(key, value)| Some((key.into_string().ok()?, Some(value))));

        let mut buf = self.append_buf;
        buf.0.extend(all);
        buf
    }

    // Make changes from this view visible across the program.
    //
    // See `std::env::set_var` for why this is unsafe.
    pub unsafe fn merge_into_global(self) {
        for entry in self.to_env_diff() {
            match entry {
                Entry::Set { key, value } => unsafe { sys::set_var(key, value) },
                Entry::Unset { key } => unsafe { sys::remove_var(key) },
            }
        }
    }
}

impl EnvContainer for OsEnv {
    fn raw_get(&self, key: &str) -> Option<OsString> {
        self.append_buf.raw_get(key).or(sys::var_os(key))
    }
}

impl MutableEnvContainer for OsEnv {
    fn raw_merge(&mut self, diff: impl Diff) {
        self.append_buf.raw_merge(diff);
    }
}

impl Diff for OsEnv {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        self.append_buf.into_diff()
    }
}

// command

impl MutableEnvContainer for std::process::Command {
    fn raw_merge(&mut self, diff: impl Diff) {
        for entry in diff.to_env_diff() {
            match entry {
                Entry::Set { key, value } => self.env(key, value),
                Entry::Unset { key } => self.env_remove(key),
            };
        }
    }
}
