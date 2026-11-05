// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IFTBaseUpgradeable } from "./IFTBaseUpgradeable.sol";
import { IFTPausableUpgradeable } from "./IFTPausableUpgradeable.sol";
import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";

/// @title IFT Access Managed
/// @notice This is the access managed and upgradable implementation of IFT
/// @dev The authority gates every authority function by its selector. Map `pause` and `unpause` to separate roles,
/// see IBCRolesLib.
/// @dev If you need a custom IFT implementation, then inherit from IFTBaseUpgradeable instead of deploying this
/// contract directly
contract IFTAccessManaged is IFTPausableUpgradeable, AccessManagedUpgradeable {
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @notice Initializes the IFTAccessManaged contract
    /// @param authority_ The address of the AccessManager contract
    /// @param erc20Name The name of the ERC20 token
    /// @param erc20Symbol The symbol of the ERC20 token
    /// @param ics27Gmp The address of the ICS27-GMP contract
    // natlint-disable-next-line MissingInheritdoc
    function initialize(
        address authority_,
        string calldata erc20Name,
        string calldata erc20Symbol,
        address ics27Gmp
    )
        external
        initializer
    {
        __AccessManaged_init(authority_);
        __Pausable_init();
        __IFTBase_init(erc20Name, erc20Symbol, ics27Gmp);
    }

    /// @inheritdoc IFTBaseUpgradeable
    /// @dev `restricted` checks the selector of the external call, so each authority function keeps its own role.
    function _onlyAuthority() internal override(IFTBaseUpgradeable) restricted { }
    // solhint-disable-previous-line no-empty-blocks
}
