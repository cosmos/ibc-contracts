// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import { Integration2TestBase } from "../solidity-ibc/Integration2Test.t.sol";
import { Integration3TestBase } from "../solidity-ibc/Integration3Test.t.sol";
import { IFTIntegrationTestBase } from "../solidity-ibc/IFTIntegrationTest.t.sol";
import { WithBesuQBFTSim, WithBesuIBFT2Sim } from "./utils/BesuSimDriver.sol";

// The integration suites on Besu light clients tracking QBFT and IBFT2 sim chains. Every relayed packet is proven
// with account and storage proofs against a sealed header, so fuzz runs are capped to keep the suites fast.

/// forge-config: default.fuzz.runs = 256
contract Integration2BesuQBFTTest is Integration2TestBase, WithBesuQBFTSim { }

/// forge-config: default.fuzz.runs = 256
contract Integration2BesuIBFT2Test is Integration2TestBase, WithBesuIBFT2Sim { }

/// forge-config: default.fuzz.runs = 256
contract Integration3BesuQBFTTest is Integration3TestBase, WithBesuQBFTSim { }

/// forge-config: default.fuzz.runs = 256
contract Integration3BesuIBFT2Test is Integration3TestBase, WithBesuIBFT2Sim { }

/// forge-config: default.fuzz.runs = 256
contract IFTIntegrationBesuQBFTTest is IFTIntegrationTestBase, WithBesuQBFTSim { }

/// forge-config: default.fuzz.runs = 256
contract IFTIntegrationBesuIBFT2Test is IFTIntegrationTestBase, WithBesuIBFT2Sim { }
