# AGENTS.md

This subtree contains the Go end-to-end test suites that drive local Ethereum, Cosmos, and Solana environments through interchaintest.

Look here first:
- `e2e/interchaintestv8/README.md`
- `e2e/interchaintestv8/go.mod`
- the relevant suite entrypoint (`ibc_eureka_test.go`, `relayer_test.go`, `sp1_ics07_test.go`, `solana_test.go`, etc.)

Use the smallest relevant validation from the repo root:
- Single targeted test: `just test-e2e TestWithSuite/Test_Name`
- Common wrappers: `just test-e2e-eureka`, `just test-e2e-relayer`, `just test-e2e-cosmos-relayer`, `just test-e2e-sp1-ics07`
- Solana e2e wrappers: `just test-e2e-solana`, `just test-e2e-solana-gmp`, `just test-e2e-solana-ift`, `just test-e2e-solana-upgrade`, `just test-e2e-solana-attestation`
- Go lint for this subtree lives in `just lint-go`; see `justfile` for the other suite wrappers

Local constraints:
- Prefer the smallest targeted test; full e2e runs are slow and environment-dependent.
- Local runs expect `.env` set up from `.env.example`; `just test-e2e` installs the relayer, and `just test-e2e-sp1-ics07` also installs the operator.
- If contract interfaces or ABI-exposed types change, run `just generate-abi` before e2e validation.
- If Solana IDLs change, run `just generate-solana-types`; if `.proto` files change, run `just generate-buf`.
- Do not hand-edit generated code under `e2e/interchaintestv8/types/` or `e2e/interchaintestv8/solana/go-anchor/`.
