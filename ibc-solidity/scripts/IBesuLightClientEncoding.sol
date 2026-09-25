// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IBesuLightClientMsgs } from "../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";

/// @title Besu Light Client Encoding Schema
/// @notice Generation-only interface exposing the types hidden inside the light client's bytes payloads.
/// @dev Never deployed or implemented by the light client. Pack function inputs without a selector to obtain
/// abi.encode. Matching outputs generate typed decoders for the same payloads.
interface IBesuLightClientEncoding {
    /// @notice Exposes the client state encoding.
    /// @param state The client state.
    function clientState(IBesuLightClientMsgs.ClientState calldata state)
        external
        pure
        returns (IBesuLightClientMsgs.ClientState memory);

    /// @notice Exposes the consensus state preimage encoding.
    /// @param state The consensus state.
    function consensusState(IBesuLightClientMsgs.ConsensusState calldata state)
        external
        pure
        returns (IBesuLightClientMsgs.ConsensusState memory);

    /// @notice Exposes the update payload encoding.
    /// @param message The update message.
    function updateClient(IBesuLightClientMsgs.MsgUpdateClient calldata message)
        external
        pure
        returns (IBesuLightClientMsgs.MsgUpdateClient memory);

    /// @notice Exposes the membership and non-membership proof encoding.
    /// @param proof The proof with its consensus state preimage.
    function membershipProof(IBesuLightClientMsgs.MembershipProof calldata proof)
        external
        pure
        returns (IBesuLightClientMsgs.MembershipProof memory);

    /// @notice Exposes the nested account proof encoding.
    /// @param nodes The ordered RLP trie nodes.
    function proofNodes(bytes[] calldata nodes) external pure returns (bytes[] memory);
}
