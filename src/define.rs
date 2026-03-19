#[macro_export]
macro_rules! define_env {
    (
        $(#[$($attributes:tt)*])?
        $vis:vis $name:ident ($repr:ty) = $(#$parse:tt)? $key:literal
    ) => {
        $(#[$($attributes)*])?
        #[derive(Debug, Clone)]
        $vis struct $name($repr);

        $crate::define_env!(@parse $($parse)? $name($repr));

        impl $crate::EnvVar for $name {
            const KEY: &str = $key;
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
                self.0.to_string()
            }

            fn env_deserialize(raw: String) -> Result<Self, Self::Error> {
                let value = raw.parse()?;
                Ok(Self(value))
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
