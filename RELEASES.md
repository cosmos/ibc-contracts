# Releases

This repository has multiple on-chain and off-chain components. Each component has its own release process.

Releases are published as GitHub releases against a component-prefixed tag, and the [release workflow](./.github/workflows/release.yml) attaches the build artifacts for that component. Release notes are assembled from the [conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) messages the tag contains, so a breaking change needs to be marked as one in the commit message; see [Release notes](./CONTRIBUTING.md#release-notes) in the contributing guidelines.

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

## Supported versions

Because each component is versioned independently, support is tracked per component rather than across the repository. Fixes, including security fixes, are made on `main` and released as a new tag for each affected component. Adopters running the latest release of a component are the priority for coordinated fixes; see [SECURITY.md](./SECURITY.md) for the disclosure process.

| Component | Tag prefix | Supported line |
| --- | --- | --- |
| Solidity contracts | `solidity-` | `solidity-v2.0.x` |
| CosmWasm Ethereum light client | `cw-ics08-wasm-eth-` | `cw-ics08-wasm-eth-v1.3.x` |
| Proof API | `proof-api-` | `proof-api-v0.7.x` |
| Solana programs | `solana-` | Not yet stated |
| SP1 light client programs | `sp1-programs-` | Not yet stated |

> [!IMPORTANT]
> Two gaps to close here. The Solana programs and the SP1 light client programs have no stated supported line, and the project has not committed to a support window for older lines — how long a previous major or minor line keeps receiving backported fixes, and how that is communicated to adopters. Both need deciding and stating in this table, because [SECURITY.md](./SECURITY.md) points at this section to answer "is my version still getting fixes?". Keep this table and the [Releases](./README.md#releases) section of the README in step; today they are two places recording the same fact.

Note that the on-chain components are deployed contracts and programs: publishing a new tag does not update a live deployment. Upgrading a deployment is a separate, permissioned action described in [UPGRADEABILITY.md](./UPGRADEABILITY.md), and whether a release requires a new initializer or a state migration is what the version bump above tells you.
