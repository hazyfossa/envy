use envy::{define_env, parse::EnvironmentParse};

// If you don't want your variable to be constrained to unicode, you can define it as `#raw`.
// Note that this typically restricts your choice of types to OsString and PathBuf.
define_env!(XAuthority(std::path::PathBuf) = #raw "XAUTHORITY");

// You can use attributes and visibility syntax as usual
// Debug, Clone and Deref<Target={inner}> are derived by default
define_env!(
    #[derive(PartialEq, Eq)]
    pub Count(u8) = "MY_COUNT"
);

// If you need custom parsing for a variable, define it as `#custom`:
define_env!(ListOfValues(Vec<u32>) = #custom "VARIABLE");

impl EnvironmentParse<String> for ListOfValues {
    type Error = std::num::ParseIntError;

    fn env_serialize(self) -> String {
        self.0
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(";")
    }

    fn env_deserialize(raw: String) -> Result<Self, Self::Error> {
        let list = raw
            .split(";")
            .map(|s| s.parse::<u32>())
            .collect::<Result<_, _>>()?;

        Ok(Self(list))
    }
}

// You can also implement EnvironmentParse<OsString>
// but parsing arbitrary-encoding strings is notoriously difficult

// Also, for now, returning references or zero-copy values (i.e Cow)
// from parsers is not supported. This might change in the future

// ---

// If you want to use a pre-existing type, it is possible:
enum Example {
    A,
    B,
}

impl std::str::FromStr for Example {
    type Err = std::convert::Infallible;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl ToString for Example {
    fn to_string(&self) -> String {
        todo!()
    }
}

define_env!(Example = "EXAMPLE");

// parse modifiers (#...) will defer to correct traits on your types

// Note, however, that it's usually a good practice to keep your env-types
// separate from logic-types. In most cases, you should use #custom instead

fn main() {}
