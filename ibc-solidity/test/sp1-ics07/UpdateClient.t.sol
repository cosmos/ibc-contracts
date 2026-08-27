// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1ICS07TendermintTest} from "./SP1ICS07TendermintTest.sol";

struct SP1ICS07UpdateClientFixtureJson {
    bytes trustedClientState;
    bytes trustedConsensusState;
    bytes updateMsg;
}

contract SP1ICS07UpdateClientTest is SP1ICS07TendermintTest {
    using stdJson for string;

    SP1ICS07UpdateClientFixtureJson public fixture;

    UpdateClientOutput public output;

    function setUpTestWithFixture(string memory fileName) public {
        fixture = loadFixture(fileName);

        setUpTest(fileName, address(0));

        MsgUpdateClient memory updateMsg = abi.decode(fixture.updateMsg, (MsgUpdateClient));
        output = abi.decode(updateMsg.sp1Proof.publicValues, (UpdateClientOutput));

        ClientState memory clientState = abi.decode(mockIcs07Tendermint.getClientState(), (ClientState));
        assert(clientState.latestHeight.revisionHeight < output.newHeight.revisionHeight);

        vm.expectRevert();
        mockIcs07Tendermint.getConsensusStateHash(output.newHeight.revisionHeight);
    }

    function fixtureTestCases() public pure returns (FixtureTestCase[] memory) {
        FixtureTestCase[] memory testCases = new FixtureTestCase[](2);
        testCases[0] = FixtureTestCase({name: "groth16", fileName: "update_client_fixture-groth16.json"});
        testCases[1] = FixtureTestCase({name: "plonk", fileName: "update_client_fixture-plonk.json"});

        return testCases;
    }

    // Confirm that submitting a real proof passes the verifier.
    function test_ValidUpdateClient() public {
        FixtureTestCase[] memory testCases = fixtureTestCases();

        for (uint256 i = 0; i < testCases.length; ++i) {
            setUpTestWithFixture(testCases[i].fileName);

            // set a correct timestamp
            vm.warp(_nanosToSeconds(output.time) + 300);

            // run verify
            UpdateResult res = ics07Tendermint.updateClient(fixture.updateMsg);

            // to console
            console.log("UpdateClient-", testCases[i].name, "gas used: ", vm.lastCallGas().gasTotalUsed);
            assert(res == UpdateResult.Update);

            ClientState memory clientState = abi.decode(ics07Tendermint.getClientState(), (ClientState));
            assert(keccak256(bytes(clientState.chainId)) == keccak256(bytes("cosmoshub-4")));
            assert(clientState.latestHeight.revisionHeight == output.newHeight.revisionHeight);
            assert(clientState.isFrozen == false);

            bytes32 consensusHash = ics07Tendermint.getConsensusStateHash(output.newHeight.revisionHeight);
            assertEq(consensusHash, keccak256(abi.encode(output.newConsensusState)));
        }
    }

    // Confirm that submitting a real proof passes the verifier.
    function test_ValidNoOpUpdateClient() public {
        // Doesn't matter which fixture we use since this is a no-op
        setUpTestWithFixture("update_client_fixture-plonk.json");
        // set a correct timestamp
        vm.warp(_nanosToSeconds(output.time) + 300);

        // run verify
        UpdateResult res = ics07Tendermint.updateClient(fixture.updateMsg);
        assert(res == UpdateResult.Update);

        // run verify again
        res = ics07Tendermint.updateClient(fixture.updateMsg);

        // to console
        console.log("UpdateClient_NoOp gas used: ", vm.lastCallGas().gasTotalUsed);
        assert(res == UpdateResult.NoOp);
    }

    function test_Invalid_UpdateClient() public {
        // Doesn't matter which fixture we use since this is a fail
        setUpTestWithFixture("update_client_fixture-plonk.json");

        vm.expectRevert();
        ics07Tendermint.updateClient(bytes("invalid"));
    }

    function test_MockMisbehavior_UpdateClient() public {
        // Doesn't matter which fixture we use since this is a mock contract
        setUpTestWithFixture("update_client_fixture-plonk.json");
        // set a correct timestamp
        vm.warp(_nanosToSeconds(output.time) + 300);

        // update mock client
        MsgUpdateClient memory updateMsg = abi.decode(fixture.updateMsg, (MsgUpdateClient));
        updateMsg.sp1Proof.proof = bytes("");

        UpdateResult res = mockIcs07Tendermint.updateClient(abi.encode(updateMsg));
        assert(res == UpdateResult.Update);

        // change output so that it is a misbehaviour
        output.newConsensusState.timestamp = output.time + 1;
        // re-encode output
        updateMsg.sp1Proof.publicValues = abi.encode(output);

        // run verify again
        res = mockIcs07Tendermint.updateClient(abi.encode(updateMsg));
        assert(res == UpdateResult.Misbehaviour);

        ClientState memory clientState = abi.decode(mockIcs07Tendermint.getClientState(), (ClientState));
        assert(clientState.isFrozen == true);
    }

    function loadFixture(string memory fileName) public view returns (SP1ICS07UpdateClientFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, FIXTURE_DIR, fileName);
        string memory json = vm.readFile(path);
        bytes memory trustedClientState = json.readBytes(".trustedClientState");
        bytes memory trustedConsensusState = json.readBytes(".trustedConsensusState");
        bytes memory updateMsg = json.readBytes(".updateMsg");

        SP1ICS07UpdateClientFixtureJson memory fix = SP1ICS07UpdateClientFixtureJson({
            trustedClientState: trustedClientState, trustedConsensusState: trustedConsensusState, updateMsg: updateMsg
        });

        return fix;
    }
}
