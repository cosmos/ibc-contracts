// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length

import { Test } from "forge-std/Test.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";

contract ICS20LibTest is Test {
    function test_hasHops() public pure {
        assert(ICS20Lib.hasHops("transfer/client-0/uatom"));
        assert(!ICS20Lib.hasHops("uatom"));
    }
}
