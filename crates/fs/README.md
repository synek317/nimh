# nimh_fs

[![crates.io](https://img.shields.io/crates/v/nimh_fs.svg)](https://crates.io/crates/nimh_fs)
[![docs.rs](https://docs.rs/nimh_fs/badge.svg)](https://docs.rs/nimh_fs)

FS utils for [NiMh](https://github.com/synek317/nimh), also available through `nimh::fs`.

`cp_r` and `cp_rl` provide command-like recursive copying. `CopyOptions` exposes separate
policies for an existing destination, nested files, nested directories, symlinks and type
mismatches. Its default rejects every collision; the command-like helpers use their own
documented preset.

NiMh: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

License: MIT
