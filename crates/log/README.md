# nimh_log

[![crates.io](https://img.shields.io/crates/v/nimh_log.svg)](https://crates.io/crates/nimh_log)
[![docs.rs](https://docs.rs/nimh_log/badge.svg)](https://docs.rs/nimh_log)

Readable logs in one line of setup for [NiMh](https://github.com/synek317/nimh), also
available through `nimh::log`.

`NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

`NiMh` crates, like much of the Rust ecosystem, describe what they are doing through the
[`log`](https://docs.rs/log) facade. Those messages go nowhere until the application
installs a logger. This crate is that logger, for applications that want readable lines
on stderr and nothing to configure:

```rust
use nimh_log::{debug, info};

nimh_log::init();

info!("listening on port {}", 8080);
debug!("this line appears when LOG_LEVEL=debug");
```

```
2026-11-11T12:41:03.123Z  INFO my_app: listening on port 8080
```

A line starts with the time in UTC, so that logs from different machines can be put side
by side. Then come the level, the module that wrote the message and the message itself.

## Choosing what is logged

`init` reads the `LOG_LEVEL` environment variable: `off`, `error`, `warn`, `info`,
`debug` or `trace`, in any letter case. Without it, messages at the `info` level and
above are logged. `init_with_level` leaves the choice to the application instead.

## Colors

Levels are colored when stderr is a terminal, and never when it is a file or a pipe.
Setting the [`NO_COLOR`](https://no-color.org) environment variable turns colors off
everywhere.

## Logging macros

The macros and level types of the `log` crate are re-exported, so an application needs
no dependency on `log` of its own.

License: MIT
