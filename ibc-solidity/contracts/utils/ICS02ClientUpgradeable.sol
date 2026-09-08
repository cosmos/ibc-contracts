// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../msgs/IICS02ClientMsgs.sol";
import { ILightClientMsgs } from "../msgs/ILightClientMsgs.sol";

import { IICS02ClientErrors } from "../errors/IICS02ClientErrors.sol";
import { IICS02Client, IICS02ClientAccessControlled } from "../interfaces/IICS02Client.sol";
import { ILightClient } from "../interfaces/ILightClient.sol";

import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";
import { IBCIdentifiers } from "../utils/IBCIdentifiers.sol";

/// @title ICS02 Client Router
/// @notice This is the ICS02 Light Client Router contract, storing the light clients and their identifiers.
/// @dev Light client migrations/upgrades are supported via `AccessControl` role-based access control
/// @dev Each client is identified by a unique identifier, hash of which also serves as the role identifier
/// @dev The light client migrator role is granted to whoever called `addClient` for the client, and can be revoked (not
/// transferred)
abstract contract ICS02ClientUpgradeable is IICS02Client, IICS02ClientErrors, AccessManagedUpgradeable {
    /// @notice Storage of the ICS02Client contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the
    /// @dev risk of storage collisions when using with upgradeable contracts.
    /// @param clients Mapping of client identifiers to light client contracts
    /// @param counterpartyInfos Mapping of client identifiers to counterparty info
    /// @param nextClientSeq The next sequence number for the next client identifier
    /// @custom:storage-location erc7201:ibc.storage.ICS02Client
    struct ICS02ClientStorage {
        mapping(string clientId => ILightClient) clients;
        mapping(string clientId => IICS02ClientMsgs.CounterpartyInfo info) counterpartyInfos;
        uint256 nextClientSeq;
    }

    /// @notice ERC-7201 slot for the ICS02Client storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.ICS02Client")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant ICS02CLIENT_STORAGE_SLOT =
        0x515a8336edcaab4ae6524d41223c1782132890f89189ba6632107a7b5a449600;

    /// @notice This function initializes the ICS02Client contract
    /// @dev This function is meant to be called by the initializer of the contract that inherits this.
    /// @param authority The address of the AccessManager contract
    function __ICS02Client_init(address authority) internal onlyInitializing {
        __AccessManaged_init(authority);
    }

    /// @inheritdoc IICS02Client
    function getNextClientSeq() external view returns (uint256) {
        return _getICS02ClientStorage().nextClientSeq;
    }

    /// @notice Generates the next client identifier
    /// @return The next client identifier
    function nextClientId() private returns (string memory) {
        ICS02ClientStorage storage $ = _getICS02ClientStorage();
        // initial client sequence should be 0, hence we use x++ instead of ++x
        // solhint-disable-next-line gas-increment-by-one
        return string.concat(IBCIdentifiers.CLIENT_ID_PREFIX, Strings.toString($.nextClientSeq++));
    }

    /// @inheritdoc IICS02Client
    function getCounterparty(string calldata clientId) public view returns (IICS02ClientMsgs.CounterpartyInfo memory) {
        IICS02ClientMsgs.CounterpartyInfo memory counterpartyInfo = _getICS02ClientStorage().counterpartyInfos[clientId];
        require(bytes(counterpartyInfo.clientId).length != 0, IBCCounterpartyClientNotFound(clientId));

        return counterpartyInfo;
    }

    /// @inheritdoc IICS02Client
    function getClient(string calldata clientId) public view returns (ILightClient) {
        ILightClient client = _getICS02ClientStorage().clients[clientId];
        require(address(client) != address(0), IBCClientNotFound(clientId));

        return client;
    }

    /// @inheritdoc IICS02Client
    function addClient(
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        external
        returns (string memory)
    {
        string memory clientId = nextClientId();
        _addClient(clientId, counterpartyInfo, client);
        return clientId;
    }

    /// @inheritdoc IICS02ClientAccessControlled
    function addClient(
        string calldata clientId,
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        external
        restricted
        returns (string memory)
    {
        require(bytes(clientId).length != 0, IBCInvalidClientId(clientId));
        require(IBCIdentifiers.validateCustomIBCIdentifier(bytes(clientId)), IBCInvalidClientId(clientId));
        _addClient(clientId, counterpartyInfo, client);
        return clientId;
    }

    /// @notice This function adds a client to the client router
    /// @dev This function assumes that the clientId has already been generated and validated.
    /// @param clientId The client identifier
    /// @param counterpartyInfo The counterparty client information
    /// @param client The address of the client contract
    function _addClient(
        string memory clientId,
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        private
    {
        ICS02ClientStorage storage $ = _getICS02ClientStorage();
        require(address($.clients[clientId]) == address(0), IBCClientAlreadyExists(clientId));

        $.clients[clientId] = ILightClient(client);
        $.counterpartyInfos[clientId] = counterpartyInfo;

        emit ICS02ClientAdded(clientId, counterpartyInfo, client);
    }

    /// @inheritdoc IICS02ClientAccessControlled
    function updateClient(
        string calldata clientId,
        bytes calldata updateMsg
    )
        external
        restricted
        returns (ILightClientMsgs.UpdateResult)
    {
        ILightClientMsgs.UpdateResult result = getClient(clientId).updateClient(updateMsg);
        emit ICS02ClientUpdated(clientId, result);
        return result;
    }

    /// @inheritdoc IICS02ClientAccessControlled
    function migrateClient(
        string calldata clientId,
        IICS02ClientMsgs.CounterpartyInfo calldata counterpartyInfo,
        address client
    )
        external
        restricted
    {
        getClient(clientId); // Ensure subject client exists

        ICS02ClientStorage storage $ = _getICS02ClientStorage();
        $.counterpartyInfos[clientId] = counterpartyInfo;
        $.clients[clientId] = ILightClient(client);

        emit ICS02ClientMigrated(clientId, counterpartyInfo, client);
    }

    /// @inheritdoc IICS02Client
    function submitMisbehaviour(string calldata clientId, bytes calldata misbehaviourMsg) external {
        getClient(clientId).misbehaviour(misbehaviourMsg);
        emit ICS02MisbehaviourSubmitted(clientId);
    }

    /// @notice Returns the storage of the ICS02Client contract
    /// @return $ The storage of the ICS02Client contract
    function _getICS02ClientStorage() private pure returns (ICS02ClientStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := ICS02CLIENT_STORAGE_SLOT
        }
    }
}
