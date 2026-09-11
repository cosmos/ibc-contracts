# ADR: Rate Limiting in IFT in Solidity

**Status**: Proposed
**Date**: 2026-09-10
**Last Updated**: 2026-09-11

## Context

IFT (Interchain Fungible Token) requires rate limiting to prevent funds from being drained quickly in the event of an IBC exploit. In this document, we document the reasons behind the rate limiting design.

The objective is to bound how quickly tokens can be minted or moved across chains, giving operators time to detect an exploit and respond. Rate limiting cannot establish that an incoming transfer is legitimate, and an attacker can continue using capacity as it refills until the affected route is disabled.

This ADR describes the proposed IFT policy. The existing [IFT base contract](../../../ibc-solidity/contracts/utils/IFTBaseUpgradeable.sol) burns on send and mints on receive, but does not yet enforce this policy. The [ICS20 escrow rate limiter](../../../ibc-solidity/contracts/utils/RateLimitUpgradeable.sol) uses a different, fixed-window design.

## Summary

|     **Type**    |  **Design Axis**  |                                                                                                  **Meaning**                                                                                                 | **Decision** |                                                          **Reason**                                                         |
|:---------------:|:-----------------:|:------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------:|:------------:|:---------------------------------------------------------------------------------------------------------------------------:|
|  **Algorithm**  |    Fixed Window   | Usage resets to zero at a period boundary (e.g. 00:00 UTC or every 6h). | ❌ | Allows two full limits to pass immediately across a window boundary. |
|  **Algorithm**  |   Sliding Window  |                                                     Every transfer is timestamped and the sum over the trailing window seconds must stay under the limit.                                                    |       ❌      |                                           Consumes more gas than refilling bucket.                                          |
|  **Algorithm**  |  Refilling Bucket | Transfers consume capacity, which refills at `capacity / window` per second. With unchanged parameters, at most `2 × capacity` can pass in a window-length interval. | ✅ | Constant-size accounting with gradual recovery and no boundary reset. |
|   **Feature**   | Absolute Capacity |                                                                                        Limit is a fixed token amount.                                                                                        |       ❌      |                           Very hard to set it right initially. Might require constant adjustment.                           |
|   **Feature**   |     % Capacity    | Limit is a share of local token supply; the supply sampling policy remains to be specified. | ✅ | Scales with local supply, reducing manual adjustments. |
| **Improvement** |   Capacity Floor  |                                                                      Limit is the `max(floor, %capacity)` in case %capacity is too low.                                                                      |       ✅      |                             When seeding a token, %capacity would evaluate to a very low number                             |
|   **Feature**   |      Inbound      |                                                                                         Mints on receive are limited.                                                                                        |       ✅      |                                          We want to be as restrictive as possible.                                          |
|   **Feature**   |      Outbound     |                                                                                          Burns on send are limited.                                                                                          |       ✅      |                                          We want to be as restrictive as possible.                                          |
|   **Feature**   |  Rewinding Usage  |                                                         Flow in the opposite direction gives capacity back, so round trips cannot exhaust the limit.                                                         |       ❌      | Rewinding usage means that an attacker who can mint tokens, can send outbound transfers to gain some capacity back to mint. |
|   **Decision**  |      Optional     | Whether IFT bridging can operate without configured rate limits. | ❌ | Require explicit configuration before bridging can be used. |
|   **Decision**  |  Per Token/Client | Whether each token shares its limits across IBC clients or has separate limits for each client. | Per token | Adding routes must not multiply the token's aggregate allowance. |

## Decisions

### Algorithm

**Decision: use a refilling bucket.** Each transfer consumes an amount of available capacity equal to its token amount. Capacity recovers with elapsed time, up to the configured maximum. The contract only needs to account for that recovery when it reads or updates the bucket; no periodic transaction is required.

For an unchanged capacity `C`, refill window `W`, available amount `A`, and elapsed time `dt`, the conceptual accounting is:

```text
available = min(C, A + C * dt / W)
require(amount <= available)
available = available - amount
```

The window is the time needed to refill an empty bucket, not a promise that only `C` tokens can pass in every trailing window. Over an interval of length `T`, the upper bound is `C + C * T / W`: the initial burst plus the refill. For example, a capacity of 1,000 tokens and a one-hour window permits 1,000 tokens immediately from a full bucket, followed by another 500 after half an hour. Up to 2,000 tokens can pass over an hour if the refill is fully consumed. This bound assumes capacity and refill parameters do not change during the interval.

An exact sliding window provides a stricter guarantee: no more than the limit can pass in any trailing window. However, recording transfers and expiring their contributions requires more storage and processing as activity grows. Coarser time buckets can reduce that cost, but introduce approximation and additional accounting choices. We choose the refilling bucket for predictable accounting cost and gradual recovery, accepting the weaker trailing-window bound. The gas advantage over an exact transfer log does not establish that it is cheaper than every fixed-window implementation.

### Capacity

**Decision: derive capacity from a percentage of local token supply, with an absolute floor.** For a percentage expressed in basis points:

```text
capacity = max(capacityFloor, referenceSupply * capacityBps / 10_000)
```

`referenceSupply` is the local supply selected by the sampling policy, which remains an implementation decision. IFT burns and mints tokens rather than holding an escrow balance, so local token supply is the relevant baseline here. It is not a measure of backing, global supply, or market liquidity.

The floor makes bootstrapping possible. A newly deployed token with zero local supply would otherwise have zero inbound capacity and could never receive its first transfer. For example, with a 1% capacity and a 1,000-token floor, capacity is 1,000 at zero supply, remains 1,000 at a supply of 50,000, and becomes 10,000 at a supply of 1,000,000. These numbers illustrate the formula; they are not proposed deployment defaults.

### Inbound and Outbound

**Decision: limit both inbound mints and outbound burns.** The proposed accounting uses an independent bucket for each direction, shared across all clients for that token. An inbound transfer consumes inbound allowance; an outbound transfer consumes outbound allowance. Receiving tokens does not spend the allowance needed to send tokens, and vice versa.

### Rewinding Usage

**Decision: do not restore allowance in response to transfers in the opposite direction.** The limiter tracks gross activity in each direction rather than net flow. With unchanged parameters, allowance recovers through time-based refill only.

Rewinding guards against round trips that would otherwise consume allowance in both directions. For example, without rewinding, if a user sends 100 tokens back and forth, then they can continue to do so until the entire allowance is consumed. Note that with refilling buckets, the allowance continuously recovers.

However, rewinding usage creates a security risk. An attacker who can cause fraudulent mints could mint until they hit the rate limit, then send them to another chain, and recover allowance to continue minting. The local burn can result in a mint elsewhere, so the attacker has moved the tokens while regaining permission to mint locally.

### Per Token vs Per Client

**Decision: share each token's directional limits across all of its registered IBC clients on a given chain.** Here, “per token” means per local IFT contract. Separate tokens have separate budgets, and deployments on other chains enforce their own limits; this does not create a synchronized global bucket.

Given that the IFT contract is the authority for its own supply, it is reasonable to treat all clients as a single source of demand. This avoids multiplying the token's aggregate allowance by the number of clients, which could

## Open Implementation Questions

The choices above establish the policy, but the implementation must still address some details:

- **Supply sampling and capacity updates:** When is reference supply sampled, and how are remaining allowance and elapsed refill handled when supply or configuration changes? Updates must not accidentally reset consumed usage or apply a new refill rate retroactively.
