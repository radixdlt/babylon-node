# CI / Release Workflows

Reference for the GitHub Actions workflows in [`.github/workflows/`](workflows/), grouped by whether they run automatically or need to be dispatched manually. Branching/release strategy is documented in [`docs/branching-strategy.md`](../docs/branching-strategy.md) — briefly: `develop` (next protocol version) ⊇ `release/XXX` (latest published protocol version, hotfixes) ⊇ `main` (public-facing latest release). Workflow/CI changes should land on `main` first and be propagated up through `release/XXX` into `develop`.

## Automatic workflows

These trigger on push/PR/schedule/release — nothing to run by hand.

| Workflow | Trigger | Purpose | Action |
|---|---|---|---|
| `ci.yml` | Every PR; push to `develop`, `main`, `release/*` | Unit/integration tests, Sonar, Codecov, cross-compile check | [Runs](https://github.com/radixdlt/babylon-node/actions/workflows/ci.yml) |
| `docker.yml` | Every PR; push to `develop`, `main`, `release/*`; GitHub Release published | Builds `.deb`, builds/pushes private multiarch Docker image (PR/branch) or public release image (on Release), Snyk image/project monitoring | [Runs](https://github.com/radixdlt/babylon-node/actions/workflows/docker.yml) |
| `deploy-and-smoke-test.yaml` | Push to `develop` | Deploys to "gilganet" testnet via Jenkins, then runs smoke tests via Jenkins | [Runs](https://github.com/radixdlt/babylon-node/actions/workflows/deploy-and-smoke-test.yaml) |
| `add-artifacts-to-release.yml` | GitHub Release published (also `workflow_dispatch`, see below) | Builds `libcorerust` for 6 targets + the Java distribution zip, attaches both to the Release | [Runs](https://github.com/radixdlt/babylon-node/actions/workflows/add-artifacts-to-release.yml) |

## Manual workflows

| Workflow | Trigger | Purpose | Action |
|---|---|---|---|
| `publish-typescript-sdk.yml` | `workflow_dispatch` (input `package_version_number`, required) | Publishes `@radixdlt/babylon-core-api-sdk` to npmjs.org. Deliberately **not** wired to the `release` event — SDK versioning is decoupled from node release tags. | [Runs](https://github.com/radixdlt/babylon-node/actions/workflows/publish-typescript-sdk.yml) |
| `add-artifacts-to-release.yml` | `workflow_dispatch` | Can be run standalone to build the `libcorerust` artifacts (the publish-to-release jobs only run `if: github.event_name == 'release'`, so manual dispatch only exercises the build, not the upload) | [Runs](https://github.com/radixdlt/babylon-node/actions/workflows/add-artifacts-to-release.yml) |

### Release sequence

1. A GitHub **Release** is published from `release/XXX` (per the branching strategy, `release/XXX` is merged into `main` as part of this).
2. This auto-fires **`add-artifacts-to-release.yml`**: builds `libcorerust` for macOS/Linux/Windows × x86_64/aarch64, zips and uploads each to the Release, then (gated behind the **`publish-artifacts`** environment) builds and uploads the Java distribution zip.
3. This also auto-fires **`docker.yml`**'s release path: builds+pushes the public `docker.io/radixdlt/babylon-node` image (AMD64 + ARM64), joins them into a multiarch manifest, then runs Snyk container/project monitoring against the release.
4. If the release includes an SDK-relevant Core API change, separately run **`publish-typescript-sdk.yml`** manually with the new `package_version_number`.

## External dependencies per job

Shared building blocks used across nearly every workflow (not repeated per-row below):
- **`RDXWorks-actions/*`** — the org's own mirror of common third-party actions (`checkout`, `cache`, `setup-java`, `setup-node`, `codecov-action`, `snyk-actions`, `notify-slack-action`, `action-gh-release`, `configure-aws-credentials`, `aws-secretsmanager-get-secrets`, etc.).
- **`radixdlt/public-iac-resuable-artifacts`** — external repo providing reusable workflows (`docker-build.yml`, `join-docker-images-all-tags.yml`) and composite actions (`fetch-secrets`, `tailnet`, `snyk-container-monitor`).
- **`./.github/actions/fetch-secrets`** — local composite action that assumes an AWS IAM role (`RDXWorks-actions/configure-aws-credentials`) and reads a named secret from Secrets Manager (`RDXWorks-actions/aws-secretsmanager-get-secrets`). This is the core AWS integration point used by several of the workflows.
- **`./.github/actions/setup-env`** — installs the pinned Rust toolchain (must match the corresponding radixdlt-scrypto branch) + JDK 17; no secrets.
- **`./.github/actions/setup-version-properties`** / **`./.github/actions/gradle-task`** — pure git/gradle wrappers computing `VERSION_*` outputs; no secrets.

| Workflow | Secrets | AWS / Secrets Manager | Self-hosted runner(s) | GH environment | Other external deps |
|---|---|---|---|---|---|
| `ci.yml` | `COMMON_SECRETS_ROLE_ARN`; `AWS_SECRET_NAME_CODECOV`; `GITHUB_TOKEN` | `build` job fetches the Codecov secret via **external** `public-iac-resuable-artifacts/fetch-secrets` using `COMMON_SECRETS_ROLE_ARN`/`AWS_SECRET_NAME_CODECOV`, and fetches the Sonar token via the **local** `fetch-secrets` action from the literal path `github-actions/common/sonar-token` | `selfhosted-ubuntu-22.04-16-cores` (build/sonar, steadystate-integration, targeted-integration) | none | SonarCloud/SonarQube, Codecov.io |
| `docker.yml` | `DOCKERHUB_RELEASER_ROLE`; `AWS_ROLE_NAME_SNYK_SECRET`; `AWS_SECRET_NAME_DOCKERHUB`; `AWS_SECRET_NAME_SNYK`; `SNYK_ORG_ID` | Private-image multiarch join uses hardcoded OIDC role `arn:aws:iam::308190735829:role/gh-common-secrets-read-access` + Secrets Manager path `github-actions/common/dockerhub-credentials`; public/release join uses `DOCKERHUB_RELEASER_ROLE` + path `github-actions/rdxworks/dockerhub-images/release-credentials`; release image builds also pass `DOCKERHUB_RELEASER_ROLE` as `role_to_assume` into the reusable `docker-build.yml`; Snyk monitor jobs fetch via `AWS_ROLE_NAME_SNYK_SECRET`/`AWS_SECRET_NAME_SNYK`(/`AWS_SECRET_NAME_DOCKERHUB`/`SNYK_ORG_ID`) | `selfhosted-ubuntu-22.04-16-cores` (`build_deb`), `selfhosted-ubuntu-22.04-arm` (both ARM image builds) | none | Docker Hub (`private-babylon-node` for PR/branch builds, public `babylon-node` for releases), Snyk (image + project monitoring) |
| `deploy-and-smoke-test.yaml` | `BABYLON_SECRETS_ROLE_ARN`; `SECRETS_ACCOUNT_ID` | Local `fetch-secrets` with `BABYLON_SECRETS_ROLE_ARN` reads Jenkins API token from `github-actions/radixdlt/babylon-node/jenkins-api-token`; Tailscale connect step assumes `arn:aws:iam::${SECRETS_ACCOUNT_ID}:role/gh-common-secrets-read-access` and reads `github-actions/common/tailscale-public-workflows-DpiE80` | none (GitHub-hosted `ubuntu-22.04`; actual deploy/test execution happens on Jenkins, external to Actions) | none | Jenkins (`v2-jobs/job/babylon-deploy-main`, `v2-jobs/job/babylon-testnet-smoke-tests` against gilganet), Tailscale (private tailnet to reach Jenkins) |
| `add-artifacts-to-release.yml` | none | none | none (`ubuntu-22.04`, `macos-14`, `windows-2025`) | **`publish-artifacts`** (distribution-zip job only) | GitHub Releases API (`action-gh-release` asset upload) |
| `publish-typescript-sdk.yml` | `BABYLON_SECRETS_ROLE_ARN` | Local `fetch-secrets` reads npm publishing token from `github-actions/radixdlt/babylon-node/npm-publishing-secret` (yields `NODE_AUTH_TOKEN`) | none | none (relies on `workflow_dispatch` alone for gating) | npmjs.org registry |

Notes:
- Any workflow gated behind an `environment:` (`publish-artifacts`) requires whatever manual-approval/reviewer rule is configured for that environment in repo settings before the job proceeds.
- `ci.yml` and `docker.yml` both use **two different** `fetch-secrets` implementations: the **external** one (`radixdlt/public-iac-resuable-artifacts/fetch-secrets@main`, used for Snyk/Codecov) and the **local** one (`./.github/actions/fetch-secrets`, used for Sonar/Jenkins/npm). Same AWS pattern (assume role → Secrets Manager `GetSecretValue`), different repos hosting the composite action.
- `build_push_container_private*` jobs in `docker.yml` run on every PR — an ARM build only runs on a PR if it's labeled `ARM-TEST` (cost-saving guard on the self-hosted ARM pool).
- `deploy-and-smoke-test.yaml` and the Jenkins jobs it triggers are the only workflows here where the actual work happens outside GitHub Actions entirely (on Jenkins) — a GitHub Actions success only means the Jenkins job was *triggered*, not that the deploy/smoke-test itself passed (check Jenkins for that).
- `publish-typescript-sdk.yml` has a dead code path referencing `github.event.release.tag_name` for its version — the `release:` trigger that would populate it is commented out, so this branch of the version-detection logic never executes; the `package_version_number` input is always what's used in practice.
