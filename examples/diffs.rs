#![allow(unused)]
use envy::container::EnvBuf;
use envy::{Env, container::OsEnv, define_env, diff::Diff};

use std::collections::HashMap;
use std::{ffi::OsString, path::PathBuf};

define_env!(pub Username(String) = "USERNAME"   );
define_env!(pub Home(PathBuf)    = #raw "HOME"  );
define_env!(pub Shell(PathBuf)   = #raw "SHELL" );

fn diffs() -> Result<(), envy::Error> {
    let mut env = OsEnv::new_view();

    // A diff is a collection of env variables

    // An individual variable is a diff
    let name = env.get::<Username>()?;
    let home = env.get::<Home>()?;

    // A tuple of variables is also a diff
    let session = (name, home);

    // Diffs can (expectedly) be passed to commands
    let mut command = std::process::Command::new("/bin/bash");
    command.envs(session.to_env_diff());

    Ok(())
}

// Instead of juggling HashMaps or unsafely writing to global env,
// you're encouraged to receive and return typed diffs in functions.

// This turns your environment from implicit to explicit.
// No more .expect("this is set somewhere above")

// without envy
fn setup_session_std(env: &mut HashMap<String, OsString>) {
    // Look here to figure out what is being modified
    todo!()
}

// with envy
type Session = (Username, Home, Shell);
fn session_envy() -> Session {
    todo!()
}

fn containers() {
    // a container is something that can hold key-value pairs

    // if you do not want to think about diff types, use a buffer container
    let mut buf = EnvBuf::new();

    // you can apply any diff to a buffer
    buf.set(Username("toor".to_string()));

    // a buffer is itself a diff
    let changes = buf.to_env_diff();

    // For convenience, OsEnv will allocate a local "view" buffer on first set
    // See function docs for more details
    let mut env = OsEnv::new_view();
    env.set(Shell("/bin/sh".into()));
}

fn main() {}
