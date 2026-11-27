# nimh_env

[![crates.io](https://img.shields.io/crates/v/nimh_env.svg)](https://crates.io/crates/nimh_env)
[![docs.rs](https://docs.rs/nimh_env/badge.svg)](https://docs.rs/nimh_env)

Typed environment variables for [NiMh](https://github.com/synek317/nimh), also available
through `nimh::env`.

`NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

Configuration read from the environment arrives as text, and three things can go wrong on
its way to a usable value: the variable is missing, it is not Unicode, or it does not
parse. `std::env::var` reports the first two without naming the variable and leaves the
third to the caller. Here one call does all of it, and the error says which variable is
wrong, what it holds and why that is not acceptable:

```
environment variable `PORT` has invalid value "http": invalid digit found in string
```

## Reading a variable

A value can be of any type that implements `FromStr`. There are three ways to ask for
it, one for each meaning a missing variable can have:

```rust
use std::net::IpAddr;

// Required: the application cannot work without it.
let port: u16 = nimh_env::var("PORT")?;

// Optional, with a value to use instead.
let workers: usize = nimh_env::var_or("WORKERS", 4)?;

// Optional, and its absence means something.
let bind_address: Option<IpAddr> = nimh_env::var_opt("BIND_ADDRESS")?;
```

A variable set to something unusable is always an error, also when a default exists. A
typo in `WORKERS=fuor` should stop the application, not start it quietly with 4 workers.

The value is parsed exactly as it was set. It is not trimmed, and an empty value
(`PORT=`) counts as set.

## Naming a variable once

`EnvVar` is a variable name that has been checked. As a `const` it is checked by the
compiler, lives in one place and can be read wherever it is needed:

```rust
use nimh_env::EnvVar;

const WORKERS: EnvVar = EnvVar::new("NIMH_DOCS_WORKERS");

let workers: usize = WORKERS.get_or(4)?;
assert_eq!(workers, 4);
```

## Logs

Falling back to a default is announced at the debug level of the `log` facade, with the
name of the variable only. Values never reach the logs, because they are often secrets.

License: MIT
