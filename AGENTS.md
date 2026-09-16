# agents-config

## Repository structure

`src/` owns shared configuration parsing, validated provider resolution, credential-file handling, and optional Rig conversion.
This is an independent Git repository under `package/agents-config` in the virtual monorepo, not a GlossShift module.
The development scaffold is adapted from `totto2727-org/template-rust-simple` at commit `08ffe1d2d6a27e8d0b4c25f98e5a89a96d8ec1c1`.
`rust-toolchain.toml`, `.envrc`, `flake.nix`, `flake.lock`, `Justfile`, and `.github/workflows/` own development and release automation.
As this is a library, the template's CLI entry point, CLI tests, `package.nix`, installable Nix package/overlay outputs, and FlakeHub publishing workflow are not used.

## Development commands

Run commands from this repository root.
Enter `nix develop` before Cargo or Just commands; for non-interactive validation use `nix develop --command just ci`.
Rustup selects the stable toolchain and Clippy/rustfmt components from `rust-toolchain.toml`; Nix provides rustup and Just rather than a second Rust toolchain.
Review `.envrc` before explicitly allowing direnv.

- `just` lists available tasks without running checks or changing files.
- `just fix` applies formatting and supported Clippy fixes.
- `just check` checks formatting, strict Clippy with and without optional features, and GitHub Actions with actionlint/ShellCheck.
- `just build` builds with all features enabled.
- `just test` runs tests with and without Rig.
- `just ci` runs the complete local validation gate.
- `just package` verifies the packaged crate with and without Rig; run from a clean committed checkout.
- `just publish-dry-run` checks registry publication without uploading or requiring a publishing token.
- `nix flake check --all-systems --no-build` evaluates all supported development-shell outputs.

## Architecture

Keep filesystem and environment discovery at the loading boundary.
Only the explicit shared configuration path or the home-directory default selects providers; do not introduce implicit project-directory discovery.
Keep provider settings independent of consumer UI and prompts.
`ProviderAdapter` permits downstream library types, while the optional `rig` feature owns Rig-specific connection and agent construction.
First-chunk and stream-idle timeout durations remain a neutral application streaming policy, not an implicit Rig timeout guarantee.

## Package-specific rules

- Never read real user credentials in tests; use explicit temporary paths.
- Preserve existing configuration and credential content when initializing missing templates.
- Keep credentials mode `0600` on Unix and avoid exposing secret values in `Debug`, error chains, or raw TOML diagnostics.
- Validate URL, path, header, credential-reference, and timeout invariants before constructing resolved providers.
- Keep both no-feature and Rig-feature builds usable without GlossShift or GPUI dependencies.
- Use placeholder credentials in examples and test fixtures only.
- Keep temporary experiments and progress logs out of commits.

## Publication

Read [the publishing guide](./docs/publishing.md) before changing or running the crates.io workflow.
Publishing is manual-only, requires an existing version-matching tag on main, and uses a token stored in the protected `crates-io` GitHub environment.
Do not dispatch publishing, create releases, or install secrets merely to test CI.
Keep the template's shared Nix action on `@main` in ordinary read-only CI; the privileged publish workflow uses directly reviewed SHA-pinned actions instead.
Never treat a local dry run as proof of registry ownership or configured GitHub environment protection.

_This AGENTS.md was generated from the [share-artifact skill](https://raw.githubusercontent.com/totto2727-org/agent/refs/heads/main/plugins/totto2727-coding/skills/share-artifact/SKILL.md) and [AGENTS template](https://raw.githubusercontent.com/totto2727-org/agent/refs/heads/main/plugins/totto2727-coding/skills/share-artifact/agents/template.md)._
