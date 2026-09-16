set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

fix: fix-rustfmt fix-clippy

fix-rustfmt:
    cargo fmt --all

fix-clippy:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings

check: check-rustfmt check-clippy check-workflows

check-workflows:
    actionlint .github/workflows/*.yml

check-rustfmt:
    cargo fmt --all --check

check-clippy:
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo clippy --locked --all-targets --no-default-features -- -D warnings

build:
    cargo build --locked --all-features

test:
    cargo test --locked --all-features
    cargo test --locked --no-default-features

# Validate both published feature configurations without uploading a crate.
package:
    cargo package --locked --no-default-features
    cargo package --locked --all-features

# Check registry publication without a token or any upload.
publish-dry-run:
    cargo publish --locked --registry crates-io --dry-run

ci: check build test
