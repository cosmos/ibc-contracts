// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,gas-small-strings

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";
import {MembershipTest} from "./MembershipTest.sol";
import {ILightClient} from "../../contracts/interfaces/ILightClient.sol";

contract SP1ICS07LargeMembershipTest is MembershipTest {
    SP1MembershipProof public proof;

    function setUpLargeMembershipTestWithFixture(string memory fileName) public {
        setUpTestWithFixtures(fileName);

        proof = abi.decode(fixture.membershipProof.proof, (SP1MembershipProof));
    }

    function getOutput() public view returns (MembershipOutput memory) {
        return abi.decode(proof.sp1Proof.publicValues, (MembershipOutput));
    }

    function test_ValidLargeCachedVerifyMembership_25_plonk() public {
        ValidCachedMulticallMembershipTest("membership_25-plonk_fixture.json", 25, "25 key-value pairs with plonk");
    }

    function test_ValidLargeCachedVerifyMembership_100_groth16() public {
        ValidCachedMulticallMembershipTest(
            "membership_100-groth16_fixture.json", 100, "100 key-value pairs with groth16"
        );
    }

    function ValidCachedMulticallMembershipTest(string memory fileName, uint32 n, string memory metadata) public {
        require(n > 0, "n must be greater than 0");

        setUpLargeMembershipTestWithFixture(fileName);

        bytes[] memory multicallData = new bytes[](n);

        for (uint32 i = 0; i < n; ++i) {
            bytes memory proofBz = bytes("");
            if (i == 0) {
                proofBz = abi.encode(fixture.membershipProof);
            }

            bytes memory value = getOutput().kvPairs[i].value;
            if (value.length > 0) {
                multicallData[i] = abi.encodeCall(
                    ILightClient.verifyMembership,
                    MsgVerifyMembership({
                        proof: proofBz, // cached kv pairs
                        proofHeight: fixture.proofHeight,
                        path: getOutput().kvPairs[i].path,
                        value: value
                    })
                );
            } else {
                multicallData[i] = abi.encodeCall(
                    ILightClient.verifyNonMembership,
                    MsgVerifyNonMembership({
                        proof: proofBz, // cached kv pairs
                        proofHeight: fixture.proofHeight,
                        path: getOutput().kvPairs[i].path
                    })
                );
            }
        }

        ics07Tendermint.multicall(multicallData);
        console.log("Proved", metadata, ", gas used: ", vm.lastCallGas().gasTotalUsed);
    }
}
