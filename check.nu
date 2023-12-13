#!/usr/bin/env nu

use std

def run [name operation] {
  print --stderr $"+ ($name)"
  do $operation
}

run 'cargo build' { cargo build --all-targets }
run 'cargo nextest run' { cargo nextest run }
run 'cargo clippy' { cargo clippy }
run 'cargo fmt' { cargo fmt --check }
run 'cargo machete' { cargo machete }

run 'nix build' { nix build }
run 'nix flake check' { nix flake check }

print ''
print "Here's a random program for you."
print ''
cargo run --quiet --bin random-program
