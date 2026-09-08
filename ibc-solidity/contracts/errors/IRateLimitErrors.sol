// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IRateLimitErrors
/// @notice Interface for rate limit errors
interface IRateLimitErrors {
    /// @notice Rate limit exceeded
    /// @param rateLimit The rate limit of the token
    /// @param usage The amount used so far
    error RateLimitExceeded(uint256 rateLimit, uint256 usage);
}
