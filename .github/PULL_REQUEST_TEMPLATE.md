<!-- < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < ☺
v                               ✰  Thanks for creating a PR! ✰
v    Before smashing the submit button please review the checkboxes.
v    If a checkbox is n/a - please still include it but + a little note why
v    Also: make sure that you are familiar with the contribution guidelines: (https://github.com/cosmos/ibc-contracts/blob/main/CONTRIBUTING.md)
v    Failure to do this can result in your PR getting closed without further discussion (we receive a lot of PRs, and it takes a lot of time to respond to everyone who doesn't read this)
☺ > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > >  -->

## PR description

<!-- Add a description of the changes that this PR introduces and the files that
are the most critical to review.
-->

## Fixed Issue(s)

<!-- Please link to fixed issue(s) here using format: fixes #<issue number> -->
<!-- Example: "fixes #1234" -->

closes: #XXXX

---

### Thanks for sending a pull request! Have you done the following?

Before we can merge this PR, please make sure that all the following items have been
checked off. If any of the checklist items are not applicable, please leave them but
write a little note why.

- [ ] Checked out our [contribution guidelines](https://github.com/cosmos/ibc-contracts/blob/main/CONTRIBUTING.md)?
- [ ] Linked to a GitHub issue with discussion and accepted design, OR link to spec that describes this work.
- [ ] [ADR](https://github.com/cosmos/ibc-contracts/tree/main/docs/adr) merged first, if this changes architecture, a public interface, or a spec.
- [ ] Kept this to one logical change, with the diff as small as practical.
- [ ] Signed off every commit for the [DCO](https://github.com/cosmos/ibc-contracts/blob/main/DCO.md) with `git commit -s`, using your legal name.
- [ ] Vendored, copied, or adapted third-party code? Updated [`NOTICE`](https://github.com/cosmos/ibc-contracts/blob/main/NOTICE) and checked the license against the allowed third-party licenses. See [Third-party code](https://github.com/cosmos/ibc-contracts/blob/main/CONTRIBUTING.md#third-party-code).
- [ ] Disclosed any AI assistance with a `Co-Authored-By` or `Assisted-By` line.
- [ ] Added an SPDX license header to every new source file (`just lint-license-fix`).
- [ ] Wrote unit and end-to-end tests if relevant, covering expected and error paths.
- [ ] Called out any breaking change, and whether it needs a new initializer or a state migration. See [RELEASES.md](https://github.com/cosmos/ibc-contracts/blob/main/RELEASES.md).
- [ ] Updated documentation (`docs/`, `README.md`, `UPGRADEABILITY.md`) if anything is changed.
- [ ] Added relevant NatSpec, `rustdoc`, and `godoc` comments.
- [ ] Self-reviewed `Files changed` in the GitHub PR explorer.
- [ ] Provide a [conventional commit message](https://www.conventionalcommits.org/en/v1.0.0/) as the PR title to follow the repository standards.
    <!-- This repository uses [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/).
    CI validates the PR title against these types: feat, fix, docs, test, ci, refactor, chore, revert, imp, deps.
    Example commit messages:
    fix: skip emission of unpopulated memo field in ics20
    deps: updating sp1-contracts to v4.0.0
    chore: removed unused variables
    test: adding e2e tests for ics20
    docs: ics27 documentation updates
    feat: add semantic version utilities for e2e tests
    feat(api)!: this is an api breaking feature
    fix(statemachine)!: this is a statemachine breaking fix
    -->
- [ ] Review `SonarCloud Report` in the comment section below once CI passes.

### Locally, you can run these tests to catch failures early:

- [ ] all linters: `just lint`
- [ ] SPDX headers: `just lint-license`
- [ ] Solidity: `just solidity::lint-solidity` and `just solidity::test-foundry`
- [ ] Solana: `just solana::lint-solana`
- [ ] Rust: `just lint-rust` and `just test-cargo`
- [ ] CosmWasm light client: `just lint-cw` and `just test-cargo-cw`
- [ ] SP1 programs: `just lint-sp1`
- [ ] end-to-end: `just test-e2e <TestName>`
- [ ] protobuf changes: `just lint-buf` and `just generate-buf`

<!-- `just lint` does not cover the SP1, CosmWasm, or Solana linters — run those
     directly. Run `just --list` for the full set of recipes, including the
     per-suite e2e tests and the `just solidity::...` / `just solana::...`
     sub-namespaces. -->
