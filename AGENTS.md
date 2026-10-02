# AGENTS.md

This repo combines Solidity/Foundry contracts, a Rust workspace, Solana Anchor programs, and Go-based end-to-end tooling for IBC Eureka.

Look here first:
- `README.md`
- `justfile`
- `ibc-solidity/foundry.toml`
- `Cargo.toml`
- `ibc-solidity/contracts/README.md`

Use the smallest relevant validation from the repo root:
- Solidity: `just solidity::lint-contracts` and `just solidity::test-foundry`
- Rust: run formatting, clippy, and tests for the affected package from its owning workspace, including affected dependents. Use that workspace's toolchain and flags from its recipes. For broad Rust changes or shared interfaces affecting multiple stacks, use `just lint-rust` and `just test-cargo`; inspect their recipes because they also invoke checks in independent SP1, CosmWasm, or Solana workspaces.
- Broad cross-stack changes: `just lint`
- Solana or e2e changes: use the subtree-specific commands in `ibc-solana/AGENTS.md` or `e2e/interchaintestv8/AGENTS.md`

Focused checks can complete local validation for a narrow change; do not repeat broad checks solely as a final step. Required CI/PR checks in `.github/workflows/` still apply.

Hard constraints:
- Do not hand-edit generated outputs in `packages/go-abigen/`, most of `packages/go-anchor/`, `e2e/interchaintestv8/solana/go-anchor/`, or protobuf outputs under `e2e/interchaintestv8/types/`; regenerate with `just solidity::generate-abi`, `just solana::generate-solana-types`, or `just generate-buf`.
- `packages/go-anchor/ics07_tendermint_patches/` is the hand-maintained exception inside `packages/go-anchor/`; preserve it across regeneration.
- If Solidity interfaces, ABI-exposed structs, or contract types change, run `just solidity::generate-abi` before validating Go or e2e code.
- If Solana program interfaces or IDLs change, run `just solana::generate-solana-types`.
- If `.proto` files change, run `just generate-buf`.
