// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,max-line-length,max-states-count,immutable-vars-naming

// This is a helper to deploy the IBC implementation for testing purposes

import {Test} from "forge-std/Test.sol";

import {IICS02ClientMsgs} from "../../../contracts/msgs/IICS02ClientMsgs.sol";
import {IICS26RouterMsgs} from "../../../contracts/msgs/IICS26RouterMsgs.sol";
import {IICS20TransferMsgs} from "../../../contracts/msgs/IICS20TransferMsgs.sol";
import {IICS27GMPMsgs} from "../../../contracts/msgs/IICS27GMPMsgs.sol";

import {IERC20} from "@openzeppelin-contracts/token/ERC20/IERC20.sol";
import {IICS26Router} from "../../../contracts/interfaces/IICS26Router.sol";
import {ISignatureTransfer} from "@uniswap/permit2/src/interfaces/ISignatureTransfer.sol";

import {ICS26Router} from "../../../contracts/ICS26Router.sol";
import {IBCERC20} from "../../../contracts/utils/IBCERC20.sol";
import {Escrow} from "../../../contracts/utils/Escrow.sol";
import {ICS20Transfer} from "../../../contracts/ICS20Transfer.sol";
import {ICS27Lib} from "../../../contracts/utils/ICS27Lib.sol";
import {ICS27GMP} from "../../../contracts/ICS27GMP.sol";
import {ICS27Account} from "../../../contracts/utils/ICS27Account.sol";
import {TestHelper} from "./TestHelper.sol";
import {SolidityLightClient} from "../utils/SolidityLightClient.sol";
import {ICS20Lib} from "../../../contracts/utils/ICS20Lib.sol";
import {ERC1967Proxy} from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {ICS24Host} from "../../../contracts/utils/ICS24Host.sol";
import {RelayerHelper} from "../../../contracts/utils/RelayerHelper.sol";
import {DeployAccessManagerWithRoles} from "../../../scripts/deployments/DeployAccessManagerWithRoles.sol";
import {AccessManager} from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import {IBCRolesLib} from "../../../contracts/utils/IBCRolesLib.sol";

