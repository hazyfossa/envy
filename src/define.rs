#[macro_export]
macro_rules! define_env {
    ($vis:vis $name:ident ($repr:ty) = auto parse $key:expr) => {
        impl $crate::parse::EnvironmentParse<String> for $name
        {
            fn env_serialize(self) -> String {
                self.0.to_string()
            }

            fn env_deserialize(raw: String) -> $crate::parse::Result<Self> {
                let value = raw.parse().map_err(|e|
                    $crate::parse::ParseErrorHatch(e)
                )?;

                Ok(Self(value))
            }
        }

        $crate::define_env!($vis $name ($repr) = $key);
    };

    ($vis:vis $name:ident ($repr:ty) = parse $key:expr) => {
        impl $crate::parse::EnvironmentParse<std::ffi::OsString> for $name {
            fn env_serialize(self) -> std::ffi::OsString { self.0.env_serialize() }

            fn env_deserialize(raw: std::ffi::OsString) -> $crate::parse::Result<Self> {
                Ok(Self(<$repr>::env_deserialize(raw)?))
            }
        }

        $crate::define_env!($vis $name ($repr) = $key);
    };

    ($vis:vis $name:ident ($repr:ty) = $key:expr) => {
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
