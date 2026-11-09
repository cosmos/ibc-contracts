# Testing

Use the smallest relevant checks during development and the full proof-backed checks for a release.
Run commands from the light-client project directory unless a command explicitly changes directory.
[Development](DEVELOPMENT.md) lists tool prerequisites; [Architecture](architecture.md) explains
which component enforces each statement.

Use `bash scripts/docker-build.sh test` to run ordinary Rust, shell, and Solidity checks inside the build container
without installing their toolchains on the host. It prepares contract dependencies first and writes
Rust build data under `target/docker/`. It preserves failing test exit statuses.

The [Makefile](../Makefile) exposes `make test ACTION=<suite>`. With no action it runs all
suites, serially: `rust`, `shell`, `contracts`, `check`, `coverage`, `sp1`, and `integration`.
`coverage-zk` and `coverage-proof-api` select individual coverage reports.
The `sp1` action builds circuits, executes guests, generates real proof fixtures, and verifies
those fixtures through the Solidity gateway. It requires a working prover backend and writes
the canonical gateway fixtures to `artifacts/proofs/`.
The `integration` action requires the external Router workspace. Coverage requires
`cargo-llvm-cov` and LLVM tools. Live networks are run separately with `make e2e`.

## What each layer establishes

| Layer | Prerequisites | Establishes | Does not establish |
| --- | --- | --- | --- |
| Native Rust | Host toolchain and native dependencies | Parser, signature, MPT, path, ABI, batching, RPC, and witness behavior | Guest execution or real-proof compatibility |
| Solidity with mock verifier | Contract dependencies and Foundry | State transitions, public-value bindings, FULL/CACHED handling, upgrade controls | SP1 cryptographic verification |
| SP1 execution | Canonical ELFs and SP1 SDK | Native/guest public-value agreement and guest rejection behavior | Groth16 generation or gateway acceptance |
| Real proofs and gateway | Compatible prover backend, ELFs, gateway contracts | Proof generation, local verification, and on-chain verifier binding | Complete live IBC flows |
| Two-chain E2E | Besu networks, deployed contracts, real proving | Integrated update, receive, acknowledgement, timeout, reuse, and negative scenarios | Unmeasured performance or other source revisions |

A skipped or unavailable layer is unverified, not passing. Historical results apply to their
recorded source/artifact versions.

