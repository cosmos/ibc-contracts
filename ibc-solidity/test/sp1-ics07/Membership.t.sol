// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";
import { MembershipTest } from "./MembershipTest.sol";

contract SP1ICS07MembershipTest is MembershipTest {
    SP1MembershipProof public proof;

    function setUpMembershipTestWithFixture(string memory fileName) public {
        setUpTestWithFixtures(fileName);

        proof = abi.decode(fixture.membershipProof.proof, (SP1MembershipProof));
    }

    function fixtureTestCases() public pure returns (FixtureTestCase[] memory) {
        FixtureTestCase[] memory testCases = new FixtureTestCase[](2);
        testCases[0] = FixtureTestCase({ name: "groth16", fileName: "memberships_fixture-groth16.json" });
        testCases[1] = FixtureTestCase({ name: "plonk", fileName: "memberships_fixture-plonk.json" });

        return testCases;
    }

    function test_ValidateFixtures() public {
        FixtureTestCase[] memory testCases = fixtureTestCases();

        for (uint256 i = 0; i < testCases.length; ++i) {
            FixtureTestCase memory tc = testCases[i];
            setUpMembershipTestWithFixture(tc.fileName);

            MembershipOutput memory output = abi.decode(proof.sp1Proof.publicValues, (MembershipOutput));

            assertEq(output.kvPairs.length, 2);
            assertEq(output.kvPairs[0].path, verifyMembershipPath);
            assert(output.kvPairs[0].value.length != 0);
            assertEq(output.kvPairs[0].value, VERIFY_MEMBERSHIP_VALUE);
            assertEq(output.kvPairs[1].path, verifyNonMembershipPath);
            assertEq(output.kvPairs[1].value.length, 0);
        }
    }

    // Confirm that submitting a real proof passes the verifier.
    function test_ValidVerifyMembership() public {
        FixtureTestCase[] memory testCases = fixtureTestCases();

        for (uint256 i = 0; i < testCases.length; ++i) {
            FixtureTestCase memory tc = testCases[i];
            setUpMembershipTestWithFixture(tc.fileName);

            MsgVerifyMembership memory membershipMsg = MsgVerifyMembership({
                proof: abi.encode(fixture.membershipProof),
                proofHeight: fixture.proofHeight,
                path: verifyMembershipPath,
                value: VERIFY_MEMBERSHIP_VALUE
            });

            ics07Tendermint.verifyMembership(membershipMsg);

            console.log("VerifyMembership-", testCases[i].name, " gas used: ", vm.lastCallGas().gasTotalUsed);
        }
    }

    // Modify the proof to make it a non-membership proof.
    function test_ValidVerifyNonMembership() public {
        FixtureTestCase[] memory testCases = fixtureTestCases();

        for (uint256 i = 0; i < testCases.length; ++i) {
            FixtureTestCase memory tc = testCases[i];
            setUpMembershipTestWithFixture(tc.fileName);

            MsgVerifyNonMembership memory nonMembershipMsg = MsgVerifyNonMembership({
                proof: abi.encode(fixture.membershipProof),
                proofHeight: fixture.proofHeight,
                path: verifyNonMembershipPath
            });

            ics07Tendermint.verifyNonMembership(nonMembershipMsg);

            console.log("VerifyNonMembership-", testCases[i].name, " gas used: ", vm.lastCallGas().gasTotalUsed);
        }
    }

    function test_ValidCachedMembership() public {
        // It doesn't matter which fixture we use, as proofs will be cached
        setUpMembershipTestWithFixture("memberships_fixture-plonk.json");

        MsgVerifyMembership memory membershipMsg = MsgVerifyMembership({
            proof: abi.encode(fixture.membershipProof),
            proofHeight: fixture.proofHeight,
            path: verifyMembershipPath,
            value: VERIFY_MEMBERSHIP_VALUE
        });

        ics07Tendermint.verifyMembership(membershipMsg);

        // resubmit cached membership proof
        MsgVerifyMembership memory cachedMembershipMsg = MsgVerifyMembership({
            proof: bytes(""),
            proofHeight: fixture.proofHeight,
            path: verifyMembershipPath,
            value: VERIFY_MEMBERSHIP_VALUE
        });
        ics07Tendermint.verifyMembership(cachedMembershipMsg);

        console.log("Cached VerifyMembership gas used: ", vm.lastCallGas().gasTotalUsed);

        // resubmit cached non-membership proof
        MsgVerifyNonMembership memory cachedNonMembershipMsg = MsgVerifyNonMembership({
            proof: bytes(""), proofHeight: fixture.proofHeight, path: verifyNonMembershipPath
        });

        ics07Tendermint.verifyNonMembership(cachedNonMembershipMsg);

        console.log("Cached VerifyNonMembership gas used: ", vm.lastCallGas().gasTotalUsed);

        // resubmit invalid cached membership proof
        MsgVerifyMembership memory invalidCachedMembershipMsg = MsgVerifyMembership({
            proof: bytes(""), proofHeight: fixture.proofHeight, path: verifyMembershipPath, value: bytes("invalid")
        });
        vm.expectRevert(abi.encodeWithSelector(KeyValuePairNotInCache.selector, verifyMembershipPath, bytes("invalid")));
        ics07Tendermint.verifyMembership(invalidCachedMembershipMsg);
    }

    // Confirm that submitting an invalid proof with the real verifier fails.
    function test_Invalid_VerifyMembership() public {
        FixtureTestCase[] memory testCases = fixtureTestCases();

        for (uint256 i = 0; i < testCases.length; ++i) {
            FixtureTestCase memory tc = testCases[i];
            setUpMembershipTestWithFixture(tc.fileName);

            SP1MembershipProof memory proofMsg = proof;
            proofMsg.sp1Proof.proof = bytes("invalid");

            MembershipProof memory membershipProof =
                MembershipProof({ proofType: MembershipProofType.SP1MembershipProof, proof: abi.encode(proofMsg) });

            MsgVerifyNonMembership memory nonMembershipMsg = MsgVerifyNonMembership({
                proof: abi.encode(membershipProof), proofHeight: fixture.proofHeight, path: verifyNonMembershipPath
            });

            vm.expectRevert();
            ics07Tendermint.verifyNonMembership(nonMembershipMsg);
        }
    }

    // solhint-disable-next-line function-max-lines
    function test_Invalid_MockMembership() public {
        // It doesn't matter which fixture we use, as we use mock verifier
        setUpMembershipTestWithFixture("memberships_fixture-plonk.json");

        MockInvalidMembershipTestCase[] memory testCases = new MockInvalidMembershipTestCase[](10);
        testCases[0] = MockInvalidMembershipTestCase({
            name: "success: valid mock",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: true
        });
        testCases[1] = MockInvalidMembershipTestCase({
            name: "Invalid proof",
            sp1Proof: SP1Proof({
                proof: bytes("invalid"), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });
        testCases[2] = MockInvalidMembershipTestCase({
            name: "Invalid proof height",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight + 1,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });
        testCases[3] = MockInvalidMembershipTestCase({
            name: "Empty path",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: new bytes[](0),
            value: bytes(""),
            expPass: false
        });
        testCases[4] = MockInvalidMembershipTestCase({
            name: "Invalid prefix",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });
        testCases[4].path[0] = bytes("abc");
        testCases[5] = MockInvalidMembershipTestCase({
            name: "Invalid prefix, same length",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });
        testCases[5].path[0] = bytes("invalid");
        testCases[6] = MockInvalidMembershipTestCase({
            name: "Invalid suffix",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });
        testCases[6].path[1] = bytes("invalid");
        testCases[7] = MockInvalidMembershipTestCase({
            name: "Invalid value",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: proof.sp1Proof.vKey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes("invalid"),
            expPass: false
        });
        testCases[8] = MockInvalidMembershipTestCase({
            name: "Invalid vKey",
            sp1Proof: SP1Proof({
                proof: bytes(""), publicValues: proof.sp1Proof.publicValues, vKey: genesisFixture.ucAndMembershipVkey
            }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });
        testCases[9] = MockInvalidMembershipTestCase({
            name: "Invalid public values",
            sp1Proof: SP1Proof({ proof: bytes(""), publicValues: bytes("invalid"), vKey: proof.sp1Proof.vKey }),
            proofHeight: fixture.proofHeight.revisionHeight,
            path: verifyNonMembershipPath,
            value: bytes(""),
            expPass: false
        });

        for (uint256 i = 0; i < testCases.length; ++i) {
            MockInvalidMembershipTestCase memory tc = testCases[i];

            SP1MembershipProof memory proofMsg = proof;
            proofMsg.sp1Proof = tc.sp1Proof;

            MembershipProof memory membershipProof =
                MembershipProof({ proofType: MembershipProofType.SP1MembershipProof, proof: abi.encode(proofMsg) });

            Height memory proofHeight = fixture.proofHeight;
            proofHeight.revisionHeight = tc.proofHeight;

            if (tc.expPass) {
                if (tc.value.length > 0) {
                    MsgVerifyMembership memory membershipMsg = MsgVerifyMembership({
                        proof: abi.encode(membershipProof), proofHeight: proofHeight, path: tc.path, value: tc.value
                    });
                    mockIcs07Tendermint.verifyMembership(membershipMsg);
                } else {
                    MsgVerifyNonMembership memory membershipMsg = MsgVerifyNonMembership({
                        proof: abi.encode(membershipProof), proofHeight: proofHeight, path: tc.path
                    });
                    mockIcs07Tendermint.verifyNonMembership(membershipMsg);
                }
            } else {
                if (tc.value.length > 0) {
                    MsgVerifyMembership memory membershipMsg = MsgVerifyMembership({
                        proof: abi.encode(membershipProof), proofHeight: proofHeight, path: tc.path, value: tc.value
                    });
                    vm.expectRevert();
                    mockIcs07Tendermint.verifyMembership(membershipMsg);
                } else {
                    MsgVerifyNonMembership memory membershipMsg = MsgVerifyNonMembership({
                        proof: abi.encode(membershipProof), proofHeight: proofHeight, path: tc.path
                    });
                    vm.expectRevert();
                    mockIcs07Tendermint.verifyNonMembership(membershipMsg);
                }
            }
        }
    }

    // solhint-disable-next-line gas-struct-packing
    struct MockInvalidMembershipTestCase {
        string name;
        SP1Proof sp1Proof;
        uint64 proofHeight;
        bytes[] path;
        bytes value;
        bool expPass;
    }
}
