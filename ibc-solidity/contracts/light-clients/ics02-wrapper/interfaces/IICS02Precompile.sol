// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../../../msgs/IICS02ClientMsgs.sol";

/// @dev The ICS02I contract's address.
// natlint-disable-next-line
address constant ICS02_PRECOMPILE_ADDRESS = 0x0000000000000000000000000000000000000807;

/// @dev The ICS02 contract's instance.
/// @dev The ICS02 Precompile contract is deployed at this fixed address in 'cosmos/evm'
// natlint-disable-next-line
IICS02Precompile constant ICS02_CONTRACT = IICS02Precompile(ICS02_PRECOMPILE_ADDRESS);

/// @author CosmosLabs
/// @title ICS02 Client Router Precompile Interface
/// @notice The interface through which solidity contracts will interact with IBC Light Clients (ICS02)
/// @dev This interface must match the interface in the cosmos/evm repository.
/// @custom:source <https://github.com/cosmos/evm/blob/main/precompiles/ics02/ICS02I.sol>
/// @custom:address 0x0000000000000000000000000000000000000807
interface IICS02Precompile {
    /// @notice The result of an update operation
    enum UpdateResult {
        /// The update was successful
        Update,
        /// A misbehaviour was detected
        Misbehaviour
    }

    /// @notice Updates the client with the given client identifier.
    /// @param clientId The client identifier
    /// @param updateMsg The encoded update message e.g., a protobuf any.
    /// @return The result of the update operation
    function updateClient(string calldata clientId, bytes calldata updateMsg) external returns (UpdateResult);

    /// @notice Querying the membership of a key-value pair
    /// @dev Notice that this message is not view, as it may update the client state for caching purposes.
    /// @param clientId The client identifier
    /// @param proof The proof of membership
    /// @param proofHeight The height of the proof
    /// @param path The path of the value in the Merkle tree
    /// @param value The value in the Merkle tree
    /// @return The unix timestamp of the verification height in the counterparty chain in seconds.
    function verifyMembership(
        string calldata clientId,
        bytes calldata proof,
        IICS02ClientMsgs.Height calldata proofHeight,
        bytes[] calldata path,
        bytes calldata value
    )
        external
        returns (uint256);

    /// @notice Querying the non-membership of a key
    /// @dev Notice that this message is not view, as it may update the client state for caching purposes.
    /// @param clientId The client identifier
    /// @param proof The proof of membership
    /// @param proofHeight The height of the proof
    /// @param path The path of the value in the Merkle tree
    /// @return The unix timestamp of the verification height in the counterparty chain in seconds.
    function verifyNonMembership(
        string calldata clientId,
        bytes calldata proof,
        IICS02ClientMsgs.Height calldata proofHeight,
        bytes[] calldata path
    )
        external
        returns (uint256);

    /// @notice Returns the client state.
    /// @param clientId The client identifier
    /// @return The client state.
    function getClientState(string calldata clientId) external view returns (bytes memory);
}
