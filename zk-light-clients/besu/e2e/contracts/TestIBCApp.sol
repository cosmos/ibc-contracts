// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { IIBCApp } from "@ibc-contracts/interfaces/IIBCApp.sol";
import { IICS26Router } from "@ibc-contracts/interfaces/IICS26Router.sol";
import { IIBCAppCallbacks } from "@ibc-contracts/msgs/IIBCAppCallbacks.sol";
import { IICS26RouterMsgs } from "@ibc-contracts/msgs/IICS26RouterMsgs.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";

contract TestIBCApp is IIBCApp {
    error UnauthorizedRouter(address caller);

    address public immutable IBC_ROUTER;
    bytes public lastPayload;
    bytes public lastAcknowledgement;
    uint64 public lastSequence;
    bool public timedOut;

    constructor(address ibcRouter) {
        IBC_ROUTER = ibcRouter;
    }

    modifier onlyRouter() {
        if (msg.sender != IBC_ROUTER) revert UnauthorizedRouter(msg.sender);
        _;
    }

    function sendPacket(
        string calldata sourceClient,
        string calldata destinationPort,
        uint64 timeoutTimestamp,
        bytes calldata payload
    ) external returns (uint64) {
        return IICS26Router(IBC_ROUTER).sendPacket(
            IICS26RouterMsgs.MsgSendPacket({
                sourceClient: sourceClient,
                timeoutTimestamp: timeoutTimestamp,
                payload: IICS26RouterMsgs.Payload({
                    sourcePort: Strings.toHexString(address(this)),
                    destPort: destinationPort,
                    version: "besu-zk-e2e-v1",
                    encoding: "application/octet-stream",
                    value: payload
                })
            })
        );
    }

    function onRecvPacket(IIBCAppCallbacks.OnRecvPacketCallback calldata msg_)
        external
        onlyRouter
        returns (bytes memory)
    {
        lastPayload = msg_.payload.value;
        lastSequence = msg_.sequence;
        return abi.encodePacked("ack:", msg_.payload.value);
    }

    function onAcknowledgementPacket(
        IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_
    ) external onlyRouter {
        lastAcknowledgement = msg_.acknowledgement;
        lastSequence = msg_.sequence;
    }

    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_)
        external
        onlyRouter
    {
        timedOut = true;
        lastSequence = msg_.sequence;
    }
}
