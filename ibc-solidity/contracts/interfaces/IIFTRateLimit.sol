// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTMsgs } from "../msgs/IIFTMsgs.sol";

/// @title IIFTRateLimit
/// @notice Interface for the per token rate limits of an IFT contract
/// @dev Each direction is a refilling bucket that refills linearly over `window` seconds. Both directions share the
/// same capacity and window but track their usage independently. Rate limits are mandatory: until they are
/// configured, both buckets are empty and every transfer reverts.
interface IIFTRateLimit {
    /// @notice Emitted when the rate limit is set
    /// @param capacity The maximum amount that can pass in each direction while its bucket is full
    /// @param window The number of seconds it takes for an empty bucket to refill completely
    // solhint-disable-next-line gas-indexed-events
    event IFTRateLimitSet(uint208 capacity, uint48 window);

    /// @notice Sets the rate limit shared by both directions
    /// @dev Only callable by the authority. The refill accrued under the previous settings is applied before the
    /// new settings take effect, so the new rate is not applied retroactively and consumed usage is preserved.
    /// @param capacity The maximum amount that can pass in each direction while its bucket is full
    /// @param window The number of seconds it takes for an empty bucket to refill completely
    function setIFTRateLimit(uint208 capacity, uint48 window) external;

    /// @notice Returns the configured rate limit shared by both directions
    /// @return capacity The maximum amount that can pass in each direction while its bucket is full
    /// @return window The number of seconds it takes for an empty bucket to refill completely
    function getIFTRateLimit() external view returns (uint208 capacity, uint48 window);

    /// @notice Returns the amount that can currently pass in a direction
    /// @param direction The direction the rate limit applies to
    /// @return The available amount, accounting for the refill accrued since the last update
    function getIFTRateLimitAvailable(IIFTMsgs.IFTRateLimitDirection direction) external view returns (uint256);
}
