// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { ILightClient } from "../../interfaces/ILightClient.sol";
import { ILightClientMsgs } from "../../msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../msgs/IICS02ClientMsgs.sol";

/// @title Dummy Light Client
/// @notice Insecure light client for local development and testing; proofs are ignored.
contract DummyLightClient is ILightClient {
    /// @notice A membership record exists for the queried height and path.
    error MembershipExists();
    /// @notice No membership record exists for the queried height and path.
    error MissingMembership();
    /// @notice No timestamp has been registered for the queried height.
    error UnknownHeight();
    /// @notice The queried value does not match the stored value hash.
    error ValueMismatch();
    /// @notice Consensus timestamps must be non-zero.
    error ZeroTimestamp();

    /// @notice Dummy client state returned by `getClientState`.
    /// @param latestHeight Latest updated height.
    /// @param latestTimestamp Timestamp for the latest updated height.
    struct ClientState {
        IICS02ClientMsgs.Height latestHeight;
        uint64 latestTimestamp;
    }

    /// @notice Membership record supplied in an update.
    /// @param path Merkle path for the record.
    /// @param value Value for the record.
    struct Membership {
        bytes[] path;
        bytes value;
    }

    /// @notice Update message accepted by the dummy client.
    /// @param height Height whose timestamp and membership records are being set.
    /// @param timestamp Consensus timestamp for the height.
    /// @param memberships Membership records for the height.
    struct MsgUpdateClient {
        IICS02ClientMsgs.Height height;
        uint64 timestamp;
        Membership[] memberships;
    }

    /// @notice Latest dummy client state.
    ClientState private _clientState;

    /// @notice Consensus timestamps by encoded height key.
    mapping(bytes32 heightKey => uint64 timestamp) private _timestamps;
    /// @notice Membership value hashes by encoded height and path key.
    mapping(bytes32 membershipKey => bytes32 valueHash) private _membershipValueHashes;

    /// @inheritdoc ILightClient
    function updateClient(bytes calldata updateMsg) external returns (ILightClientMsgs.UpdateResult) {
        MsgUpdateClient memory msg_ = abi.decode(updateMsg, (MsgUpdateClient));
        if (msg_.timestamp == 0) {
            revert ZeroTimestamp();
        }

        IICS02ClientMsgs.Height memory height = msg_.height;
        bytes32 heightKey = _heightKey(height.revisionNumber, height.revisionHeight);
        _timestamps[heightKey] = msg_.timestamp;
        _clientState = ClientState({ latestHeight: height, latestTimestamp: msg_.timestamp });

        for (uint256 i = 0; i < msg_.memberships.length; ++i) {
            Membership memory membership = msg_.memberships[i];

            bytes32 pathHash = keccak256(abi.encode(membership.path));
            bytes32 membershipKey = _membershipKey(height.revisionNumber, height.revisionHeight, pathHash);
            _membershipValueHashes[membershipKey] = keccak256(membership.value);
        }

        return ILightClientMsgs.UpdateResult.Update;
    }

    /// @inheritdoc ILightClient
    function verifyMembership(ILightClientMsgs.MsgVerifyMembership calldata msg_) external returns (uint256) {
        uint64 timestamp = _timestamp(msg_.proofHeight);

        bytes32 valueHash = _membershipValueHashes[_membershipKey(msg_.proofHeight, msg_.path)];
        if (valueHash == bytes32(0)) {
            revert MissingMembership();
        }
        if (valueHash != keccak256(msg_.value)) {
            revert ValueMismatch();
        }

        return timestamp;
    }

    /// @inheritdoc ILightClient
    function verifyNonMembership(ILightClientMsgs.MsgVerifyNonMembership calldata msg_) external returns (uint256) {
        uint64 timestamp = _timestamp(msg_.proofHeight);

        bytes32 valueHash = _membershipValueHashes[_membershipKey(msg_.proofHeight, msg_.path)];
        if (valueHash != bytes32(0)) {
            revert MembershipExists();
        }

        return timestamp;
    }

    /// @inheritdoc ILightClient
    function misbehaviour(bytes calldata) external {
        return;
    }

    /// @inheritdoc ILightClient
    function getClientState() external view returns (bytes memory) {
        return abi.encode(_clientState);
    }

    /// @notice Returns the registered timestamp for a height.
    /// @param height Height to query.
    /// @return Registered consensus timestamp.
    function _timestamp(IICS02ClientMsgs.Height calldata height) private view returns (uint64) {
        uint64 timestamp = _timestamps[_heightKey(height.revisionNumber, height.revisionHeight)];
        if (timestamp == 0) {
            revert UnknownHeight();
        }
        return timestamp;
    }

    /// @notice Builds a membership storage key.
    /// @param height Height for the record.
    /// @param path Path for the record.
    /// @return Membership storage key.
    function _membershipKey(
        IICS02ClientMsgs.Height calldata height,
        bytes[] calldata path
    )
        private
        pure
        returns (bytes32)
    {
        return _membershipKey(height.revisionNumber, height.revisionHeight, keccak256(abi.encode(path)));
    }

    /// @notice Builds a height storage key.
    /// @param revisionNumber Revision number of the height.
    /// @param revisionHeight Revision height.
    /// @return Height storage key.
    function _heightKey(uint64 revisionNumber, uint64 revisionHeight) private pure returns (bytes32) {
        return keccak256(abi.encode(revisionNumber, revisionHeight));
    }

    /// @notice Builds a membership storage key from height fields and path hash.
    /// @param revisionNumber Revision number of the height.
    /// @param revisionHeight Revision height.
    /// @param pathHash Hash of the path.
    /// @return Membership storage key.
    function _membershipKey(
        uint64 revisionNumber,
        uint64 revisionHeight,
        bytes32 pathHash
    )
        private
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(revisionNumber, revisionHeight, pathHash));
    }
}
