// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IPausable } from "../interfaces/IPausable.sol";
import { IFTBaseUpgradeable } from "./IFTBaseUpgradeable.sol";
import {
    ERC20BurnableUpgradeable
} from "@openzeppelin-upgradeable/token/ERC20/extensions/ERC20BurnableUpgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { PausableUpgradeable } from "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

/// @title IFT Pausable Upgradeable
/// @notice Shared base of the reference IFT implementations: a burnable, pausable and UUPS upgradeable IFT whose
/// issuer mint, pause, unpause and upgrades are gated by `_onlyAuthority`
/// @dev The authority can pause and unpause every bridge flow: transfers, mints and refunds. Admin functions, the
/// issuer's `mint` and local token transfers stay available while paused.
/// @dev Inheriters bind `_onlyAuthority` to their access control and initialize it together with `__Pausable_init` and
/// `__IFTBase_init`
abstract contract IFTPausableUpgradeable is
    IPausable,
    IFTBaseUpgradeable,
    ERC20BurnableUpgradeable,
    UUPSUpgradeable,
    PausableUpgradeable
{
    /// @notice Mints tokens to an account
    /// @dev Only callable by the authority
    /// @param mintAddress Address to mint tokens to
    /// @param amount Amount of tokens to mint
    // natlint-disable-next-line MissingInheritdoc
    function mint(address mintAddress, uint256 amount) external {
        _onlyAuthority();
        _mint(mintAddress, amount);
    }

    /// @inheritdoc IPausable
    function pause() external {
        _onlyAuthority();
        _pause();
    }

    /// @inheritdoc IPausable
    function unpause() external {
        _onlyAuthority();
        _unpause();
    }

    /* solhint-disable no-empty-blocks */
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

    /* solhint-enable no-empty-blocks */

    /// @inheritdoc UUPSUpgradeable
    function _authorizeUpgrade(address) internal override(UUPSUpgradeable) {
        _onlyAuthority();
    }
}
