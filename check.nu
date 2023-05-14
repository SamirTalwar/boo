#!/usr/bin/env nu

use std

cargo build --all-targets
cargo test
cargo clippy
cargo fmt --check

if 'IN_NIX_SHELL' in $env {
  print ''
  print 'Checking rust-toolchain.toml'
  std assert equal (open rust-toolchain.toml | get toolchain.channel) (rustc --version | split row (char space) | $in.1)
} else {
  print 'Skipping Nix checks.'
}
