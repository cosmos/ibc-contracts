// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { ISP1Verifier } from "@sp1-contracts/ISP1Verifier.sol";

contract MockSP1Verifier is ISP1Verifier {
    error RejectedProof();

    bytes32 private expectedVKey;
    bytes32 private expectedPublicValuesHash;
    bytes32 private expectedProofHash;
    bool private rejectAll;

    function setExpected(bytes32 vkey, bytes calldata publicValues, bytes calldata proofBytes)
        external
    {
        expectedVKey = vkey;
        expectedPublicValuesHash = keccak256(publicValues);
        expectedProofHash = keccak256(proofBytes);
        rejectAll = false;
    }

    function setRejectAll(bool value) external {
        rejectAll = value;
    }

    function verifyProof(
        bytes32 programVKey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view {
        if (
            rejectAll || programVKey != expectedVKey
                || keccak256(publicValues) != expectedPublicValuesHash
                || keccak256(proofBytes) != expectedProofHash
        ) {
            revert RejectedProof();
        }
    }
}
