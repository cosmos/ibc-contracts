# Governance

IBC operates under a Technical Steering Committee (TSC) defined in the [Technical Charter](./CHARTER.md). The Charter is authoritative; this file is the contributor-facing summary of it.

Governance is lightweight by design: decisions run on lazy consensus, with a vote as the fallback, and the TSC's powers are a closed, enumerated list. Everything not reserved to the TSC defaults to the maintainers and their domains.

## Contributor ladder

- **Contributor:** anyone who opens an issue, a discussion, or a pull request. Open to all.
- **Committer:** a Contributor granted merge rights after a sustained record of substantial contributions, confirmed by a Maintainer vote.
- **Maintainer:** a Committer who holds the governance franchise and votes on promotions, removals, and committee representation.

Merge authority is earned through the same public record for everyone. The full criteria for each rung live in the [Technical Charter](./CHARTER.md), and the process for joining and leaving the group that holds merge rights on this repository is in [MAINTAINERS.md](./MAINTAINERS.md).

## Technical Steering Committee

The TSC has five seats:

- three Maintainer seats that the Maintainers elect from their pool;
- two Committer seats that the Committers elect from their pool.

A Related Parties cap prevents any single group from holding more than two of the five seats. The cap does not apply to the initial TSC and takes effect at the first election. The Charter names the founding TSC, which is listed in [TSC_MEMBERS.md](./TSC_MEMBERS.md), with a clear path for external contributors to earn seats as the community grows.

## Decision-making

Lazy consensus is the default: a change proceeds unless someone with standing objects within the stated window. Where consensus cannot be reached, the TSC votes.

A change that affects architecture, a public interface, or a specification routes through the [ADR process](./CONTRIBUTING.md#architecture-decision-records-adr). Requirements that arrive from external working groups are proposals like any other; the Maintainers and the TSC decide what to take up and when.

Where a vote is needed, the default threshold is 51% of TSC voting members, one vote each. Two decisions require a two-thirds vote of *all* TSC voting members rather than a majority of those voting:

- approving an inbound or outbound license exception ([Charter](./CHARTER.md) §7.c);
- amending the Charter (§8.a), which additionally requires approval by Cosmos Projects.

Quorum for a TSC meeting is fifty percent of voting members. A meeting may proceed without quorum but cannot decide anything.

## Meetings

TSC meetings and community calls are open to the public. Agendas are filed ahead of time using the [contributor call template](./.github/ISSUE_TEMPLATE/contributor-call.md), anyone may add an item, and notes are published afterward so that decisions are visible to people who could not attend.

Every call opens with the project's antitrust policy notice and a statement that the call is recorded. Both are obligations rather than formalities.

## Code of Conduct

The [Code of Conduct](./CODE_OF_CONDUCT.md) applies in every project space, including this repository, chat, the mailing list, and community calls. Reporting routes, including how to escalate a report that concerns a Maintainer or the Chair, are in that file.

## Full charter

See the [Technical Charter](./CHARTER.md) for the complete governance model, including the enumerated TSC powers, election procedures, and the amendment process.
