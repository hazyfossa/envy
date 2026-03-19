# envy
Envy is a toolkit for working with environment variables. It replaces common boilerplate and provides user-friendly errors.

# getting started
The most important part of envy is the `define_env!` macro.
When you want to use an environment variable, declare this as such:

```rust
define_env!(pub Vt(u8) = "XDG_VTNR");
```

By default, this will generate code to parse this variable to a proper type using FromStr and ToString traits.
This is usually what you'd to as part of boilerplate. To learn about other options, see `examples/parsing.rs`

As a bonus, this allows you to check all variables the program interacts with by grep'ing the source code for `define_env!`

A brief example of usage:
```rust
fn main() {
    // This is a cheap way to reference local env from anywhere
    let env = OsEnv::new_view();

    let vt = env.get::<Vt>().unwrap();
    println!("Running on vt {}", *vt)
}
```

See `examples/diffs.rs` to learn about intuitive env modification.
