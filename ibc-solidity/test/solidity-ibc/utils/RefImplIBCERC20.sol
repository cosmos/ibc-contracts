// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IMintableAndBurnable } from "../../../contracts/interfaces/IMintableAndBurnable.sol";

import { ERC20Upgradeable } from "@openzeppelin-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { OwnableUpgradeable } from "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";

/// @title Reference IBC ERC20 Implementation
/// @notice This implementation is intended to serve as a base reference for developers creating their own
/// IBC-compatible upgradeable ERC20 tokens.
contract RefImplIBCERC20 is IMintableAndBurnable, UUPSUpgradeable, ERC20Upgradeable, OwnableUpgradeable {
    /// @notice Caller is not the ICS20 contract
    /// @param caller The address of the caller
    error CallerIsNotICS20(address caller);

    /// @notice Storage of the RefIBCERC20 contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _ics20 The ICS20 contract address, can burn and mint tokens
    struct RefIBCERC20Storage {
        address _ics20;
    }

    /// @notice ERC-7201 slot for the RefIBCERC20 storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.RefIBCERC20")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant REFIBCERC20_STORAGE_SLOT =
        0x7f1f4ef08fb1ecf5e6ce5f1511ee420a1716a929ca6536d77be0398bd880e400;

    /// @dev This contract is meant to be deployed by a proxy, so the constructor is not used
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @notice Initializes the RefIBCERC20 contract
    /// @param owner_ The owner of the contract, allowing it to be upgraded
    /// @param ics20_ The ICS20 contract address
    /// @param name_ The name of the token
    /// @param symbol_ The symbol of the token
    function initialize(
        address owner_,
        address ics20_,
        string calldata name_,
        string calldata symbol_
    )
        external
        initializer
    {
        __ERC20_init(name_, symbol_);
        __Ownable_init(owner_);

        RefIBCERC20Storage storage $ = _getRefIBCERC20Storage();
        $._ics20 = ics20_;
    }

    /// @notice Returns the ICS20 contract address
    /// @return The ICS20 contract address
    function ics20() external view returns (address) {
        return _getRefIBCERC20Storage()._ics20;
    }

    /**
     * @inheritdoc ERC20Upgradeable
     *
     * @dev Returns the number of decimals used to get its user representation.
     * For example, if `decimals` equals `2`, a balance of `505` tokens should
     * be displayed to a user as `5.05` (`505 / 10 ** 2`).
     *
     * Cosmos SDK tokens usually opt for a value of 6, imitating the relationship
     * between ATOM and uatom.
     *
     * NOTE: This information is only used for _display_ purposes such as by wallets:
     * it in no way affects any of the arithmetic of the contract, including
     * {IERC20-balanceOf} and {IERC20-transfer}.
     */
    function decimals() public pure override(ERC20Upgradeable) returns (uint8) {
        return 6;
    }

    /// @inheritdoc IMintableAndBurnable
    function mint(address mintAddress, uint256 amount) external onlyICS20 {
        _mint(mintAddress, amount);
    }

    /// @inheritdoc IMintableAndBurnable
    function burn(address mintAddress, uint256 amount) external onlyICS20 {
        _burn(mintAddress, amount);
    }

    /// @notice Returns the storage of the RefIBCERC20 contract
    /// @return $ The storage of the RefIBCERC20 contract
    function _getRefIBCERC20Storage() private pure returns (RefIBCERC20Storage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := REFIBCERC20_STORAGE_SLOT
        }
    }

    /// @inheritdoc UUPSUpgradeable
    function _authorizeUpgrade(address) internal view override(UUPSUpgradeable) onlyOwner { }
    // solhint-disable-previous-line no-empty-blocks

    /// @notice Modifier to check if the caller is the ICS20 contract
    modifier onlyICS20() {
        require(_msgSender() == _getRefIBCERC20Storage()._ics20, CallerIsNotICS20(_msgSender()));
        _;
    }
}
