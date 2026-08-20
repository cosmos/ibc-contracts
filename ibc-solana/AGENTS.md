# AGENTS.md

This subtree contains the Anchor-based Solana IBC programs and their ProgramTest integration harness.

Look here first:
- `programs/solana/README.md`
- `programs/solana/Anchor.toml`
- `programs/solana/Cargo.toml`
- `programs/solana/integration-tests/README.md`

Use the smallest relevant validation from the repo root:
- Build and refresh IDLs: `just build-solana`
- Solana unit tests: `just test-solana`
- Anchor tests: `just test-anchor-solana`
- Lint: `just lint-solana`

Local constraints:
- Prefer the repo `just` recipes over raw `anchor` commands; they auto-detect `anchor-nix` when available.
- If program interfaces or IDLs change, run `just generate-solana-types`; do not hand-edit `packages/go-anchor/` except `packages/go-anchor/ics07_tendermint_patches/`, and do not hand-edit `e2e/interchaintestv8/solana/go-anchor/`.
- Cluster-specific program IDs live in `programs/solana/Anchor.toml`, and keypairs live under `solana-keypairs/<cluster>/`; keep non-`localnet` keypairs out of git.
