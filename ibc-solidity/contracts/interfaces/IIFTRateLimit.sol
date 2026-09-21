// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-indexed-events

import { IIFTMsgs } from "../msgs/IIFTMsgs.sol";

/// @title IIFTRateLimit
/// @notice Interface for the per token, per direction rate limits of an IFT contract
/// @dev Each direction is a refilling bucket with a fixed capacity that refills linearly over `window` seconds.
/// Rate limits are mandatory: until a direction is configured, its bucket is empty and every transfer in that
/// direction reverts.
interface IIFTRateLimit {
    /// @notice Emitted when the rate limit of a direction is set
    /// @param direction The direction the rate limit applies to
    /// @param capacity The maximum amount that can pass while the bucket is full
    /// @param window The number of seconds it takes for an empty bucket to refill completely
    event IFTRateLimitSet(IIFTMsgs.IFTRateLimitDirection direction, uint208 capacity, uint48 window);

    /// @notice Sets the rate limit of a direction
    /// @dev Only callable by the authority. The refill accrued under the previous settings is applied before the
    /// new settings take effect, so the new rate is not applied retroactively and consumed usage is preserved.
    /// @param direction The direction the rate limit applies to
    /// @param capacity The maximum amount that can pass while the bucket is full
    /// @param window The number of seconds it takes for an empty bucket to refill completely
    function setIFTRateLimit(IIFTMsgs.IFTRateLimitDirection direction, uint208 capacity, uint48 window) external;

    /// @notice Returns the configured rate limit of a direction
    /// @param direction The direction the rate limit applies to
    /// @return capacity The maximum amount that can pass while the bucket is full
    /// @return window The number of seconds it takes for an empty bucket to refill completely
    function getIFTRateLimit(IIFTMsgs.IFTRateLimitDirection direction)
        external
        view
        returns (uint208 capacity, uint48 window);

    /// @notice Returns the amount that can currently pass in a direction
    /// @param direction The direction the rate limit applies to
    /// @return The available amount, accounting for the refill accrued since the last update
    function getIFTRateLimitAvailable(IIFTMsgs.IFTRateLimitDirection direction) external view returns (uint256);
}
