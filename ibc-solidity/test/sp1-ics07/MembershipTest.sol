// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { SP1ICS07TendermintTest } from "./SP1ICS07TendermintTest.sol";

abstract contract MembershipTest is SP1ICS07TendermintTest {
    bytes[] public verifyMembershipPath = [bytes("ibc"), bytes("clients/07-tendermint-0/clientState")];

    bytes[] public verifyNonMembershipPath = [bytes("ibc"), bytes("clients/07-tendermint-001/clientState")];

    bytes public constant VERIFY_MEMBERSHIP_VALUE =
        hex"0a2b2f6962632e6c69676874636c69656e74732e74656e6465726d696e742e76312e436c69656e74537461746512750a056f6b776d651204080110031a040880ea4922040880df6e2a0308d80432003a02101442190a090801180120012a0100120c0a02000110211804200c300142190a090801180120012a0100120c0a02000110201801200130014a07757067726164654a1075706772616465644942435374617465";

    struct SP1ICS07MembershipFixtureJson {
        Height proofHeight;
        MembershipProof membershipProof;
    }

    using stdJson for string;

    SP1ICS07MembershipFixtureJson public fixture;

    function setUpTestWithFixtures(string memory fileName) public {
        fixture = loadFixture(fileName);

        setUpTest(fileName, address(0));
    }

    function loadFixture(string memory fileName) public view returns (SP1ICS07MembershipFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, FIXTURE_DIR, fileName);
        string memory json = vm.readFile(path);
        bytes memory proofHeightBz = json.readBytes(".proofHeight");
        bytes memory membershipProofBz = json.readBytes(".membershipProof");

        SP1ICS07MembershipFixtureJson memory fix = SP1ICS07MembershipFixtureJson({
            proofHeight: abi.decode(proofHeightBz, (Height)),
            membershipProof: abi.decode(membershipProofBz, (MembershipProof))
        });

        return fix;
    }
}
