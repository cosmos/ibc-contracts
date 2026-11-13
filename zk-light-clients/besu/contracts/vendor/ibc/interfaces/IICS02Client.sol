// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../msgs/IICS02ClientMsgs.sol";
import { ILightClientMsgs } from "../msgs/ILightClientMsgs.sol";
import { ILightClient } from "./ILightClient.sol";

/// @title ICS02 Client Access Controlled Interface
/// @notice Interface for the access controlled functions of the IBC Eureka light client router
interface IICS02ClientAccessControlled {
    /// @notice Adds a client to the client router.
    /// @dev Only a caller with `CLIENT_ID_CUSTOMIZER_ROLE` can call this function.
    /// @param clientId The custom client identifier
    /// @param counterpartyInfo The counterparty client information
    /// @param client The address of the client contract
    /// @return The client identifier
    function addClient(
        string calldata clientId,
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        external
        returns (string memory);

    /// @notice Updates the client with the given client identifier.
    /// @dev Can only be called with the `RELAYER_ROLE`.
    /// @param clientId The client identifier
    /// @param updateMsg The encoded update message e.g., an SP1 proof.
    /// @return The result of the update operation
    function updateClient(
        string calldata clientId,
        bytes calldata updateMsg
    )
        external
        returns (ILightClientMsgs.UpdateResult);

    /// @notice Migrate a client by replacing the existing counterparty information and contract address.
    /// @dev This is a privilaged operation, only one with `getLightClientMigratorRole(clientId)` can call this.
    /// @param clientId The client identifier of the client to migrate
    /// @param counterpartyInfo The new counterparty client information
    /// @param client The address of the new client contract
    function migrateClient(
        string calldata clientId,
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        external;
}

/// @title ICS02 Light Client Router Interface
/// @notice Interface for the IBC Eureka light client router
interface IICS02Client is IICS02ClientAccessControlled {
    /// @notice Returns the counterparty client information given the client identifier.
    /// @param clientId The client identifier
    /// @return The counterparty client information
    function getCounterparty(string calldata clientId) external view returns (IICS02ClientMsgs.CounterpartyInfo memory);

    /// @notice Returns the address of the client contract given the client identifier.
    /// @param clientId The client identifier
    /// @return The address of the client contract
    function getClient(string calldata clientId) external view returns (ILightClient);

    /// @notice Returns the next client sequence number.
    /// @dev This function can be used to determine when to stop iterating over clients.
    /// @return The next client sequence number
    function getNextClientSeq() external view returns (uint256);

    /// @notice Adds a client to the client router.
    /// @param counterpartyInfo The counterparty client information
    /// @param client The address of the client contract
    /// @return The client identifier
    function addClient(
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        external
        returns (string memory);

    /// @notice Submits misbehaviour to the client with the given client identifier.
    /// @param clientId The client identifier
    /// @param misbehaviourMsg The misbehaviour message
    function submitMisbehaviour(string calldata clientId, bytes calldata misbehaviourMsg) external;

    // ============ Events ============

    /// @notice Emitted when a new client is added to the client router.
    /// @param clientId The newly created client identifier
    /// @param counterpartyInfo The counterparty client information
    /// @param client The address of the client contract
    event ICS02ClientAdded(string clientId, IICS02ClientMsgs.CounterpartyInfo counterpartyInfo, address client);

    /// @notice Emitted when a client is migrated to a new client.
    /// @param clientId The client identifier of the migrated client
    /// @param counterpartyInfo The new counterparty client information
    /// @param client The address of the new client contract
    event ICS02ClientMigrated(string clientId, IICS02ClientMsgs.CounterpartyInfo counterpartyInfo, address client);

    /// @notice Emitted when a client is updated.
    /// @param clientId The client identifier of the updated ILightClientMsgs
    /// @param result The result of the update operation
    event ICS02ClientUpdated(string clientId, ILightClientMsgs.UpdateResult result);

    /// @notice Emitted when a misbehaviour is submitted to a client and the client is frozen.
    /// @param clientId The client identifier of the frozen client
    event ICS02MisbehaviourSubmitted(string clientId);
}
