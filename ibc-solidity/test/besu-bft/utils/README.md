# Besu BFT simulation suite

Test-only, in-process simulator of a Besu QBFT or IBFT 2.0 chain. It replaces docker-captured fixtures
when a test needs headers signed by an arbitrary validator set, exotic consensus scenarios (double signing,
clock misbehaviour, low quorum, validator churn), or real state roots and Merkle Patricia proofs for IBC
commitments that `BesuQBFTLightClient` and `BesuIBFT2LightClient` verify.

This directory was AI generated and reviewed for correctness. Its behaviour is pinned by
`SimHeader.t.sol` (byte-exact round trips of live Besu headers), `MerklePatriciaTrie.t.sol`
(OpenZeppelin trie vectors and round trips through `TrieProof`), and `../QBFTSimSuite.t.sol`
(scenarios run against the production light clients).

| File | Role |
| --- | --- |
| `QBFTSimSuite.sol` | Entry point: validator keys, block production and sealing, light client deployment and message builders. |
| `SimHeader.sol` | Besu header model with RLP encode/decode, mode-specific commit seal digest and block hash. |
| `SimWorldState.sol` | Height-versioned accounts and storage; state roots, account and storage proofs, IBC commitment slots. |
| `MerklePatriciaTrie.sol` | Trie builder computing roots and membership or exclusion proofs from a flat key/value set. |

Typical use:

```solidity
QBFTSimSuite sim = new QBFTSimSuite(SimHeader.Mode.QBFT);
sim.addValidators(4);
sim.produceBlocks(2);
IBesuLightClient client = sim.deployLightClient(1 days, 10);
sim.commitPacket(packet);
sim.produceBlock();
client.updateClient(sim.updateMsg(2, 3));
client.verifyMembership(sim.membershipMsg(3, path));
```
