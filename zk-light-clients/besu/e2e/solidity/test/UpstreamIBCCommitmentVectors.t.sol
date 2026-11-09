// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";
import { IICS20TransferMsgs } from "@ibc-contracts/msgs/IICS20TransferMsgs.sol";
import { IICS26RouterMsgs } from "@ibc-contracts/msgs/IICS26RouterMsgs.sol";
import { ICS20Lib } from "@ibc-contracts/utils/ICS20Lib.sol";
import { ICS24Host } from "@ibc-contracts/utils/ICS24Host.sol";

contract UpstreamIBCCommitmentVectorsTest is Test {
    function testPinnedUpstreamTerminalPathVectors() public pure {
        assertEq(
            ICS24Host.packetCommitmentPathCalldata("client-0", 1),
            hex"636c69656e742d30010000000000000001"
        );
        assertEq(
            ICS24Host.packetReceiptCommitmentPathCalldata("client-1", 2),
            hex"636c69656e742d31020000000000000002"
        );
        assertEq(
            ICS24Host.packetAcknowledgementCommitmentPathCalldata("client-2", 3),
            hex"636c69656e742d32030000000000000003"
        );
    }

    function testPinnedUpstreamPacketCommitmentVector() public pure {
        IICS20TransferMsgs.FungibleTokenPacketData memory packetData =
            IICS20TransferMsgs.FungibleTokenPacketData({
                denom: "uatom",
                amount: 1_000_000,
                sender: "sender",
                receiver: "receiver",
                memo: "memo"
            });
        IICS26RouterMsgs.Payload[] memory payloads = new IICS26RouterMsgs.Payload[](1);
        payloads[0] = IICS26RouterMsgs.Payload({
            sourcePort: ICS20Lib.DEFAULT_PORT_ID,
            destPort: ICS20Lib.DEFAULT_PORT_ID,
            version: ICS20Lib.ICS20_VERSION,
            encoding: ICS20Lib.ICS20_ENCODING,
            value: abi.encode(packetData)
        });
        IICS26RouterMsgs.Packet memory packet = IICS26RouterMsgs.Packet({
            sequence: 1,
            sourceClient: "client-0",
            destClient: "client-1",
            timeoutTimestamp: 100,
            payloads: payloads
        });

        assertEq(
            ICS24Host.packetCommitmentBytes32(packet),
            0x2e6c14ba0b7da7433158ba2356eeb1dd6ecde71a9f69ce0f2d9c5e38073146eb
        );
    }

    function testPinnedUpstreamAcknowledgementCommitmentVector() public pure {
        bytes[] memory acknowledgements = new bytes[](1);
        acknowledgements[0] = bytes("some bytes");
        assertEq(
            ICS24Host.packetAcknowledgementCommitmentBytes32(acknowledgements),
            0xf03b4667413e56aaf086663267913e525c442b56fa1af4fa3f3dab9f37044c5b
        );
    }
}
