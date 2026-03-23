#[macro_export]
macro_rules! define_env {
    // newtype
    (
        $(#[$($attributes:tt)*])?
        $vis:vis $name:ident ($repr:ty) = $(#$parse_modifier:tt)? $key:literal
    ) => {
        $crate::define_env!(@newtype
            $(#[$($attributes)*])?
            $vis $name($repr)
        );

        $crate::define_env!(@parse $($parse_modifier)? $name($repr));

        impl $crate::EnvVar for $name {
            const KEY: &str = $key;
        }
    };

    // existing
    (
        $name:ident = $(#$parse_modifier:tt)? $key:literal
    ) => {
        $crate::define_env!(@parse $($parse_modifier)? $name($name));

        impl $crate::EnvVar for $name {
            const KEY: &str = $key;
        }
    };

    (@newtype
        $(#[$($attributes:tt)*])?
        $vis:vis $name:ident ($repr:ty)
    ) => {
        $(#[$($attributes)*])?
        #[derive(Debug, Clone)]
        $vis struct $name($repr);

        impl From<$repr> for $name {
            fn from(value: $repr) -> Self {
                Self(value)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $repr;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };

    (@parse $name:ident ($repr:ty)) => {
        impl $crate::parse::EnvironmentParse<String> for $name {
            type Error = <$repr as std::str::FromStr>::Err;

            fn env_serialize(self) -> String {
                self.to_string()
            }

            fn env_deserialize(raw: String) -> Result<Self, Self::Error> {
                let value = raw.parse::<$repr>()?;
                Ok(Self::from(value))
            }
        }
    };

    (@parse raw $name:ident ($repr:ty)) => {
        impl $crate::parse::EnvironmentParse<std::ffi::OsString> for $name {
            type Error = <$repr as TryFrom<std::ffi::OsString>>::Error;

            fn env_serialize(self) -> std::ffi::OsString {
                self.0.into()
            }

            fn env_deserialize(raw: std::ffi::OsString) -> Result<Self, Self::Error> {
                let value = <$repr>::try_from(raw)?;
                Ok(Self(value))
            }
        }
    };

    // It's the caller responsibility to implement `EnvironmentParse`
    (@parse custom $name:ident ($repr:ty)) => {};
}
