// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import { Integration2Suite } from "./integration/suites/Integration2Suite.sol";
import { Integration3Suite } from "./integration/suites/Integration3Suite.sol";
import { IFTIntegrationSuite } from "./integration/suites/IFTIntegrationSuite.sol";
import { WithSolidityLightClient } from "./integration/drivers/SolidityLightClientDriver.sol";

// The integration suites on `SolidityLightClient`, which reads the counterparty router directly.

contract Integration2Test is Integration2Suite, WithSolidityLightClient { }

contract Integration3Test is Integration3Suite, WithSolidityLightClient { }

contract IFTIntegrationTest is IFTIntegrationSuite, WithSolidityLightClient { }
