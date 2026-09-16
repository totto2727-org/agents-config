# username/project

## Repository structure

```text
.github/workflows/  Validation and publishing workflows
Cargo.toml          Binary crate metadata, lint policy, and dependencies
flake.nix           Development shell, package, and overlay outputs
package.nix         Installable Nix package
rust-toolchain.toml Rust channel and rustup components
src/main.rs         Command-line entry point
tests/              End-to-end command tests
```

Replace the sample paths with the copied project's actual source and test layout.

## Development commands

### Execution rules

- Run commands from the repository root.
- Enter the environment with `nix develop` before running Cargo or Just tasks.
- Use Just recipes for the standard Rust workflow; use Nix commands directly for package and flake validation.

### Standard tasks

- `nix develop` — Enter the Nix-defined development environment; rustup resolves the configured Rust channel separately.
- `just fix` — Format source and apply supported lint fixes.
- `just check` — Verify formatting and lint findings.
- `just build` — Build the binary with every feature enabled.
- `just test` — Run all tests with every feature enabled.
- `just run` — Run the command locally.
- `just ci` — Run the complete local Rust validation gate.
- `nix build` — Build the installable Nix package.
- `nix run .` — Run the packaged command.
- `nix flake check --all-systems --no-build` — Evaluate every supported flake output without building it.

## Architecture

### Command boundaries

- Replace this item with the copied project's command parsing, application logic, and side-effect ownership.
- Keep recoverable failures explicit, preserve the documented exit behavior, and keep `unsafe` code forbidden unless a reviewed exception is necessary.

### Public interface

- Keep the complete command surface documented with caller-visible inputs, outputs, exit behavior, and constraints.
- If the project exposes a library API, inspect the canonical crate registry documentation first and use its maintained API index when one exists; otherwise provide complete inline coverage or link a substantive existing guide.
- Cover externally observable command behavior with focused end-to-end tests.

### Packaging

- Keep `package.nix` metadata, `Cargo.toml`, the flake package name, and the executable name aligned.
- Keep package and overlay outputs for distributable CLI applications so every documented Nix path remains valid; remove them only when the project is no longer distributed as a CLI.

## Development tools

- **Rustup**: Installs and selects the toolchain declared in `rust-toolchain.toml`.
- **Rust and Cargo**: Build, check, test, and run the command-line application.
- **Clippy and rustfmt**: Enforce lint and formatting policy.
- **Just**: Defines the standard development task surface.
- **Nix flakes**: Build the package and expose its overlay while pinning Nix inputs.

## Package-specific rules

- Replace this section with repository-specific invariants and remove placeholder guidance before handoff.
- Keep README Usage centered on the installed command with no options when possible.
- Keep README Setup complete: present direct `nix run`, the applicable crates.io or Git `cargo install`, `nix profile install`, and a declarative `flake.nix` example using `overlays.default` as mutually exclusive choices; state that only one setup method is required.
- Keep publishing workflows disabled until every mutable `uses:` reference in each privileged publishing workflow is pinned to an audited full commit SHA.
- Before enabling FlakeHub publication, use the official FlakeHub publishing wizard to verify the repository name, set visibility to `public`, and verify the trusted GitHub organization binding; run `nix flake check --all-systems --no-build` and confirm the workflow triggers only from protected `main`.
- Update `Cargo.lock` after dependency or toolchain changes, update `rust-toolchain.toml` when the channel or components change, and run `nix flake update` after Nix input changes.

_This AGENTS.md was generated from the [share-artifact skill](https://raw.githubusercontent.com/totto2727-org/agent/refs/heads/main/plugins/totto2727-coding/skills/share-artifact/SKILL.md) and [AGENTS template](https://raw.githubusercontent.com/totto2727-org/agent/refs/heads/main/plugins/totto2727-coding/skills/share-artifact/agents/template.md)._
