# Development

Start with [Architecture](architecture.md) for component responsibilities and [Testing](TESTING.md)
for verification commands. Paths below are relative to the light-client project directory.

## Docker build workflow

`make build-docker` builds the three guest ELFs and matching VKeys in containers. A standalone
`tooling/generate-vkeys` package derives keys locally without SP1 network/AWS features.
The same command also builds Linux host tools, the Proof API binary, and Solidity contracts.
Only Docker, Make, a shell, and a checked-out working tree are needed on the host. See
[Docker build instructions](../README.md#build-with-docker) for output paths and individual targets.
The wrapper is `scripts/docker-build.sh`; build dependencies are in `deploy/build/Dockerfile`,
and `scripts/docker-build-entrypoint.sh` calls the existing build/dependency scripts.

Native tools and macOS SDK settings are not inherited by the container. Linux compilation and
container tool caches use `target/docker/`, separately from native `target/` outputs. Guest ELFs
and Solidity outputs retain their normal host directories. The image bundles the guest compiler
from SP1 v6.1.0 and registers it as the `succinct` toolchain. The container entrypoint calls
`scripts/build-sp1.sh --local`, so compilation runs directly without a Docker CLI or socket inside
the builder. The host wrapper mounts the project at `/workspace` and runs the builder using the
caller's UID/GID; generated files need no root ownership cleanup.

Docker builds default to one Cargo job to limit peak memory during SP1 release links.
On machines with more Docker memory, use `CARGO_BUILD_JOBS=2 make build-docker` to increase
parallelism. Make's `-j` setting does not control Cargo's internal jobs. If a compiler
still exits with `SIGKILL` with one job, increase Docker Desktop's memory allocation.

The guest script's default `--docker` mode remains available for the native workflow and existing
reproducibility scripts. `--local` requires an installed `succinct` compiler and never installs one
at runtime. The image verifies the compiler and CLI during image construction. Both modes retain
`--locked` builds and the pinned CLI revision gate. Revalidate ELF/VKey identities when changing
the compilation workflow; matching compiler versions alone does not prove identical binaries.

`bash scripts/docker-build.sh test` runs ordinary
native Rust, script, and contract checks in the builder; it does not start live chains or prove jobs.
Missing dependencies are downloaded inside containers, with sources/caches retained at the host
paths described above. No host package manager or global SDK override is needed.
For contract builds, the host wrapper first initializes the pinned SP1 submodule using Git.
Its mapping lives in the parent repository's root `.gitmodules`; the builder mounts only this
light-client directory and does not need the parent repository's Git metadata.

## Toolchains and dependencies

Contract builds report compiler diagnostics without running the optional Foundry linter.
Run `make test ACTION=check` for production contracts and deployment scripts; high, medium,
and low severity findings fail this check. Contract linting is included in both that action and `bash scripts/docker-build.sh test`.
Run `cd contracts && forge lint` for the full inventory, including test code and style/gas
suggestions. Test calls that intentionally discard results are outside the production lint gate.
Narrow lint exceptions in production code explain protocol timestamp checks, checked casts,
and verifier calls enforced by the EVM as static calls. Keep `contracts/foundry.lock` aligned
with the SP1 submodule gitlink when updating that dependency.

| Input | Configuration | Purpose |
| --- | --- | --- |
| Host Rust 1.97.1 | `rust-toolchain.toml` | CLI, host service, native tests, formatting, Clippy |
| Minimum Rust 1.85 | root `Cargo.toml`, inherited by workspace packages; `e2e/Cargo.toml` | Cargo rejects older compilers unless explicitly overridden |
| SP1 6.1.0 | root dependencies, `Cargo.lock`, `scripts/build-sp1.sh` | Guest compilation, execution, proving |
| SP1 CLI 6.1.0 build identity | checked by `scripts/build-sp1.sh` | Canonical Docker guest builds |
| Solidity 0.8.28, Cancun | `contracts/foundry.toml` | Contract compilation |
| Foundry 1.8.1 | pinned tool stage in `deploy/build/Dockerfile` | Current deployment builds; match this for local contract work |
| Contract inputs | Solidity sources in `contracts/vendor/ibc/`, SP1 gitlink, pinned OpenZeppelin packages | IBC interfaces, verifier and upgrade support |

`cargo-prove` is SP1's Cargo extension. Installing it exposes `cargo prove`; standard Rust/Cargo
alone do not provide that command. Its `--version` output includes a short source revision. The
build script expects `d454975`, identifying the reviewed SP1 6.1.0 CLI build. This is not another
package/version to install manually; it is an exact-build check for reproducible guest artifacts.
Do not remove that check just because the readable version string matches.

Foundry supplies `forge` (Solidity builds/tests/scripts) and `cast` (EVM RPC/transaction commands).
`contracts/foundry.toml` selects Solidity 0.8.28. Versioned installation uses Foundry's installer;
see [Foundry installation](https://getfoundry.sh/introduction/installation/). Container deployment
uses the pinned image instead of a separately installed local Foundry.

The minimum Rust version does not select the guest compiler. Guest builds use SP1's pinned
Docker image/toolchain, not the host compiler selected by `rust-toolchain.toml`. Keep guest-reachable
APIs compatible with the declared minimum; Clippy's MSRV check catches newer standard-library APIs.
A declaration alone does not establish that every locked host dependency builds on Rust 1.85.

Native SP1 SDK dependencies require a C/C++ toolchain, linker, CMake, OpenSSL development support,
and `protoc` with its standard includes. Install platform equivalents before building host tooling.
The prover [Dockerfile](../deploy/build/Dockerfile) records its Linux build dependencies.
Shell/deployment tools also use Bash 4+, `jq`, OpenSSL, Git, and Docker. Contract dependency setup
uses npm and `sha256sum`; macOS users need compatible GNU utilities for these scripts.

```bash
git submodule update --init --recursive
bash scripts/install-contract-deps.sh
rustc --version
cargo prove --version
forge --version
```

Only the tasks using SP1 CLI, Foundry, or Docker need those tools. Native crate tests do not require
live chains or real proving. Dependency fetching requires network access for public SP1/OpenZeppelin sources. Router source is
used only by the optional E2E workspace; see [Testing](TESTING.md#optional-router-integration-workspace).

## Code boundaries

The [architecture component table](architecture.md#components-and-trust-boundary) maps source
directories to their responsibilities. Keep deterministic verification in `crates/`, guest entry
points thin, and RPC acquisition in host tooling. `e2e/` is a separate Cargo workspace.
Read each `patches/*/PATCH.md` before changing a pinned cryptographic dependency.

## Everyday workflow

Run focused tests for the code you change, then the standard gates:

```bash
cargo test --locked -p besu-qbft
cargo test --locked -p evm-mpt -p ibc-storage -p batch-merkle
cargo test --locked -p prover-script --lib --bins --test rpc --test witness
```

The [Makefile](../Makefile) provides setup, builds, Docker packaging, tests, and deployment wrappers;
run `make help` for the target list and see [setup instructions](../README.md#tests-and-development).
Make is the single command interface. Equivalent explicit commands are in [Testing](TESTING.md).
Select `make test ACTION=rust|shell|contracts|check` for focused development runs.
The default `make test` includes coverage, real proofs, and external Router integration.
`make e2e` starts live Besu networks for the complete scenario suite.

## Changing verification code

Preserve accepted/rejected statements, canonical encoding, error precedence where intentional,
and public-value bytes. Add regression cases for changed security boundaries. Check both general
and optimized implementations when their results overlap; an optimization must not skip validating
unreturned fields or turn malformed proof data into absence.

The [architecture guide](architecture.md) describes the verification invariants, and
[Testing](TESTING.md#required-regression-areas) lists regression areas to retain.

Native equivalence is necessary but insufficient for guest-reachable changes. Rebuild all canonical
ELFs, execute the differential/rejection suites, compare identities, and run release proof checks
before publishing new artifacts. Shared-crate changes may alter more than one ELF even when the
protocol schemas remain unchanged. Generated VKeys must travel with their ELFs; rebuilding does not update existing contracts.

## Artifacts and configuration

The canonical ELF paths and `artifacts/vkeys.json` resolve through the completed set selected
by `artifacts/current`. `make build-docker` stages compilation and key generation before switching
that pointer. A failed rebuild preserves the previous set. Old generations remain in `.sets/`.

Deployment reads generated identities at runtime. The unified Proof API additionally requires
explicit YAML approvals matching the loaded ELFs/VKeys and deployed contract keys. It retains its
loaded ELFs and identities until restarted; see the [runtime guide](../tooling/proof-api/README.md).
Deployment profiles hold public chain configuration; deployment combines them with a private
snapshot of the generated VKeys. `release/expected-vkeys.json` is retained as
historical release evidence and is not an operational build, proving, or deployment input.

The service and deployment containers mount existing Linux build outputs. The service runtime
image supplies libraries only; deployment reuses the pinned build tool image. No separate
artifact packaging or committed-HEAD deployment build is required.

## Common setup failures

| Symptom | Check |
| --- | --- |
| `Could not find protoc` | Install the protobuf compiler/includes or set `PROTOC` to the compiler path. |
| Linker cannot read the selected SDK | Use a compiler/linker and platform SDK that support each other; do not change protocol dependencies to fix a local SDK mismatch. |
| `cargo prove` unavailable or version rejected | Install the pinned SP1 CLI/toolchain before running `scripts/build-sp1.sh`. |
| VKey/ELF comparison fails | Confirm the source revision and canonical build inputs; follow [Release](RELEASE.md). |
| Deployment cannot find generated outputs | Run `make build-docker`; ensure Docker can mount this project directory. |

## Standalone contracts and E2E separation

`contracts/` builds without Router source. The IBC interface/message bundle is vendored unchanged
with its license and source revision. OpenZeppelin packages live under `contracts/node_modules`.
`DeployLightClient.s.sol` creates only the implementation/proxy; it accepts configured nonzero
Router addresses without requiring code at them. `RegisterLightClient.s.sol` is an explicit,
separate integration action against an existing Router.

`e2e/solidity/` contains actual Router deployments and upstream comparison tests. It is a test
workspace and is excluded from standard Foundry builds. Its wrapper
requires a revision-selected external checkout with unchanged Router contract sources and never
clones or modifies that checkout. See [Testing](TESTING.md#optional-router-integration-workspace).
