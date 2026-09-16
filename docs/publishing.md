# Publishing agents-config

The repository has two independent workflows:

- `CI` runs validation, and packaged-crate verification on pull requests and pushes to `main`.
- `Publish crate` is manual-only (`workflow_dispatch`). A push, tag push, pull request, or merge does not publish a crate.

No version has been published by adding these workflows.

## Required repository and registry setup

1. Create the repository as `totto2727-org/agents-config` from `totto2727-org/template-rust-simple` and merge the initialized library and workflow changes through review.
2. Protect `main`, and require the `CI` check before merging release commits.
3. Create a GitHub environment named `crates-io` with required reviewers, prevent self-review where available, and restrict deployment branches to `main`.
4. Confirm that the crates.io account controls the `agents-config` name or may create it. Checking name availability is not a reservation.
5. Create an expiring crates.io API token scoped to the exact `agents-config` crate. First publication needs `publish-new`; subsequent versions need `publish-update`. Grant only the needed scopes.
6. Store the token as `CARGO_REGISTRY_TOKEN` in the protected `crates-io` environment, not as a repository-wide secret or a checked-in file.

The environment name in workflow YAML does not configure reviewers or branch protection automatically.
An absent token causes the publishing step to fail before invoking `cargo publish`.
Do not dispatch the workflow until the environment and token scope have been reviewed.

GitHub OAuth App restrictions may prevent template generation or pull-request creation through an integration even when public repository reads succeed.
An organization administrator must approve the integration when that policy applies; do not substitute a different repository owner or bypass the approved API gateway.

## Release procedure

1. Update the package version and `Cargo.lock`, validate the change, and merge it into `main`.
2. Open Actions → Publish crate → Run workflow and select the `main` branch.
3. Review the validation result and immutable commit SHA before approving the `crates-io` deployment.

The validation job checks that:

- The workflow runs only in `totto2727-org/agents-config` from `main`.
- Formatting, Clippy, library tests with and without Rig pass.
- The packaged source builds with and without Rig, and `cargo publish --dry-run` succeeds.

Both jobs check out the same immutable `github.sha` captured when the workflow is dispatched.
All actions in the publishing workflow are pinned to reviewed commit SHAs.
The crates.io token is passed only to the final upload step, after the protected-environment approval.
The workflow grants only `contents: read`, does not request OIDC permissions, disables persisted checkout credentials, and serializes publication attempts without cancelling an in-progress upload.
If a version already exists, Cargo will reject publishing it again; create a new version instead of rewriting a published release.

## Why an API token initially

The current crates.io trusted-publishing documentation states that the first version of a crate cannot be published through trusted publishing.
This workflow therefore supports first publication through a restricted API token.
After the crate exists, maintainers may replace token authentication with a separately reviewed trusted-publisher configuration.
Merely enabling `id-token: write` does not register a trusted publisher.

## Local validation

From the repository root, use the template-derived environment:

```bash
nix develop --command just ci
nix develop --command just package
nix develop --command just publish-dry-run
```

Run packaging commands from a clean committed checkout.
The dry-run command does not upload or reserve the crate name and is not evidence that registry authentication or GitHub environment protection has been configured.

## Sources

- [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io trusted-publishing documentation](https://crates.io/docs/trusted-publishing)
- [Canonical trusted-publishing documentation source](https://github.com/rust-lang/crates.io/blob/main/svelte/src/routes/docs/trusted-publishing/+page.svelte)
- [GitHub environment protection](https://docs.github.com/en/actions/deployment/targeting-different-environments/managing-environments-for-deployment)
- [Pinned checkout action](https://github.com/actions/checkout/tree/3d3c42e5aac5ba805825da76410c181273ba90b1)
- [Pinned Determinate Nix action](https://github.com/DeterminateSystems/determinate-nix-action/tree/2a0be2498974c2b6327e19780488744384637d88)
