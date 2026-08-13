// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IRateLimit
/// @notice Interface for a rate limiting contract that manages token usage limits
interface IRateLimit {
    /// @notice Sets the rate limit for a token
    /// @dev The caller must have the rate limiter role
    /// @param token The token address
    /// @param rateLimit The rate limit to set
    function setRateLimit(address token, uint256 rateLimit) external;

    /// @notice Gets the rate limit for a token
    /// @param token The token address
    /// @return The rate limit for the token
    function getRateLimit(address token) external view returns (uint256);

    /// @notice Gets a token's actual usage for the current date
    /// @param token The token address
    /// @return The daily usage for the token
    function getDailyUsage(address token) external view returns (uint256);
}
