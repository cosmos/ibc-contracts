// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IPausable } from "../interfaces/IPausable.sol";
import { IFTBaseUpgradeable } from "./IFTBaseUpgradeable.sol";
import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";
import {
    ERC20BurnableUpgradeable
} from "@openzeppelin-upgradeable/token/ERC20/extensions/ERC20BurnableUpgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { PausableUpgradeable } from "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

/// @title IFT Access Managed
/// @notice This is the access managed and upgradable implementation of IFT
/// @dev The authority can pause and unpause every bridge flow: transfers, mints and refunds. Map `pause` and `unpause`
/// to separate roles, see IBCRolesLib. Admin functions and local token transfers stay available while paused.
/// @dev If you need a custom IFT implementation, then inherit from IFTBaseUpgradeable instead of deploying this
/// contract directly
contract IFTAccessManaged is
    IPausable,
    IFTBaseUpgradeable,
    ERC20BurnableUpgradeable,
    AccessManagedUpgradeable,
    UUPSUpgradeable,
    PausableUpgradeable
{
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

    /// @notice Mints tokens to an account
    /// @dev Only callable by the configured authority
    /// @param mintAddress Address to mint tokens to
    /// @param amount Amount of tokens to mint
    // natlint-disable-next-line MissingInheritdoc
    function mint(address mintAddress, uint256 amount) external restricted {
        _mint(mintAddress, amount);
    }

    /// @inheritdoc IPausable
    function pause() external restricted {
        _pause();
    }

    /// @inheritdoc IPausable
    function unpause() external restricted {
        _unpause();
    }

    /// @inheritdoc IFTBaseUpgradeable
    /// @dev Outbound transfers revert while paused
    function _beforeIFTTransfer(
        address,
        string calldata,
        string calldata,
        uint256,
        uint64
    )
        internal
        virtual
        override(IFTBaseUpgradeable)
        whenNotPaused
    { }
    // solhint-disable-previous-line no-empty-blocks

    /// @inheritdoc IFTBaseUpgradeable
    /// @dev Inbound mints revert while paused, so ICS27-GMP writes an error acknowledgement
    function _beforeIFTMint(
        string memory,
        address,
        uint256
    )
        internal
        virtual
        override(IFTBaseUpgradeable)
        whenNotPaused
    { }
    // solhint-disable-previous-line no-empty-blocks

    /// @inheritdoc IFTBaseUpgradeable
    /// @dev Refunds revert while paused, so the acknowledgement or timeout can be relayed again after unpausing
    function _beforeIFTRefund(
        string memory,
        uint64,
        address,
        uint256
    )
        internal
        virtual
        override(IFTBaseUpgradeable)
        whenNotPaused
    { }
    // solhint-disable-previous-line no-empty-blocks

    /// @inheritdoc IFTBaseUpgradeable
    function _onlyAuthority() internal override(IFTBaseUpgradeable) restricted { }
    // solhint-disable-previous-line no-empty-blocks

    /// @inheritdoc UUPSUpgradeable
    function _authorizeUpgrade(address) internal override(UUPSUpgradeable) restricted { }
    // solhint-disable-previous-line no-empty-blocks
}
