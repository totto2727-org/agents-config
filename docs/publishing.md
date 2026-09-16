# Publishing agents-config

`CI` validates the crate and its packaged source on pull requests and pushes to `main`.
`Publish crate` runs on every push to `main`, including merged pull requests.
It reads the package name and version using `cargo metadata` and checks the exact version on crates.io.
HTTP 200 skips publication, HTTP 404 runs `cargo publish --registry crates-io`, and other responses fail the job rather than assuming the version is unpublished.
Registry lookup failures and Cargo publication failures are reported by the job.
The local `.github/actions/publish-crate/action.yaml` composite action owns version lookup and publication.
Like `publish-npm`, it accepts a required `working-directory` input, loads the caller workspace’s Nix environment, and inherits authentication from the caller’s environment.
Its version-check step writes its decision to `GITHUB_OUTPUT`; a separate publish step runs only for an unpublished version.
The calling workflow owns checkout, Nix installation, the main-push trigger, concurrency, and the `crates-io` environment.
The action has no repository-specific crate name or path, allowing it to move to the shared monorepo later.
Currently it targets crates.io only, not custom Cargo registries.

## Setup

- Protect `main` and require the PR CI check before merging.
- Configure the `crates-io` GitHub environment and store `CARGO_REGISTRY_TOKEN` there.
- Use an expiring token scoped to `agents-config`, with `publish-new` for initial publication and `publish-update` for subsequent versions.
- Configure environment reviewers if approval before upload is desired.

The workflow does not create environment protections or credentials automatically.
A missing or invalid token is handled by Cargo when publication is attempted.
Both workflows load the same Nix development environment using `eval "$(nix print-dev-env "$GITHUB_WORKSPACE#default")"`.

## Release procedure

1. Update the version in `Cargo.toml` and `Cargo.lock` through a reviewed pull request.
2. Merge into `main` to trigger publishing. No release tag or manual workflow dispatch is required.
3. Check the publishing workflow result. An already-published version is a successful no-op.

A push directly to `main` also triggers this workflow, so enforce merge policies using branch protection.
Publication is serialized to avoid overlapping runs of this workflow.
If an upload times out, check crates.io before retrying.

## Local validation

```bash
nix develop --command just ci
nix develop --command just publish-dry-run
```

Run packaging commands from a clean committed checkout.
A dry run does not upload or reserve a crate name, and does not verify publishing credentials.
Do not merge or publish merely to test the workflow.

## Sources

- [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub environment protection](https://docs.github.com/en/actions/deployment/targeting-different-environments/managing-environments-for-deployment)
