# Maintainers

<!-- Please keep all lists sorted alphabetically by github -->

This file lists the people who hold merge rights on this repository, and the process by which that changes. Governance roles and the Technical Steering Committee roster are in [GOVERNANCE.md](./GOVERNANCE.md) and [TSC_MEMBERS.md](./TSC_MEMBERS.md).

The [contributor ladder](./GOVERNANCE.md#contributor-ladder) has three rungs. A **Contributor** is anyone who opens an issue, a discussion, or a pull request. A **Committer** has earned merge rights. A **Maintainer** is a Committer who also holds the governance franchise and votes on promotions, removals, and committee representation.

> [!IMPORTANT]
> The roster below is derived from the `@cosmos/foundations-team` code owners for this repository. Each person needs to confirm their own row — name, community ID, Discord ID, email, and affiliation. The table format is mandated by Cosmos and all seven columns are required.

## Scopes

| Scope | Definition | GitHub Role | GitHub Team |
| --- | --- | --- | --- |
| Maintainer | Commit rights across the repository, plus the governance franchise: votes on promotions, removals, and TSC representation | Maintain | `@cosmos/foundations-team` |
| Committer | Commit rights, no governance franchise | Write | `@cosmos/foundations-team` |

## Active Maintainers

| Name | GitHub ID | Scope | Community ID | Discord ID | Email | Company Affiliation |
| --- | --- | --- | --- | --- | --- | --- |


> [!WARNING]
> **Maintainer diversity does not currently meet the Cosmos bar.** Every maintainer above is affiliated with a single organization. The project lifecycle criteria require maintainers drawn from **at least 2** organizations, with more expected as the project matures, assessed directly from the Company Affiliation column above. This is an entry criterion, and it cannot be resolved by editing this file — it needs maintainers promoted from outside Cosmos Labs. Confirm affiliations before publishing; if any of the handles above are in fact unaffiliated or employed elsewhere, this warning may need revising rather than removing.

## Emeritus Maintainers

There are no emeritus Maintainers yet. Maintainers moved to emeritus status under [Removing Maintainers](#removing-maintainers) are listed here, in the same seven-column format as the active list.

| Name | GitHub ID | Scope | Community ID | Discord ID | Email | Company Affiliation |
| --- | --- | --- | --- | --- | --- | --- |

## Becoming a Maintainer

`ibc-contracts` welcomes community contribution. Each community member may progress to become a Committer and then a Maintainer, and merge authority is earned through the same public record for everyone.

How to become a Maintainer:

- Contribute significantly to the code in this repository.
- Review other contributors' pull requests. Review history counts toward standing as much as authorship does.

### Contribution requirement

The requirement to be able to be proposed as a Maintainer is:

- 5 significant changes have been authored in this repository by the proposed Maintainer and accepted (merged PRs).

### Approval process

The following steps must occur for a contributor to be promoted:

#### PR proposed

- The proposed Maintainer has the sponsorship of at least one current Maintainer.
    - This sponsoring Maintainer will create a proposal PR modifying the list of Maintainers. See [proposal PR template](#proposal-pr-template).
    - The proposed Maintainer accepts the nomination and expresses a willingness to be a long-term (more than 6 month) contributor by adding a comment in the proposal PR.
    - The PR will be communicated in all appropriate communication channels, including at least the community chat, [GitHub Discussions](https://github.com/cosmos/ibc-contracts/discussions), and any maintainer or community call.

#### Voting

- Approval by at least 3 current Maintainers within two weeks of the proposal, or an absolute majority (half the total + 1) of current Maintainers.
    - Maintainers vote by approving the proposal PR.
- No veto raised by another Maintainer within the voting timeframe.
    - All vetoes must be accompanied by a public explanation as a comment on the proposal PR.
    - The explanation of the veto must be reasonable and follow the [Code of Conduct](./CODE_OF_CONDUCT.md).
    - A veto can be retracted; in that case the voting timeframe is reset and all approvals are removed.
    - It is bad form to veto, retract, and veto again.

The proposed Maintainer becomes a Maintainer either:

- when two weeks have passed without a veto since the third approval of the proposal PR, or
- when an absolute majority of Maintainers has approved the proposal PR.

In either case, no Maintainer raised and stood by a veto.

#### After the merge

Merging the roster PR does not grant access. A current Maintainer must add the new Maintainer to the corresponding GitHub Team by hand. Nothing automates this, so it belongs in the merge checklist — otherwise this file and the actual permissions drift apart, and [CODEOWNERS](./.github/CODEOWNERS) rules silently stop matching the people they name.

## Removing Maintainers

Being a Maintainer is not a status symbol or a title to be maintained indefinitely.

It will occasionally be necessary and appropriate to move a Maintainer to emeritus status. This can occur in the following situations:

- Resignation of a Maintainer.
- Violation of the Code of Conduct warranting removal.
- Inactivity.
    - A general measure of inactivity will be no commits or code review comments for two reporting quarters, although this will not be strictly enforced if the Maintainer expresses a reasonable intent to continue contributing.
    - Reasonable exceptions to inactivity will be granted for known long-term leave such as parental leave and medical leave.
- Other unspecified circumstances.

### PR proposed

- A PR listing the justification for moving the Maintainer to emeritus, most commonly inactivity, is proposed.

### Voting

The voting process for moving a Maintainer to emeritus status is the same as for adding a new Maintainer.

Returning to active status from emeritus status uses the same steps as adding a new Maintainer. Note that the emeritus Maintainer always already has the required significant contributions, so there is no contribution prescription delay.

### After the merge

Remove the Maintainer from the GitHub Team, and audit every other access path they held: package registries, release signing keys, CI secrets, deployer and admin keys for any live deployment, and the security disclosure channels in [SECURITY.md](./SECURITY.md). Moving a row between tables in this file changes nothing on its own.

## Proposal PR template

```markdown
I propose to add [maintainer github handle] as a ibc-contracts project maintainer.

[maintainer github handle] contributed with many high quality commits:

- [list significant achievements]

Here are [their past contributions on the ibc-contracts project](https://github.com/cosmos/ibc-contracts/commits?author=[user github handle]).

Voting ends two weeks from today.

For more information on this process, see the MAINTAINERS.md file.
```

## Modifying this file

Other than adding and removing Maintainers, any changes to this file (MAINTAINERS.md) must be proposed in a PR and agreed upon by two thirds of current Maintainers.
