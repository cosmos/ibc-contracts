// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { BesuZKTypes } from "./BesuZKTypes.sol";

/// @notice Strict codecs for every relayer-controlled ABI payload.
library PublicValuesCodec {
    error EmptyProof();
    error EmptyPublicValues();
    error NonCanonicalEncoding();
    error UnsupportedSchemaVersion(uint32 version);

    uint256 private constant UPDATE_PUBLIC_VALUES_LENGTH = 7 * 32;
    uint256 private constant MEMBERSHIP_PUBLIC_VALUES_LENGTH = 6 * 32;
    uint256 private constant NON_MEMBERSHIP_PUBLIC_VALUES_LENGTH = 5 * 32;

    function decodeEnvelope(bytes calldata encoded)
        internal
        pure
        returns (BesuZKTypes.SP1ProofEnvelope memory envelope)
    {
        envelope = abi.decode(encoded, (BesuZKTypes.SP1ProofEnvelope));
        if (envelope.publicValues.length == 0) revert EmptyPublicValues();
        if (envelope.proofBytes.length == 0) revert EmptyProof();
        _requireCanonicalCalldata(encoded, abi.encode(envelope));
    }

    function decodeMembershipEnvelope(bytes calldata encoded)
        internal
        pure
        returns (BesuZKTypes.MembershipBatchProofEnvelope memory envelope)
    {
        envelope = abi.decode(encoded, (BesuZKTypes.MembershipBatchProofEnvelope));
        _requireCanonicalCalldata(encoded, abi.encode(envelope));
    }

    function decodeUpdate(bytes memory encoded)
        internal
        pure
        returns (BesuZKTypes.UpdatePublicValues memory values)
    {
        if (encoded.length != UPDATE_PUBLIC_VALUES_LENGTH) revert NonCanonicalEncoding();
        values = abi.decode(encoded, (BesuZKTypes.UpdatePublicValues));
        _requireCanonicalMemory(encoded, abi.encode(values));
        _requireSchema(values.schemaVersion);
    }

    function decodeMembership(bytes memory encoded)
        internal
        pure
        returns (BesuZKTypes.MembershipPublicValues memory values)
    {
        if (encoded.length != MEMBERSHIP_PUBLIC_VALUES_LENGTH) revert NonCanonicalEncoding();
        values = abi.decode(encoded, (BesuZKTypes.MembershipPublicValues));
        _requireCanonicalMemory(encoded, abi.encode(values));
        if (values.schemaVersion != BesuZKTypes.MEMBERSHIP_SCHEMA_VERSION) {
            revert UnsupportedSchemaVersion(values.schemaVersion);
        }
    }

    function decodeNonMembership(bytes memory encoded)
        internal
        pure
        returns (BesuZKTypes.NonMembershipPublicValues memory values)
    {
        if (encoded.length != NON_MEMBERSHIP_PUBLIC_VALUES_LENGTH) {
            revert NonCanonicalEncoding();
        }
        values = abi.decode(encoded, (BesuZKTypes.NonMembershipPublicValues));
        _requireCanonicalMemory(encoded, abi.encode(values));
        _requireSchema(values.schemaVersion);
    }

    function _requireSchema(uint32 version) private pure {
        if (version != BesuZKTypes.SCHEMA_VERSION) revert UnsupportedSchemaVersion(version);
    }

    function _requireCanonicalCalldata(bytes calldata original, bytes memory canonical)
        private
        pure
    {
        if (original.length != canonical.length || keccak256(original) != keccak256(canonical)) {
            revert NonCanonicalEncoding();
        }
    }

    function _requireCanonicalMemory(bytes memory original, bytes memory canonical) private pure {
        if (original.length != canonical.length || keccak256(original) != keccak256(canonical)) {
            revert NonCanonicalEncoding();
        }
    }
}
