# AGENTS.md

This subtree contains the Anchor-based Solana IBC programs and their ProgramTest integration harness.

Look here first:
- `ibc-solana/README.md`
- `ibc-solana/Anchor.toml`
- `ibc-solana/Cargo.toml`
- `ibc-solana/integration-tests/README.md`

Use the smallest relevant validation from the repo root:
- Build and refresh IDLs: `just solana::build-solana-programs`
- Solana unit tests: `just solana::test-solana`
- Anchor tests: `just solana::test-anchor-solana`
- Lint: `just solana::lint-solana`

Local constraints:
- Prefer the repo `just` recipes over raw `anchor` commands; they auto-detect `anchor-nix` when available.
- If program interfaces or IDLs change, run `just solana::generate-solana-types`; do not hand-edit `packages/go-anchor/` except `packages/go-anchor/ics07_tendermint_patches/`, and do not hand-edit `e2e/interchaintestv8/solana/go-anchor/`.
- Cluster-specific program IDs live in `ibc-solana/Anchor.toml`, and keypairs live under `solana-keypairs/<cluster>/`; keep non-`localnet` keypairs out of git.
