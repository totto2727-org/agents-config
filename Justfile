set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

fix: fix-rustfmt fix-clippy

fix-rustfmt:
    cargo fmt --all

fix-clippy:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings

check: check-rustfmt check-clippy

check-rustfmt:
    cargo fmt --all --check

check-clippy:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings

build:
    cargo build --all-features

test:
    cargo test --all-features
    cargo test --no-default-features

# Validate both published feature configurations without uploading a crate.
package:
    cargo package --no-default-features
    cargo package --all-features

# Check registry publication without a token or any upload.
publish-dry-run:
    cargo publish --registry crates-io --dry-run

ci: check build test
