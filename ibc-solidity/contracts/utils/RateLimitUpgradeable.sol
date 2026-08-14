// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IRateLimitErrors } from "../errors/IRateLimitErrors.sol";
import { IRateLimit } from "../interfaces/IRateLimit.sol";

import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";

/// @title Rate Limit Upgradeable contract
/// @notice This contract is an abstract contract for adding rate limiting to escrow contracts.
/// @dev Rate limits are set per token address by the rate limiter role and are enforced per day.
/// @dev Rate limits are applied to tokens leaving the escrow contract.
abstract contract RateLimitUpgradeable is IRateLimitErrors, IRateLimit, AccessManagedUpgradeable {
    /// @notice Storage of the RateLimit contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _rateLimits Mapping of token addresses to their rate limits, 0 means no limit
    /// @param _dailyUsage Mapping of daily token keys to their usage
    struct RateLimitStorage {
        mapping(address token => uint256 limit) _rateLimits;
        mapping(bytes32 dailyTokenKey => uint256 usage) _dailyUsage;
    }

    /// @notice ERC-7201 slot for the RateLimit storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.RateLimit")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant RATELIMIT_STORAGE_SLOT =
        0xcb05b6cb8e6c87c443cb04d44193d7d46d51c1198725a0ee3478d5baa736c100;

    /// @notice The period for rate limiting
    uint256 private constant RATE_LIMIT_PERIOD = 1 days;

    /// @notice The initializer for the RateLimit contract
    /// @param authority_ The address of the AccessManager contract
    function __RateLimit_init(address authority_) internal onlyInitializing {
        __AccessManaged_init(authority_);
    }

    /// @inheritdoc IRateLimit
    function setRateLimit(address token, uint256 rateLimit) external restricted {
        _getRateLimitStorage()._rateLimits[token] = rateLimit;
    }

    /// @inheritdoc IRateLimit
    function getRateLimit(address token) external view returns (uint256) {
        return _getRateLimitStorage()._rateLimits[token];
    }

    /// @inheritdoc IRateLimit
    function getDailyUsage(address token) external view returns (uint256) {
        return _getRateLimitStorage()._dailyUsage[_getDailyTokenKey(token)];
    }

    /// @notice Checks the rate limit for a token and updates the daily usage
    /// @param token The token address
    /// @param amount The amount to check against the rate limit
    function _assertAndUpdateRateLimit(address token, uint256 amount) internal {
        RateLimitStorage storage $ = _getRateLimitStorage();

        uint256 rateLimit = $._rateLimits[token];
        if (rateLimit == 0) {
            return;
        }

        bytes32 dailyTokenKey = _getDailyTokenKey(token);
        uint256 usage = $._dailyUsage[dailyTokenKey] + amount;
        // solhint-disable-next-line gas-strict-inequalities
        require(usage <= rateLimit, RateLimitExceeded(rateLimit, usage));

        $._dailyUsage[dailyTokenKey] = usage;
    }

    /// @notice Reduces the daily usage for a token
    /// @dev This function is used in order to track the net usage a token
    /// @param token The token address
    /// @param amount The amount to reduce from the daily usage
    function _reduceDailyUsage(address token, uint256 amount) internal {
        RateLimitStorage storage $ = _getRateLimitStorage();

        uint256 rateLimit = $._rateLimits[token];
        if (rateLimit == 0) {
            return;
        }

        bytes32 dailyTokenKey = _getDailyTokenKey(token);
        uint256 usage = $._dailyUsage[dailyTokenKey];
        if (usage > amount) {
            $._dailyUsage[dailyTokenKey] = usage - amount;
        } else {
            $._dailyUsage[dailyTokenKey] = 0;
        }
    }

    /// @notice Returns the daily token key for the current timestamp and token
    /// @param token The token address
    /// @return The daily token key
    function _getDailyTokenKey(address token) internal view returns (bytes32) {
        return keccak256(abi.encodePacked(block.timestamp / RATE_LIMIT_PERIOD, token));
    }

    /// @notice Returns the storage of the RateLimit contract
    /// @return $ The storage of the RateLimit contract
    function _getRateLimitStorage() internal pure returns (RateLimitStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := RATELIMIT_STORAGE_SLOT
        }
    }
}