contract IbcImpl is Test, DeployAccessManagerWithRoles {
    AccessManager public immutable accessManager;
    ICS26Router public immutable ics26Router;
    ICS20Transfer public immutable ics20Transfer;
    ICS27GMP public immutable ics27Gmp;
    RelayerHelper public immutable relayerHelper;

    mapping(string counterpartyId => IbcImpl ibcImpl) public counterpartyImpls;

    TestHelper private _th = new TestHelper();

    constructor(address permit2) {
        // ============ Step 1: Deploy the logic contracts ==============
        address escrowLogic = address(new Escrow());
        address ibcERC20Logic = address(new IBCERC20());
        address ics27AccountLogic = address(new ICS27Account());
        ICS26Router ics26RouterLogic = new ICS26Router();
        ICS20Transfer ics20TransferLogic = new ICS20Transfer();
        ICS27GMP ics27GmpLogic = new ICS27GMP();

        // ============== Step 2: Deploy ERC1967 Proxies ==============
        accessManager = new AccessManager(msg.sender);

        ERC1967Proxy routerProxy = new ERC1967Proxy(
            address(ics26RouterLogic), abi.encodeCall(ICS26Router.initialize, (address(accessManager)))
        );

        ERC1967Proxy transferProxy = new ERC1967Proxy(
            address(ics20TransferLogic),
            abi.encodeCall(
                ICS20Transfer.initialize,
                (address(routerProxy), escrowLogic, ibcERC20Logic, permit2, address(accessManager))
            )
        );

        ERC1967Proxy gmpProxy = new ERC1967Proxy(
            address(ics27GmpLogic),
            abi.encodeCall(
                ICS27GMP.initialize, (address(routerProxy), address(ics27AccountLogic), address(accessManager))
            )
        );

        ics26Router = ICS26Router(address(routerProxy));
        ics20Transfer = ICS20Transfer(address(transferProxy));
        ics27Gmp = ICS27GMP(address(gmpProxy));
        relayerHelper = new RelayerHelper(address(ics26Router));

        // ============== Step 3: Wire up the contracts ==============
        vm.startPrank(msg.sender);
        accessManagerSetTargetRoles(
            accessManager, address(routerProxy), address(transferProxy), address(gmpProxy), true
        );
        accessManager.grantRole(IBCRolesLib.ID_CUSTOMIZER_ROLE, msg.sender, 0);
        accessManager.grantRole(IBCRolesLib.ERC20_CUSTOMIZER_ROLE, msg.sender, 0);

        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, address(ics20Transfer));
        ics26Router.addIBCApp(ICS27Lib.DEFAULT_PORT_ID, address(ics27Gmp));
        vm.stopPrank();
    }

    /// @notice Adds a counterparty implementation by creating a solidity light client
    /// @param counterparty The counterparty implementation
    /// @param counterpartyId The counterparty identifier
    function addCounterpartyImpl(IbcImpl counterparty, string calldata counterpartyId) public returns (string memory) {
        ICS26Router counterpartyIcs26 = counterparty.ics26Router();
        SolidityLightClient lightClient = new SolidityLightClient(counterpartyIcs26);

        // Set the light client as the counterparty for the current implementation
        counterpartyImpls[counterpartyId] = counterparty;

        return ics26Router.addClient(
            IICS02ClientMsgs.CounterpartyInfo(counterpartyId, _th.EMPTY_MERKLE_PREFIX()), address(lightClient)
        );
    }

    function sendTransferAsUser(IERC20 token, address sender, string calldata receiver, uint256 amount)
        external
        returns (IICS26RouterMsgs.Packet memory)
    {
        return sendTransferAsUser(
            token, sender, receiver, amount, _th.DEFAULT_TIMEOUT_TIMESTAMP(), _th.FIRST_CLIENT_ID()
        );
    }

    function sendTransferAsUser(
        IERC20 token,
        address sender,
        string calldata receiver,
        uint256 amount,
        uint64 timeoutTimestamp
    ) external returns (IICS26RouterMsgs.Packet memory) {
        return sendTransferAsUser(token, sender, receiver, amount, timeoutTimestamp, _th.FIRST_CLIENT_ID());
    }

    function sendTransferAsUser(
        IERC20 token,
        address sender,
        string calldata receiver,
        uint256 amount,
        string memory sourceClient
    ) public returns (IICS26RouterMsgs.Packet memory) {
        return sendTransferAsUser(token, sender, receiver, amount, _th.DEFAULT_TIMEOUT_TIMESTAMP(), sourceClient);
    }

    function sendTransferAsUser(
        IERC20 token,
        address sender,
        string calldata receiver,
        uint256 amount,
        uint64 timeoutTimestamp,
        string memory sourceClient
    ) public returns (IICS26RouterMsgs.Packet memory) {
        vm.startPrank(sender);
        token.approve(address(ics20Transfer), amount);
        vm.recordLogs();
        ics20Transfer.sendTransfer(
            IICS20TransferMsgs.SendTransferMsg({
                denom: address(token),
                amount: amount,
                receiver: receiver,
                sourceClient: sourceClient,
                destPort: ICS20Lib.DEFAULT_PORT_ID,
                timeoutTimestamp: timeoutTimestamp,
                memo: _th.randomString()
            })
        );
        vm.stopPrank();

        bytes memory packetBz = _th.getValueFromEvent(IICS26Router.SendPacket.selector);
        return abi.decode(packetBz, (IICS26RouterMsgs.Packet));
    }

    function sendTransferAsUser(
        IERC20 token,
        address sender,
        string calldata receiver,
        ISignatureTransfer.PermitTransferFrom memory permit,
        bytes memory signature
    ) public returns (IICS26RouterMsgs.Packet memory) {
        return sendTransferAsUser(token, sender, receiver, _th.FIRST_CLIENT_ID(), permit, signature);
    }

    function sendTransferAsUser(
        IERC20 token,
        address sender,
        string calldata receiver,
        string memory sourceClient,
        ISignatureTransfer.PermitTransferFrom memory permit,
        bytes memory signature
    ) public returns (IICS26RouterMsgs.Packet memory) {
        vm.startPrank(sender);
        vm.recordLogs();
        ics20Transfer.sendTransferWithPermit2(
            IICS20TransferMsgs.SendTransferMsg({
                denom: address(token),
                amount: permit.permitted.amount,
                receiver: receiver,
                sourceClient: sourceClient,
                destPort: ICS20Lib.DEFAULT_PORT_ID,
                timeoutTimestamp: _th.DEFAULT_TIMEOUT_TIMESTAMP(),
                memo: ""
            }),
            permit,
            signature
        );
        vm.stopPrank();

        bytes memory packetBz = _th.getValueFromEvent(IICS26Router.SendPacket.selector);
        return abi.decode(packetBz, (IICS26RouterMsgs.Packet));
    }

    function sendGmpAsUser(address sender, string calldata receiver, bytes calldata payload)
        external
        returns (IICS26RouterMsgs.Packet memory)
    {
        return sendGmpAsUser(sender, receiver, payload, "", "", _th.DEFAULT_TIMEOUT_TIMESTAMP(), _th.FIRST_CLIENT_ID());
    }

    function sendGmpAsUser(address sender, string calldata receiver, bytes calldata payload, bytes calldata salt)
        external
        returns (IICS26RouterMsgs.Packet memory)
    {
        return
            sendGmpAsUser(sender, receiver, payload, salt, "", _th.DEFAULT_TIMEOUT_TIMESTAMP(), _th.FIRST_CLIENT_ID());
    }

    function sendGmpAsUser(
        address sender,
        string calldata receiver,
        bytes calldata payload,
        bytes calldata salt,
        string calldata memo
    ) external returns (IICS26RouterMsgs.Packet memory) {
        return sendGmpAsUser(
            sender, receiver, payload, salt, memo, _th.DEFAULT_TIMEOUT_TIMESTAMP(), _th.FIRST_CLIENT_ID()
        );
    }

    function sendGmpAsUser(
        address sender,
        string calldata receiver,
        bytes calldata payload,
        bytes memory salt,
        string memory memo,
        uint64 timeoutTimestamp
    ) public returns (IICS26RouterMsgs.Packet memory) {
        return sendGmpAsUser(sender, receiver, payload, salt, memo, timeoutTimestamp, _th.FIRST_CLIENT_ID());
    }

    function sendGmpAsUser(
        address sender,
        string calldata receiver,
        bytes calldata payload,
        bytes memory salt,
        string memory memo,
        uint64 timeoutTimestamp,
        string memory sourceClient
    ) public returns (IICS26RouterMsgs.Packet memory) {
        vm.startPrank(sender);
        vm.recordLogs();
        ics27Gmp.sendCall(
            IICS27GMPMsgs.SendCallMsg({
                receiver: receiver,
                payload: payload,
                salt: salt,
                memo: memo,
                timeoutTimestamp: timeoutTimestamp,
                sourceClient: sourceClient
            })
        );
        vm.stopPrank();

        bytes memory packetBz = _th.getValueFromEvent(IICS26Router.SendPacket.selector);
        return abi.decode(packetBz, (IICS26RouterMsgs.Packet));
    }

    function recvPacket(IICS26RouterMsgs.Packet calldata packet) external returns (bytes[] memory acks) {
        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket;
        msgRecvPacket.packet = packet;
        vm.recordLogs();
        ics26Router.recvPacket(msgRecvPacket);

        bytes memory ackBz = _th.getValueFromEvent(IICS26Router.WriteAcknowledgement.selector);
        (, acks) = abi.decode(ackBz, (IICS26RouterMsgs.Packet, bytes[]));
        return acks;
    }

    function ackPacket(IICS26RouterMsgs.Packet calldata packet, bytes[] calldata acks) external {
        require(acks.length == 1, "multiple acks not supported");
        IICS26RouterMsgs.MsgAckPacket memory msgWriteAck;
        msgWriteAck.packet = packet;
        msgWriteAck.acknowledgement = acks[0];

        ics26Router.ackPacket(msgWriteAck);
    }

    function timeoutPacket(IICS26RouterMsgs.Packet calldata packet) external {
        IICS26RouterMsgs.MsgTimeoutPacket memory msgTimeoutPacket;
        msgTimeoutPacket.packet = packet;
        vm.recordLogs();
        ics26Router.timeoutPacket(msgTimeoutPacket);
    }

    function cheatPacketCommitment(IICS26RouterMsgs.Packet calldata packet) external {
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(packet.sourceClient, packet.sequence);
        bytes32 value = ICS24Host.packetCommitmentBytes32(packet);
        _cheatCommit(path, value);
    }

    function _cheatCommit(bytes32 path, bytes32 value) private {
        bytes32 erc7201Slot = 0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;
        bytes32 commitmentSlot = keccak256(abi.encodePacked(path, erc7201Slot));
        // This is a cheat code to commit a value to the light client
        // It should only be used for testing purposes
        vm.store(address(ics26Router), commitmentSlot, value);
    }
}
