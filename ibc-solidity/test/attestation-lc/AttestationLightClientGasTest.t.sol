// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { AttestationLightClient } from "../../contracts/light-clients/attestation/AttestationLightClient.sol";
import { IAttestationMsgs as AM } from "../../contracts/light-clients/attestation/msgs/IAttestationMsgs.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";

contract AttestationLightClientGasTest is Test {
    uint64 internal constant INITIAL_HEIGHT = 100;
    uint64 internal constant INITIAL_TS = 1_700_000_000;

    error NeedAtLeastOneCommitment();

    function testGas_VerifyMembership_1of1_1Commit() public {
        _runScenario({ quorum: 1, attestorCount: 1, commitmentCount: 1, label: "verifyMembership 1of1 - 1 commit" });
    }

    function testGas_VerifyMembership_1of1_5Commits() public {
        _runScenario({ quorum: 1, attestorCount: 1, commitmentCount: 5, label: "verify 1of1 - 5 commits" });
    }

    function testGas_VerifyMembership_1of1_20Commits() public {
        _runScenario({ quorum: 1, attestorCount: 1, commitmentCount: 20, label: "verify 1of1 - 20 commits" });
    }

    function testGas_VerifyMembership_3of5_1Commit() public {
        _runScenario({ quorum: 3, attestorCount: 5, commitmentCount: 1, label: "verifyMembership 3of5 - 1 commit" });
    }

    function testGas_VerifyMembership_3of5_5Commits() public {
        _runScenario({ quorum: 3, attestorCount: 5, commitmentCount: 5, label: "verify 3of5 - 5 commits" });
    }

    function testGas_VerifyMembership_3of5_20Commits() public {
        _runScenario({ quorum: 3, attestorCount: 5, commitmentCount: 20, label: "verify 3of5 - 20 commits" });
    }

    function testGas_VerifyMembership_5of7_1Commit() public {
        _runScenario({ quorum: 5, attestorCount: 7, commitmentCount: 1, label: "verifyMembership 5of7 - 1 commit" });
    }

    function testGas_VerifyMembership_5of7_5Commits() public {
        _runScenario({ quorum: 5, attestorCount: 7, commitmentCount: 5, label: "verify 5of7 - 5 commits" });
    }

    function testGas_VerifyMembership_5of7_20Commits() public {
        _runScenario({ quorum: 5, attestorCount: 7, commitmentCount: 20, label: "verify 5of7 - 20 commits" });
    }

    function _runScenario(uint8 quorum, uint256 attestorCount, uint256 commitmentCount, string memory label) internal {
        (address[] memory attestorAddrs, uint256[] memory attestorPrivs) = _generateAttestors(attestorCount);

        AttestationLightClient client = new AttestationLightClient({
            attestorAddresses: attestorAddrs,
            minRequiredSigs: quorum,
            initialHeight: INITIAL_HEIGHT,
            initialTimestampSeconds: INITIAL_TS,
            roleManager: address(0)
        });

        (AM.PacketCompact[] memory packets, AM.PacketCompact memory target) = _makeCommitments(commitmentCount);

        AM.PacketAttestation memory p = AM.PacketAttestation({ height: INITIAL_HEIGHT, packets: packets });
        bytes memory attestationData = abi.encode(p);
        bytes32 digest = sha256(abi.encodePacked(bytes1(0x02), sha256(attestationData)));

        bytes[] memory signatures = new bytes[](quorum);
        for (uint256 i = 0; i < signatures.length; ++i) {
            signatures[i] = _sig(attestorPrivs[i], digest);
        }

        AM.AttestationProof memory proof =
            AM.AttestationProof({ attestationData: attestationData, signatures: signatures });

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;
        msgVerify.proof = abi.encode(proof);
        msgVerify.proofHeight = IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: INITIAL_HEIGHT });
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = abi.encodePacked("packet-path", commitmentCount - 1);
        msgVerify.value = abi.encode(target.commitment);

        uint256 gasBefore = gasleft();
        uint256 ts = client.verifyMembership(msgVerify);
        uint256 gasUsed = gasBefore - gasleft();
        emit log_named_uint(label, gasUsed);

        assertEq(ts, INITIAL_TS);
    }

    function _generateAttestors(uint256 n) internal pure returns (address[] memory addrs, uint256[] memory privs) {
        addrs = new address[](n);
        privs = new uint256[](n);
        for (uint256 i = 0; i < n; ++i) {
            // deterministic but distinct private keys
            uint256 pk = uint256(keccak256(abi.encodePacked("attestor-pk", i + 1)));
            privs[i] = pk;
            addrs[i] = vm.addr(pk);
        }
    }

    function _makeCommitments(uint256 k)
        internal
        pure
        returns (AM.PacketCompact[] memory packets, AM.PacketCompact memory target)
    {
        if (k == 0) {
            revert NeedAtLeastOneCommitment();
        }

        packets = new AM.PacketCompact[](k);

        for (uint256 i = 0; i < k; ++i) {
            packets[i] = AM.PacketCompact({
                path: keccak256(abi.encodePacked("packet-path", i)),
                commitment: keccak256(abi.encodePacked("packet-", i))
            });
            target = packets[i];
        }
    }

    function _sig(uint256 privKey, bytes32 digest) internal pure returns (bytes memory) {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privKey, digest);
        return abi.encodePacked(r, s, v);
    }
}
