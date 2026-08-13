// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable
// custom-errors,max-line-length,no-inline-assembly,gas-small-strings,function-max-lines,gas-struct-packing

import { Test } from "forge-std/Test.sol";

import { IIFTMsgs } from "../../contracts/msgs/IIFTMsgs.sol";
import { IIBCAppCallbacks } from "../../contracts/msgs/IIBCAppCallbacks.sol";
import { IICS26RouterMsgs } from "../../contracts/msgs/IICS26RouterMsgs.sol";
import { IICS27GMPMsgs } from "../../contracts/msgs/IICS27GMPMsgs.sol";

import { IIFT } from "../../contracts/interfaces/IIFT.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { IIFTErrors } from "../../contracts/errors/IIFTErrors.sol";
import { IICS27GMP } from "../../contracts/interfaces/IICS27GMP.sol";
import { IIBCSenderCallbacks } from "../../contracts/interfaces/IIBCSenderCallbacks.sol";
import { IERC20 } from "@openzeppelin-contracts/token/ERC20/IERC20.sol";
import { IERC165 } from "@openzeppelin-contracts/utils/introspection/IERC165.sol";
import { IERC20Metadata } from "@openzeppelin-contracts/token/ERC20/extensions/IERC20Metadata.sol";

import { IFTOwnable } from "../../contracts/utils/IFTOwnable.sol";
import { IFTAccessManaged } from "../../contracts/utils/IFTAccessManaged.sol";
import { EVMIFTSendCallConstructor } from "../../contracts/utils/EVMIFTSendCallConstructor.sol";
import { CosmosIFTSendCallConstructor } from "../../contracts/utils/CosmosIFTSendCallConstructor.sol";
import { IIFTSendCallConstructor } from "../../contracts/interfaces/IIFTSendCallConstructor.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { TestHelper } from "./utils/TestHelper.sol";
import { ICS27Lib } from "../../contracts/utils/ICS27Lib.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { OwnableUpgradeable } from "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";

