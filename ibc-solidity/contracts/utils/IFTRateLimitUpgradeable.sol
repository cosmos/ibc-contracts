// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTMsgs } from "../msgs/IIFTMsgs.sol";
import { IIFTRateLimit } from "../interfaces/IIFTRateLimit.sol";

import { RateLimiter } from "@openzeppelin-contracts/utils/RateLimiter.sol";

/// @title IFT Rate Limit Upgradeable
/// @notice Abstract contract that rate limits inbound mints and outbound burns of an IFT contract
/// @dev A single refilling bucket limiter shared across all bridges of the token, with one entry per direction.
/// Both directions share the same capacity and window but track their usage independently. Usage is gross per
/// direction: flow in one direction never restores allowance in the other, and refunds of pending transfers consume
/// inbound allowance like any other mint. See the IFT rate limiting ADR for the rationale.
abstract contract IFTRateLimitUpgradeable is IIFTRateLimit {
    using RateLimiter for RateLimiter.RefillingBucket;

    /// @notice Storage of the IFTRateLimit contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _bucket The limiter, keyed by direction
    struct IFTRateLimitStorage {
        RateLimiter.RefillingBucket _bucket;
    }

    /// @notice ERC-7201 slot for the IFTRateLimit storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.IFTRateLimit")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant IFT_RATELIMIT_STORAGE_SLOT =
        0x74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00;

    /// @notice Prefix hashed with a direction to derive its limiter key
    string private constant DIRECTION_KEY_PREFIX = "ibc.ift.ratelimit.direction";

    /// @inheritdoc IIFTRateLimit
    function getIFTRateLimit() external view returns (uint208 capacity, uint48 window) {
        RateLimiter.RefillingBucket storage bucket = _getIFTRateLimitStorage()._bucket;
        return (bucket._capacity, bucket._window);
    }

    /// @inheritdoc IIFTRateLimit
    function getIFTRateLimitAvailable(IIFTMsgs.IFTRateLimitDirection direction) external view returns (uint256) {
        return _getIFTRateLimitStorage()._bucket.available(_directionKey(direction));
    }

    /// @notice Sets the rate limit shared by both directions
    /// @dev Both directions are synced before the settings change so that the refill accrued under the old rate is
    /// kept and the new rate only applies from now on. Lowering the capacity below the current usage of a direction
    /// leaves it empty until it refills.
    /// @param capacity The maximum amount that can pass in each direction while its bucket is full
    /// @param window The number of seconds it takes for an empty bucket to refill completely
    function _setIFTRateLimit(uint208 capacity, uint48 window) internal {
        RateLimiter.RefillingBucket storage bucket = _getIFTRateLimitStorage()._bucket;
        bucket.sync(_directionKey(IIFTMsgs.IFTRateLimitDirection.Inbound));
        bucket.sync(_directionKey(IIFTMsgs.IFTRateLimitDirection.Outbound));
        bucket.updateSettings(window, capacity);

        emit IFTRateLimitSet(capacity, window);
    }

    /// @notice Consumes `amount` from the bucket of a direction
    /// @param direction The direction the amount flows in
    /// @param amount The amount to consume
    function _consumeIFTRateLimit(IIFTMsgs.IFTRateLimitDirection direction, uint256 amount) internal {
        _getIFTRateLimitStorage()._bucket.consume(_directionKey(direction), amount);
    }

    /// @notice Returns the limiter key of a direction
    /// @dev The key is derived from the enum value, so `IFTRateLimitDirection` values must only be appended, never
    /// reordered, or stored usage would be remapped across an upgrade.
    /// @param direction The direction
    /// @return The limiter key
    function _directionKey(IIFTMsgs.IFTRateLimitDirection direction) private pure returns (bytes32) {
        return keccak256(abi.encodePacked(DIRECTION_KEY_PREFIX, direction));
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
