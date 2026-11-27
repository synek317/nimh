# nimh_error

[![crates.io](https://img.shields.io/crates/v/nimh_error.svg)](https://crates.io/crates/nimh_error)
[![docs.rs](https://docs.rs/nimh_error/badge.svg)](https://docs.rs/nimh_error)

Readable error reports for [NiMh](https://github.com/synek317/nimh), also available through
`nimh::error`.

`NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

An error rarely tells the whole story on its own. *"No such file or directory"* is how the
story ends; how it began (which file, and what the application wanted with it) is kept in
the errors that wrap it, one `source()` after another. Printing an error with `{}` shows
only the outermost one, and returning it from `main` shows its `Debug` form. This crate
prints the whole story instead.

## An error with its causes

`DisplayChain` adds `display_chain()` to every error. `{}` puts the chain on one line,
which suits logs. `{:#}` gives every cause its own line, which suits a terminal.

```rust
use nimh_error::DisplayChain;
#
#
#
#

// `load_config` fails with a `ConfigError` whose source is a `ParseIntError`.
let error = load_config().unwrap_err();

assert_eq!(
    error.display_chain().to_string(),
    "cannot load the configuration: invalid digit found in string",
);
assert_eq!(
    format!("{:#}", error.display_chain()),
    "cannot load the configuration\n  caused by: invalid digit found in string",
);
```

## Reporting from `main`

`report` turns the result of the whole program into an exit code and, on failure, prints
the error with its causes to stderr.

```rust
use std::error::Error;
use std::process::ExitCode;

fn main() -> ExitCode {
    nimh_error::report(run())
}

fn run() -> Result<(), Box<dyn Error>> {
    let port: u16 = "http".parse()?;
    println!("listening on port {port}");
    Ok(())
}
```

```
error: invalid digit found in string
```

License: MIT
