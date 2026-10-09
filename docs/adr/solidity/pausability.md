# ADR: Pausing and Freezing in IBC-Solidity

**Status**: Proposed
**Date**: 2026-10-08

## Context

Rate limits bound how quickly funds can move during an exploit, but they cannot stop one (see the [IFT rate limit ADR](./ift-ratelimit.md)). Once an exploit is detected, operators need a way to halt the affected flows while they investigate, patch, or migrate. In this document, we decide which IBC-Solidity contracts can be halted, what a halt blocks, and who is allowed to trigger it.

Some of this already exists. [`ICS20Transfer`](../../../ibc-solidity/contracts/ICS20Transfer.sol) and [`ICS27GMP`](../../../ibc-solidity/contracts/ICS27GMP.sol) inherit OpenZeppelin's `PausableUpgradeable`, and the [deployment script](../../../ibc-solidity/scripts/deployments/DeployAccessManagerWithRoles.sol) wires their `pause()` and `unpause()` to `PAUSER_ROLE` and `UNPAUSER_ROLE` from [`IBCRolesLib`](../../../ibc-solidity/contracts/utils/IBCRolesLib.sol). Light clients freeze permanently when misbehaviour is proven. The router, the IFT contracts, and the light clients have no halt that an operator can trigger. Pausing was added without an ADR, so this document records the reasoning behind the existing behaviour and decides on new knobs.

Two terms are used throughout:

- **Pause**: a reversible halt triggered by an authority.
- **Freeze**: the permanent halt of a light client after proven misbehaviour. A frozen client is never unfrozen; it is replaced.

The guiding principle is that a pause affects liveness, never safety. A pauser cannot move funds, only stop them from moving. To react within minutes, the pauser is meant to be a fast, low-threshold key, such as a monitoring bot or a small multisig. A key like that must only be able to halt contracts whose operator chose to trust it. Pause power therefore sits on the contract that carries the risk, and is scoped so that it cannot halt parties who never opted into it.

## Summary

|   **Category**   |           **Option**           |                             **Meaning**                             | **Decision** |                                                            **Reason**                                                            |
|:----------------:|:------------------------------:|:-------------------------------------------------------------------:|:------------:|:--------------------------------------------------------------------------------------------------------------------------------:|
|    **Scope**     |          ICS26Router           |        A router-level pause that halts every app and client.        |      ❌       |        Apps and light clients are registered permissionlessly. A router pauser could halt parties that never trusted it.         |
|    **Scope**     |          Light Client          |       Each light client can be paused by its own role manager.      |      ✅       |              Halts a single connection without affecting others, and is controlled by whoever deployed the client.               |
|    **Scope**     |         ICS20Transfer          |                      App-wide pause (existing).                     |      ✅       |                                Holds escrowed funds and mints vouchers on behalf of its operator.                                |
|    **Scope**     |            ICS27GMP            |                      App-wide pause (existing).                     |      ✅       |              Executes arbitrary calls through accounts. IFTs built on it inherit its pauser as a trust assumption.               |
|    **Scope**     |       IFTBaseUpgradeable       |                  Pause logic in the abstract base.                  |      ❌       |                         Access control belongs to the inheriter. The base exposes virtual hooks instead.                         |
|    **Scope**     |  IFTOwnable, IFTAccessManaged  |             Pause in the reference IFT implementations.             |      ✅       |                           Issuers deploying the reference contracts get a pause without writing code.                            |
|    **Scope**     | Escrow, IBCERC20, ICS27Account |                    Pause in the helper contracts.                   |      ❌       |                      Their bridge functions are only reachable through their parent app, which is pausable.                      |
| **Granularity**  |         Contract-wide          |           A single flag halts every flow of the contract.           |      ✅       |                                           Easy to operate under pressure. OZ default.                                            |
| **Granularity**  |         Per Direction          |             Sends and receives can be paused separately.            |      ❌       | Containing an exploit usually requires halting both, and every extra state is one more thing to reason about during an incident. |
| **Granularity**  |       Per Client in Apps       |                 An app can pause individual clients.                |      ❌       |                            Covered by the light client pause. IFT can already remove a single bridge.                            |
|    **Effect**    |             Sends              |                       Outbound packets revert.                      |      ✅       |                                                  Stops new funds from leaving.                                                   |
|    **Effect**    |            Receives            |      Inbound packets are written with an error acknowledgement.     |      ✅       |                      The router turns app reverts into error acknowledgements, so the source chain refunds.                      |
|    **Effect**    |       Acks and Timeouts        | Acknowledgements and timeouts revert, and the packet stays pending. |      ✅       |                            Refunds can be forged too. Pending packets can be retried after unpausing.                            |
|    **Effect**    |        Admin Functions         |             Configuration remains callable while paused.            |      ✅       |                                  Operators must be able to fix configuration before unpausing.                                   |
|    **Effect**    |        Token Transfers         |                  Local ERC20 transfers are paused.                  |      ❌       |                 A pause halts the bridge, it does not freeze the token. Token-level policy belongs in `_update`.                 |
|  **Authority**   |     Pauser/Unpauser Split      |                  Different roles pause and unpause.                 |      ✅       |                                        Pausing must be fast, resuming must be deliberate.                                        |
|  **Authority**   |          Pause Expiry          |          A pause lifts automatically after a fixed period.          |      ❌       |        Whoever can hold a pause can already halt the contract by other means. Expiry could reopen an unpatched contract.         |
|  **Authority**   |          Close Target          |    Use AccessManager's `setTargetClosed` instead of a pause flag.   |      ❌       |                    Admin-only, subject to the target admin delay, and it only affects `restricted` functions.                    |
| **Light Client** |        Reversible Pause        |                       The pause can be lifted.                      |      ✅       |                                    A false alarm can be undone without migrating the client.                                     |
| **Light Client** |   Misbehaviour While Paused    |       Misbehaviour can still be submitted to a paused client.       |      ✅       |                                A paused client must still be able to freeze permanently on proof.                                |
| **Light Client** |        Send-Side Check         |        The router rejects sends on paused or frozen clients.        |      ❌       |   Not part of this decision. Sends are still delivered; only their acknowledgements wait. Documented as a possible follow-up.    |