contract IFTTest is Test {
    // solhint-disable gas-indexed-events
    IIFT public ift;

    EVMIFTSendCallConstructor public evmCallConstructor = new EVMIFTSendCallConstructor();
    TestHelper public th = new TestHelper();

    string public constant TOKEN_NAME = "Test IFT";
    string public constant TOKEN_SYMBOL = "TIFT";

    string public constant COUNTERPARTY_IFT_ADDRESS = "0x123";

    address public mockICS27 = makeAddr("mockICS27");
    // admin is the owner of the IFTOwnable and authority of the access manager
    address public admin = makeAddr("admin");

    function setUpOwnable() public {
        address impl = address(new IFTOwnable());
        ERC1967Proxy proxy = new ERC1967Proxy(
            impl, abi.encodeCall(IFTOwnable.initialize, (admin, TOKEN_NAME, TOKEN_SYMBOL, mockICS27))
        );
        ift = IIFT(address(proxy));
    }

    function setUpAccessManaged() public {
        address impl = address(new IFTAccessManaged());
        AccessManager manager = new AccessManager(admin);
        ERC1967Proxy proxy = new ERC1967Proxy(
            impl, abi.encodeCall(IFTAccessManaged.initialize, (address(manager), TOKEN_NAME, TOKEN_SYMBOL, mockICS27))
        );
        ift = IIFT(address(proxy));
    }

    function test_Ownable_deployment() public {
        setUpOwnable();
        assertEq(address(ift.ics27()), mockICS27);
        assertEq(IERC20Metadata(address(ift)).name(), TOKEN_NAME);
        assertEq(IERC20Metadata(address(ift)).symbol(), TOKEN_SYMBOL);
        assertEq(IFTOwnable(address(ift)).owner(), admin);
    }

    function test_AccessManaged_deployment() public {
        setUpAccessManaged();
        assertEq(address(ift.ics27()), mockICS27);
        assertEq(IERC20Metadata(address(ift)).name(), TOKEN_NAME);
        assertEq(IERC20Metadata(address(ift)).symbol(), TOKEN_SYMBOL);

        AccessManager manager = AccessManager(IFTAccessManaged(address(ift)).authority());
        (bool isAdmin,) = manager.hasRole(IBCRolesLib.ADMIN_ROLE, admin);
        assertTrue(isAdmin);
    }

    function testFuzz_Ownable_authorityCanMint(uint256 amount) public {
        setUpOwnable();

        address receiver = makeAddr("receiver");

        vm.prank(admin);
        IFTOwnable(address(ift)).mint(receiver, amount);

        assertEq(IERC20(address(ift)).balanceOf(receiver), amount);
        assertEq(IERC20(address(ift)).totalSupply(), amount);
    }

    function testFuzz_Ownable_unauthorizedCannotMint(uint256 amount) public {
        setUpOwnable();

        address unauthorized = makeAddr("unauthorized");

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, unauthorized));
        vm.prank(unauthorized);
        IFTOwnable(address(ift)).mint(makeAddr("receiver"), amount);
    }

    function testFuzz_Ownable_holderCanBurn(uint256 amount) public {
        setUpOwnable();

        address holder = makeAddr("holder");

        vm.prank(admin);
        IFTOwnable(address(ift)).mint(holder, amount);

        vm.prank(holder);
        IFTOwnable(address(ift)).burn(amount);

        assertEq(IERC20(address(ift)).balanceOf(holder), 0);
        assertEq(IERC20(address(ift)).totalSupply(), 0);
    }

    function testFuzz_Ownable_approvedSpenderCanBurnFrom(uint256 amount) public {
        setUpOwnable();

        address holder = makeAddr("holder");
        address spender = makeAddr("spender");

        vm.prank(admin);
        IFTOwnable(address(ift)).mint(holder, amount);

        vm.prank(holder);
        IERC20(address(ift)).approve(spender, amount);

        vm.prank(spender);
        IFTOwnable(address(ift)).burnFrom(holder, amount);

        assertEq(IERC20(address(ift)).balanceOf(holder), 0);
        assertEq(IERC20(address(ift)).totalSupply(), 0);
    }

    function testFuzz_Ownable_unapprovedSpenderCannotBurnFrom(uint256 amount) public {
        vm.assume(amount > 0);
        setUpOwnable();

        address holder = makeAddr("holder");
        address spender = makeAddr("spender");

        vm.prank(admin);
        IFTOwnable(address(ift)).mint(holder, amount);

        vm.expectRevert();
        vm.prank(spender);
        IFTOwnable(address(ift)).burnFrom(holder, amount);
    }

    function testFuzz_AccessManaged_authorityCanMint(uint256 amount) public {
        setUpAccessManaged();

        address receiver = makeAddr("receiver");

        vm.prank(admin);
        IFTAccessManaged(address(ift)).mint(receiver, amount);

        assertEq(IERC20(address(ift)).balanceOf(receiver), amount);
        assertEq(IERC20(address(ift)).totalSupply(), amount);
    }

    function testFuzz_AccessManaged_unauthorizedCannotMint(uint256 amount) public {
        setUpAccessManaged();

        address unauthorized = makeAddr("unauthorized");

        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        vm.prank(unauthorized);
        IFTAccessManaged(address(ift)).mint(makeAddr("receiver"), amount);
    }

    function testFuzz_AccessManaged_holderCanBurn(uint256 amount) public {
        setUpAccessManaged();

        address holder = makeAddr("holder");

        vm.prank(admin);
        IFTAccessManaged(address(ift)).mint(holder, amount);

        vm.prank(holder);
        IFTAccessManaged(address(ift)).burn(amount);

        assertEq(IERC20(address(ift)).balanceOf(holder), 0);
        assertEq(IERC20(address(ift)).totalSupply(), 0);
    }

    function testFuzz_AccessManaged_approvedSpenderCanBurnFrom(uint256 amount) public {
        setUpAccessManaged();

        address holder = makeAddr("holder");
        address spender = makeAddr("spender");

        vm.prank(admin);
        IFTAccessManaged(address(ift)).mint(holder, amount);

        vm.prank(holder);
        IERC20(address(ift)).approve(spender, amount);

        vm.prank(spender);
        IFTAccessManaged(address(ift)).burnFrom(holder, amount);

        assertEq(IERC20(address(ift)).balanceOf(holder), 0);
        assertEq(IERC20(address(ift)).totalSupply(), 0);
    }

    function testFuzz_AccessManaged_unapprovedSpenderCannotBurnFrom(uint256 amount) public {
        vm.assume(amount > 0);
        setUpAccessManaged();

        address holder = makeAddr("holder");
        address spender = makeAddr("spender");

        vm.prank(admin);
        IFTAccessManaged(address(ift)).mint(holder, amount);

        vm.expectRevert();
        vm.prank(spender);
        IFTAccessManaged(address(ift)).burnFrom(holder, amount);
    }

    function fixtureregisterBridgeTC() public returns (RegisterIFTBridgeTestCase[] memory) {
        address unauthorized = makeAddr("unauthorized");

        RegisterIFTBridgeTestCase[] memory testCases = new RegisterIFTBridgeTestCase[](8);

        testCases[0] = RegisterIFTBridgeTestCase({
            name: "success: ownable admin registers",
            caller: admin,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: address(evmCallConstructor),
            expectedRevert: ""
        });
        testCases[1] = RegisterIFTBridgeTestCase({
            name: "success: access managed admin registers",
            caller: admin,
            ownable: false,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: address(evmCallConstructor),
            expectedRevert: ""
        });
        testCases[2] = RegisterIFTBridgeTestCase({
            name: "revert: ownable unauthorized caller",
            caller: unauthorized,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: address(evmCallConstructor),
            expectedRevert: abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, unauthorized)
        });
        testCases[3] = RegisterIFTBridgeTestCase({
            name: "revert: access managed unauthorized caller",
            caller: makeAddr("unauthorized"),
            ownable: false,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: address(evmCallConstructor),
            expectedRevert: abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized)
        });
        testCases[4] = RegisterIFTBridgeTestCase({
            name: "revert: empty clientId",
            caller: admin,
            ownable: true,
            clientId: "",
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: address(evmCallConstructor),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTEmptyClientId.selector)
        });
        testCases[5] = RegisterIFTBridgeTestCase({
            name: "revert: empty counterparty IFT address",
            caller: admin,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: "",
            iftSendCallConstructor: address(evmCallConstructor),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTEmptyCounterpartyAddress.selector)
        });
        testCases[6] = RegisterIFTBridgeTestCase({
            name: "revert: empty iftSendCallConstructor address",
            caller: admin,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: address(0),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTZeroAddressConstructor.selector)
        });
        testCases[7] = RegisterIFTBridgeTestCase({
            name: "revert: iftSendCallConstructor does not support IIFTSendCallConstructor",
            caller: admin,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            counterpartyIFT: COUNTERPARTY_IFT_ADDRESS,
            iftSendCallConstructor: makeAddr("invalidConstructor"),
            expectedRevert: abi.encodeWithSelector(
                IIFTErrors.IFTInvalidConstructorInterface.selector, makeAddr("invalidConstructor")
            )
        });

        return testCases;
    }

    function tableRegisterIFTBridgeTest(RegisterIFTBridgeTestCase memory registerBridgeTC) public {
        if (registerBridgeTC.ownable) {
            setUpOwnable();
        } else {
            setUpAccessManaged();
        }
        vm.startPrank(registerBridgeTC.caller);

        if (registerBridgeTC.expectedRevert.length != 0) {
            vm.expectRevert(registerBridgeTC.expectedRevert);
        } else {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTBridgeRegistered(
                registerBridgeTC.clientId, registerBridgeTC.counterpartyIFT, registerBridgeTC.iftSendCallConstructor
            );
        }

        ift.registerIFTBridge(
            registerBridgeTC.clientId, registerBridgeTC.counterpartyIFT, registerBridgeTC.iftSendCallConstructor
        );

        if (registerBridgeTC.expectedRevert.length != 0) {
            return;
        }

        IIFTMsgs.IFTBridge memory bridge = ift.getIFTBridge(registerBridgeTC.clientId);
        assertEq(bridge.clientId, registerBridgeTC.clientId);
        assertEq(bridge.counterpartyIFTAddress, registerBridgeTC.counterpartyIFT);
        assertEq(address(bridge.iftSendCallConstructor), address(evmCallConstructor));
    }

    function fixtureRemoveBridgeTC() public returns (RemoveIFTBridgeTestCase[] memory) {
        address unauthorized = makeAddr("unauthorized");

        RemoveIFTBridgeTestCase[] memory testCases = new RemoveIFTBridgeTestCase[](5);

        testCases[0] = RemoveIFTBridgeTestCase({
            name: "success: ownable admin removes",
            caller: admin,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            expectedRevert: ""
        });
        testCases[1] = RemoveIFTBridgeTestCase({
            name: "success: access managed admin removes",
            caller: admin,
            ownable: false,
            clientId: th.FIRST_CLIENT_ID(),
            expectedRevert: ""
        });
        testCases[2] = RemoveIFTBridgeTestCase({
            name: "revert: ownable unauthorized caller",
            caller: unauthorized,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            expectedRevert: abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, unauthorized)
        });
        testCases[3] = RemoveIFTBridgeTestCase({
            name: "revert: access managed unauthorized caller",
            caller: unauthorized,
            ownable: false,
            clientId: th.FIRST_CLIENT_ID(),
            expectedRevert: abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized)
        });
        testCases[4] = RemoveIFTBridgeTestCase({
            name: "revert: clientId not registered",
            caller: admin,
            ownable: true,
            clientId: th.INVALID_ID(),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTBridgeNotFound.selector, th.INVALID_ID())
        });

        return testCases;
    }

    function tableRemoveIFTBridgeTest(RemoveIFTBridgeTestCase memory removeBridgeTC) public {
        if (removeBridgeTC.ownable) {
            setUpOwnable();
        } else {
            setUpAccessManaged();
        }
        // First register the bridge
        vm.startPrank(admin);
        ift.registerIFTBridge(th.FIRST_CLIENT_ID(), COUNTERPARTY_IFT_ADDRESS, address(evmCallConstructor));
        vm.stopPrank();

        // Now attempt to remove the bridge
        vm.startPrank(removeBridgeTC.caller);

        if (removeBridgeTC.expectedRevert.length != 0) {
            vm.expectRevert(removeBridgeTC.expectedRevert);
        } else {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTBridgeRemoved(removeBridgeTC.clientId);
        }

        ift.removeIFTBridge(removeBridgeTC.clientId);

        if (removeBridgeTC.expectedRevert.length != 0) {
            return;
        }

        vm.expectRevert(abi.encodeWithSelector(IIFTErrors.IFTBridgeNotFound.selector, removeBridgeTC.clientId));
        ift.getIFTBridge(removeBridgeTC.clientId);
    }

    function fixtureTransferTC() public returns (IFTTransferTestCase[] memory) {
        address sender = makeAddr("sender");
        string memory receiver = Strings.toHexString(makeAddr("receiver"));

        uint64 timeout = th.DEFAULT_TIMEOUT_TIMESTAMP();
        uint64 pastTimeout = uint64(block.timestamp) - 1;
        uint256 transferAmount = 100;

        IFTTransferTestCase[] memory testCases = new IFTTransferTestCase[](7);

        testCases[0] = IFTTransferTestCase({
            name: "success: ownable transfer",
            caller: sender,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            receiver: receiver,
            amount: transferAmount,
            timeoutTimestamp: timeout,
            expectedRevert: ""
        });
        testCases[1] = IFTTransferTestCase({
            name: "success: access managed transfer",
            caller: sender,
            ownable: false,
            clientId: th.FIRST_CLIENT_ID(),
            receiver: receiver,
            amount: transferAmount,
            timeoutTimestamp: timeout,
            expectedRevert: ""
        });
        testCases[2] = IFTTransferTestCase({
            name: "revert: empty receiver",
            caller: sender,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            receiver: "",
            amount: transferAmount,
            timeoutTimestamp: timeout,
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTEmptyReceiver.selector)
        });
        testCases[3] = IFTTransferTestCase({
            name: "revert: zero amount",
            caller: sender,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            receiver: receiver,
            amount: 0,
            timeoutTimestamp: timeout,
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTZeroAmount.selector)
        });
        testCases[4] = IFTTransferTestCase({
            name: "revert: timeout in past",
            caller: sender,
            ownable: true,
            clientId: th.FIRST_CLIENT_ID(),
            receiver: receiver,
            amount: transferAmount,
            timeoutTimestamp: pastTimeout,
            expectedRevert: abi.encodeWithSelector(
                IIFTErrors.IFTTimeoutInPast.selector, pastTimeout, uint64(block.timestamp)
            )
        });
        testCases[5] = IFTTransferTestCase({
            name: "revert: unregistered clientId",
            caller: sender,
            ownable: true,
            clientId: th.INVALID_ID(),
            receiver: receiver,
            amount: transferAmount,
            timeoutTimestamp: timeout,
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTBridgeNotFound.selector, th.INVALID_ID())
        });
        testCases[6] = IFTTransferTestCase({
            name: "revert: empty clientId",
            caller: sender,
            ownable: true,
            clientId: "",
            receiver: receiver,
            amount: transferAmount,
            timeoutTimestamp: timeout,
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTEmptyClientId.selector)
        });

        return testCases;
    }

    function tableIFTTransferTest(IFTTransferTestCase memory transferTC) public {
        if (transferTC.ownable) {
            setUpOwnable();
        } else {
            setUpAccessManaged();
        }

        // First register the bridge
        vm.startPrank(admin);
        ift.registerIFTBridge(th.FIRST_CLIENT_ID(), COUNTERPARTY_IFT_ADDRESS, address(evmCallConstructor));
        vm.stopPrank();

        // random sequence number
        uint64 seq = uint64(vm.randomUint(1, type(uint64).max));
        vm.mockCall(address(mockICS27), IICS27GMP.sendCall.selector, abi.encode(seq));

        // Mint some tokens to the caller
        uint256 initialBalance = 1_000_000 ether;
        deal(address(ift), transferTC.caller, initialBalance, true);

        if (transferTC.expectedRevert.length != 0) {
            vm.expectRevert(transferTC.expectedRevert);
        } else {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTTransferInitiated(
                transferTC.clientId, seq, transferTC.caller, transferTC.receiver, transferTC.amount
            );
        }

        vm.startPrank(transferTC.caller);
        ift.iftTransfer(transferTC.clientId, transferTC.receiver, transferTC.amount, transferTC.timeoutTimestamp);
        vm.stopPrank();

        if (transferTC.expectedRevert.length != 0) {
            return;
        }

        IIFTMsgs.PendingTransfer memory pending = ift.getPendingTransfer(transferTC.clientId, seq);
        assertEq(pending.sender, transferTC.caller);
        assertEq(pending.amount, transferTC.amount);
    }

    function fixtureAckTC() public returns (OnAckPacketTestCase[] memory) {
        address unauthorized = makeAddr("unauthorized");
        address relayer = makeAddr("relayer");

        IICS26RouterMsgs.Payload memory payload = IICS26RouterMsgs.Payload({
            sourcePort: ICS27Lib.DEFAULT_PORT_ID,
            destPort: ICS27Lib.DEFAULT_PORT_ID,
            version: ICS27Lib.ICS27_VERSION,
            encoding: ICS27Lib.ICS27_ENCODING,
            value: ""
        });

        OnAckPacketTestCase[] memory testCases = new OnAckPacketTestCase[](5);

        testCases[0] = OnAckPacketTestCase({
            name: "success: ack completes transfer",
            caller: mockICS27,
            success: true,
            callback: IIBCAppCallbacks.OnAcknowledgementPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                acknowledgement: hex"01",
                relayer: relayer
            }),
            expectedRevert: ""
        });
        testCases[1] = OnAckPacketTestCase({
            name: "success: ack failure refunds transfer",
            caller: mockICS27,
            success: false,
            callback: IIBCAppCallbacks.OnAcknowledgementPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                acknowledgement: ICS24Host.UNIVERSAL_ERROR_ACK,
                relayer: relayer
            }),
            expectedRevert: ""
        });
        testCases[2] = OnAckPacketTestCase({
            name: "revert: unauthorized caller",
            caller: unauthorized,
            success: true,
            callback: IIBCAppCallbacks.OnAcknowledgementPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                acknowledgement: hex"01",
                relayer: relayer
            }),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTOnlyICS27GMP.selector, unauthorized)
        });
        testCases[3] = OnAckPacketTestCase({
            name: "revert: incorrect sequence",
            caller: mockICS27,
            success: true,
            callback: IIBCAppCallbacks.OnAcknowledgementPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 43,
                payload: payload,
                acknowledgement: hex"01",
                relayer: relayer
            }),
            expectedRevert: abi.encodeWithSelector(
                IIFTErrors.IFTPendingTransferNotFound.selector, th.FIRST_CLIENT_ID(), 43
            )
        });
        testCases[4] = OnAckPacketTestCase({
            name: "revert: incorrect clientId",
            caller: mockICS27,
            success: true,
            callback: IIBCAppCallbacks.OnAcknowledgementPacketCallback({
                sourceClient: th.INVALID_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                acknowledgement: hex"01",
                relayer: relayer
            }),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTPendingTransferNotFound.selector, th.INVALID_ID(), 42)
        });

        return testCases;
    }

    function tableOnAckPacketTest(OnAckPacketTestCase memory ackTC) public {
        setUpOwnable();

        // First register the bridge and initiate a transfer
        vm.startPrank(admin);
        ift.registerIFTBridge(th.FIRST_CLIENT_ID(), COUNTERPARTY_IFT_ADDRESS, address(evmCallConstructor));
        vm.stopPrank();

        uint64 seq = 42;
        uint256 transferAmount = 1000;
        address sender = makeAddr("sender");
        vm.mockCall(address(mockICS27), IICS27GMP.sendCall.selector, abi.encode(seq));

        // Mint some tokens to the caller
        deal(address(ift), sender, transferAmount, true);

        vm.startPrank(sender);
        ift.iftTransfer(th.FIRST_CLIENT_ID(), Strings.toHexString(makeAddr("receiver")), transferAmount);
        vm.stopPrank();

        if (ackTC.expectedRevert.length != 0) {
            vm.expectRevert(ackTC.expectedRevert);
        } else if (ackTC.success) {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTTransferCompleted(ackTC.callback.sourceClient, ackTC.callback.sequence, sender, transferAmount);
        } else {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTTransferRefunded(ackTC.callback.sourceClient, ackTC.callback.sequence, sender, transferAmount);
        }

        vm.prank(ackTC.caller);
        IIBCSenderCallbacks(address(ift)).onAckPacket(ackTC.success, ackTC.callback);

        if (ackTC.expectedRevert.length != 0) {
            return;
        }

        vm.expectRevert(
            abi.encodeWithSelector(
                IIFTErrors.IFTPendingTransferNotFound.selector, ackTC.callback.sourceClient, ackTC.callback.sequence
            )
        );
        ift.getPendingTransfer(ackTC.callback.sourceClient, ackTC.callback.sequence);
    }

    function fixtureTimeoutTC() public returns (OnTimeoutPacketTestCase[] memory) {
        address unauthorized = makeAddr("unauthorized");
        address relayer = makeAddr("relayer");

        IICS26RouterMsgs.Payload memory payload = IICS26RouterMsgs.Payload({
            sourcePort: ICS27Lib.DEFAULT_PORT_ID,
            destPort: ICS27Lib.DEFAULT_PORT_ID,
            version: ICS27Lib.ICS27_VERSION,
            encoding: ICS27Lib.ICS27_ENCODING,
            value: ""
        });

        OnTimeoutPacketTestCase[] memory testCases = new OnTimeoutPacketTestCase[](4);

        testCases[0] = OnTimeoutPacketTestCase({
            name: "success: timeout refunds transfer",
            caller: mockICS27,
            callback: IIBCAppCallbacks.OnTimeoutPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                relayer: relayer
            }),
            expectedRevert: ""
        });
        testCases[1] = OnTimeoutPacketTestCase({
            name: "revert: unauthorized caller",
            caller: unauthorized,
            callback: IIBCAppCallbacks.OnTimeoutPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                relayer: relayer
            }),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTOnlyICS27GMP.selector, unauthorized)
        });
        testCases[2] = OnTimeoutPacketTestCase({
            name: "revert: incorrect sequence",
            caller: mockICS27,
            callback: IIBCAppCallbacks.OnTimeoutPacketCallback({
                sourceClient: th.FIRST_CLIENT_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 43,
                payload: payload,
                relayer: relayer
            }),
            expectedRevert: abi.encodeWithSelector(
                IIFTErrors.IFTPendingTransferNotFound.selector, th.FIRST_CLIENT_ID(), 43
            )
        });
        testCases[3] = OnTimeoutPacketTestCase({
            name: "revert: incorrect clientId",
            caller: mockICS27,
            callback: IIBCAppCallbacks.OnTimeoutPacketCallback({
                sourceClient: th.INVALID_ID(),
                destinationClient: th.SECOND_CLIENT_ID(),
                sequence: 42,
                payload: payload,
                relayer: relayer
            }),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTPendingTransferNotFound.selector, th.INVALID_ID(), 42)
        });

        return testCases;
    }

    function tableOnTimeoutPacketTest(OnTimeoutPacketTestCase memory timeoutTC) public {
        setUpOwnable();

        // First register the bridge and initiate a transfer
        vm.startPrank(admin);
        ift.registerIFTBridge(th.FIRST_CLIENT_ID(), COUNTERPARTY_IFT_ADDRESS, address(evmCallConstructor));
        vm.stopPrank();

        uint64 seq = 42;
        uint256 transferAmount = 1000;
        address sender = makeAddr("sender");
        vm.mockCall(address(mockICS27), IICS27GMP.sendCall.selector, abi.encode(seq));

        // Mint some tokens to the caller
        deal(address(ift), sender, transferAmount, true);

        vm.startPrank(sender);
        ift.iftTransfer(th.FIRST_CLIENT_ID(), Strings.toHexString(makeAddr("receiver")), transferAmount);
        vm.stopPrank();

        if (timeoutTC.expectedRevert.length != 0) {
            vm.expectRevert(timeoutTC.expectedRevert);
        } else {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTTransferRefunded(
                timeoutTC.callback.sourceClient, timeoutTC.callback.sequence, sender, transferAmount
            );
        }

        vm.prank(timeoutTC.caller);
        IIBCSenderCallbacks(address(ift)).onTimeoutPacket(timeoutTC.callback);

        if (timeoutTC.expectedRevert.length != 0) {
            return;
        }

        vm.expectRevert(
            abi.encodeWithSelector(
                IIFTErrors.IFTPendingTransferNotFound.selector,
                timeoutTC.callback.sourceClient,
                timeoutTC.callback.sequence
            )
        );
        ift.getPendingTransfer(timeoutTC.callback.sourceClient, timeoutTC.callback.sequence);
    }

    function fixtureMintTC() public returns (IFTMintTestCase[] memory) {
        address authorizedCaller = makeAddr("authorizedCaller");
        address unauthorizedCaller = makeAddr("unauthorizedCaller");
        address receiver = makeAddr("receiver");

        IICS27GMPMsgs.AccountIdentifier memory accountId = IICS27GMPMsgs.AccountIdentifier({
            clientId: th.FIRST_CLIENT_ID(), sender: COUNTERPARTY_IFT_ADDRESS, salt: ""
        });

        IFTMintTestCase[] memory testCases = new IFTMintTestCase[](6);

        testCases[0] = IFTMintTestCase({
            name: "success: ownable mint by authorized caller",
            ownable: true,
            caller: authorizedCaller,
            accountId: accountId,
            receiver: receiver,
            amount: vm.randomUint(1, uint256(type(uint128).max)),
            expectedRevert: ""
        });
        testCases[1] = IFTMintTestCase({
            name: "success: access managed mint by authorized caller",
            ownable: false,
            caller: authorizedCaller,
            accountId: accountId,
            receiver: receiver,
            amount: vm.randomUint(1, uint256(type(uint128).max)),
            expectedRevert: ""
        });
        testCases[2] = IFTMintTestCase({
            name: "revert: mint by unauthorized caller",
            ownable: true,
            caller: unauthorizedCaller,
            accountId: accountId,
            receiver: receiver,
            amount: vm.randomUint(1, uint256(type(uint128).max)),
            expectedRevert: "mock revert"
        });
        testCases[3] = IFTMintTestCase({
            name: "revert: incorrect account identifier clientId",
            ownable: true,
            caller: authorizedCaller,
            accountId: IICS27GMPMsgs.AccountIdentifier({
                clientId: th.INVALID_ID(), sender: COUNTERPARTY_IFT_ADDRESS, salt: ""
            }),
            receiver: receiver,
            amount: vm.randomUint(1, uint256(type(uint128).max)),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTBridgeNotFound.selector, th.INVALID_ID())
        });
        testCases[4] = IFTMintTestCase({
            name: "revert: unexpected salt in account identifier",
            ownable: true,
            caller: authorizedCaller,
            accountId: IICS27GMPMsgs.AccountIdentifier({
                clientId: th.FIRST_CLIENT_ID(), sender: COUNTERPARTY_IFT_ADDRESS, salt: hex"01"
            }),
            receiver: receiver,
            amount: vm.randomUint(1, uint256(type(uint128).max)),
            expectedRevert: abi.encodeWithSelector(IIFTErrors.IFTUnexpectedSalt.selector, hex"01")
        });
        testCases[5] = IFTMintTestCase({
            name: "revert: incorrect account identifier sender",
            ownable: true,
            caller: authorizedCaller,
            accountId: IICS27GMPMsgs.AccountIdentifier({ clientId: th.FIRST_CLIENT_ID(), sender: "0x456", salt: "" }),
            receiver: receiver,
            amount: vm.randomUint(1, uint256(type(uint128).max)),
            expectedRevert: abi.encodeWithSelector(
                IIFTErrors.IFTUnauthorizedMint.selector, COUNTERPARTY_IFT_ADDRESS, "0x456"
            )
        });

        return testCases;
    }

    function tableIFTMintTest(IFTMintTestCase memory mintTC) public {
        if (mintTC.ownable) {
            setUpOwnable();
        } else {
            setUpAccessManaged();
        }

        // First register the bridge
        vm.startPrank(admin);
        ift.registerIFTBridge(th.FIRST_CLIENT_ID(), COUNTERPARTY_IFT_ADDRESS, address(evmCallConstructor));
        vm.stopPrank();

        address authorizedCaller = makeAddr("authorizedCaller");
        address unauthorizedCaller = makeAddr("unauthorizedCaller");

        vm.mockCall(
            address(mockICS27),
            abi.encodeCall(IICS27GMP.getAccountIdentifier, (authorizedCaller)),
            abi.encode(mintTC.accountId)
        );
        vm.mockCallRevert(
            address(mockICS27), abi.encodeCall(IICS27GMP.getAccountIdentifier, (unauthorizedCaller)), "mock revert"
        );

        if (mintTC.expectedRevert.length != 0) {
            vm.expectRevert(mintTC.expectedRevert);
        } else {
            vm.expectEmit(true, true, true, true);
            emit IIFT.IFTMintReceived(th.FIRST_CLIENT_ID(), mintTC.receiver, mintTC.amount);
        }

        vm.prank(mintTC.caller);
        ift.iftMint(mintTC.receiver, mintTC.amount);

        if (mintTC.expectedRevert.length != 0) {
            return;
        }

        uint256 receiverBalance = IERC20(address(ift)).balanceOf(mintTC.receiver);
        assertEq(receiverBalance, mintTC.amount);
    }

    // Upgrade Tests

    function fixtureUpgradeTC() public returns (UpgradeTestCase[] memory) {
        address unauthorized = makeAddr("unauthorized");

        UpgradeTestCase[] memory testCases = new UpgradeTestCase[](2);

        testCases[0] = UpgradeTestCase({
            name: "success: ownable admin upgrades", caller: admin, ownable: true, expectedRevert: ""
        });
        testCases[1] = UpgradeTestCase({
            name: "revert: ownable unauthorized caller",
            caller: unauthorized,
            ownable: true,
            expectedRevert: abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, unauthorized)
        });

        return testCases;
    }

    function tableUpgradeTest(UpgradeTestCase memory upgradeTC) public {
        if (upgradeTC.ownable) {
            setUpOwnable();
        } else {
            setUpAccessManaged();
        }

        IFTOwnable newImpl = new IFTOwnable();

        if (upgradeTC.expectedRevert.length != 0) {
            vm.expectRevert(upgradeTC.expectedRevert);
        }

        vm.prank(upgradeTC.caller);
        UUPSUpgradeable(address(ift)).upgradeToAndCall(address(newImpl), "");

        if (upgradeTC.expectedRevert.length != 0) {
            return;
        }

        assertEq(IFTOwnable(address(ift)).owner(), admin, "owner should be preserved after upgrade");
    }

    // EVMIFTSendCallConstructor Tests

    function testFuzz_evmCallConstructor_constructMintCall(uint256 amount) public {
        address receiver = makeAddr("receiver");
        string memory receiverStr = Strings.toHexString(receiver);

        bytes memory callData = evmCallConstructor.constructMintCall(receiverStr, amount);
        bytes memory expected = abi.encodeCall(IIFT.iftMint, (receiver, amount));

        assertEq(callData, expected);
    }

    function testFuzz_evmCallConstructor_invalidReceiver_reverts(uint256 amount) public {
        vm.expectRevert();
        evmCallConstructor.constructMintCall("invalid-address", amount);
    }

    function test_evmCallConstructor_supportsInterface() public view {
        assertTrue(evmCallConstructor.supportsInterface(type(IIFTSendCallConstructor).interfaceId));

        bytes4 erc165Id = 0x01ffc9a7;
        assertTrue(evmCallConstructor.supportsInterface(erc165Id));

        bytes4 randomId = 0xdeadbeef;
        assertFalse(evmCallConstructor.supportsInterface(randomId));
    }

    // ERC165 Interface Tests

    function test_IFT_supportsInterface() public {
        setUpOwnable();

        // IIBCSenderCallbacks interface ID
        bytes4 senderCallbacksId = type(IIBCSenderCallbacks).interfaceId;
        assertTrue(IERC165(address(ift)).supportsInterface(senderCallbacksId));

        // ERC165 interface ID
        bytes4 erc165Id = 0x01ffc9a7;
        assertTrue(IERC165(address(ift)).supportsInterface(erc165Id));
    }

    function test_cosmosCallConstructor_constructor() public {
        string memory typeUrl = "/wfchain.ift.MsgIFTMint";
        string memory tokenDenom = "uatom";
        string memory ica = "cosmos1abc123";

        CosmosIFTSendCallConstructor constructor_ = new CosmosIFTSendCallConstructor(typeUrl, tokenDenom, ica);

        assertEq(constructor_.bridgeReceiveTypeUrl(), typeUrl);
        assertEq(constructor_.denom(), tokenDenom);
        assertEq(constructor_.icaAddress(), ica);
    }

    function fixtureCosmosMintCallTC() public returns (CosmosMintCallTestCase[] memory) {
        CosmosMintCallTestCase[] memory testCases = new CosmosMintCallTestCase[](19);

        testCases[0] = CosmosMintCallTestCase({
            name: "success: bech32 receiver", receiver: "wf1address234", amount: 1_000_000, expectedRevert: ""
        });
        testCases[1] = CosmosMintCallTestCase({
            name: "success: hex address receiver",
            receiver: Strings.toHexString(makeAddr("receiver")),
            amount: 2,
            expectedRevert: ""
        });
        testCases[2] = CosmosMintCallTestCase({
            name: "success: cosmos16...",
            receiver: "cosmos16a87dpmkutsfx7cddxt749urv62nkcajmr8nw2",
            amount: 3,
            expectedRevert: ""
        });
        testCases[3] = CosmosMintCallTestCase({
            name: "success: cosmos1xmx...",
            receiver: "cosmos1xmxsh7td3ga53afs2qpzskq4dl4g9a4x3uck7j",
            amount: 4,
            expectedRevert: ""
        });
        testCases[4] = CosmosMintCallTestCase({
            name: "success: cosmos1t5u...",
            receiver: "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd",
            amount: 5,
            expectedRevert: ""
        });
        testCases[5] = CosmosMintCallTestCase({
            name: "success: cosmos1hs6...",
            receiver: "cosmos1hs6sl3u2lmrwr58khexm4v9yjk333c7ts0rcuz",
            amount: 6,
            expectedRevert: ""
        });
        testCases[6] = CosmosMintCallTestCase({
            name: "success: osmo10c3...",
            receiver: "osmo10c3r27nekd9w7s3swwg3t9sf5hzrvqjftf2djk",
            amount: 7,
            expectedRevert: ""
        });
        testCases[7] = CosmosMintCallTestCase({
            name: "success: osmo14dy...",
            receiver: "osmo14dy272jhqz3gqduujt0gngqm76sg3e0fqhvzxa",
            amount: 8,
            expectedRevert: ""
        });
        testCases[8] = CosmosMintCallTestCase({
            name: "success: osmo1hd97...",
            receiver: "osmo1hd97pvymu9xx63xg3y4qn2sy4cfywtzv0527jg",
            amount: 9,
            expectedRevert: ""
        });
        testCases[9] = CosmosMintCallTestCase({
            name: "success: osmo1txtz...",
            receiver: "osmo1txtzjqtmgwjhgkdvwtdgfwhsda77ewhxtsclce",
            amount: 10,
            expectedRevert: ""
        });
        testCases[10] = CosmosMintCallTestCase({
            name: "revert: empty receiver",
            receiver: "",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "")
        });
        testCases[11] = CosmosMintCallTestCase({
            name: "revert: missing separator",
            receiver: "wf2345a",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf2345a"
            )
        });
        testCases[12] = CosmosMintCallTestCase({
            name: "revert: invalid hrp capital",
            receiver: "WF12345a",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "WF12345a"
            )
        });
        testCases[13] = CosmosMintCallTestCase({
            name: "revert: invalid data char 1",
            receiver: "wf1rece1ver",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf1rece1ver"
            )
        });
        testCases[14] = CosmosMintCallTestCase({
            name: "revert: invalid data char i",
            receiver: "wf1receiver",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf1receiver"
            )
        });
        testCases[15] = CosmosMintCallTestCase({
            name: "revert: invalid data char o",
            receiver: "wf1receiver",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf1receiver"
            )
        });
        testCases[16] = CosmosMintCallTestCase({
            name: "revert: invalid data char b",
            receiver: "wf1recebr",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf1recebr"
            )
        });
        testCases[17] = CosmosMintCallTestCase({
            name: "revert: invalid data char {",
            receiver: "wf1address{",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf1address{"
            )
        });
        testCases[18] = CosmosMintCallTestCase({
            name: "revert: invalid data char \"",
            receiver: "wf1address\"",
            amount: 10,
            expectedRevert: abi.encodeWithSelector(
                CosmosIFTSendCallConstructor.CosmosIFTInvalidReceiver.selector, "wf1address\""
            )
        });

        return testCases;
    }

    function tableCosmosMintCallTest(CosmosMintCallTestCase memory cosmosMintCallTC) public {
        string memory typeUrl = "/wfchain.ift.MsgIFTMint";
        string memory tokenDenom = "testift";
        string memory ica = "wf1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5a9p63";

        CosmosIFTSendCallConstructor constructor_ = new CosmosIFTSendCallConstructor(typeUrl, tokenDenom, ica);

        if (cosmosMintCallTC.expectedRevert.length != 0) {
            vm.expectRevert(cosmosMintCallTC.expectedRevert);
            constructor_.constructMintCall(cosmosMintCallTC.receiver, cosmosMintCallTC.amount);
            return;
        }

        bytes memory callData = constructor_.constructMintCall(cosmosMintCallTC.receiver, cosmosMintCallTC.amount);

        bytes memory expected = abi.encodePacked(
            "{\"messages\":[{\"@type\":\"",
            typeUrl,
            "\",\"signer\":\"",
            ica,
            "\",\"denom\":\"",
            tokenDenom,
            "\",\"receiver\":\"",
            cosmosMintCallTC.receiver,
            "\",\"amount\":\"",
            Strings.toString(cosmosMintCallTC.amount),
            "\"}]}"
        );

        assertEq(callData, expected);
    }

    function test_cosmosCallConstructor_supportsInterface() public {
        CosmosIFTSendCallConstructor constructor_ =
            new CosmosIFTSendCallConstructor("/wfchain.ift.MsgIFTMint", "testift", "wf1ica");

        assertTrue(constructor_.supportsInterface(type(IIFTSendCallConstructor).interfaceId));

        bytes4 erc165Id = 0x01ffc9a7;
        assertTrue(constructor_.supportsInterface(erc165Id));

        bytes4 randomId = 0xdeadbeef;
        assertFalse(constructor_.supportsInterface(randomId));
    }
}

