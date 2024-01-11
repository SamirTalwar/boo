#!/usr/bin/env nu

use std

def run [name operation] {
  print --stderr $"+ ($name)"
  do $operation
}

run 'cargo build' { cargo build --all-targets }
run 'cargo nextest run' { cargo nextest run --no-fail-fast }
run 'cargo clippy' { cargo clippy }
run 'cargo fmt' { cargo fmt --check }
run 'cargo machete' { cargo machete }

run 'nix build' { nix --no-warn-dirty build }
run 'nix flake check' { nix --no-warn-dirty flake check }
run 'nix run' {
  let result = ('1 + 1' | nix --no-warn-dirty run)
  std assert ($result == '2')
}

print ''
print "Here's a random program for you."
print ''
cargo run --quiet --bin random-program
