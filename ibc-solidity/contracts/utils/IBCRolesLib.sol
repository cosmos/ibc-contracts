// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterAccessControlled } from "../interfaces/IICS26Router.sol";
import { IICS02ClientAccessControlled } from "../interfaces/IICS02Client.sol";
import { IICS20TransferAccessControlled } from "../interfaces/IICS20Transfer.sol";
import { IICS27GMP } from "../interfaces/IICS27GMP.sol";
import { IPausable } from "../interfaces/IPausable.sol";
import { IRateLimit } from "../interfaces/IRateLimit.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";

/// @title IBCRolesLib
/// @notice This library is used to define the shared roles for IBC contracts.
library IBCRolesLib {
    /// @notice The admin role as per defined by AccessManager.
    uint64 internal constant ADMIN_ROLE = type(uint64).min;

    /// @notice The relayer role as per defined by AccessManager.
    uint64 internal constant PUBLIC_ROLE = type(uint64).max;

    /// @notice Only addresses with this role may relay packets.
    uint64 internal constant RELAYER_ROLE = 1;

    /// @notice The pauser role can pause IBC applications.
    uint64 internal constant PAUSER_ROLE = 2;

    /// @notice The unpauser role can unpause IBC applications.
    uint64 internal constant UNPAUSER_ROLE = 3;

    /// @notice Has permission to call `ICS20Transfer.sendTransferWithSender`.
    uint64 internal constant DELEGATE_SENDER_ROLE = 4;

    /// @notice Can set withdrawal rate limits per ERC20 token.
    uint64 internal constant RATE_LIMITER_ROLE = 5;

    /// @notice Can set custom port ids and client ids in ICS26Router.
    uint64 internal constant ID_CUSTOMIZER_ROLE = 6;

    /// @notice Can set custom ERC20 contracts for IBC denoms in ICS20Transfer.
    uint64 internal constant ERC20_CUSTOMIZER_ROLE = 7;

    /// @notice The functions that can be called by the RELAYER_ROLE in ICS26Router.
    /// @return An array of function selectors that can be called by the RELAYER_ROLE.
    function ics26RelayerSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory relayerFunctions = new bytes4[](4);
        relayerFunctions[0] = IICS26RouterAccessControlled.recvPacket.selector;
        relayerFunctions[1] = IICS26RouterAccessControlled.timeoutPacket.selector;
        relayerFunctions[2] = IICS26RouterAccessControlled.ackPacket.selector;
        relayerFunctions[3] = IICS02ClientAccessControlled.updateClient.selector;
        return relayerFunctions;
    }

    /// @notice The functions that can be called by the ID_CUSTOMIZER_ROLE in ICS26Router.
    /// @return An array of function selectors that can be called by the ID_CUSTOMIZER_ROLE.
    function ics26IdCustomizerSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory idCustomizerFunctions = new bytes4[](2);
        idCustomizerFunctions[0] = IICS26RouterAccessControlled.addIBCApp.selector;
        idCustomizerFunctions[1] = IICS02ClientAccessControlled.addClient.selector;
        return idCustomizerFunctions;
    }

    /// @notice The functions that can be used to migrate clients in ICS26Router.
    /// @dev These functions are not associated with a specific role, but are restricted to the ADMIN_ROLE.
    /// @return An array of function selectors that can be used to migrate clients.
    function ics26ClientMigrationSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory migrationFunctions = new bytes4[](1);
        migrationFunctions[0] = IICS02ClientAccessControlled.migrateClient.selector;
        return migrationFunctions;
    }

    /// @notice The functions that can be called by the ERC20_CUSTOMIZER_ROLE in ICS20Transfer.
    /// @return An array of function selectors that can be called by the ERC20_CUSTOMIZER_ROLE.
    function erc20CustomizerSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory erc20CustomizerFunctions = new bytes4[](1);
        erc20CustomizerFunctions[0] = IICS20TransferAccessControlled.setCustomERC20.selector;
        return erc20CustomizerFunctions;
    }

    /// @notice The functions that can be called by the DELEGATE_SENDER_ROLE in ICS20Transfer.
    /// @return An array of function selectors that can be called by the DELEGATE_SENDER_ROLE.
    function delegateSenderSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory delegateSenderFunctions = new bytes4[](1);
        delegateSenderFunctions[0] = IICS20TransferAccessControlled.sendTransferWithSender.selector;
        return delegateSenderFunctions;
    }

    /// @notice The functions that can be called by the PAUSER_ROLE.
    /// @return An array of function selectors that can be called by the PAUSER_ROLE.
    function pauserSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory pauserFunctions = new bytes4[](1);
        pauserFunctions[0] = IPausable.pause.selector;
        return pauserFunctions;
    }

    /// @notice The functions that can be called by the UNPAUSER_ROLE.
    /// @return An array of function selectors that can be called by the UNPAUSER_ROLE.
    function unpauserSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory unpauserFunctions = new bytes4[](1);
        unpauserFunctions[0] = IPausable.unpause.selector;
        return unpauserFunctions;
    }

    /// @notice The functions that can be called by the RATE_LIMITER_ROLE in Escrow.
    /// @return An array of function selectors that can be called by the RATE_LIMITER_ROLE.
    function rateLimiterSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory rateLimiterFunctions = new bytes4[](1);
        rateLimiterFunctions[0] = IRateLimit.setRateLimit.selector;
        return rateLimiterFunctions;
    }

    /// @notice The functions that can be used to upgrade the beacon contracts in ICS20Transfer.
    /// @dev These functions are not associated with a specific role, but are restricted to the ADMIN_ROLE.
    /// @return An array of function selectors that can be used to upgrade the beacon contracts.
    function ics20BeaconUpgradeSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory beaconUpgradeFunctions = new bytes4[](2);
        beaconUpgradeFunctions[0] = IICS20TransferAccessControlled.upgradeEscrowTo.selector;
        beaconUpgradeFunctions[1] = IICS20TransferAccessControlled.upgradeIBCERC20To.selector;
        return beaconUpgradeFunctions;
    }

    /// @notice The functions that can be used to upgrade the beacon contracts in ICS27GMP.
    /// @dev These functions are not associated with a specific role, but are restricted to the ADMIN_ROLE.
    /// @return An array of function selectors that can be used to upgrade the beacon contracts.
    function ics27BeaconUpgradeSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory beaconUpgradeFunctions = new bytes4[](1);
        beaconUpgradeFunctions[0] = IICS27GMP.upgradeAccountTo.selector;
        return beaconUpgradeFunctions;
    }

    /// @notice The functions that can be used to upgrade UUPS contracts such as ICS26Router and ICS20Transfer.
    /// @dev These functions are not associated with a specific role, but are restricted to the ADMIN_ROLE.
    /// @return An array of function selectors that can be used to upgrade UUPS contracts.
    function uupsUpgradeSelectors() internal pure returns (bytes4[] memory) {
        bytes4[] memory uupsUpgradeFunctions = new bytes4[](1);
        uupsUpgradeFunctions[0] = UUPSUpgradeable.upgradeToAndCall.selector;
        return uupsUpgradeFunctions;
    }
}
