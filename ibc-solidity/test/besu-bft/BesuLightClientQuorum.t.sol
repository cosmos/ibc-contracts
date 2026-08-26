// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { BesuLightClientBase } from "../../contracts/light-clients/besu/BesuLightClientBase.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";

contract BesuLightClientQuorumHarness is BesuLightClientBase {
    constructor(
        uint64 initialTrustedTimestamp,
        address[] memory initialValidators
    )
        BesuLightClientBase(address(1), 1, initialTrustedTimestamp, bytes32(0), initialValidators, 0, 0, address(0))
    { }

    function checkValidatorQuorum(address[] calldata signers, address[] calldata validators) external pure {
        _checkValidatorQuorum(signers, validators);
    }

    function _commitSealDigest(ParsedHeader memory) internal pure override returns (bytes32) {
        return bytes32(0);
    }
}

contract BesuLightClientQuorumTest is Test {
    BesuLightClientQuorumHarness private harness;

    function setUp() public {
        harness = new BesuLightClientQuorumHarness(1, _addresses(1));
    }

    function test_constructor_rejectsZeroTrustedTimestamp() public {
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderTimestamp.selector, 0));
        new BesuLightClientQuorumHarness(0, _addresses(1));
    }

    function test_checkValidatorQuorum_acceptsBesuFourOfSixQuorum() public view {
        harness.checkValidatorQuorum(_addresses(4), _addresses(6));
    }

    function test_checkValidatorQuorum_rejectsThreeOfSixValidators() public {
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 3, 4));
        harness.checkValidatorQuorum(_addresses(3), _addresses(6));
    }

    function _addresses(uint256 length) private pure returns (address[] memory addresses) {
        addresses = new address[](length);
        for (uint256 i = 0; i < length; ++i) {
            addresses[i] = address(uint160(i + 1));
        }
    }
}
