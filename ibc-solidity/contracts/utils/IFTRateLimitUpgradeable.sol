// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTMsgs } from "../msgs/IIFTMsgs.sol";
import { IIFTRateLimit } from "../interfaces/IIFTRateLimit.sol";
import { IIFTErrors } from "../errors/IIFTErrors.sol";

import { RateLimiter } from "@openzeppelin-contracts/utils/RateLimiter.sol";

/// @title IFT Rate Limit Upgradeable
/// @notice Abstract contract that rate limits inbound mints and outbound burns of an IFT contract
/// @dev Each direction is an independent refilling bucket shared across all bridges of the token. Usage is gross
/// per direction: flow in one direction never restores allowance in the other, and refunds of pending transfers
/// consume inbound allowance like any other mint. See the IFT rate limiting ADR for the rationale.
abstract contract IFTRateLimitUpgradeable is IIFTErrors, IIFTRateLimit {
    using RateLimiter for RateLimiter.RefillingBucket;

    /// @notice Storage of the IFTRateLimit contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @dev Inbound and outbound are separate limiters rather than two keys of one limiter because a limiter shares
    /// its capacity and window across all of its keys, and each direction needs its own capacity.
    /// @param _inbound The bucket consumed by mints received from counterparty chains
    /// @param _outbound The bucket consumed by transfers sent to counterparty chains
    struct IFTRateLimitStorage {
        RateLimiter.RefillingBucket _inbound;
        RateLimiter.RefillingBucket _outbound;
    }

    /// @notice ERC-7201 slot for the IFTRateLimit storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.IFTRateLimit")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant IFT_RATELIMIT_STORAGE_SLOT =
        0x74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00;

    /// @notice The single entry used in each bucket, since limits are per token rather than per bridge
    bytes32 private constant BUCKET_KEY = bytes32(0);

    /// @inheritdoc IIFTRateLimit
    function getIFTRateLimit(IIFTMsgs.IFTRateLimitDirection direction)
        external
        view
        returns (uint208 capacity, uint48 window)
    {
        RateLimiter.RefillingBucket storage bucket = _getBucket(direction);
        return (bucket._capacity, bucket._window);
    }

    /// @inheritdoc IIFTRateLimit
    function getIFTRateLimitAvailable(IIFTMsgs.IFTRateLimitDirection direction) external view returns (uint256) {
        return _getBucket(direction).available(BUCKET_KEY);
    }

    /// @notice Sets the rate limit of a direction
    /// @dev The bucket is synced before the settings change so that the refill accrued under the old rate is kept
    /// and the new rate only applies from now on. Lowering the capacity below the current usage leaves the bucket
    /// empty until it refills.
    /// @param direction The direction the rate limit applies to
    /// @param capacity The maximum amount that can pass while the bucket is full
    /// @param window The number of seconds it takes for an empty bucket to refill completely
    function _setIFTRateLimit(IIFTMsgs.IFTRateLimitDirection direction, uint208 capacity, uint48 window) internal {
        RateLimiter.RefillingBucket storage bucket = _getBucket(direction);
        bucket.sync(BUCKET_KEY);
        bucket.updateSettings(window, capacity);

        emit IFTRateLimitSet(direction, capacity, window);
    }

    /// @notice Consumes `amount` from the bucket of a direction
    /// @param direction The direction the amount flows in
    /// @param amount The amount to consume
    function _consumeIFTRateLimit(IIFTMsgs.IFTRateLimitDirection direction, uint256 amount) internal {
        RateLimiter.RefillingBucket storage bucket = _getBucket(direction);
        bucket.consume(BUCKET_KEY, amount);
    }

    /// @notice Returns the bucket of a direction
    /// @param direction The direction of the bucket
    /// @return The bucket
    function _getBucket(IIFTMsgs.IFTRateLimitDirection direction)
        private
        view
        returns (RateLimiter.RefillingBucket storage)
    {
        IFTRateLimitStorage storage $ = _getIFTRateLimitStorage();
        if (direction == IIFTMsgs.IFTRateLimitDirection.Inbound) {
            return $._inbound;
        } else if (direction == IIFTMsgs.IFTRateLimitDirection.Outbound) {
            return $._outbound;
        }
        revert IFTUnknownRateLimitDirection(direction);
    }

    /// @notice Returns the storage of the IFTRateLimit contract
    /// @return $ The storage of the IFTRateLimit contract
    function _getIFTRateLimitStorage() private pure returns (IFTRateLimitStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := IFT_RATELIMIT_STORAGE_SLOT
        }
    }
}