## Native and shell checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --exclude prover-script
cargo test --locked -p prover-script --lib --bins --test rpc --test witness
bash scripts/tests/preflight-deploy.sh
bash scripts/tests/collection-scripts.sh
bash scripts/tests/release-metadata.sh
bash scripts/tests/entrypoint.sh
bash scripts/tests/e2e-live-tooling.sh
bash scripts/tests/router-workspace.sh
python3 -m unittest discover -s scripts/tests -p '*.py'
```

The release-metadata test uses temporary fixtures.
`make release ACTION=metadata` checks actual release outputs and belongs after manifest
creation in [Release](RELEASE.md).

The separate E2E workspace can be checked without running live scenarios:

```bash
cargo fmt --manifest-path e2e/Cargo.toml -- --check
cargo clippy --locked --manifest-path e2e/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path e2e/Cargo.toml
```

Live scenario tests are ignored in this ordinary invocation.

## Proof API checks

The ordinary workspace tests above include the unified API and scheduler. To run them alone:

```bash
cargo test --locked -p proof-api -p proof-api-integration
```

The API tests cover configuration, transport encodings/deadlines, packet fixture parity, RPC
limits and orchestration. Engine tests inject a fake proof backend with authenticated MPT data;
they do not generate SP1 proofs. Scheduler tests exercise admission, splitting, deduplication,
conflicts, cancellation, timeouts and shutdown. See the
[implementation source map](PROOF_API_IMPLEMENTATION.MD) and
[protocol provenance](../tooling/proof-api/proto/README.md).

For a running deployment, use the runtime guide's
[validation commands](../tooling/proof-api/README.md#validation-and-endpoints), then exercise
all three ProverService methods through the intended relayer/client. Retain command results and
transaction evidence against the configured approved artifacts. The live E2E harness below calls
proving tooling directly; it does not start the unified API or test the external relayer.

## Coverage reports

Install `cargo-llvm-cov` (`cargo install cargo-llvm-cov --locked`) and the pinned Rust
toolchain's LLVM tools (`rustup component add llvm-tools-preview`). Native host prerequisites,
including `protoc`, are listed in [Development](DEVELOPMENT.md#toolchains-and-dependencies).

```bash
make test ACTION=coverage           # Generate both reports, serially
make test ACTION=coverage-zk        # Only native circuit logic
make test ACTION=coverage-proof-api # Only Proof API and host proving code
```

Add `SUMMARY=1` to any of these commands for a compact Markdown table instead of
the per-file terminal summary:

```bash
make test ACTION=coverage SUMMARY=1
make test ACTION=coverage-zk SUMMARY=1
make test ACTION=coverage-proof-api SUMMARY=1
```

The table lists covered/total counts and percentages for lines, functions and LLVM
regions. Statements are `N/A`: LLVM does not export a separate statement metric,
and regions are not equivalent to statements. Branch percentages are `N/A` when no
branches were instrumented (including a `0 / 0` count). Rust branch instrumentation
currently requires nightly with `cargo llvm-cov --branch`; these targets retain the
repository's pinned toolchain and do not enable it. Summary mode also saves the
filtered LLVM JSON as `summary.json` beside `summary.txt`. HTML generation is unchanged.

Each target prints per-file coverage and a TOTAL row, saves that summary, and generates
an HTML report with uncovered source lines (with `SUMMARY=1`, the compact table replaces
the per-file summary):

| Report | Measured source | Summary | HTML entry point |
| --- | --- | --- | --- |
| ZK circuit logic (native) | The six libraries under `crates/`: types, public-values, evm-mpt, qbft, ibc-storage and batch-merkle | `target/coverage/zk/summary.txt` | `target/coverage/zk/html/index.html` |
| Proof API | `tooling/proof-api`, `tooling/proof-api-integration` and `tooling/prover-script` | `target/coverage/proof-api/summary.txt` | `target/coverage/proof-api/html/index.html` |

The API report collects API/integration tests plus the prover library, binaries, RPC and
witness tests. Shared circuit libraries are excluded from its report to keep the source
groups separate. The circuit report runs the six libraries' native tests. It does not
measure `programs/*/src/main.rs`, zkVM-specific branches, guest execution or proof generation.
Guest binaries have native test harnesses disabled; do not interpret the native percentage
as SP1 guest coverage. Solidity, scripts and the separate E2E workspace are outside both reports.

Each target clears intermediate coverage data first, so run these targets serially and
avoid overlapping other coverage jobs. Previously generated HTML reports and summaries
under `target/coverage/` remain available when generating the other report.

## Required regression areas

| Component | Cases to retain when changing it |
| --- | --- |
| QBFT | Canonical header/extra-data encoding, round-preserving digest, supported header tails, scalar/recovery-ID bounds, high-S parity, non-validator/duplicate signers, independent target quorum and trusted overlap |
| MPT | Existing account authentication; canonical nonce/balance; inline and hashed children; exact-key inclusion/exclusion; missing, duplicate, unused, malformed and truncated nodes; present zero versus absence |
| IBC paths/slots | One path element, identifier grammar, packet/ACK versus receipt types, sequence/namespace binding, independent slot derivation |
| Membership batches | Shared account root, valid sizes through 32, empty/33-entry rejection, duplicate digests, strict ordering, commitment/proof/path mutations at every entry position |
| Merkle batches | Roots and positional branches for every count 1–32, padding, index/length errors, changed leaves/roots/siblings |
| Public values | ABI widths, address padding, schema 1 versus membership schema 2, canonical bytes, malformed/trailing data, FULL/CACHED payload rules |
| Solidity | Stored-state/Router/height/VKey binding, cache isolation, FULL before CACHED, exact item branch, time policies, replay/conflicting state and upgrade authorization |
| Host/scheduler | RPC response/key matching, deterministic witness bytes, allowlisting/authentication, backpressure, deduplication, frozen height, artifact shape |

Existing tests are evidence for their actual assertions; this table is a maintenance checklist, not
an assertion that every requested scenario has already passed. Sections 17 and 19–22 of
[SPEC.md](../SPEC.md) define the detailed acceptance matrix.

## Contract checks

```bash
bash scripts/install-contract-deps.sh
(cd contracts && forge fmt --check && forge build && forge test)
```

Read real-verifier test gating before interpreting a successful ordinary `forge test` as a
proof-backed result.

## Optional Router integration workspace

Full Router implementations are test dependencies, supplied externally rather than as submodules.
Provide an existing checkout with its Solidity dependencies installed according to that project's
instructions. This harness never installs dependencies into or modifies that checkout:

```bash
export E2E_ROUTER_DIR=/absolute/path/to/ibc-contracts
# Default tested-baseline selection; alternate revisions require explicit compatibility validation.
export E2E_ROUTER_REV=1d129afa1305069cb4ed709357922e5b44799dd8
make test ACTION=integration
make release ACTION=storage-layout
```

`e2e/scripts/forge.sh` selects the external Router remapping and the separate `e2e/solidity`
Foundry workspace. The external checkout must match the selected commit, with no tracked or untracked changes under
`ibc-solidity/contracts/`. Changes elsewhere are not checked by the wrapper. Actual
Router deployment and profile-registration tests, plus upstream commitment vectors, live there.
Ordinary `make test ACTION=contracts` uses the interface bundle and requires no Router checkout.
The standalone deployment regression checks deployment with no code at either Router address.

## Build and execute guests

```bash
make build-docker
SP1_PROVER=cpu cargo test --locked -p prover-script --test sp1_execute -- --test-threads=1
```

Use the pinned guest compiler and CLI identity described in
[Development](DEVELOPMENT.md#toolchains-and-dependencies). Do not substitute native binaries for
the three canonical ELFs. Identity mismatches require review as described in
[Release](RELEASE.md).

## Generate real proofs and check the gateway

On a host/backend capable of completing Groth16 jobs, run:

```bash
SP1_PROVER=cpu bash scripts/tests/tools/generate-golden-proofs.sh
(cd contracts && RUN_REAL_SP1=true forge test --match-contract SP1VerifierIntegrationTest)
```

Record the backend, source revision, ELF identities, witness provenance, elapsed time, peak memory,
exit status, and relevant transaction results. Run heavy jobs serially unless the machine/backend
has been measured for the intended concurrency. Measure resource needs for the selected program and backend before choosing host capacity.

## Live two-chain validation

```bash
E2E_ROUTER_DIR=/absolute/path/to/ibc-contracts RUN_HEAVY_E2E=1 bash scripts/tests/tools/run-e2e.sh
```

The script creates two four-validator Besu networks, deploys both directions, and selects ignored
live tests serially. It refuses an existing `.e2e-state`. It tears down its Compose stack on exit;
retain logs and deployment/artifact evidence needed to interpret the result before discarding state.

For existing test nodes, follow [README deployment](../README.md#deploy-to-existing-test-nodes)
and [Deployment](DEPLOYMENT.md). An asymmetric four/one-validator topology is diagnostic evidence;
it does not substitute for the required four/four release topology.

For remote runs, use an isolated source snapshot, record its revision and tool versions, preserve
command exit statuses/logs, and avoid overlapping heavy jobs. Confirm Docker socket/network access,
container UID permissions, historical RPC state retention, and Cancun-compatible test-chain genesis.
Resume from the last verified stage after a connection failure; do not infer success from a missing
process or incomplete log.

## Performance and completion

Performance acceptance requires separately collected measurements; see
[Release](RELEASE.md) for the required evidence. Neither `make test` nor `make release`
establishes performance conformance.

Keep a compact result record: source revision/dirty-tree description, exact commands, tool versions,
ELF/VKey identities where relevant, pass/fail/skipped results, and locations of retained evidence.
Attach results to their source and artifact versions in the release record. Historical runs do not
establish current guest-binary validity, and a documentation edit does not refresh that evidence.
