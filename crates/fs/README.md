# nimh_fs

[![crates.io](https://img.shields.io/crates/v/nimh_fs.svg)](https://crates.io/crates/nimh_fs)
[![docs.rs](https://docs.rs/nimh_fs/badge.svg)](https://docs.rs/nimh_fs)

File system helpers modelled after familiar Unix commands, for
[NiMh](https://github.com/synek317/nimh), also available through `nimh::fs`.

`NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

The standard library copies one file at a time and creates a symbolic link only where
nothing exists yet. Scripts reach for `cp -R` and `ln -sfn` instead, and so can Rust code:

```rust
nimh_fs::cp_r("target/release", "/srv/releases/v1.2.0")?;
nimh_fs::ln_sfn("/srv/releases/v1.2.0", "/srv/current")?;
```

Errors name every path involved, so that a failure can be understood without a debugger:

```
cannot copy file 'target/release/app' to '/srv/releases/v1.2.0/app': Permission denied (os error 13)
```

## Copying a tree

| Function | Like | Symbolic links |
| --- | --- | --- |
| `cp_r` | `cp -R` | are copied as links, with the same target path |
| `cp_rl` | `cp -RL` | are followed, and what they point to is copied |

Both copy into an existing destination directory, overwrite files, merge directories and
refuse to put a file where a directory is, or the other way round.

When that is not the right behavior, `CopyOptions` spells out every decision: what to do
with an existing destination, with each kind of collision inside the tree and with
symbolic links. Its default rejects every collision, so nothing is lost by accident:

```rust
use nimh_fs::{
    ConflictPolicy, CopyOptions, DestinationDirectory, DestinationPolicy,
    DirectoryConflict, FileConflict,
};

let options = CopyOptions::new()
    .with_destination(DestinationPolicy {
        directory: DestinationDirectory::Merge,
        ..DestinationPolicy::new()
    })
    .with_conflicts(ConflictPolicy {
        directory: DirectoryConflict::Merge,
        file: FileConflict::Overwrite,
        ..ConflictPolicy::new()
    });

options.copy("assets", "public")?;
```

## Switching a symbolic link

`ln_sfn` replaces a link atomically: whoever follows it at any moment finds either the
old target or the new one. That makes it safe to switch a `current` link while a server
is reading through it.

License: MIT
