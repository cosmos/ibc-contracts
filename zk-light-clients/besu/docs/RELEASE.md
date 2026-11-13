# Release

Use one reviewed source revision and the pinned build inputs from [Development](DEVELOPMENT.md).
Follow [Testing](TESTING.md) for exact validation commands.

`make release` runs all release actions in order: `reproducible`, `storage-layout`,
`manifest`, then `metadata`. Select one with `make release ACTION=manifest` (for example).
Set `RELEASE_MANIFEST=/path/to/manifest.json` to use a custom manifest path for generation
and validation. Build outputs must already exist; reproducibility needs Docker/native SP1 tools, and storage
layout validation requires the external Router workspace. This workflow generates and checks
release evidence; it does not publish a release or replace test/E2E validation.
Manifest generation and validation live in `scripts/release-workflow.sh`.

1. Pass native Rust, shell, E2E-driver, and Solidity checks.
2. Build all three canonical SP1 ELFs. Execute native/guest differential and rejection tests.
3. Retain the completed ELF/VKey set from `make build-docker` (`artifacts/current`). Review identities
   with the source/build changes and compare them with the previous reviewed release manifest.
   Historical expected VKeys are not a build/deployment gate.
4. Generate and locally verify real proofs; pass real-gateway tests and both directions of the
   required four/four-validator Besu E2E matrix. Include v1.1 membership sizes and FULL/CACHED cases.
5. Check storage layout and clean-build reproducibility. Record the required performance matrix
   with witness identities, provenance, hardware, backend, time, and memory.
6. After the final contract build, generate and check release metadata:

   ```bash
   make release ACTION=manifest
   make release ACTION=metadata
   ```

7. Record the source revision, build inputs, artifact identities, validation outcomes, and remaining
   blockers. Archive the reviewed ELF/VKey set and deployment outputs. Runtime containers mount those outputs.

Performance measurements remain separate release evidence. The benchmark runners and baseline
recording/finalization utilities have been removed. Manifest generation checks ELF hashes against
`artifacts/vkeys.json` and records contract ABI/bytecode hashes. The `metadata` action validates selected metadata values, identity formats and VKey
uniqueness; it does not recompute recorded hashes or compare them to deployed contracts. Review
the generated manifest against the retained outputs and deployment records. Neither action
establishes performance conformance. The historical
[performance baseline](../release/performance-baseline.json) is not a completed current measurement.

A guest-source refactor can change ELF/VKey identities without changing public-value schemas.
Rebuilding `artifacts/vkeys.json` does not update existing contracts. Batched membership also changes
its public values/envelope and requires the compatible light-client implementation; see
[Deployment](DEPLOYMENT.md#v11-batched-membership-migration).

Skipped guest, proof, gateway, or E2E checks remain unverified. Preserve the old release's evidence;
attach new results to the new source/artifact identities.

The checked-in interfaces are covered by the release's repository revision. Retain the selected
external Router revision and integration results with the release evidence; a Router checkout is
not an input to standalone contract compilation or deployment.

Reproducibility and storage-layout helpers remain under `scripts/tests/tools/`. Release validation
is separate from `make build-docker` and `make deploy`.
