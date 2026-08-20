# Contributing to ibc-contracts

## 🎉 Thanks for taking the time to contribute! 🎉

Welcome to the `ibc-contracts` repository! The following is a set of guidelines for contributing to this repo and its components. These are mostly guidelines, not rules. Use your best judgement, and feel free to propose changes to this document in a pull request. Contributions come in the form of code submissions, writing documentation, raising issues, helping others in chat, and any other actions that help develop `ibc-contracts`.

IBC is an open standard for trust-minimized interoperability between independent ledgers, stewarded as an open, public project. Every contributor follows the same public path: there is no internal fast lane, and no direct pushes to `main`.

This document covers everything you need to know to decide whether and how you want to contribute.

## Contents

- [GitHub and chat accounts](#github-and-chat-accounts)
- [I just have a quick question](#i-just-have-a-quick-question)
- [Ways to contribute](#ways-to-contribute)
- [Where to engage](#where-to-engage)
- [Where to start?](#where-to-start)
- [Contribution workflow](#contribution-workflow)
- [Reporting bugs](#reporting-bugs)
- [Suggesting enhancements](#suggesting-enhancements)
- [Architecture Decision Records (ADR)](#architecture-decision-records-adr)
- [Pull requests](#pull-requests)
- [Release notes](#release-notes)
- [Code reviews](#code-reviews)
- [Developer Certificate of Origin (DCO)](#developer-certificate-of-origin-dco)
- [Copyright and license](#copyright-and-license)
- [Third-party code](#third-party-code)
- [Security contributions](#security-contributions)
- [Guidelines for non-code and other trivial contributions](#guidelines-for-non-code-and-other-trivial-contributions)
- [Specifications](#specifications)
- [Governance](#governance)
- [Other important information](#other-important-information)

## GitHub and chat accounts

A GitHub account is required for code and issue contributions. Chat is optional, but useful for asking questions and coordinating with maintainers.

- Create a [GitHub account](https://github.com) if you don't already have one.
- Join the community Discord to ask questions or chat with us.

## I just have a quick question

You might find the answer in the [README](./README.md), the rest of the [documentation in this repository](./docs), or the wider [IBC documentation](https://ibc.cosmos.network). Otherwise, ask in [GitHub Discussions](https://github.com/cosmos/ibc-contracts/discussions) or in the community chat. [SUPPORT.md](./SUPPORT.md) lists every place to get help.

> [!NOTE]
> Please don't file an issue to ask a question. You'll get faster results by using the resources above.

## Ways to contribute

A contribution does not have to be code:

- **Report a bug** as a GitHub issue using the [bug template](https://github.com/cosmos/ibc-contracts/blob/main/.github/ISSUE_TEMPLATE/bug-report.md), with enough detail to reproduce. See [Reporting bugs](#reporting-bugs).
- **Propose a feature or enhancement** as a GitHub issue using the [feature template](https://github.com/cosmos/ibc-contracts/blob/main/.github/ISSUE_TEMPLATE/feature-request.md). See [Suggesting enhancements](#suggesting-enhancements).
- **Improve documentation**, which follows the same pull request path as code and is the easiest first contribution.
- **Submit code** against an accepted issue.
- **Review other contributors' pull requests**, which is one of the most useful ways to build standing toward Committer.

Merge authority is earned through the same public record for everyone, and review history counts toward it as much as authorship does. See [GOVERNANCE.md](./GOVERNANCE.md#contributor-ladder) for the contributor ladder.

## Where to engage

Every contribution decision happens in the open. These are the venues, and which is for what:

- [GitHub Discussions](https://github.com/cosmos/ibc-contracts/discussions) for ideas, questions, and early proposals.
- [GitHub Issues](https://github.com/cosmos/ibc-contracts/issues) for bugs and scoped, accepted work, opened through the bug and feature templates.
- GitHub pull requests for ADRs, code, and docs.
- Community calls, open to all and recorded, with a public calendar, for synchronous design and roadmap discussion. Agendas are filed using the [contributor call template](./.github/ISSUE_TEMPLATE/contributor-call.md).
- Community chat on Discord for questions and coordination.
- [GitHub Discussions](https://github.com/cosmos/ibc-contracts/discussions) also hosts formal and asynchronous governance decisions and their lazy-consensus windows.
- Security disclosures follow the private path in [SECURITY.md](./SECURITY.md) and never go through public issues. This is the single documented exception to the public-by-default rule.

## Where to start?

The first step is deciding what to work on! We use labels to identify issues that are a good place to start.

- Browse the [good first issues](https://github.com/cosmos/ibc-contracts/labels/good%20first%20issue). These have a clearly specified scope of work and require no deep knowledge of IBC or the light client stack. Examples include improving logging, emitting new events, or removing unused code.
- Browse the [help wanted issues](https://github.com/cosmos/ibc-contracts/labels/help%20wanted). These are a bit more involved than good first issues and benefit from some familiarity with the codebase. Examples include extending the Solidity or Solana interfaces, bumping a dependency such as SP1 or a light client, or fixing bugs.

If you find an issue you'd like to work on, comment on it or raise it in the community chat and we can assign it to you. If you have a different idea, open a [GitHub Discussion](https://github.com/cosmos/ibc-contracts/discussions) or raise it on a community call and we can discuss it. This step is expected, not optional, and it protects you from investing in work that overlaps or conflicts with existing plans.

Prioritized use cases from external working groups enter here too, as ordinary public proposals on the same footing as any other contribution.

## Contribution workflow

The code and documentation are maintained using the same *contributor workflow*, where everyone without exception contributes change proposals using pull requests (PRs) from a fork. This is the same path for maintainers and first-time contributors, and it facilitates social contribution, easy testing, and peer review.

Before you start writing code, three things need to be true. Skipping them is the most common reason work gets discarded:

1. **Rough agreement exists** that the problem is worth solving, reached in a [Discussion](https://github.com/cosmos/ibc-contracts/discussions), an issue, or a community call. No private pre-alignment substitutes for this.
2. **An ADR is merged**, if your change touches architecture, a public interface, or a specification. See [Architecture Decision Records (ADR)](#architecture-decision-records-adr). Trivial changes, bug fixes, and documentation corrections skip this step.
3. **An accepted, scoped GitHub issue exists and is assigned to you.** The issue is the record of truth for status, and it is visible to everyone.

With those in place, use the following workflow:

1. [**Fork the repository**](https://github.com/cosmos/ibc-contracts/fork). Make sure you also [add an upstream remote](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/syncing-a-fork) so you can keep your fork up to date.
2. **Clone your fork** to your computer and install the toolchain described under [Build Requirements](./README.md#build-requirements). Most tasks in this repository are driven through [`just`](https://github.com/casey/just); run `just --list` to see what is available.
3. **Create a topic branch** and name it appropriately. Starting the branch name with the issue number is a good practice and a reminder to fix only one issue per PR. Maintainers working within the `ibc-contracts` repo use the convention `{moniker}/{issue#}-branch-name`.
4. **Make your changes**, following the conventions of the surrounding code. This repository spans Solidity contracts, Solana programs and Rust crates, and Go end-to-end tests, and each area follows the idiom already established there. In general a commit serves a single purpose, and diffs should be easily comprehensible. For this reason, do not mix formatting fixes or code moves with actual code changes. New source files need an SPDX license header; see [Copyright and license](#copyright-and-license).
5. **Commit your changes**. Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) form, for example `fix(ics20): reject transfers with an empty denom`. Make sure to add a DCO sign-off to each commit (for example, `git commit -s -m "..."`); see [Developer Certificate of Origin (DCO)](#developer-certificate-of-origin-dco) below.
6. **Test your changes** locally before pushing to ensure you are not breaking another part of the software. Running `just lint` and the relevant test recipes locally helps you be confident that your changes will pass CI once pushed as a PR. See [Locally, you can run these](#locally-you-can-run-these) for the commands.
7. **Push your changes** to your remote fork (usually labeled `origin`).
8. **Create a pull request** against `main`. If it's not ready for review, make it a `Draft` PR. If the PR addresses an existing issue, link it in the PR description using GitHub keywords such as `fixes #1234` or `refs #1234`. The PR *title* must be a valid conventional commit — CI checks this.
9. **Add labels** to identify the type of your PR, if you have permission. For example, if your PR fixes a bug, add the "bug" label. If you don't have permission, maintainers will label the PR during triage.
10. **Ensure your changes are reviewed**. Mark the PR `Ready for Review` and let us know in the community chat. If you are a maintainer, you can choose reviewers; otherwise this is done by one of the maintainers.
11. **Make any required changes** based on reviewer feedback. Make the changes, commit to your branch, and push to your remote fork.
12. **When your PR is approved and validated**, all tests pass, and your branch has no conflicts, it can be merged. This is done by a maintainer, usually the same person who approves it.

You contributed to `ibc-contracts`! Thanks!

### Locally, you can run these

| Task | Command |
| --- | --- |
| All linters | `just lint` |
| SPDX headers | `just lint-license`, or `just lint-license-fix` to insert them |
| Solidity contracts | `just solidity::lint-solidity`, `just solidity::test-foundry <TestName>` |
| Solana programs | `just solana::lint-solana`, `just solana::build-solana-programs` |
| Rust crates | `just lint-rust`, `just test-cargo` |
| CosmWasm light client | `just lint-cw`, `just test-cargo-cw` |
| SP1 programs | `just lint-sp1`, `just build-sp1-programs` |
| Go end-to-end tests | `just lint-go`, `just test-e2e <TestName>` |
| Protobuf | `just lint-buf`, `just generate-buf` |

Note that `just lint` does not cover everything: the SP1, CosmWasm, and Solana linters are separate recipes, so run those directly when you touch those components. Run `just --list` for the full set, including the per-suite end-to-end recipes and the `just solidity::...` and `just solana::...` sub-namespaces.

Do not hand-edit generated output. Protobuf output, the Go ABI bindings under `packages/go-abigen`, the Anchor IDL bindings in any `generated/` directory, and the test fixtures all have a generating recipe — change the source and regenerate.

## Reporting bugs

This section guides you through submitting a bug report. Following these guidelines helps maintainers and the community understand your report, reproduce the behavior, and find related reports.

Bugs are tracked as [GitHub issues](https://github.com/cosmos/ibc-contracts/issues). Before submitting a bug report:

- **Confirm the problem** is reproducible in the latest release of the affected component.
- **Check the [documentation](./docs) and the [README](./README.md)**. You might be able to find the cause of the problem and fix it yourself.
- **Search [existing issues](https://github.com/cosmos/ibc-contracts/issues)** to see if the problem has already been reported. If it has and the issue is still open, add a comment to the existing issue instead of opening a new one. If you find a closed issue that seems like the same thing you're experiencing, open a new issue and include a link to the original issue.
- **Check whether the bug is eligible for a bug bounty** under [SECURITY.md](./SECURITY.md). Bugs with a security impact must not be reported publicly; see [Security contributions](#security-contributions).
- **Check the [audit reports](./docs/audits)**, in case the behavior is a known and accepted finding.

When you create a bug report, please include as many details as possible:

- **Use a clear and descriptive summary** to identify the problem.
- **Describe the exact steps that reproduce the problem** in as much detail as possible.
- **Provide specific examples to demonstrate the steps**. Include links to files or GitHub projects, or copy-pasteable snippets. If you're providing snippets in the issue, use Markdown code blocks. An executable test case is ideal — a failing Foundry test or end-to-end test is the most useful thing you can attach.
- **Describe the behavior you observed** and point out exactly what the problem is.
- **Explain which behavior you expected to see instead and why.**
- **Include logs, traces, or reverted transaction data** where they help demonstrate the problem.

Provide more context by answering these questions:

- **Did the problem start happening recently** (for example, after updating to a new version) or was it always a problem? If recent, can you reproduce it in an older version? What is the most recent version in which the problem doesn't happen?
- **Can you reliably reproduce the issue?** If not, provide details about how often it happens and under which conditions.

Include details about your configuration and environment:

- **Which component and version are affected?** Give the release tag or the exact commit hash. See [RELEASES.md](./RELEASES.md) for how each component is versioned.
- **Which toolchain versions are you running?** For example Foundry, Rust, SP1, Solana, or Go, depending on the component.
- **Which network is involved?** Mainnet, testnet, or a local devnet, and the relevant contract or program addresses.
- **What is the network context?** Which chains, light clients, relayer, and attestor set are involved, and which IBC version each side runs.
- **What OS and version are you running?** For Linux, include the kernel (`uname -a`).

## Suggesting enhancements

This section guides you through submitting an enhancement suggestion. Following these guidelines helps maintainers and the community understand your suggestion and find related ones.

Enhancement suggestions are tracked as [GitHub issues](https://github.com/cosmos/ibc-contracts/issues). Before submitting:

- **Check the [documentation](./docs)**. The enhancement might already exist.
- **Search [existing issues](https://github.com/cosmos/ibc-contracts/issues) and [merged ADRs](./docs/adr)** to see if it has already been suggested. If it has and the issue is still open, add a comment instead of opening a new one.

When you create an enhancement suggestion, please include:

- **A clear and descriptive title** to identify the suggestion.
- **A step-by-step description of the suggested enhancement** in as much detail as possible.
- **Specific examples to demonstrate the steps**, using Markdown code blocks for any snippets.
- **A description of the current behavior** and an explanation of which behavior you expected to see instead, and why.
- **An explanation of why this enhancement would be useful**, pointing to a use case or adopter need, and which user group benefits: chains, IBC app developers, relayers, attestor operators, or light client developers.
- **Whether this enhancement exists in other IBC implementations**, such as [ibc-go](https://github.com/cosmos/ibc-go).
- **Any gas, proving cost, or upgrade implications**, since these often decide whether a change to the on-chain components is viable.
- **The version of the affected component you're using** and the name and version of your OS.

Proposals are discussed in the open and can have different outcomes:

- the change is accepted and added to the project's planning;
- the change is accepted and an external contributor is supported in implementing it, with the goal of merging it into `ibc-contracts`;
- the change is declined because it is not aligned with the objectives of the project; or
- in the case of applications or light clients, the change is developed and maintained in a separate repository.

## Architecture Decision Records (ADR)

Anything that changes architecture, a public interface, or a specification requires an ADR before implementation, so that all involved parties are in agreement before any party begins coding.

An ADR is a markdown file submitted as a pull request under [`docs/adr`](./docs/adr), in the subdirectory for the component it concerns, and reviewed under lazy consensus. See the existing ADRs for the expected shape. ADRs document solidified designs that will be implemented and do not have a spec. They should document the architecture that will be built. Most design feedback should be gathered before the initial draft. An ADR can and should be written for any design decision that may be revisited in the future.

Changes to the upgrade model of the deployed contracts belong in an ADR too, and should be reflected in [UPGRADEABILITY.md](./UPGRADEABILITY.md) in the same or a follow-up pull request.

Trivial changes, bug fixes, and documentation corrections do not need an ADR.

## Pull requests

The pull request process has several goals:

- Maintain product quality.
- Fix problems that are important to users.
- Engage the community in working toward the best possible product.
- Enable a sustainable system for maintainers to review contributions.

Please follow these steps to have your contribution considered by the maintainers:

1. Ensure all commits have a sign-off for the DCO, as described in [Developer Certificate of Origin (DCO)](#developer-certificate-of-origin-dco).
2. Ensure an accepted, scoped GitHub issue exists and is assigned to you. The issue is the record of truth for status. Non-minor pull requests without an assigned issue may be closed.
3. Follow all instructions in the [pull request template](https://github.com/cosmos/ibc-contracts/blob/main/.github/PULL_REQUEST_TEMPLATE.md).
4. Include appropriate test coverage, covering both the expected path and error handling. Changes to the contracts or programs need unit tests, and changes that cross a chain boundary need an end-to-end test.
5. Give the pull request a title in [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) form. CI validates it against the accepted types: `feat`, `fix`, `docs`, `test`, `ci`, `refactor`, `chore`, `revert`, `imp`, `deps`.
6. Document public interfaces: NatSpec on Solidity, rustdoc on Rust, and godoc on Go.
7. After you submit your pull request, verify that all [status checks](https://docs.github.com/articles/about-status-checks) are passing.

### What makes a good pull request?

#### One pull request, one change

- This limits the surface area of the change and makes it easier to identify root causes when issues arise.
- Make sure your PR doesn't include commits that are not part of it. This can happen if [your fork is not up to date](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/syncing-a-fork).

#### Minimize lines of code (LOC) per PR

- PRs get near-exponentially longer to review as the number of lines of code increases. Ideally, keep your changes under 300 LOC. If that's not possible, try to break your PR into smaller ones for reviewers to review sequentially.
- Generated code — protobuf output, ABI bindings, and test fixtures — should land in its own commit, so reviewers can see the hand-written change on its own.

#### Write meaningful commit messages

- Your commit messages should be meaningful and follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), and the PR description should link to the related issue and comprehensively describe the changes. The history is a record people rely on to understand why a change was made, and it feeds release notes.

#### Keep history clean

- Rebase onto the latest `main` rather than merging it in, and use `git push --force-with-lease` when updating your branch. Squash incidental fixup commits.

#### Be responsive

- Don't let a PR sit idle with unaddressed comments until it needs a full rebase. If you are pausing work on an issue, indicate it in the PR comments or change the PR to draft status.

#### What if the status checks are failing?

- If a status check is failing and you believe the failure is unrelated to your change, leave a comment on the pull request explaining why. A maintainer will re-run the status check for you. If we conclude that the failure was a false positive, we will open an issue to track the problem with our status check suite.

## Release notes

This repository does not keep a single `CHANGELOG.md`. Each on-chain and off-chain component is tagged and released separately, as described in [RELEASES.md](./RELEASES.md), and the release notes for a tag are assembled from the conventional commit messages it contains. Your commit message and PR title are therefore what users will read.

Two things follow from that:

- **Mark breaking changes.** Use the `!` suffix, for example `feat(api)!: rename ICS26Router entry point`, and say in the PR description what adopters need to do. For the contracts and programs, state whether the change requires a new initializer or a state migration, since that determines the version bump under [RELEASES.md](./RELEASES.md).
- **Write the message for the reader of the release notes**, not for the reviewer. Chores and changes with no user-facing effect should be described as such so they can be filtered out.

## Code reviews

All changes must be code reviewed, preferably (and, for non-trivial changes, obligatorily) from someone who knows the areas the change touches. For non-trivial changes we may want two reviewers. The primary reviewer makes this decision and nominates a second reviewer if needed. Except for trivial changes, PRs should not be merged until relevant parties (for example, owners of the affected component) have had a reasonable chance to look at the PR in their local business hours.

Most PRs will find reviewers organically. If a maintainer intends to be the primary reviewer of a PR, they should set themselves as the assignee on GitHub. Only the primary reviewer of a change should do the merge, except in rare cases (for example, they are unavailable in a reasonable timeframe).

If a PR has gone five working days without a reviewer emerging, you can ask in the community chat, however please don't ping or message individual maintainers.

Disagreements that cannot be resolved among reviewers escalate to the maintainers, and only genuinely cross-cutting disputes reach the Technical Steering Committee.

## Developer Certificate of Origin (DCO)

All commits must be signed off to satisfy the [Developer Certificate of Origin](https://developercertificate.org/) (DCO). This certifies that you are able to submit your contribution under the license of the repository, and for it to be redistributed under that same license.

**TL;DR:** ensure all your commits have a sign-off. Git has a built-in mechanism for this via the `-s` (or `--signoff`) argument to `git commit`, provided your `user.name` and `user.email` are set up correctly:

```bash
git config user.name "FIRST_NAME LAST_NAME"
git config user.email "MY_NAME@example.com"
```

The sign-off must use your legal name, not a pseudonym. If you use the GitHub web UI for commits, make sure the `Signed-off-by` line uses the same email address as the commit author. This can be your GitHub `users.noreply.github.com` email if you keep your email address private.

For more detail, including how to set up a git alias and what to do when the DCO check fails on your PR, see [DCO.md](./DCO.md).

### Guidelines for submitting agentic contributions

DCO sign-offs are required for all contributions, and only a human may sign off on a commit. Agents are encouraged to use the `Co-Authored-By` or `Assisted-By` keys in DCO statements, and to include their model name, version, and context size. The human contributor still signs off and remains responsible for the change. Example:

```text
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Signed-off-by: Jane Contributor <jane@example.com>
```

## Copyright and license

All new code submitted to `ibc-contracts` must be under the [Apache License, Version 2.0](./LICENSE.md), and all new documentation must be under the Creative Commons Attribution 4.0 International License. You may maintain copyright to your works under these clauses.

Unlike some IBC repositories, this one **enforces a per-file license header**. Every source file carries the SPDX short-form identifier, and [CI fails without it](./.github/workflows/license.yml):

```solidity
// SPDX-License-Identifier: Apache-2.0
```

Use the comment syntax of the language in question. `just lint-license` reports files that are missing or have an incorrect header, and `just lint-license-fix` inserts them in place — run it before pushing rather than fixing the CI failure afterward.

## Third-party code

If your change vendors, copies, or adapts code from another project, raise it on the issue before writing the code rather than at review. Two things are required before the pull request can merge.

**1. Confirm the license is cleared.** `ibc-contracts` follows the Cosmos allowed third-party licenses policy:

- **Apache-2.0 code needs no license review.** It is Cosmos's recommended license, and this is the common case.
- **Anything else is an exception that must be approved by the Cosmos Governing Board**, unless it is *automatically* approved under the Allowlist License Policy. Automatic approval requires **all three** of the policy's conditions, not just the first:
  1. the component is fully licensable under the policy's list of approved licenses (MIT, ISC, the BSD licenses, Zlib and similar permissive terms); **and**
  2. it is either not stored in this repository at all, or else stored **unmodified, in source code form, in a designated third-party folder**; **and**
  3. it shows substantial outside use — part of the language's standard library, or created on GitHub at least 12 months ago with at least 10 stars or 10 forks.

  Condition 2 is the one that usually decides it. Copying non-Apache-2.0 code inline and editing it to fit this repository fails condition 2 no matter how permissive the license, so it needs a Governing Board exception rather than a judgement call in review. Note also that no copyleft license appears on the approved list at all.

  This applies to Solidity libraries pulled in as dependencies as much as to Rust crates: a contract copied out of another project and edited is vendored code, even if it is only a few dozen lines.

**2. Update [`NOTICE`](./NOTICE).** Record what was taken, where it came from, the copyright holder and years, and the license. Where the upstream project ships its own `NOTICE`, its readable contents must be reproduced — that is an obligation under section 4(d) of the Apache License, not a courtesy. Where the upstream is under MIT or a BSD license, the copyright and permission notice must be retained verbatim. The existing entries show the expected shape, down to naming the specific files that were adapted.

Two things in `NOTICE` are worth reading before you add to it. The Open Ethereum entry records GPL-3.0-or-later code in `packages/ethereum/trie-db/` that is a live license incompatibility rather than a settled attribution — do not use it as a precedent for adding more copyleft code. And the "Open items" section at the end lists what is not yet settled; if your change resolves one of those, say so in the pull request.

## Security contributions

If you think you have discovered a security issue in `ibc-contracts`, please follow the coordinated disclosure process described in [SECURITY.md](./SECURITY.md). Please do not open a public issue, pull request, or discussion for security vulnerabilities.

## Guidelines for non-code and other trivial contributions

Small documentation and typo fixes are welcome and follow the normal pull request path. Where you have several tiny fixes, please combine them into a single pull request rather than opening one per fix, so that reviewers and our continuous delivery systems are not put under unnecessary pressure. A pull request that fixes our tooling so that CI catches a class of mistake automatically is more valuable still.

## Specifications

The `ibc-specs` repository follows this same fork-and-pull path, with two differences. Contributions are made under the Community Specification License 1.0 rather than Apache 2.0, and a specification change is substantial by nature, so it always carries an [ADR](#architecture-decision-records-adr) rather than going straight to a pull request.

## Governance

The project is governed by a Technical Steering Committee under the [Technical Charter](./CHARTER.md). Decisions run on lazy consensus, with a TSC vote as the fallback: a change proceeds unless someone with standing objects within the stated window.

See [GOVERNANCE.md](./GOVERNANCE.md) for the roles, the contributor ladder, and how decisions are made, [TSC_MEMBERS.md](./TSC_MEMBERS.md) for the current committee, and [MAINTAINERS.md](./MAINTAINERS.md) for who holds merge rights on this repository and how that changes.

## Other important information

- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Governance](./GOVERNANCE.md)
- [Technical Charter](./CHARTER.md)
- [TSC members](./TSC_MEMBERS.md)
- [Maintainers](./MAINTAINERS.md)
- [Support](./SUPPORT.md)
- [Security policy](./SECURITY.md)
- [Releases](./RELEASES.md)
- [Upgradeability](./UPGRADEABILITY.md)
- [Adopters](./ADOPTERS.md)
- [DCO](./DCO.md)
