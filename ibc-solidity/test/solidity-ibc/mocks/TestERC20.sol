// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks,gas-custom-errors

import { ERC20 } from "@openzeppelin-contracts/token/ERC20/ERC20.sol";

contract TestERC20 is ERC20 {
    constructor() ERC20("Test ERC20", "TERC") { }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract MalfunctioningERC20 is TestERC20 {
    bool public malfunction = false;

    function setMalfunction(bool _malfunction) external {
        malfunction = _malfunction;
    }

    // _update is doing nothing so that a transfer seems to have gone through,
    // but the internal state of the ERC20 contract is not updated - i.e. no transfer really happened
    function _update(address from, address to, uint256 value) internal virtual override {
        if (malfunction) {
            // Do nothing 😱
            return;
        }

        super._update(from, to, value);
    }
}

// Test contract to deploy ERC20 with different decimals value
contract TestERC20Metadata is ERC20 {
    uint8 private _decimals;

    constructor(uint8 decimals_) ERC20("MetadataToken", "MTK") {
        _decimals = decimals_;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }
}

// Test contract to deploy ERC20 with fee on transfer, it takes a fee of 1 unit on every transfer and burns it, so the
// recipient receives amount - 1
contract FeeOnTransferERC20 is ERC20 {
    constructor() ERC20("Fee Token", "FEE") { }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function _update(address from, address to, uint256 amount) internal override {
        if (from != address(0) && to != address(0)) {
            uint256 fee = 1;
            super._update(from, address(0), fee); // burn fee
            super._update(from, to, amount - fee); // deliver remainder
        } else {
            super._update(from, to, amount);
        }
    }
}
