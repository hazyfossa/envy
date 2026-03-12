#[macro_export]
macro_rules! define_env {
    ($vis:vis $name:ident ($repr:ty) = $key:expr) => {
        impl $crate::parse::EnvironmentParse<String> for $name
        {
            type Error = <$repr as std::str::FromStr>::Err;
            fn env_serialize(self) -> String {
                self.0.to_string()
            }

            fn env_deserialize(raw: String) -> Result<Self, Self::Error> {
                let value = raw.parse()?;
                Ok(Self(value))
            }
        }

        $crate::define_env!(@impl $vis $name ($repr) = $key);
    };

    ($vis:vis $name:ident ($repr:ty) = raw $key:expr) => {
        impl $crate::parse::EnvironmentParse<std::ffi::OsString> for $name {
            type Error = <$repr as TryFrom<std::ffi::OsString>>::Error;

            fn env_serialize(self) -> std::ffi::OsString { self.0.into() }

            fn env_deserialize(raw: std::ffi::OsString) -> Result<Self, Self::Error> {
                Ok(Self(<$repr>::try_from(raw)?))
            }
        }

        $crate::define_env!(@impl $vis $name ($repr) = $key);
    };

    ($vis:vis $name:ident ($repr:ty) = custom $key:expr) => {
        $crate::define_env!(@impl $vis $name ($repr) = $key);
    };

    (@impl $vis:vis $name:ident ($repr:ty) = $key:expr) => {
        $vis struct $name($repr);

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
}
