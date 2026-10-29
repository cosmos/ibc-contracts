// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import { Integration2Suite } from "../solidity-ibc/integration/suites/Integration2Suite.sol";
import { Integration3Suite } from "../solidity-ibc/integration/suites/Integration3Suite.sol";
import { IFTIntegrationSuite } from "../solidity-ibc/integration/suites/IFTIntegrationSuite.sol";
import { WithSolidityLightClient } from "../solidity-ibc/integration/drivers/SolidityLightClientDriver.sol";
import { WithMainnetFork } from "./utils/MainnetIbcImpl.sol";

// The integration suites on a mainnet fork, with the upgraded mainnet deployment in every slot and fresh deployments
// as its counterparties. Every new account a fuzz run touches is fetched over RPC, so fuzz runs are capped.

/// forge-config: default.fuzz.runs = 32
contract Integration2MainnetATest is Integration2Suite, WithSolidityLightClient, WithMainnetFork(0) { }

/// forge-config: default.fuzz.runs = 32
contract Integration2MainnetBTest is Integration2Suite, WithSolidityLightClient, WithMainnetFork(1) { }

/// forge-config: default.fuzz.runs = 32
contract Integration3MainnetATest is Integration3Suite, WithSolidityLightClient, WithMainnetFork(0) { }

/// forge-config: default.fuzz.runs = 32
contract Integration3MainnetBTest is Integration3Suite, WithSolidityLightClient, WithMainnetFork(1) { }

/// forge-config: default.fuzz.runs = 32
contract Integration3MainnetCTest is Integration3Suite, WithSolidityLightClient, WithMainnetFork(2) { }

/// forge-config: default.fuzz.runs = 32
contract IFTIntegrationMainnetATest is IFTIntegrationSuite, WithSolidityLightClient, WithMainnetFork(0) { }

/// forge-config: default.fuzz.runs = 32
contract IFTIntegrationMainnetBTest is IFTIntegrationSuite, WithSolidityLightClient, WithMainnetFork(1) { }
