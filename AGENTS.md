# template-rust-simple initialization

## Template files

| File | Meaning |
| --- | --- |
| `README.md` | Template entry point: asks an AI agent to follow this initialization guide. |
| `AGENTS.md` | Template file map and initialization instructions. Replaced after initialization. |
| `README_TEMPLATE.md` | End-user README skeleton to customize and promote to README.md. |
| `AGENTS_TEMPLATE.md` | Copied-project development guidance to customize and promote to AGENTS.md. |
| `src/`, `tests/` | Starter command and tests to replace with the new project. |
| `Cargo.toml` | Crate identity, repository metadata, dependencies, and lint policy. |
| `Cargo.lock` | Locked Rust dependencies and root package identity. |
| `rust-toolchain.toml` | Rustup toolchain, components, and targets. |
| `Justfile` | Standard development tasks for the copied project. |
| `flake.nix`, `flake.lock` | Nix development environment, package/overlay outputs, and pinned inputs. |
| `package.nix` | Nix CLI package definition and metadata. |
| `.envrc` | Optional direnv entry point for the Nix shell. |
| `.github/workflows/ci.yml` | Pre-merge Rust validation. |
| `.github/workflows/flakehub-publish-rolling.yml.disabled` | Disabled public rolling FlakeHub publication workflow. |
| `.gitignore` | Local outputs excluded from Git. |
| `LICENSE` | License and copyright holder to review for the new project. |

## Initialization

### 1. Establish the project

Use the requested repository name, crate and command names, purpose, license, and publication targets.
Resolve missing project-specific decisions with the user rather than inventing registry ownership or publishing credentials.
Work from the copied repository root and enter `nix develop` before running Cargo or Just tasks.
If using direnv, review `.envrc` before explicitly running `direnv allow`.

### 2. Replace metadata and starter files

Replace `project` in `Cargo.toml`, `package.nix`, the package/overlay attributes in `flake.nix`, and the binary reference in `tests/cli.rs`.
Replace `username/project`, version, description, keywords, repository URL, license, Nix metadata, and copyright holder with the copied project's values.
Keep or adjust the Rust channel, components, and targets in `rust-toolchain.toml` and the minimum Rust version in `Cargo.toml` to match the project's requirements.
Replace the starter source and tests with the actual project.

Retain Nix package/overlay outputs for a distributable CLI.
If the copied project is no longer a CLI and those outputs are removed, remove the corresponding README installation paths as well.
Keep shared `totto2727-org/monorepo` action references on `@main`, matching the other simple templates.
Do not create `CLAUDE.md`.

### 3. Create the project's documentation

Customize `README_TEMPLATE.md` for the actual user-facing features, usage, prerequisites, supported installation methods, and complete public command surface.
Keep Usage focused on the installed command and present supported acquisition methods as alternatives, stating that only one setup method is required.
Document direct `nix run`, the applicable crates.io or Git `cargo install`, `nix profile install`, and declarative overlay-based `flake.nix` setup when supported.
If the project introduces a library API, inspect its canonical registry documentation first and link a maintained API index when available.
Customize `AGENTS_TEMPLATE.md` for the actual file layout, development commands, boundaries, and project-specific rules.
Remove placeholders and unsupported setup methods, retaining the customized project documents' provenance footers.
Keep template initialization instructions out of the copied project's final documents.

Configure or delete the disabled publishing workflow using the instructions below before replacing this guide with the customized documents:

```bash
rm README.md AGENTS.md
mv README_TEMPLATE.md README.md
mv AGENTS_TEMPLATE.md AGENTS.md
```

### 4. Validate and hand off

Update `Cargo.lock` after changing the package name, dependencies, or toolchain, and run `nix flake update` after Nix input changes.
Run `just ci` in the Nix development environment and `nix flake check --all-systems --no-build` to validate the initialized project.
Nix package builds are not required initialization validation.
Review the final documents for remaining placeholders, obsolete template file references, and valid links, then commit the initialized project.

## Publication setup

### crates.io

- Publishing to crates.io is optional and independent of FlakeHub.
- This template does not include a crates.io publishing workflow. Do not imply that Git pushes publish the crate.
- Before publication, confirm the crate name and ownership under an account the user controls, complete the package metadata, and review the package contents with `cargo package --list` and `cargo publish --dry-run` inside `nix develop`.
- Do not publish or add publishing credentials without explicit user authorization. If crates.io distribution is not intended, set `publish = false` in `Cargo.toml` and omit crates.io installation instructions.

Reference: [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html).

### FlakeHub rolling publication

- Keep `.github/workflows/flakehub-publish-rolling.yml.disabled` disabled until the copied project explicitly enables publication. Delete it if FlakeHub publication is not needed.
- Use the [official FlakeHub publishing wizard](https://flakehub.com/new) to verify the repository name, public visibility, and trusted GitHub organization binding.
- Keep shared `totto2727-org/monorepo` action references on `@main`. Review their current implementation and pin third-party actions to audited full commit SHAs before enabling publication.
- Protect `main`, verify that the workflow publishes only pushes to `main` with job-scoped OIDC permissions, and complete `just ci` and `nix flake check --all-systems --no-build` before renaming the disabled file to `flakehub-publish-rolling.yml`.
- Do not enable publication before the required repository settings are configured. Enabling FlakeHub does not require crates.io publication.
