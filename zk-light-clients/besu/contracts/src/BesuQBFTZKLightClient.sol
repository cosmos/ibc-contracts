// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { ILightClient } from "@ibc-contracts/interfaces/ILightClient.sol";
import { ILightClientMsgs } from "@ibc-contracts/msgs/ILightClientMsgs.sol";
import { ISP1Verifier } from "@sp1-contracts/ISP1Verifier.sol";
import { AccessManagedUpgradeable } from
    "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import { BesuZKTypes } from "./BesuZKTypes.sol";
import { PublicValuesCodec } from "./PublicValuesCodec.sol";
import { BatchMembershipLib } from "./BatchMembershipLib.sol";
import { IBesuQBFTZKLightClient } from "./interfaces/IBesuQBFTZKLightClient.sol";

/// @dev Deployed behind an ERC1967Proxy, initialized in the proxy's constructor. Upgrades are
/// `restricted` by the configured AccessManager. Router integration may require matching authorities.
contract BesuQBFTZKLightClient is
    ILightClient,
    IBesuQBFTZKLightClient,
    AccessManagedUpgradeable,
    UUPSUpgradeable
{
    error UnauthorizedCaller(address caller);
    error ZeroAddress();
    error InvalidProgramVKey();
    error InvalidBootstrapState();
    error UnsupportedRevisionNumber(uint64 revisionNumber);
    error InvalidPathLength(uint256 actual);
    error InvalidMembershipValueLength(uint256 actual);
    error ConsensusStateNotFound(uint64 height);
    error TrustedValidatorSetHashMismatch();
    error InvalidHeightTransition(uint64 trustedHeight, uint64 verifiedHeight);
    error NonIncreasingTimestamp(uint64 trustedTimestamp, uint64 verifiedTimestamp);
    error TrustedStateExpired(uint64 trustedHeight);
    error TimestampTooFarInFuture(uint64 blockTimestamp);
    error StateRootMismatch();
    error SourceRouterMismatch();
    error PathDigestMismatch();
    error UnsupportedMembershipEnvelopeMode(uint8 mode);
    error InvalidMembershipEnvelopePayload();
    error UnknownMembershipBatch(bytes32 batchId);
    error ConflictingConsensusState(uint64 height);
    error UnsupportedMisbehaviour();

    event ClientUpdated(
        uint64 indexed height, bytes32 stateRoot, uint64 blockTimestamp, bytes32 validatorSetHash
    );
    event MembershipBatchVerified(
        bytes32 indexed batchId, uint64 indexed proofHeight, bytes32 batchRoot, uint32 entryCount
    );

    /// @custom:storage-location erc7201:ibc.lightclient.storage.BesuQBFTZKLightClient
    struct LightClientStorage {
        address localIbcRouter;
        address sourceIbcRouter;
        address sp1Verifier;
        bytes32 updateClientVKey;
        bytes32 membershipVKey;
        bytes32 nonMembershipVKey;
        BesuZKTypes.ClientState clientState;
        mapping(uint64 height => BesuZKTypes.ConsensusState state) consensusStates;
        mapping(bytes32 batchId => bool verified) verifiedMembershipBatches;
    }

    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.lightclient.storage.BesuQBFTZKLightClient"))
    /// - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant LIGHT_CLIENT_STORAGE_SLOT =
        0x94ee77a41f9e95e3b69a8107d85bc451d194ece6aa4d138bad1e127452480400;

    modifier onlyLocalRouter() {
        if (msg.sender != _getLightClientStorage().localIbcRouter) {
            revert UnauthorizedCaller(msg.sender);
        }
        _;
    }

    constructor() {
        _disableInitializers();
    }

    /// @notice Everything `initialize` binds besides the authority.
    struct InitParams {
        address localIbcRouter;
        address sourceIbcRouter;
        address sp1Verifier;
        bytes32 updateClientVKey;
        bytes32 membershipVKey;
        bytes32 nonMembershipVKey;
        uint64 initialHeight;
        bytes32 initialStateRoot;
        uint64 initialBlockTimestamp;
        bytes32 initialValidatorSetHash;
        uint64 trustingPeriodSeconds;
        uint64 maxClockDriftSeconds;
    }

    /// @param authority The AccessManager that authorizes upgrades.
    function initialize(address authority, InitParams calldata p) external initializer {
        if (
            authority == address(0) || p.localIbcRouter == address(0)
                || p.sourceIbcRouter == address(0) || p.sp1Verifier == address(0)
        ) {
            revert ZeroAddress();
        }
        if (
            p.updateClientVKey == bytes32(0) || p.membershipVKey == bytes32(0)
                || p.nonMembershipVKey == bytes32(0) || p.updateClientVKey == p.membershipVKey
                || p.updateClientVKey == p.nonMembershipVKey
                || p.membershipVKey == p.nonMembershipVKey
        ) {
            revert InvalidProgramVKey();
        }
        if (
            p.initialHeight == 0 || p.initialStateRoot == bytes32(0)
                || p.initialBlockTimestamp == 0 || p.initialValidatorSetHash == bytes32(0)
        ) {
            revert InvalidBootstrapState();
        }

        __AccessManaged_init(authority);
        LightClientStorage storage $ = _getLightClientStorage();
        $.localIbcRouter = p.localIbcRouter;
        $.sourceIbcRouter = p.sourceIbcRouter;
        $.sp1Verifier = p.sp1Verifier;
        $.updateClientVKey = p.updateClientVKey;
        $.membershipVKey = p.membershipVKey;
        $.nonMembershipVKey = p.nonMembershipVKey;
        $.clientState = BesuZKTypes.ClientState({
            latestHeight: p.initialHeight,
            trustingPeriodSeconds: p.trustingPeriodSeconds,
            maxClockDriftSeconds: p.maxClockDriftSeconds
        });
        $.consensusStates[p.initialHeight] = BesuZKTypes.ConsensusState({
            stateRoot: p.initialStateRoot,
            blockTimestamp: p.initialBlockTimestamp,
            validatorSetHash: p.initialValidatorSetHash
        });
    }

    // Expiry/drift intentionally use destination time. The verifier is view (STATICCALL),
    // so its call tree cannot change state or emit events before ClientUpdated.
    // forge-lint: disable-next-item(block-timestamp, reentrancy-events)
    function updateClient(bytes calldata updateMsg)
        external
        override
        onlyLocalRouter
        returns (ILightClientMsgs.UpdateResult)
    {
        LightClientStorage storage $ = _getLightClientStorage();
        BesuZKTypes.SP1ProofEnvelope memory envelope = PublicValuesCodec.decodeEnvelope(updateMsg);
        BesuZKTypes.UpdatePublicValues memory values =
            PublicValuesCodec.decodeUpdate(envelope.publicValues);
        BesuZKTypes.ConsensusState memory trusted = _existingState(values.trustedHeight);

        if (values.trustedValidatorSetHash != trusted.validatorSetHash) {
            revert TrustedValidatorSetHashMismatch();
        }
        if (
            $.clientState.trustingPeriodSeconds != 0
                && uint256(trusted.blockTimestamp) + uint256($.clientState.trustingPeriodSeconds)
                    <= block.timestamp
        ) {
            revert TrustedStateExpired(values.trustedHeight);
        }
        if (values.verifiedHeight <= values.trustedHeight) {
            revert InvalidHeightTransition(values.trustedHeight, values.verifiedHeight);
        }
        if (values.blockTimestamp <= trusted.blockTimestamp) {
            revert NonIncreasingTimestamp(trusted.blockTimestamp, values.blockTimestamp);
        }
        if (
            uint256(values.blockTimestamp)
                > block.timestamp + uint256($.clientState.maxClockDriftSeconds)
        ) {
            revert TimestampTooFarInFuture(values.blockTimestamp);
        }

        ISP1Verifier($.sp1Verifier).verifyProof(
            $.updateClientVKey, envelope.publicValues, envelope.proofBytes
        );

        BesuZKTypes.ConsensusState storage current = $.consensusStates[values.verifiedHeight];
        if (_existsStorage(current)) {
            if (
                current.stateRoot == values.stateRoot
                    && current.blockTimestamp == values.blockTimestamp
                    && current.validatorSetHash == values.newValidatorSetHash
            ) {
                return ILightClientMsgs.UpdateResult.NoOp;
            }
            revert ConflictingConsensusState(values.verifiedHeight);
        }

        current.stateRoot = values.stateRoot;
        current.blockTimestamp = values.blockTimestamp;
        current.validatorSetHash = values.newValidatorSetHash;
        if (values.verifiedHeight > $.clientState.latestHeight) {
            $.clientState.latestHeight = values.verifiedHeight;
        }
        emit ClientUpdated(
            values.verifiedHeight,
            values.stateRoot,
            values.blockTimestamp,
            values.newValidatorSetHash
        );
        return ILightClientMsgs.UpdateResult.Update;
    }

    function verifyMembership(ILightClientMsgs.MsgVerifyMembership calldata msg_)
        external
        override
        onlyLocalRouter
        returns (uint256)
    {
        uint64 height = _validateHeightAndPath(
            msg_.proofHeight.revisionNumber, msg_.proofHeight.revisionHeight, msg_.path.length
        );
        BesuZKTypes.ConsensusState memory state = _existingState(height);
        if (msg_.value.length != 32) revert InvalidMembershipValueLength(msg_.value.length);
        LightClientStorage storage $ = _getLightClientStorage();
        BesuZKTypes.MembershipBatchProofEnvelope memory envelope =
            PublicValuesCodec.decodeMembershipEnvelope(msg_.proof);
        if (envelope.entryCount == 0 || envelope.entryCount > BesuZKTypes.MAX_BATCH_MESSAGES) {
            revert BatchMembershipLib.InvalidBatchEntryCount(envelope.entryCount);
        }
        bytes32 batchId = _membershipBatchId(
            height, state.stateRoot, envelope.batchRoot, envelope.entryCount, $.membershipVKey,
            $.sourceIbcRouter
        );
        bytes32 itemLeaf = BatchMembershipLib.leaf(
            keccak256(abi.encode(msg_.path)), abi.decode(msg_.value, (bytes32))
        );
        BatchMembershipLib.verify(
            envelope.batchRoot,
            itemLeaf,
            envelope.leafIndex,
            envelope.entryCount,
            envelope.merkleProof
        );
        if (envelope.mode == BesuZKTypes.MEMBERSHIP_ENVELOPE_FULL) {
            if (envelope.publicValues.length == 0 || envelope.proofBytes.length == 0) {
                revert InvalidMembershipEnvelopePayload();
            }
            BesuZKTypes.MembershipPublicValues memory values =
                PublicValuesCodec.decodeMembership(envelope.publicValues);
            if (values.proofHeight != height) {
                revert InvalidHeightTransition(height, values.proofHeight);
            }
            if (values.stateRoot != state.stateRoot) revert StateRootMismatch();
            if (values.sourceRouter != $.sourceIbcRouter) revert SourceRouterMismatch();
            if (values.batchRoot != envelope.batchRoot) revert BatchMembershipLib.InvalidMembershipBranch();
            if (values.entryCount != envelope.entryCount) {
                revert BatchMembershipLib.InvalidBatchEntryCount(values.entryCount);
            }
            ISP1Verifier($.sp1Verifier).verifyProof(
                $.membershipVKey, envelope.publicValues, envelope.proofBytes
            );
            $.verifiedMembershipBatches[batchId] = true;
            // verifyProof is view (STATICCALL); state-changing reentry is prohibited by the EVM.
            // forge-lint: disable-next-line(reentrancy-events)
            emit MembershipBatchVerified(batchId, height, envelope.batchRoot, envelope.entryCount);
        } else if (envelope.mode == BesuZKTypes.MEMBERSHIP_ENVELOPE_CACHED) {
            if (envelope.publicValues.length != 0 || envelope.proofBytes.length != 0) {
                revert InvalidMembershipEnvelopePayload();
            }
            if (!$.verifiedMembershipBatches[batchId]) revert UnknownMembershipBatch(batchId);
        } else {
            revert UnsupportedMembershipEnvelopeMode(envelope.mode);
        }
        return uint256(state.blockTimestamp);
    }

    function verifyNonMembership(ILightClientMsgs.MsgVerifyNonMembership calldata msg_)
        external
        view
        override
        onlyLocalRouter
        returns (uint256)
    {
        uint64 height = _validateHeightAndPath(
            msg_.proofHeight.revisionNumber, msg_.proofHeight.revisionHeight, msg_.path.length
        );
        BesuZKTypes.ConsensusState memory state = _existingState(height);
        BesuZKTypes.SP1ProofEnvelope memory envelope = PublicValuesCodec.decodeEnvelope(msg_.proof);
        BesuZKTypes.NonMembershipPublicValues memory values =
            PublicValuesCodec.decodeNonMembership(envelope.publicValues);

        _bindCommonMembershipValues(
            values.proofHeight,
            values.stateRoot,
            values.sourceRouter,
            values.pathDigest,
            height,
            state.stateRoot,
            msg_.path
        );
        LightClientStorage storage $ = _getLightClientStorage();
        ISP1Verifier($.sp1Verifier).verifyProof(
            $.nonMembershipVKey, envelope.publicValues, envelope.proofBytes
        );
        return uint256(state.blockTimestamp);
    }

    function misbehaviour(bytes calldata) external pure override {
        revert UnsupportedMisbehaviour();
    }

    function getClientState() external view override returns (bytes memory) {
        return abi.encode(_getLightClientStorage().clientState);
    }

    function latestHeight() external view override returns (uint64) {
        return _getLightClientStorage().clientState.latestHeight;
    }

    function trustingPeriodSeconds() external view override returns (uint64) {
        return _getLightClientStorage().clientState.trustingPeriodSeconds;
    }

    function maxClockDriftSeconds() external view override returns (uint64) {
        return _getLightClientStorage().clientState.maxClockDriftSeconds;
    }

    // solhint-disable func-name-mixedcase
    // The getter names of the fields' earlier immutable form.

    function LOCAL_IBC_ROUTER() external view returns (address) {
        return _getLightClientStorage().localIbcRouter;
    }

    function SOURCE_IBC_ROUTER() external view returns (address) {
        return _getLightClientStorage().sourceIbcRouter;
    }

    function SP1_VERIFIER() external view returns (address) {
        return _getLightClientStorage().sp1Verifier;
    }

    function UPDATE_CLIENT_VKEY() external view returns (bytes32) {
        return _getLightClientStorage().updateClientVKey;
    }

    function MEMBERSHIP_VKEY() external view returns (bytes32) {
        return _getLightClientStorage().membershipVKey;
    }

    function NON_MEMBERSHIP_VKEY() external view returns (bytes32) {
        return _getLightClientStorage().nonMembershipVKey;
    }

    // solhint-enable func-name-mixedcase

    function getConsensusState(uint64 height)
        external
        view
        override
        returns (bool found, BesuZKTypes.ConsensusState memory state)
    {
        state = _getLightClientStorage().consensusStates[height];
        found = _existsMemory(state);
    }

    function isMembershipBatchVerified(bytes32 batchId) external view override returns (bool) {
        return _getLightClientStorage().verifiedMembershipBatches[batchId];
    }

    function membershipBatchId(uint64 height, bytes32 batchRoot, uint32 entryCount)
        external
        view
        returns (bytes32)
    {
        LightClientStorage storage $ = _getLightClientStorage();
        BesuZKTypes.ConsensusState memory state = _existingState(height);
        return _membershipBatchId(
            height, state.stateRoot, batchRoot, entryCount, $.membershipVKey, $.sourceIbcRouter
        );
    }

    function _validateHeightAndPath(
        uint64 revisionNumber,
        uint64 revisionHeight,
        uint256 pathLength
    ) private pure returns (uint64) {
        if (revisionNumber != 0) revert UnsupportedRevisionNumber(revisionNumber);
        if (pathLength != 1) revert InvalidPathLength(pathLength);
        return revisionHeight;
    }

    function _bindCommonMembershipValues(
        uint64 proofHeight,
        bytes32 stateRoot,
        address sourceRouter,
        bytes32 pathDigest,
        uint64 expectedHeight,
        bytes32 expectedStateRoot,
        bytes[] calldata path
    ) private view {
        if (proofHeight != expectedHeight) {
            revert InvalidHeightTransition(expectedHeight, proofHeight);
        }
        if (stateRoot != expectedStateRoot) revert StateRootMismatch();
        if (sourceRouter != _getLightClientStorage().sourceIbcRouter) {
            revert SourceRouterMismatch();
        }
        if (pathDigest != keccak256(abi.encode(path))) revert PathDigestMismatch();
    }

    function _membershipBatchId(
        uint64 height,
        bytes32 stateRoot,
        bytes32 batchRoot,
        uint32 entryCount,
        bytes32 membershipVKey,
        address sourceRouter
    ) private pure returns (bytes32) {
        return keccak256(
            abi.encode(
                uint32(2),
                height,
                stateRoot,
                sourceRouter,
                batchRoot,
                entryCount,
                membershipVKey
            )
        );
    }

    function _existingState(uint64 height)
        private
        view
        returns (BesuZKTypes.ConsensusState memory state)
    {
        state = _getLightClientStorage().consensusStates[height];
        if (!_existsMemory(state)) revert ConsensusStateNotFound(height);
    }

    function _existsMemory(BesuZKTypes.ConsensusState memory state) private pure returns (bool) {
        return state.blockTimestamp != 0;
    }

    function _existsStorage(BesuZKTypes.ConsensusState storage state) private view returns (bool) {
        return state.blockTimestamp != 0;
    }

    /// @dev The authority's `canCall` for the entry point, `upgradeToAndCall`.
    function _authorizeUpgrade(address) internal override restricted { }
    // solhint-disable-previous-line no-empty-blocks

    function _getLightClientStorage() private pure returns (LightClientStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := LIGHT_CLIENT_STORAGE_SLOT
        }
    }
}
