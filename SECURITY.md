# Security Policy

## Reporting a Security Bug

If you think you have discovered a security issue in `ibc-contracts`, we'd love to hear from you. We will take all security bugs seriously and, if confirmed upon investigation, we will patch it within a reasonable amount of time and release a public security advisory discussing the impact and crediting the discoverer.

> [!IMPORTANT]
> Do not open a public issue, pull request, or discussion for a suspected vulnerability. Public disclosure before a fix is available puts adopters at risk.

`ibc-contracts` accepts security bugs through the following channels:

- The [Cosmos HackerOne Bug Bounty program](https://hackerone.com/cosmos?type=team) is the primary vulnerability disclosure channel, and the only one eligible for bounty rewards.
- [GitHub private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability) on this repository.
<!-- markdown-link-check-disable-next-line -->
- Email to [security@interchain.io](mailto:security@interchain.io). Please submit only one unique email thread per vulnerability. Reports submitted by email are ineligible for bounty rewards.

When sending information through any of these channels, please include a description of the flaw and any related information: reproduction steps, the affected versions, the impact, and whether the flaw is under known active use. For on-chain components, state which network and contract or program addresses are affected.

Artifacts from an email report are saved at the time the email is triaged. We are not able to monitor dynamic content, such as a document link that is edited after receipt, throughout the lifecycle of a report. To share additional information or correct earlier information, send it as an additional reply or attachment.

## What to expect

- Acknowledgement of your report within a few working days.
- An initial assessment and severity classification.
- Coordinated disclosure: the report stays confidential within the Technical Steering Committee and its designees while a fix is prepared. We will agree an embargo and disclosure timeline with you, and coordinate the release of the fix and an advisory.
- Credit to reporters who wish to be named.

## Coordinated Vulnerability Disclosure Policy and Safe Harbor

For the most up-to-date version of the policies that govern vulnerability disclosure, consult the [HackerOne program page](https://hackerone.com/cosmos?type=team&view_policy=true).

The policy hosted on HackerOne is the official Coordinated Vulnerability Disclosure policy and Safe Harbor for the Interchain Stack, and the teams and infrastructure it supports. It supersedes previous security policies used by individual teams and projects with targets in scope of the program.

## Supported versions

Each on-chain and off-chain component in this repository is released and versioned separately. Security fixes are provided for the release lines listed under [Supported versions](./RELEASES.md#supported-versions) in `RELEASES.md`. Adopters running supported, up-to-date releases are the priority for coordinated fixes.

## Deployed contracts and privileged roles

The security model of a live deployment depends on more than the code in this repository. Before reporting, it is worth reading:

- [Security Assumptions](./README.md#security-assumptions) for the trust assumptions the protocol makes, including how frozen light clients are handled.
- [Security Council and Governance Admin](./README.md#security-council-and-governance-admin) for the privileged roles that can pause and upgrade a deployment.
- [UPGRADEABILITY.md](./UPGRADEABILITY.md) for the upgrade paths and their constraints.

A finding that depends on a privileged role behaving maliciously is in scope only where it breaks a stated assumption in those documents. Say which assumption you believe is broken.

## Audits

Third-party audit reports are published in [`docs/audits`](./docs/audits). A finding already recorded in one of those reports, and accepted or fixed there, is not a new report.

## Artifact signing

Releases are signed so adopters can verify provenance. The project is adopting [Sigstore](https://www.sigstore.dev) for release artifact signing as part of its supply-chain hardening work.

> [!IMPORTANT]
> This is an open gap, not a completed item. The OpenSSF Scorecard currently reports no signed releases for this repository, and the Cosmos lifecycle criteria set `Signed-Releases` as a **MUST**. Signing needs to be wired into the [release workflow](./.github/workflows/release.yml), with published verification instructions and a key fingerprint reachable without trusting the release itself, before this section can be stated as fact. This repository publishes several artifact types per release — Solidity contract tags, SP1 program ELF files, the CosmWasm light client blob, and Solana programs — and each needs to be covered.

## Scope

This policy covers the IBC code repositories and the relayer and attestor infrastructure. For this repository that includes the Solidity contracts, the Solana programs, the SP1 light client programs, the CosmWasm Ethereum light client, and the off-chain proof API and attestor. Specification-only concerns with no security impact should go through the normal [ADR process](./CONTRIBUTING.md#architecture-decision-records-adr).

## More information

- [Maintenance and Security](https://github.com/cosmos/security) for detailed policies.
