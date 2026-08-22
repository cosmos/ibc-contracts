# AGENTS.md

This repo combines Solidity/Foundry contracts, a Rust workspace, Solana Anchor programs, and Go-based end-to-end tooling for IBC Eureka.

Look here first:
- `README.md`
- `justfile`
- `ibc-solidity/foundry.toml`
- `Cargo.toml`
- `ibc-solidity/contracts/README.md`

Use the smallest relevant validation from the repo root:
- Solidity: `just solidity::lint-solidity` and `just solidity::test-foundry`
- Rust workspace, relayer, operator, and shared packages: `just lint-rust` and `just test-cargo`
- Broad cross-stack changes: `just lint`
- Solana or e2e changes: use the subtree-specific commands in `ibc-solana/AGENTS.md` or `e2e/interchaintestv8/AGENTS.md`

Hard constraints:
- Do not hand-edit generated outputs in `packages/go-abigen/`, most of `packages/go-anchor/`, `e2e/interchaintestv8/solana/go-anchor/`, or protobuf outputs under `e2e/interchaintestv8/types/`; regenerate with `just solidity::generate-abi`, `just solana::generate-solana-types`, or `just generate-buf`.
- `packages/go-anchor/ics07_tendermint_patches/` is the hand-maintained exception inside `packages/go-anchor/`; preserve it across regeneration.
- If Solidity interfaces, ABI-exposed structs, or contract types change, run `just solidity::generate-abi` before validating Go or e2e code.
- If Solana program interfaces or IDLs change, run `just solana::generate-solana-types`.
- If `.proto` files change, run `just generate-buf`.
