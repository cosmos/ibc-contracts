# Releases

This repository has multiple on-chain and off-chain components. Each component has its own release process.

## On-chain components

### Solidity Contracts

The solidity contracts releases are tagged with the `solidity` prefix. For example, `solidity-v2.0.1`. Solidity contract releases follow semantic versioning.

- **Major version bump**: A major version bump indicates that there are breaking changes in the API (regardless of whether they require a new initializer or not).
- **Minor version bump**: A minor version bump indicates that there are non-breaking changes in the API that require a new initializer (due to storage layout changes).
- **Patch version bump**: A patch version bump indicates that there are non-breaking changes in the API that do not require a new initializer (no storage layout changes).

### CosmWasm Ethereum Light Client

The CosmWasm Ethereum Light Client releases are tagged with the `cw-ics08-wasm-eth` prefix. For example, `cw-ics08-wasm-eth-v1.3.0`. CosmWasm Ethereum Light Client releases follow semantic versioning.

- **Major version bump**: A major version bump indicates that there are breaking changes in the API. (This can only happen if there is an API breaking change in `ibc-go`'s `08-wasm` module, since the API is defined by the `ibc-go` interface.)
- **Minor version bump**: A minor version bump indicates that there are non-breaking changes in the API that require a state migration (`migrate` entry point) due to storage layout changes.
- **Patch version bump**: A patch version bump indicates that there are non-breaking changes in the API that do not require a state migration (no storage layout changes).

### Solana Programs

The Solana programs releases are tagged with the `solana` prefix. For example, `solana-v1.0.0`. Solana program releases follow semantic versioning.

- **Major version bump**: A major version bump indicates that there are breaking changes in the API.
- **Minor version bump**: A minor version bump indicates that there are non-breaking changes in the API that require storage layout changes.
- **Patch version bump**: A patch version bump indicates that there are non-breaking changes in the API that do not require storage layout changes.

## Off-chain components

### Proof API

The proof API releases are tagged with the `proof-api` prefix. For example, `proof-api-v0.7.0`. Proof API releases follow semantic versioning.

The proof API does not have a major release yet, since we want to reserve the right to make breaking changes to the proof API until we have a stable API that we are confident will not require breaking changes in the future.

- **Minor version bump**: A minor version bump indicates that there are new features or improvements in the proof API.
- **Patch version bump**: A patch version bump indicates that there are bug fixes or minor improvements in the proof API.

## Release artifacts

| Tag | Artifact | Description |
| --- | --- | --- |
| `solidity-vX.Y.Z` | `solidity-contracts-solidity-vX.Y.Z.tar.gz` | Forge build artifacts (ABI and bytecode) of the deployable contracts, with `LICENSE.md` and a `TAG_NAME` file |
|  | `solidity-vX.Y.Z.spdx.json` | SPDX SBOM generated from `ibc-solidity/package.json` and `bun.lock` at the tagged commit |
| `sp1-programs-vX.Y.Z` | `sp1-ics07-tendermint-update-client` | SP1 program that verifies the next block header with the Tendermint light client |
|  | `sp1-ics07-tendermint-membership` | SP1 program that verifies the membership or non-membership of a value in a commitment root |
|  | `sp1-ics07-tendermint-uc-and-membership` | SP1 program that verifies a client update and membership in a single proof |
|  | `sp1-ics07-tendermint-misbehaviour` | SP1 program that verifies misbehaviour evidence |
|  | `sp1-programs-vX.Y.Z.spdx.json` | SPDX SBOM generated from the SP1 programs' sources and `Cargo.lock` at the tagged commit |
| `cw-ics08-wasm-eth-vX.Y.Z` | `cw_ics08_wasm_eth.wasm.gz` | Gzipped CosmWasm Ethereum light client, for the 08-wasm module of Cosmos chains |
|  | `cw-ics08-wasm-eth-vX.Y.Z.spdx.json` | SPDX SBOM generated from the light client's sources and `Cargo.lock` at the tagged commit |
| `proof-api-vX.Y.Z` | `proof-api-vX.Y.Z-linux-amd64.tar.gz` | `proof-api` binary for linux/amd64, with `LICENSE.md` |
|  | `proof-api-vX.Y.Z-linux-arm64.tar.gz` | `proof-api` binary for linux/arm64, with `LICENSE.md` |
|  | `proof-api-vX.Y.Z-linux-amd64.spdx.json` | SPDX SBOM of the linux/amd64 archive, generated from the workspace `Cargo.toml`, `Cargo.lock` and `programs/proof-api`, so it lists every crate in the workspace lockfile, not only those `proof-api` links |
|  | `proof-api-vX.Y.Z-linux-arm64.spdx.json` | SPDX SBOM of the linux/arm64 archive, generated from the same sources |
|  | `ghcr.io/cosmos/proof-api:vX.Y.Z` | Proof API container image (linux/amd64, linux/arm64) holding the binary from the archives, signed with cosign and carrying provenance and SBOM attestations |
| Every tag above | `checksums.txt` | SHA-256 checksums of the release files, the SBOMs and the attestation bundle |
|  | `checksums.txt.sigstore.json` | Cosign signature over `checksums.txt` |
|  | `<tag>.intoto.jsonl` | Build provenance and SBOM attestations, for verification without the attestations API with `gh attestation verify --bundle` |

The files are attached to the tag's [GitHub release](https://github.com/cosmos/ibc-contracts/releases) and the image is published to [GHCR](https://github.com/cosmos/ibc-contracts/pkgs/container/proof-api).

The image is also published, unsigned, for pull requests and manual runs of the [docker workflow](./.github/workflows/docker.yml), tagged with the pull request (`pr-<number>`) or the short commit SHA.

See [Signed release assets](#signed-release-assets) for how to verify the artifacts.

## Signed release assets

Pushing a `solidity-v*`, `sp1-programs-v*`, `cw-ics08-wasm-eth-v*` or `proof-api-v*` tag runs the [release workflow](./.github/workflows/release.yml). It builds the release assets of the component from the tagged commit, generates an SPDX SBOM from the component's sources and lockfiles, attests the build provenance and SBOM of each asset (for proof-api, of each archive, not of the binary inside it), signs a `checksums.txt` over them with cosign, and attaches everything to a draft release for the tag. For proof-api, it also publishes the signed [container image](#container-images) built from the same linux archives.

A tag produces `<tag>` assets and a title naming the component and version, e.g. "Proof API vX.Y.Z" for `proof-api-vX.Y.Z`; a tag with a pre-release suffix such as `-rc.1` creates a pre-release. Other tags, such as `relayer-v*`, `attestor-wf-v*`, `relayer-api-wf-v*` and `deployed-attestor`, do not run the release workflow.

The attesting, signing and publishing steps are the [shared release actions](https://github.com/cosmos/ibc/blob/main/.github/actions/README.md) from [cosmos/ibc](https://github.com/cosmos/ibc), which [cosmos/ibc-go](https://github.com/cosmos/ibc-go) uses too; each repository only defines its tag patterns and its build. Here extra jobs build the proof-api image from the linux archives and publish its multi-platform tag once the assets verify.

### Release procedure

1. Tag the release commit and push the tag (e.g. `git tag -s solidity-v3.0.3 -m solidity-v3.0.3 origin/main && git push origin solidity-v3.0.3`).
2. Wait for the [release workflow](https://github.com/cosmos/ibc-contracts/actions/workflows/release.yml) to create the draft release in [GitHub](https://github.com/cosmos/ibc-contracts/releases) with the signed assets attached. For proof-api, every asset is verified against its attestations before any image is pushed, and the draft is only created once the multi-platform image is tagged and signed, so a draft always comes with a signed image. If a job fails, re-run the failed jobs within 7 days, while the run's artifacts are kept; after that, re-run all jobs.
3. Edit the draft and write the release notes.
4. Leave `Set as a pre-release` as the workflow set it: checked for tags with a pre-release suffix such as `-rc.1`, unchecked otherwise. Every run of the release job sets it again from the tag, so a manual change does not survive a re-run.
5. Publish the release.

Push the tag with `git` before publishing the release. A release drafted from the GitHub UI only creates its tag when it is published, so its assets would be attached after publishing. The workflow runs from the tagged commit, so a release branch must contain the current `.github/workflows/release.yml`.

The workflow only accepts a commit that is on `main` or a `release/*` branch. The check runs from the tagged commit, so it only guards against tagging a commit outside those branches by mistake. What limits who can get a commit signed is the tag ruleset, which must cover `refs/tags/solidity-v*`, `refs/tags/sp1-programs-v*`, `refs/tags/cw-ics08-wasm-eth-v*` and `refs/tags/proof-api-v*` and restrict creations, updates and deletions. This repository has no tag ruleset yet, so anyone with write access can push a release tag and get its commit signed.

### Verifying release files

Download the release files, where `<tag>` is the pushed tag:

```sh
gh release download <tag> --repo cosmos/ibc-contracts --dir <tag> && cd <tag>
```

Check the signature over `checksums.txt` with cosign v3 or later, then check the files against it (on macOS, use `shasum -a 256 -c checksums.txt`):

```sh
cosign verify-blob checksums.txt \
  --bundle checksums.txt.sigstore.json \
  --certificate-identity https://github.com/cosmos/ibc-contracts/.github/workflows/release.yml@refs/tags/<tag> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
sha256sum -c checksums.txt
```

Check the build provenance of a release file:

```sh
gh attestation verify proof-api-vX.Y.Z-linux-amd64.tar.gz --repo cosmos/ibc-contracts \
  --signer-workflow cosmos/ibc-contracts/.github/workflows/release.yml \
  --source-ref refs/tags/<tag>
```

To check its SBOM attestation instead, add `--predicate-type https://spdx.dev/Document/v<version>`, where `<version>` is the SPDX version the SBOM declares: `jq -r .spdxVersion proof-api-vX.Y.Z-linux-amd64.spdx.json` prints `SPDX-<version>`. Add `--bundle <tag>.intoto.jsonl` to verify against the attestation bundle attached to the release rather than the GitHub attestations API.

### Container images

The [release workflow](./.github/workflows/release.yml) publishes `ghcr.io/cosmos/proof-api:vX.Y.Z` for a `proof-api-vX.Y.Z` tag from the release binaries, attests its build provenance and per-platform SBOMs, and signs it with cosign. The images published by the [docker workflow](./.github/workflows/docker.yml) are not signed. The workflow pushes the index by digest, signs it, and only then tags it `vX.Y.Z`, so the release tag never points at an unsigned image. Verify a release image with cosign v3 or later:

```sh
cosign verify ghcr.io/cosmos/proof-api:vX.Y.Z \
  --certificate-identity https://github.com/cosmos/ibc-contracts/.github/workflows/release.yml@refs/tags/<tag> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
gh attestation verify oci://ghcr.io/cosmos/proof-api:vX.Y.Z --repo cosmos/ibc-contracts \
  --signer-workflow cosmos/ibc-contracts/.github/workflows/release.yml \
  --source-ref refs/tags/<tag>
```

Image SBOMs are attached to the platform digests, not the multi-platform tag. List the platform digests with `docker buildx imagetools inspect ghcr.io/cosmos/proof-api:vX.Y.Z`, then pass one as `oci://ghcr.io/cosmos/proof-api@<digest>` with `--predicate-type https://spdx.dev/Document/v<version>`, using the same SPDX version as the release SBOMs.
