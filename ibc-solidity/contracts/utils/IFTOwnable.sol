// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IFTBaseUpgradeable } from "./IFTBaseUpgradeable.sol";
import { IFTPausableUpgradeable } from "./IFTPausableUpgradeable.sol";
import { OwnableUpgradeable } from "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";

/// @title IFT Ownable
/// @notice This is the ownable and upgradable implementation of IFT
/// @dev The owner is the authority: it mints, pauses, unpauses, upgrades and administers the bridges.
/// @dev If you need a custom IFT implementation, then inherit from IFTBaseUpgradeable instead of deploying this
/// contract directly
contract IFTOwnable is IFTPausableUpgradeable, OwnableUpgradeable {
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @notice Initializes the IFTOwnable contract
    /// @param owner_ The owner of the contract
    /// @param erc20Name The name of the ERC20 token
    /// @param erc20Symbol The symbol of the ERC20 token
    /// @param ics27Gmp The address of the ICS27-GMP contract
    // natlint-disable-next-line MissingInheritdoc
    function initialize(
        address owner_,
        string calldata erc20Name,
        string calldata erc20Symbol,
        address ics27Gmp
    )
        external
        initializer
    {
        __Ownable_init(owner_);
        __Pausable_init();
        __IFTBase_init(erc20Name, erc20Symbol, ics27Gmp);
    }

    /// @inheritdoc IFTBaseUpgradeable
    function _onlyAuthority() internal view override(IFTBaseUpgradeable) onlyOwner { }
    // solhint-disable-previous-line no-empty-blocks
}