struct IFTMintTestCase {
    string name;
    bool ownable;
    address caller;
    IICS27GMPMsgs.AccountIdentifier accountId;
    address receiver;
    uint256 amount;
    bytes expectedRevert;
}

struct OnAckPacketTestCase {
    string name;
    address caller;
    bool success;
    IIBCAppCallbacks.OnAcknowledgementPacketCallback callback;
    bytes expectedRevert;
}

struct OnTimeoutPacketTestCase {
    string name;
    address caller;
    IIBCAppCallbacks.OnTimeoutPacketCallback callback;
    bytes expectedRevert;
}

struct IFTTransferTestCase {
    string name;
    address caller;
    bool ownable;
    string clientId;
    string receiver;
    uint256 amount;
    uint64 timeoutTimestamp;
    bytes expectedRevert;
}

struct RemoveIFTBridgeTestCase {
    string name;
    address caller;
    bool ownable;
    string clientId;
    bytes expectedRevert;
}

struct RegisterIFTBridgeTestCase {
    string name;
    address caller;
    bool ownable;
    string clientId;
    address iftSendCallConstructor;
    string counterpartyIFT;
    bytes expectedRevert;
}

struct UpgradeTestCase {
    string name;
    address caller;
    bool ownable;
    bytes expectedRevert;
}

struct CosmosMintCallTestCase {
    string name;
    string receiver;
    uint256 amount;
    bytes expectedRevert;
}
