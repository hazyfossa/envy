use std::{ffi::OsString, marker::PhantomData};

use crate::EnvVariable;

pub enum Entry {
    Set { key: String, value: OsString },
    Unset { key: String },
}

impl Entry {
    pub fn key(&self) -> &str {
        match self {
            Self::Set { key, .. } => key,
            Self::Unset { key } => key,
        }
    }

    pub fn to_tuple(self) -> (String, Option<OsString>) {
        match self {
            Self::Set { key, value } => (key, Some(value)),
            Self::Unset { key } => (key, None),
        }
    }

    pub fn to_os_string(self) -> OsString {
        match self {
            Self::Set { key, value } => {
                let mut s = OsString::new();
                s.push(key);
                s.push("=");
                s.push(value);
                s
            }
            Self::Unset { key } => key.into(),
        }
    }
}

pub trait Diff {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry>;
}

impl<T: EnvVariable> Diff for T {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        [Entry::Set {
            key: Self::KEY.to_string(),
            value: self.env_serialize(),
        }]
    }
}

impl<T: Diff> Diff for Option<T> {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        match self {
            Some(diff) => diff.to_env_diff().into_iter().collect::<Vec<_>>(),
            None => Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unset<T>(pub PhantomData<T>);
pub fn unset<T>() -> Unset<T> {
    Unset(PhantomData)
}

impl<T: EnvVariable> Diff for Unset<T> {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        [Entry::Unset {
            key: T::KEY.to_string(),
        }]
    }
}

// NOTE: this is for untyped variables
// you would usually prefer typed ones instead
impl Diff for &str {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        let parts: Vec<&str> = self.split("=").collect();
        if parts.len() != 2 {
            panic!("Invalid environment update: {self}");
        }

        [Entry::Set {
            key: parts[0].into(),
            value: parts[1].into(),
        }]
    }
}

#[rustfmt::skip]
mod env_container_variadics {
    use super::*;

    macro_rules! var_impl {
        ( $( $name:ident )+ ) => {
            #[allow(non_camel_case_types)]
            impl<$($name: Diff),+> Diff for ($($name,)+)
            {
                fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
                    let iter = std::iter::empty();
                    let ($($name,)+) = self;
                    $(let iter = iter.chain($name.to_env_diff());)+
                    iter
                }
            }
        };
    }

    var_impl!           { a b }
    var_impl!          { a b c }
    var_impl!         { a b c d }
    var_impl!        { a b c d e }
    var_impl!       { a b c d e f }
    var_impl!      { a b c d e f g }
    var_impl!     { a b c d e f g h }
    var_impl!    { a b c d e f g h i }
    var_impl!   { a b c d e f g h i j }
    var_impl!  { a b c d e f g h i j k }
    var_impl! { a b c d e f g h i j k l }
}
