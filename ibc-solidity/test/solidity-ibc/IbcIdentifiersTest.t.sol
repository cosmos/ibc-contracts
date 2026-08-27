// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable max-line-length,gas-custom-errors,gas-small-strings

import { Test } from "forge-std/Test.sol";
import { IBCIdentifiers } from "../../contracts/utils/IBCIdentifiers.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";

contract IBCIdentifiersTest is Test {
    struct ValidateCustomIBCIdentifierTestCase {
        string m;
        string id;
        bool expPass;
    }

    function test_validateCustomIBCIdentifier() public {
        // The following test cases are based on the test cases of ibc-go:
        // https://github.com/cosmos/ibc-go/blob/e443a88e0f2c84c131c5a1de47945a5733ff9c91/modules/core/24-host/validate_test.go#L57
        ValidateCustomIBCIdentifierTestCase[] memory testCases = new ValidateCustomIBCIdentifierTestCase[](15);
        testCases[0] = ValidateCustomIBCIdentifierTestCase({ m: "valid lowercase", id: "transfer", expPass: true });
        testCases[1] =
            ValidateCustomIBCIdentifierTestCase({
            m: "valid id special chars", id: "._+-#[]<>._+-#[]<>", expPass: true
        });
        testCases[2] = ValidateCustomIBCIdentifierTestCase({
            m: "valid id lower and special chars", id: "lower._+-#[]<>", expPass: true
        });
        testCases[3] = ValidateCustomIBCIdentifierTestCase({ m: "numeric id", id: "1234567890", expPass: true });
        testCases[4] = ValidateCustomIBCIdentifierTestCase({ m: "uppercase id", id: "NOTLOWERCASE", expPass: true });
        testCases[5] = ValidateCustomIBCIdentifierTestCase({ m: "numeric id", id: "1234567890", expPass: true });
        testCases[6] = ValidateCustomIBCIdentifierTestCase({ m: "blank id", id: "               ", expPass: false });
        testCases[7] = ValidateCustomIBCIdentifierTestCase({ m: "id length out of range", id: "1", expPass: false });
        testCases[8] = ValidateCustomIBCIdentifierTestCase({
            m: "id is too long",
            id: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Duis eros neque, ultricies vel ligula ac, convallis porttitor elit. Maecenas tincidunt turpis elit, vel faucibus nisl pellentesque sodales",
            expPass: false
        });
        testCases[9] = ValidateCustomIBCIdentifierTestCase({ m: "path-like id", id: "lower/case/id", expPass: false });
        testCases[10] = ValidateCustomIBCIdentifierTestCase({ m: "invalid id", id: "(clientid)", expPass: false });
        testCases[11] = ValidateCustomIBCIdentifierTestCase({ m: "empty string", id: "", expPass: false });
        testCases[12] = ValidateCustomIBCIdentifierTestCase({ m: "client prefix id", id: "client-5", expPass: false });
        testCases[13] = ValidateCustomIBCIdentifierTestCase({ m: "channel prefix id", id: "channel-0", expPass: false });
        testCases[14] = ValidateCustomIBCIdentifierTestCase({
            m: "contract address", id: Strings.toHexString(makeAddr("test")), expPass: true
        });

        for (uint256 i = 0; i < testCases.length; ++i) {
            ValidateCustomIBCIdentifierTestCase memory tc = testCases[i];
            bool res = IBCIdentifiers.validateCustomIBCIdentifier(bytes(tc.id));
            if (tc.expPass) {
                require(res, tc.m);
            } else {
                require(!res, tc.m);
            }
        }
    }
}
