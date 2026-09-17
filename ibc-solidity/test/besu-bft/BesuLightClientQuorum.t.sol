// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { BesuLightClientBase } from "../../contracts/light-clients/besu/BesuLightClientBase.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";

struct BesuConstructorTestCase {
    string name;
    uint64 timestamp;
    uint64 trustingPeriod;
    address[] validators;
    bytes expectedRevert;
}

contract BesuLightClientQuorumHarness is BesuLightClientBase {
    constructor(
        uint64 initialTrustedTimestamp,
        uint64 trustingPeriod,
        address[] memory initialValidators
    )
        BesuLightClientBase(
            address(1), 1, initialTrustedTimestamp, bytes32(0), initialValidators, trustingPeriod, 0, address(0)
        )
    { }

    function checkValidatorQuorum(address[] calldata signers, address[] calldata validators) external pure {
        _checkValidatorQuorum(signers, validators);
    }

    function checkTrustedValidatorOverlap(
        address[] calldata signers,
        address[] calldata trustedValidators
    )
        external
        pure
    {
        _checkTrustedValidatorOverlap(signers, trustedValidators);
    }

    function _commitSealDigest(ParsedHeader memory) internal pure override returns (bytes32) {
        return bytes32(0);
    }
}

contract BesuLightClientQuorumTest is Test {
    BesuLightClientQuorumHarness private harness;

    function setUp() public {
        harness = new BesuLightClientQuorumHarness(1, 1, _addresses(1));
    }

    function tableConstructorTest(BesuConstructorTestCase memory deployment) public {
        if (deployment.expectedRevert.length != 0) {
            vm.expectRevert(deployment.expectedRevert);
        }
        new BesuLightClientQuorumHarness(deployment.timestamp, deployment.trustingPeriod, deployment.validators);
    }

    function fixtureDeployment() public pure returns (BesuConstructorTestCase[] memory testCases) {
        testCases = new BesuConstructorTestCase[](8);
        testCases[0] = BesuConstructorTestCase("success: single validator", 1, 1, _addresses(1), "");
        testCases[1] = BesuConstructorTestCase("success: sorted validators", 1, 1, _addresses(4), "");
        testCases[2] = BesuConstructorTestCase(
            "failure: zero timestamp",
            0,
            1,
            _addresses(1),
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderTimestamp.selector)
        );
        testCases[3] = BesuConstructorTestCase(
            "failure: zero trusting period",
            1,
            0,
            _addresses(1),
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidTrustingPeriod.selector)
        );
        testCases[4] = BesuConstructorTestCase(
            "failure: empty validators",
            1,
            1,
            _addresses(0),
            abi.encodeWithSelector(IBesuLightClientErrors.EmptyValidatorSet.selector)
        );

        address[] memory validators = _addresses(2);
        validators[0] = address(0);
        testCases[5] = BesuConstructorTestCase(
            "failure: zero validator",
            1,
            1,
            validators,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidValidatorAddress.selector, address(0))
        );

        validators = _addresses(3);
        (validators[1], validators[2]) = (validators[2], validators[1]);
        testCases[6] = BesuConstructorTestCase(
            "failure: descending validators",
            1,
            1,
            validators,
            abi.encodeWithSelector(IBesuLightClientErrors.UnsortedValidatorSet.selector, 1)
        );

        validators = _addresses(3);
        validators[2] = validators[1];
        testCases[7] = BesuConstructorTestCase(
            "failure: duplicate validators",
            1,
            1,
            validators,
            abi.encodeWithSelector(IBesuLightClientErrors.UnsortedValidatorSet.selector, 1)
        );
    }

    function test_checkValidatorQuorum_acceptsBesuFourOfSixQuorum() public view {
        harness.checkValidatorQuorum(_addresses(4), _addresses(6));
    }

    function test_checkValidatorQuorum_rejectsThreeOfSixValidators() public {
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 3, 4));
        harness.checkValidatorQuorum(_addresses(3), _addresses(6));
    }

    function test_checkValidatorQuorum_acceptsSingleValidator() public view {
        harness.checkValidatorQuorum(_addresses(1), _addresses(1));
    }

    function test_checkTrustedValidatorOverlap_acceptsThreeOfFour() public view {
        harness.checkTrustedValidatorOverlap(_addresses(3), _addresses(4));
    }

    function test_checkTrustedValidatorOverlap_rejectsTwoOfFour() public {
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 2, 3)
        );
        harness.checkTrustedValidatorOverlap(_addresses(2), _addresses(4));
    }

    function test_checkTrustedValidatorOverlap_acceptsFourOfSix() public view {
        harness.checkTrustedValidatorOverlap(_addresses(4), _addresses(6));
    }

    function test_checkTrustedValidatorOverlap_rejectsThreeOfSix() public {
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 3, 4)
        );
        harness.checkTrustedValidatorOverlap(_addresses(3), _addresses(6));
    }

    function test_checkTrustedValidatorOverlap_acceptsSingleValidator() public view {
        harness.checkTrustedValidatorOverlap(_addresses(1), _addresses(1));
    }

    function _addresses(uint256 length) private pure returns (address[] memory addresses) {
        addresses = new address[](length);
        for (uint256 i = 0; i < length; ++i) {
            addresses[i] = address(uint160(i + 1));
        }
    }
}
