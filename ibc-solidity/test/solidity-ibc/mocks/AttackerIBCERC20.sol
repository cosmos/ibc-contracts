// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks,gas-custom-errors

import { IIBCERC20 } from "../../../contracts/interfaces/IIBCERC20.sol";
import { IMintableAndBurnable } from "../../../contracts/interfaces/IMintableAndBurnable.sol";
import { ERC20 } from "@openzeppelin-contracts/token/ERC20/ERC20.sol";

contract AttackerIBCERC20 is IIBCERC20, ERC20 {
    address private escrowAddress;

    constructor(string memory fullDenomPath_, address escrowAddress_) ERC20(fullDenomPath_, fullDenomPath_) {
        escrowAddress = escrowAddress_;
    }

    /// @inheritdoc IIBCERC20
    function initialize(address, address, string calldata) external { }

    /// @inheritdoc IIBCERC20
    function fullDenomPath() public pure returns (string memory) {
        return "transfer/client-0/uatom";
    }

    /// @inheritdoc IMintableAndBurnable
    function mint(address, uint256) external { }

    function mintTo(address to, uint256 amount) external {
        _mint(to, amount);
    }

    /// @inheritdoc IMintableAndBurnable
    function burn(address, uint256) external { }

    /// @inheritdoc IIBCERC20
    function escrow() external view returns (address) {
        return escrowAddress;
    }

    /// @inheritdoc IIBCERC20
    function ics20() external pure returns (address) {
        return address(0);
    }
}
