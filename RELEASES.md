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

## Cut a release

Pushing a `solidity-v*`, `sp1-programs-v*`, `cw-ics08-wasm-eth-v*`, or `proof-api-v*` tag runs the [release workflow](./.github/workflows/release.yml). The workflow builds the component from the tagged commit, signs it, and creates a draft [GitHub release](https://github.com/cosmos/ibc-contracts/releases) with the files. For `proof-api`, it also publishes the signed image `ghcr.io/cosmos/proof-api:vX.Y.Z`. The [shared release actions](https://github.com/cosmos/ibc/blob/main/.github/actions/README.md) explain the SBOMs, attestations, and signatures it produces.

Each SBOM is generated from the component's sources and lockfiles, so it lists every dependency in the lockfile, including ones the artifact does not link. Other tags, such as `relayer-v*`, `attestor-wf-v*`, `relayer-api-wf-v*`, and `deployed-attestor`, do not run the workflow.

1. Create the tag on a commit on `main` or a `release/*` branch. Push it with `git`:

   ```sh
   git tag -s solidity-v3.0.3 -m solidity-v3.0.3 origin/main
   git push origin solidity-v3.0.3
   ```

   Or create it in the GitHub UI while publishing a release, with the notes and **Set as a pre-release** filled in. The UI creates the tag only on publish, so the release is live before its files are signed and, for `proof-api`, before its image exists. The workflow attaches the signed files to the published release a few minutes later. Steps 3 and 4 are then already done.

2. Wait for the workflow to attach the signed files. For a tag pushed with `git`, it creates a draft release only after every file is verified and, for `proof-api`, the image is signed. If a job fails, re-run the failed jobs within 7 days, while the run still keeps its build artifacts. After that, re-run all jobs. If the release is already published with some of its files, delete them first. A full re-run stops for a published release that has files.
3. Write the release notes in the draft. Leave **Set as a pre-release** as it is. The workflow sets it from the tag on every run: checked for a suffix such as `-rc.1`, unchecked otherwise.
4. Publish the release.

The workflow runs from the tagged commit. A release branch therefore needs the current `.github/workflows/release.yml`. For the same reason, the workflow's branch check only catches mistakes.

A tag ruleset decides who can release. It must cover the four tag prefixes above and restrict creating, updating, and deleting those tags. This repository has no tag ruleset yet, so anyone with write access can push a release tag and get its commit signed.

## Verify a release

The commands below need the GitHub CLI and cosign v3 or later, the version the workflow signs with. Replace `<tag>` with the release tag, such as `proof-api-v0.7.0`, and `vX.Y.Z` with its version.

### Verify the files

Download the release:

```sh
gh release download <tag> --repo cosmos/ibc-contracts --dir <tag>
```

In the `<tag>` directory, check the signature on `checksums.txt`:

```sh
cosign verify-blob checksums.txt \
  --bundle checksums.txt.sigstore.json \
  --certificate-identity https://github.com/cosmos/ibc-contracts/.github/workflows/release.yml@refs/tags/<tag> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

The two `--certificate` flags accept only a signature from this repository's release workflow, run for `<tag>`.

Check that every file matches `checksums.txt`, which proves it is unchanged since signing. On macOS, use `shasum -a 256 -c checksums.txt` instead.

```sh
sha256sum -c checksums.txt
```

Check the build provenance of a release file. For `proof-api`, the attestation covers the `.tar.gz` file, not the binary inside it, so check the archive before you extract it.

```sh
gh attestation verify proof-api-vX.Y.Z-linux-amd64.tar.gz --repo cosmos/ibc-contracts \
  --signer-workflow cosmos/ibc-contracts/.github/workflows/release.yml \
  --source-ref refs/tags/<tag>
```

`--signer-workflow` and `--source-ref` accept only an attestation from the release workflow, run for `<tag>`.

Two flags change what this command checks:

- `--predicate-type https://spdx.dev/Document/v<version>` checks the SBOM attestation instead of the build provenance. `<version>` is the SPDX version the SBOM declares. `jq -r .spdxVersion proof-api-vX.Y.Z-linux-amd64.spdx.json` prints it as `SPDX-<version>`.
- `--bundle <tag>.intoto.jsonl` checks against the attestation file attached to the release, without GitHub's attestations API.

### Verify the image

Check the signature, with the same `--certificate` flags as for `checksums.txt`:

```sh
cosign verify ghcr.io/cosmos/proof-api:vX.Y.Z \
  --certificate-identity https://github.com/cosmos/ibc-contracts/.github/workflows/release.yml@refs/tags/<tag> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

Check the build provenance:

```sh
gh attestation verify oci://ghcr.io/cosmos/proof-api:vX.Y.Z --repo cosmos/ibc-contracts \
  --signer-workflow cosmos/ibc-contracts/.github/workflows/release.yml \
  --source-ref refs/tags/<tag>
```

The image tag points at one image per platform. Each platform image is a separate build, so each has its own SBOM attestation. List the platform digests with `docker buildx imagetools inspect ghcr.io/cosmos/proof-api:vX.Y.Z`. Then run the command above on `oci://ghcr.io/cosmos/proof-api@<digest>` with the `--predicate-type` flag.

The `pr-<number>` and short commit SHA images from the [docker workflow](./.github/workflows/docker.yml) are unsigned development builds.
