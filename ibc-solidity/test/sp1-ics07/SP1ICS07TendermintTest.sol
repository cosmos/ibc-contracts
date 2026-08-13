// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";
import { Test, stdStorage, StdStorage } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { IICS07TendermintMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IICS07TendermintMsgs.sol";
import { IUpdateClientMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IUpdateClientMsgs.sol";
import { IMembershipMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IMembershipMsgs.sol";
import { IUpdateClientAndMembershipMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IUcAndMembershipMsgs.sol";
import { IMisbehaviourMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IMisbehaviourMsgs.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { ISP1Msgs } from "../../contracts/light-clients/sp1-ics07/msgs/ISP1Msgs.sol";
import { SP1ICS07Tendermint } from "../../contracts/light-clients/sp1-ics07/SP1ICS07Tendermint.sol";
import {
    ISP1ICS07TendermintErrors
} from "../../contracts/light-clients/sp1-ics07/errors/ISP1ICS07TendermintErrors.sol";
import { SP1MockVerifier } from "@sp1-contracts/SP1MockVerifier.sol";
import { SP1Verifier as SP1VerifierPlonk } from "@sp1-contracts/v6.1.0/SP1VerifierPlonk.sol";
import { SP1Verifier as SP1VerifierGroth16 } from "@sp1-contracts/v6.1.0/SP1VerifierGroth16.sol";

struct SP1ICS07GenesisFixtureJson {
    bytes trustedClientState;
    bytes trustedConsensusState;
    bytes32 updateClientVkey;
    bytes32 membershipVkey;
    bytes32 ucAndMembershipVkey;
    bytes32 misbehaviourVkey;
}

abstract contract SP1ICS07TendermintTest is
    Test,
    IICS02ClientMsgs,
    ISP1Msgs,
    IICS07TendermintMsgs,
    IUpdateClientMsgs,
    IMembershipMsgs,
    IUpdateClientAndMembershipMsgs,
    IMisbehaviourMsgs,
    ISP1ICS07TendermintErrors,
    ILightClientMsgs
{
    using stdJson for string;
    using stdStorage for StdStorage;

    SP1ICS07Tendermint public ics07Tendermint;
    SP1ICS07Tendermint public mockIcs07Tendermint;

    SP1ICS07GenesisFixtureJson internal genesisFixture;

    string internal constant FIXTURE_DIR = "/test/sp1-ics07/fixtures/";

    function setUpTest(string memory fileName, address roleManager) public {
        genesisFixture = loadGenesisFixture(fileName);

        ConsensusState memory trustedConsensusState = abi.decode(genesisFixture.trustedConsensusState, (ConsensusState));

        bytes32 trustedConsensusHash = keccak256(abi.encode(trustedConsensusState));
        ClientState memory trustedClientState = abi.decode(genesisFixture.trustedClientState, (ClientState));

        address verifier;
        if (trustedClientState.zkAlgorithm == SupportedZkAlgorithm.Plonk) {
            verifier = address(new SP1VerifierPlonk());
        } else if (trustedClientState.zkAlgorithm == SupportedZkAlgorithm.Groth16) {
            verifier = address(new SP1VerifierGroth16());
        } else {
            revert("Unsupported zk algorithm");
        }

        ics07Tendermint = new SP1ICS07Tendermint(
            genesisFixture.updateClientVkey,
            genesisFixture.membershipVkey,
            genesisFixture.ucAndMembershipVkey,
            genesisFixture.misbehaviourVkey,
            verifier,
            genesisFixture.trustedClientState,
            trustedConsensusHash,
            roleManager
        );

        mockIcs07Tendermint = new SP1ICS07Tendermint(
            genesisFixture.updateClientVkey,
            genesisFixture.membershipVkey,
            genesisFixture.ucAndMembershipVkey,
            genesisFixture.misbehaviourVkey,
            address(new SP1MockVerifier()),
            genesisFixture.trustedClientState,
            trustedConsensusHash,
            roleManager
        );

        ClientState memory clientState = abi.decode(mockIcs07Tendermint.getClientState(), (ClientState));
        assert(keccak256(abi.encode(clientState)) == keccak256(genesisFixture.trustedClientState));

        bytes32 consensusHash = mockIcs07Tendermint.getConsensusStateHash(clientState.latestHeight.revisionHeight);
        assert(consensusHash == trustedConsensusHash);
    }

    function loadGenesisFixture(string memory fileName) public view returns (SP1ICS07GenesisFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, FIXTURE_DIR, fileName);
        string memory json = vm.readFile(path);
        bytes memory trustedClientState = json.readBytes(".trustedClientState");
        bytes memory trustedConsensusState = json.readBytes(".trustedConsensusState");
        bytes32 updateClientVkey = json.readBytes32(".updateClientVkey");
        bytes32 membershipVkey = json.readBytes32(".membershipVkey");
        bytes32 ucAndMembershipVkey = json.readBytes32(".ucAndMembershipVkey");
        bytes32 misbehaviourVkey = json.readBytes32(".misbehaviourVkey");

        SP1ICS07GenesisFixtureJson memory fix = SP1ICS07GenesisFixtureJson({
            trustedClientState: trustedClientState,
            trustedConsensusState: trustedConsensusState,
            updateClientVkey: updateClientVkey,
            membershipVkey: membershipVkey,
            ucAndMembershipVkey: ucAndMembershipVkey,
            misbehaviourVkey: misbehaviourVkey
        });

        return fix;
    }

    function _nanosToSeconds(uint256 nanos) internal pure returns (uint256) {
        return nanos / 1e9;
    }

    struct FixtureTestCase {
        string name;
        string fileName;
    }
}
