use std::marker::PhantomData;

use crate::EnvVar;

pub use entry::Entry;
pub mod entry {
    use std::ffi::OsString;

    pub type Entry = (String, Option<OsString>);

    pub fn set(key: String, value: OsString) -> Entry {
        (key, Some(value))
    }

    pub fn unset(key: String) -> Entry {
        (key, None)
    }
}

pub trait Diff {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry>;
}

impl<T: EnvVar> Diff for T {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        [entry::set(Self::KEY.to_string(), self.env_serialize())]
    }
}

pub struct Unset<T>(pub PhantomData<T>);
pub fn unset<T>() -> Unset<T> {
    Unset(PhantomData)
}

impl<T: EnvVar> Diff for Unset<T> {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        [entry::unset(T::KEY.to_string())]
    }
}

// NOTE: this is for untyped variables
// you would usually prefer typed ones instead
impl Diff for &'static str {
    fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
        let parts: Vec<&str> = self.split("=").collect();
        if parts.len() != 2 {
            panic!("Invalid environment update: {self}");
        }

        [entry::set(parts[0].into(), parts[1].into())]
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

            #[allow(non_camel_case_types)]
            impl<$($name: EnvVar),+> Diff for Unset<($($name,)+)>
            {
                fn to_env_diff(self) -> impl IntoIterator<Item = Entry> {
                    let iter = std::iter::empty();
                    $(let iter = iter.chain([entry::unset($name::KEY.to_string())]);)+
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

// misc

// pub trait EnvVecExt: Diff + Sized {
//     fn to_vec(self) -> Vec<OsString> {
//         self.to_env_diff()
//             .into_iter()
//             .map(|pair| {
//                 let mut merged = OsString::new();

//                 merged.push(pair.0);
//                 merged.push("=");
//                 merged.push(pair.1);

//                 merged
//             })
//             .collect()
//     }
// }

// impl<T: Diff> EnvVecExt for T {}
