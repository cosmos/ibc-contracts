// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";
import { SP1ICS07TendermintTest } from "./SP1ICS07TendermintTest.sol";
import { IMisbehaviourMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IMisbehaviourMsgs.sol";
import { SP1Verifier } from "@sp1-contracts/v6.1.0/SP1VerifierPlonk.sol";
import { stdJson } from "forge-std/StdJson.sol";

struct SP1ICS07MisbehaviourFixtureJson {
    bytes trustedClientState;
    bytes trustedConsensusState;
    bytes submitMsg;
}

contract SP1ICS07MisbehaviourTest is SP1ICS07TendermintTest {
    using stdJson for string;

    SP1ICS07MisbehaviourFixtureJson public fixture;
    MsgSubmitMisbehaviour public submitMsg;
    MisbehaviourOutput public output;

    function setUpMisbehaviour(string memory fileName) public {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, FIXTURE_DIR, fileName);
        string memory json = vm.readFile(path);
        bytes memory trustedClientStateBz = json.readBytes(".trustedClientState");
        bytes memory trustedConsensusStateBz = json.readBytes(".trustedConsensusState");
        bytes memory submitMsgBz = json.readBytes(".submitMsg");

        fixture = SP1ICS07MisbehaviourFixtureJson({
            trustedClientState: trustedClientStateBz,
            trustedConsensusState: trustedConsensusStateBz,
            submitMsg: submitMsgBz
        });

        setUpTest(fileName, address(0));

        submitMsg = abi.decode(fixture.submitMsg, (IMisbehaviourMsgs.MsgSubmitMisbehaviour));
        output = abi.decode(submitMsg.sp1Proof.publicValues, (IMisbehaviourMsgs.MisbehaviourOutput));
    }

    function test_ValidDoubleSignMisbehaviour() public {
        setUpMisbehaviour("misbehaviour_double_sign-plonk_fixture.json");

        // set a correct timestamp
        vm.warp(_nanosToSeconds(output.time));
        ics07Tendermint.misbehaviour(fixture.submitMsg);

        // to console
        console.log("Misbehaviour gas used: ", vm.lastCallGas().gasTotalUsed);

        // verify that the client is frozen
        ClientState memory clientState = abi.decode(ics07Tendermint.getClientState(), (ClientState));
        assertTrue(clientState.isFrozen);
    }

    function test_ValidBreakingTimeMonotonicityMisbehaviour() public {
        setUpMisbehaviour("misbehaviour_breaking_time_monotonicity-groth16_fixture.json");

        // set a correct timestamp
        vm.warp(_nanosToSeconds(output.time));
        ics07Tendermint.misbehaviour(fixture.submitMsg);

        // to console
        console.log("Misbehaviour gas used: ", vm.lastCallGas().gasTotalUsed);

        // verify that the client is frozen
        ClientState memory clientState = abi.decode(ics07Tendermint.getClientState(), (ClientState));
        assertTrue(clientState.isFrozen);
    }

    function test_FrozenClientState() public {
        setUpMisbehaviour("misbehaviour_double_sign-plonk_fixture.json");

        // set a correct timestamp
        vm.warp(_nanosToSeconds(output.time));
        ics07Tendermint.misbehaviour(fixture.submitMsg);

        // verify that the client is frozen
        ClientState memory clientState = abi.decode(ics07Tendermint.getClientState(), (ClientState));
        assertTrue(clientState.isFrozen);

        // try to submit a updateClient msg
        vm.expectRevert(abi.encodeWithSelector(FrozenClientState.selector));
        ics07Tendermint.updateClient(bytes(""));

        // try to submit a membership msg
        MsgVerifyMembership memory membership;
        vm.expectRevert(abi.encodeWithSelector(FrozenClientState.selector));
        ics07Tendermint.verifyMembership(membership);

        // try to submit a non-membership msg
        MsgVerifyNonMembership memory nonMembership;
        vm.expectRevert(abi.encodeWithSelector(FrozenClientState.selector));
        ics07Tendermint.verifyNonMembership(nonMembership);

        // try to submit a misbehaviour msg
        vm.expectRevert(abi.encodeWithSelector(FrozenClientState.selector));
        ics07Tendermint.misbehaviour(fixture.submitMsg);
    }

    // solhint-disable-next-line function-max-lines
    function test_InvalidMisbehaviour() public {
        setUpMisbehaviour("misbehaviour_double_sign-plonk_fixture.json");

        // proof is in the future
        vm.warp(_nanosToSeconds(output.time) - 300);
        vm.expectRevert(
            abi.encodeWithSelector(ProofIsInTheFuture.selector, block.timestamp, _nanosToSeconds(output.time))
        );
        ics07Tendermint.misbehaviour(fixture.submitMsg);

        // proof is too old
        vm.warp(output.time + ics07Tendermint.ALLOWED_SP1_CLOCK_DRIFT() + 300);
        vm.expectRevert(abi.encodeWithSelector(ProofIsTooOld.selector, block.timestamp, _nanosToSeconds(output.time)));
        ics07Tendermint.misbehaviour(fixture.submitMsg);

        // set a correct timestamp
        vm.warp(_nanosToSeconds(output.time) + 300);

        // wrong vkey
        MsgSubmitMisbehaviour memory badSubmitMsg = cloneSubmitMsg();
        badSubmitMsg.sp1Proof.vKey = bytes32(0);
        bytes memory submitMsgBz = abi.encode(badSubmitMsg);
        vm.expectRevert(
            abi.encodeWithSelector(
                VerificationKeyMismatch.selector,
                ics07Tendermint.MISBEHAVIOUR_PROGRAM_VKEY(),
                badSubmitMsg.sp1Proof.vKey
            )
        );
        ics07Tendermint.misbehaviour(submitMsgBz);

        // chain id mismatch
        badSubmitMsg = cloneSubmitMsg();
        MisbehaviourOutput memory badOutput = cloneOutput();
        badOutput.clientState.chainId = "bad-chain-id";
        badSubmitMsg.sp1Proof.publicValues = abi.encode(badOutput);
        submitMsgBz = abi.encode(badSubmitMsg);
        vm.expectRevert(
            abi.encodeWithSelector(ChainIdMismatch.selector, output.clientState.chainId, badOutput.clientState.chainId)
        );
        ics07Tendermint.misbehaviour(submitMsgBz);

        // trust threshold mismatch
        badSubmitMsg = cloneSubmitMsg();
        badOutput = cloneOutput();
        badOutput.clientState.trustLevel = TrustThreshold({ numerator: 1, denominator: 2 });
        badSubmitMsg.sp1Proof.publicValues = abi.encode(badOutput);
        submitMsgBz = abi.encode(badSubmitMsg);
        vm.expectRevert(
            abi.encodeWithSelector(
                TrustThresholdMismatch.selector,
                output.clientState.trustLevel.numerator,
                output.clientState.trustLevel.denominator,
                badOutput.clientState.trustLevel.numerator,
                badOutput.clientState.trustLevel.denominator
            )
        );
        ics07Tendermint.misbehaviour(submitMsgBz);

        // trusting period mismatch
        badSubmitMsg = cloneSubmitMsg();
        badOutput = cloneOutput();
        badOutput.clientState.trustingPeriod = 1;
        badSubmitMsg.sp1Proof.publicValues = abi.encode(badOutput);
        submitMsgBz = abi.encode(badSubmitMsg);
        vm.expectRevert(
            abi.encodeWithSelector(
                TrustingPeriodMismatch.selector, output.clientState.trustingPeriod, badOutput.clientState.trustingPeriod
            )
        );
        ics07Tendermint.misbehaviour(submitMsgBz);

        // invalid proof
        badSubmitMsg = cloneSubmitMsg();
        badOutput = cloneOutput();
        ++badOutput.time;
        badSubmitMsg.sp1Proof.publicValues = abi.encode(badOutput);
        submitMsgBz = abi.encode(badSubmitMsg);
        vm.expectRevert(abi.encodeWithSelector(SP1Verifier.InvalidProof.selector));
        ics07Tendermint.misbehaviour(submitMsgBz);

        // client state is frozen
        ics07Tendermint.misbehaviour(fixture.submitMsg); // freeze the client
        vm.expectRevert(abi.encodeWithSelector(FrozenClientState.selector));
        ics07Tendermint.misbehaviour(fixture.submitMsg);
    }

    function cloneSubmitMsg() private view returns (MsgSubmitMisbehaviour memory) {
        MsgSubmitMisbehaviour memory clone = MsgSubmitMisbehaviour({
            sp1Proof: SP1Proof({
                vKey: submitMsg.sp1Proof.vKey,
                publicValues: submitMsg.sp1Proof.publicValues,
                proof: submitMsg.sp1Proof.proof
            })
        });
        return clone;
    }

    function cloneOutput() private view returns (MisbehaviourOutput memory) {
        MisbehaviourOutput memory clone = MisbehaviourOutput({
            clientState: output.clientState,
            time: output.time,
            trustedHeight1: output.trustedHeight1,
            trustedHeight2: output.trustedHeight2,
            trustedConsensusState1: output.trustedConsensusState1,
            trustedConsensusState2: output.trustedConsensusState2
        });
        return clone;
    }
}