## Decisions

### Router

**Decision: the router is not pausable.** Anyone can register an IBC application through `addIBCApp(address)` and a light client through `addClient(counterpartyInfo, client)`. A router pause would hand a single fast key the power to halt every one of these parties, including those that never trusted it, which is exactly what the guiding principle rules out. Each layer that carries risk can halt itself instead: applications pause themselves, and light clients pause themselves (see [Light Clients](#light-clients)).

This does not make the router free of trusted levers, and we document the ones that already exist. The AccessManager admin can:

- upgrade the router through UUPS,
- replace the light client behind any client identifier with `migrateClient`, including clients that were registered permissionlessly,
- revoke relayers, when relaying is restricted to `RELAYER_ROLE`,
- close the router as a target in the AccessManager. This blocks every `restricted` function (`recvPacket`, `ackPacket`, `timeoutPacket`, `updateClient`, the custom-identifier registrations, and `migrateClient`), but not `sendPacket` or the permissionless registrations.

These are admin powers, intended for governance and protected by AccessManager execution delays. This ADR does not add a fast path to them.

Mature implementations made the same choice. The ibc-go core has no pause, the [Hyperlane v3 `Mailbox`](https://github.com/hyperlane-xyz/hyperlane-monorepo/blob/main/solidity/contracts/isms/PausableIsm.sol) dropped the pause its predecessor had in favour of opt-in pausable modules, and the [LayerZero V2 endpoint](https://github.com/LayerZero-Labs/LayerZero-v2/blob/main/packages/layerzero-v2/evm/protocol/contracts/MessagingChannel.sol) cannot be paused.

### Applications

**Decision: `ICS20Transfer` and `ICS27GMP` remain pausable with a contract-wide flag.** ICS20 escrows native tokens and mints vouchers on behalf of its operator, so the operator carries the risk and holds the pause. GMP executes arbitrary calls through its accounts, so a bug in GMP is high-impact and needs a halt.

GMP is also shared infrastructure for every IFT. While GMP is paused, `iftTransfer` reverts because `sendCall` reverts, inbound mints fail with an error acknowledgement because `onRecvPacket` reverts, and refunds revert because the ack and timeout callbacks revert. Every IFT issuer therefore trusts the GMP pauser and unpauser for liveness. GMP binds to a fixed port, so issuers cannot simply deploy a private instance next to the shared one. We accept this trust assumption, because removing the pause from GMP would leave no way to halt arbitrary execution, and a pause cannot move funds.

The helper contracts are not pausable. `Escrow` releases funds only on calls from ICS20, `IBCERC20` mints and burns only on calls from ICS20, and `ICS27Account` executes only on calls from GMP. Pausing the parent app halts all of their bridge activity.

### IFT

**Decision: [`IFTBaseUpgradeable`](../../../ibc-solidity/contracts/utils/IFTBaseUpgradeable.sol) holds no pause state, but exposes internal virtual hooks around every bridge flow. [`IFTOwnable`](../../../ibc-solidity/contracts/utils/IFTOwnable.sol) and [`IFTAccessManaged`](../../../ibc-solidity/contracts/utils/IFTAccessManaged.sol) implement a pause on top of them.**

The base deliberately leaves access control to the inheriter, as it already does with `_onlyAuthority`. A pause in the base would have to pick an authority for it. Inheriters must still be able to add their own pause, and today they cannot. `iftTransfer`, `iftMint`, `onAckPacket` and `onTimeoutPacket` are not virtual, so the only available extension point is the ERC20 `_update` override. That override also gates local transfers and the issuer's own `mint`, and it cannot tell which client a mint came from.

The base will call a hook at each point where tokens cross the bridge. These are the same points where the rate limit is consumed. Each hook runs before any state change, receives the full context of the flow, and does nothing by default. The proposed shape is:

```solidity
/// @dev Called before an outbound transfer burns `amount` from `sender`.
function _beforeIFTTransfer(string memory clientId, address sender, string memory receiver, uint256 amount) internal virtual {}

/// @dev Called before an inbound transfer mints `amount` to `receiver`.
function _beforeIFTMint(string memory clientId, address receiver, uint256 amount) internal virtual {}

/// @dev Called before a pending transfer is refunded after an error acknowledgement or a timeout.
function _beforeIFTRefund(string memory clientId, uint64 sequence, address sender, uint256 amount) internal virtual {}
```

The hooks are general purpose. Pausing is the first use, but the same hooks can enforce per-client policies, receiver allowlists, or extra accounting. Refunds get their own hook because they carry a different context from mints: the original sender and the packet sequence. A successful acknowledgement is not hooked, because it only clears the pending record and moves no funds. Exact parameter types are settled in the implementation.

`IFTOwnable` and `IFTAccessManaged` inherit `PausableUpgradeable`, implement [`IPausable`](../../../ibc-solidity/contracts/interfaces/IPausable.sol), and override all three hooks with `_requireNotPaused()`. `PausableUpgradeable` keeps its state in an ERC-7201 namespace, so adding it to deployed proxies is storage-safe. In `IFTAccessManaged`, `pause()` and `unpause()` are `restricted`, and the existing `IBCRolesLib.pauserSelectors()` and `unpauserSelectors()` map them to `PAUSER_ROLE` and `UNPAUSER_ROLE`. `IFTOwnable` has a single owner, which both pauses and unpauses. Issuers who need the roles split should use `IFTAccessManaged`.

Two existing knobs are not a substitute for the pause. Setting the IFT rate limit capacity to zero also blocks every flow, but only the authority can do it, it needs a fresh configuration to undo, and it shares a setting with the rate limit policy. `removeIFTBridge` disables a single client while refunds keep working, which is useful for retiring a route but not for stopping an incident.

### Granularity

**Decision: an application pause is a single contract-wide flag.** The flag halts sends, receives, acknowledgements and timeouts together. OpenZeppelin's `Pausable` works this way, and it leaves operators a single question during an incident: is the contract paused or not?

We considered pausing sends and receives separately, similar to ibc-go's ICS20 `SendEnabled` and `ReceiveEnabled` parameters, or per message type like the [Cosmos SDK `x/circuit`](https://github.com/cosmos/cosmos-sdk/blob/main/contrib/x/circuit/README.md) module. A forged receive and a forged refund are equally dangerous, so a one-directional pause rarely contains an exploit, and every extra combination of states is one more thing to reason about under pressure. More granularity can be added later without changing the contract-wide flag.

We also considered pausing individual clients inside applications. The light client pause already halts a single connection for every app that uses it, and IFT can remove a single bridge, so a per-client app pause would duplicate both.

### What a Pause Blocks

**Decision: a paused application rejects sends, receives, acknowledgements and timeouts, while admin functions stay available.** How each kind of packet behaves follows from how the router calls the application:

- **Sends** revert, so no packet is committed.
- **Receives**: the router wraps `onRecvPacket` in a try/catch and writes the universal error acknowledgement when the app reverts. A receive on a paused app is therefore not left pending. The source chain refunds the sender when it processes the error acknowledgement. Leaving the packet pending is not possible without changing the router, and would gain little: timeouts are capped at one day (`MAX_TIMEOUT_DURATION`), and IFT transfers default to fifteen minutes.
- **Acknowledgements and timeouts**: app reverts here propagate through the router, so the whole transaction reverts and the packet commitment is kept. After the app is unpaused, a relayer can submit the same acknowledgement or timeout again. Blocking refunds is deliberate. A refund mints or releases funds, and as the [refund section of the IFT ADR](./ift-ratelimit.md#refund-handling) shows, an attacker who can forge timeouts can use refunds to double spend. While an exploit is suspected, refunds are as dangerous as receives.
- **Admin functions** stay callable, for example `setRateLimit`, `setCustomERC20`, beacon upgrades, `registerIFTBridge`, `setIFTRateLimit` and contract upgrades. Operators need them to fix the configuration before they unpause.

A pause does not freeze tokens. ICS20 vouchers and IFT tokens remain transferable on the local chain. A token issuer who also wants a token freeze can override `_update`, which is a token policy rather than a bridge policy.

### Authority

**Decision: pausing and unpausing are held by separate roles, and a pause does not expire.**

`PAUSER_ROLE` is meant for a fast key without an execution delay, so it can react within minutes. `UNPAUSER_ROLE` is meant for the same body that holds admin rights, or one with a higher threshold, because resuming after an incident is a deliberate decision. Wormhole NTT uses the same split: the owner or a pauser can pause, but only the owner can unpause ([`ManagerBase`](https://github.com/wormhole-foundation/native-token-transfers/blob/main/evm/src/NttManager/ManagerBase.sol)). Light clients use the same two role names in their own `AccessControl` (see [Light Clients](#light-clients)).

We considered letting pauses expire automatically, like Optimism's [`SuperchainConfig`](https://github.com/ethereum-optimism/optimism/blob/op-contracts/v4.0.0/packages/contracts-bedrock/src/L1/SuperchainConfig.sol), where a guardian pause lifts after three months and cannot be triggered again until it is reset. That bound exists so that a guardian cannot hold rollup withdrawals hostage. In our case, whoever can hold a pause can already halt the contract by other means: applications and IFTs can be upgraded by their authority, and a light client with a role manager only serves the proof submitters that the role manager authorizes. Expiry would add no guarantee, and it could reopen a contract before the fix is ready.

We also considered using AccessManager's `setTargetClosed` as the application pause, instead of a pause flag. Closing a target needs `ADMIN_ROLE` and is subject to the target admin delay, so it is not a fast lever. It also blocks only `restricted` functions, while `sendTransfer`, `sendCall` and `iftTransfer` are public and the app callbacks are guarded by `onlyRouter`.

### Light Clients

**Decision: [`SP1ICS07Tendermint`](../../../ibc-solidity/contracts/light-clients/sp1-ics07/SP1ICS07Tendermint.sol), [`BesuLightClientBase`](../../../ibc-solidity/contracts/light-clients/besu/BesuLightClientBase.sol) and [`AttestationLightClient`](../../../ibc-solidity/contracts/light-clients/attestation/AttestationLightClient.sol) get a reversible pause, controlled by roles that are granted to the client's role manager.**

A per-client pause halts a single connection, for every application that uses it, without touching any other connection. It is the IBC equivalent of cursing a single lane in Chainlink CCIP's [`RMNRemote`](https://github.com/smartcontractkit/chainlink-ccip/blob/contracts-ccip-v1.6.4/chains/evm/contracts/rmn/RMNRemote.sol), and it covers what ibc-go handles through client status. It is the response to a suspected light client problem that cannot be proven on chain: a prover bug, a compromised attestor key, or a validator set behaving strangely.

The pause belongs in the light client, not in the router. A router-level client pause would be held by the router authority, which could then halt clients it does not own. Every light client is deployed with a role manager that already administers it, so the pause sits with the client's owner.

- **Mechanism**: the light clients are immutable, so they inherit OpenZeppelin's non-upgradeable `Pausable` and implement `IPausable`. `PAUSER_ROLE` and `UNPAUSER_ROLE` are granted to the role manager at construction, and the role manager can grant them to other accounts. A client deployed without a role manager has no admin and cannot be paused, which keeps fully permissionless clients permissionless.
- **Existing lever**: `updateClient`, `verifyMembership` and `verifyNonMembership` are already gated by `PROOF_SUBMITTER_ROLE`, so a role manager could halt a client today by revoking that role from the router. That mixes up who may submit proofs with whether the client is halted. It needs `DEFAULT_ADMIN_ROLE` rather than a dedicated fast key, and undoing it means reconstructing the submitter set exactly. A separate pause flag keeps the two concerns apart.
- **Blocked**: `updateClient`, `verifyMembership` and `verifyNonMembership` revert while paused. The router verifies proofs before it calls the application and outside the try/catch, so `recvPacket`, `ackPacket` and `timeoutPacket` revert on a paused client. Packets stay pending on both chains instead of producing error acknowledgements.
- **Allowed**: `misbehaviour` still works for whoever may submit it, so a paused client can still be frozen permanently when misbehaviour is proven.
- **State**: the pause flag lives in its own storage, not in the client state. `isFrozen` keeps its meaning, a permanent freeze on proven misbehaviour, and the encoding returned by `getClientState` does not change for relayers.

**The pause is reversible, and there is no admin-triggered permanent freeze.** After an investigation, the outcome is either a false alarm or a confirmed problem. A false alarm is resolved by unpausing. A confirmed problem is resolved by migrating the client identifier to a new light client with `migrateClient`, which keeps the identifier and its packet state, as `MsgRecoverClient` does in ibc-go. A permanent admin freeze would remove the first option, and recovering from it would always need a router-admin migration, even for clients registered permissionlessly.

The [`ICS02PrecompileWrapper`](../../../ibc-solidity/contracts/light-clients/ics02-wrapper/ICS02PrecompileWrapper.sol) is not pausable. It forwards to an ibc-go client that lives in the Cosmos chain's state, and that client is halted and recovered through the chain's own mechanisms: client status, `MsgRecoverClient` and `x/circuit`.

### Sending on a Paused or Frozen Client

**Decision: the router does not check client status on send. This is documented, not changed.** `sendPacket` only looks up the counterparty of the source client and never calls the light client. A packet sent from chain A over a paused or frozen client is still committed, and chain B can still receive it, because B verifies it with its own client of A, which is unaffected. The acknowledgement or timeout back on A needs A's client of B, so it waits until that client is unpaused or migrated. In the meantime, the sender's pending state, such as an IFT pending transfer, stays open.

This is a liveness issue, not a safety issue. A light client pause stops exactly the operations where chain A trusts B's state, and a send does not depend on that trust. This already happens today with clients frozen by misbehaviour. Applications and front ends should check a client's `paused()` and frozen status before sending. ibc-go rejects sends on non-active clients, so a status view on `ILightClient`, checked by `sendPacket`, is a possible follow-up. It would change the light client interface, so it is out of scope here.

## Consequences

### Trust Assumptions

|        **Actor**        |                                  **Can Halt**                                  |                  **Cannot**                  |
|:-----------------------:|:------------------------------------------------------------------------------:|:--------------------------------------------:|
|   AccessManager admin   | Everything, slowly: upgrades, `migrateClient`, closing targets, granting roles |        Move funds without an upgrade         |
|   ICS20 or GMP pauser   |            That application. A GMP pause halts every IFT using it.             |            Move funds, or unpause            |
|   Light client pauser   |              A single connection, for every application using it               |   Unpause, or undo stored consensus states   |
| IFT authority or pauser |                           That token's bridge flows                            | Halt local token transfers through the pause |

### Operational Notes

- Pausing a destination application turns packets in flight into error acknowledgements, and the senders are refunded on the source chain.
- Pausing a source application delays acknowledgements and refunds until it is unpaused. They are not lost.
- A paused light client is not updated. If a client stays paused for longer than its trusting period (SP1 ICS07 and Besu), it expires and has to be migrated even if the alarm was false.
- Unpausing does not undo anything the client stored before the pause. If a fraudulent consensus state may have been stored, migrate the client instead of unpausing it.

## Implementation Notes

This ADR does not change any code. Implementation follows in separate pull requests:

- **IFT**: add the three hooks to `IFTBaseUpgradeable`, next to the existing `_consumeIFTRateLimit` calls. Add `PausableUpgradeable` and `IPausable` to `IFTOwnable` and `IFTAccessManaged`, and override the hooks.
- **Light clients**: add `Pausable`, `PAUSER_ROLE` and `UNPAUSER_ROLE` to `SP1ICS07Tendermint`, `BesuLightClientBase` and `AttestationLightClient`, and gate `updateClient`, `verifyMembership` and `verifyNonMembership` with `whenNotPaused`.
- **Deployment**: wire `IFTAccessManaged` deployments to `PAUSER_ROLE` and `UNPAUSER_ROLE` using the existing `IBCRolesLib` selectors.
- **Tooling**: run `just solidity::generate-abi` after the interface changes, and add tests for each flow while paused, including retrying acknowledgements and timeouts after unpausing.
- **Docs**: link this ADR from [`ibc-solidity/contracts/README.md`](../../../ibc-solidity/contracts/README.md).

Unrelated inconsistencies found while writing this ADR:

- The `ICS02ClientUpgradeable` contract comment still describes a migrator role granted to the caller of `addClient`. In fact, `migrateClient` is restricted by the AccessManager.
- A zero limit means unlimited in the ICS20 escrow rate limiter but blocks every transfer in the IFT rate limiter.
