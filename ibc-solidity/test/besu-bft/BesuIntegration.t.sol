// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import { Integration2Suite } from "../solidity-ibc/integration/suites/Integration2Suite.sol";
import { Integration3Suite } from "../solidity-ibc/integration/suites/Integration3Suite.sol";
import { IFTIntegrationSuite } from "../solidity-ibc/integration/suites/IFTIntegrationSuite.sol";
import { WithBesuQBFTSim, WithBesuIBFT2Sim } from "./utils/BesuSimDriver.sol";

// The integration suites on Besu light clients tracking QBFT and IBFT2 sim chains. Every relayed packet is proven
// with account and storage proofs against a sealed header, so fuzz runs are capped to keep the suites fast.

/// forge-config: default.fuzz.runs = 256
contract Integration2BesuQBFTTest is Integration2Suite, WithBesuQBFTSim { }

/// forge-config: default.fuzz.runs = 256
contract Integration2BesuIBFT2Test is Integration2Suite, WithBesuIBFT2Sim { }

/// forge-config: default.fuzz.runs = 256
contract Integration3BesuQBFTTest is Integration3Suite, WithBesuQBFTSim { }

/// forge-config: default.fuzz.runs = 256
contract Integration3BesuIBFT2Test is Integration3Suite, WithBesuIBFT2Sim { }

/// forge-config: default.fuzz.runs = 256
contract IFTIntegrationBesuQBFTTest is IFTIntegrationSuite, WithBesuQBFTSim { }

/// forge-config: default.fuzz.runs = 256
contract IFTIntegrationBesuIBFT2Test is IFTIntegrationSuite, WithBesuIBFT2Sim { }
