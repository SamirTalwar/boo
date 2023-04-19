#!/usr/bin/env bash

set -e
set -u
set -x

cargo build
cargo test
cargo clippy
cargo fmt --check
