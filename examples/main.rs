#![allow(dead_code)]
use envy::{Env, define_env, diff::Diff, parse::EnvironmentParse};

// The most important part of envy is the `define_env!` macro.
// When you want to interact with an environment variable, declare this as such:
define_env!(pub Vt(u8) = "XDG_VTNR");

// By default, this will generate code to parse this variable to a proper type using FromStr and ToString traits.
// This is usually what you'd to as part of boilerplate.

// If you don't want your variable to be constrained to unicode, you can define it as `#raw`.
// Note that this typically restricts your choice of types to OsString and PathBuf.
define_env!(pub XAuthority(std::path::PathBuf) = #raw "XAUTHORITY");

// If you need custom parsing for a variable, define it as `#custom`:
define_env!(pub ListOfValues(Vec<u32>) = #custom "VARIABLE");

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

fn main() {
    // this function creates a new view of the system environment
    // this allows for env modification with zero `unsafe`
    //
    // it is also cheap to call
    // no allocation is performed until (unless) a new variable is set
    let env = envy::container::OsEnv::new_view();

    use envy::Env;
    let vt = env.get::<Vt>().unwrap();

    // The * here is to abandon the newtype temporarily
    // You can also do vt.0 to do the same permanently
    println!("Running on vt {}", *vt)
}

fn diffs() -> Result<(), envy::Error> {
    // A diff is a collection of env variables
    let mut env = envy::container::OsEnv::new_view();

    let vt = env.get::<Vt>()?;
    let xauthority = env.get::<XAuthority>()?;

    let session = (vt, xauthority);

    // An individual variable is a diff
    // A tuple of variables is a diff
    // An env container is a diff

    // Diffs can be passed to commands
    let _command = std::process::Command::new("/bin/bash").envs(session.to_env_diff());

    // Diffs can also be applied to containers
    env.set(ListOfValues(vec![1, 2, 3]));

    Ok(())
}
