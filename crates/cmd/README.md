# nimh_cmd

[![crates.io](https://img.shields.io/crates/v/nimh_cmd.svg)](https://crates.io/crates/nimh_cmd)
[![docs.rs](https://docs.rs/nimh_cmd/badge.svg)](https://docs.rs/nimh_cmd)

Running external commands for [NiMh](https://github.com/synek317/nimh), also available
through `nimh::cmd`.

`NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

`std::process::Command` is precise and verbose. Starting a program, checking how it ended
and decoding what it printed are three separate steps, and forgetting the second one is an
easy mistake to make. `Cmd` folds them into one call that gives either the result or an
error that tells the whole story:

```
`git rev-parse HEAD` failed (exit status: 128):
  fatal: not a git repository (or any of the parent directories): .git
```

## Three ways to run a command

| The command is run to | Call | What it prints |
| --- | --- | --- |
| do some work | `run` | is shown as it appears |
| produce a value | `read` | is captured and returned |
| answer a yes/no question | `succeeds` | is discarded |

```rust
use nimh_cmd::Cmd;

Cmd::new("cargo")
    .args(["build", "--release"])
    .dir("/srv/app")
    .run()?;

let commit = Cmd::new("git").args(["rev-parse", "HEAD"]).read()?;

if !Cmd::new("git").args(["diff", "--quiet"]).succeeds()? {
    println!("{commit} has uncommitted changes");
}
```

For a program with a list of arguments and nothing else, `nimh_cmd::run` and
`nimh_cmd::read` are shorter still:

```rust
nimh_cmd::run("git", ["fetch", "--all"])?;
let branch = nimh_cmd::read("git", ["branch", "--show-current"])?;
```

## No shell

Arguments reach the program exactly as they were given. No shell reads them on the way,
so spaces, quotes and `$` need no escaping and cannot smuggle in another command. Errors
and logs do show the command the way a shell would expect it, quotes included, so that it
can be copied and tried by hand.

## Logs

Every command is announced at the debug level of the `log` facade before it starts.
Values of environment variables given to a command never reach the logs, because they
are often secrets.

License: MIT
