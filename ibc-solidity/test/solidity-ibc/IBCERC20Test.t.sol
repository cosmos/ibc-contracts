// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length

import { Test } from "forge-std/Test.sol";

import { IERC20Errors } from "@openzeppelin-contracts/interfaces/draft-IERC6093.sol";
import { IIBCERC20Errors } from "../../contracts/errors/IIBCERC20Errors.sol";

import { IBCERC20 } from "../../contracts/utils/IBCERC20.sol";
import { Escrow } from "../../contracts/utils/Escrow.sol";
import { BeaconProxy } from "@openzeppelin-contracts/proxy/beacon/BeaconProxy.sol";
import { UpgradeableBeacon } from "@openzeppelin-contracts/proxy/beacon/UpgradeableBeacon.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";

contract IBCERC20Test is Test {
    IBCERC20 public ibcERC20;
    Escrow public escrow;
    AccessManager public accessManager;

    address public metadataCustomizer = makeAddr("metadataCustomizer");

    function setUp() public {
        address _escrowLogic = address(new Escrow());
        address escrowBeacon = address(new UpgradeableBeacon(_escrowLogic, address(this)));

        accessManager = new AccessManager(address(this));

        escrow = Escrow(
            address(
                new BeaconProxy(
                    escrowBeacon, abi.encodeCall(Escrow.initialize, (address(this), address(accessManager)))
                )
            )
        );

        IBCERC20 _ibcERC20Logic = new IBCERC20();
        address ibcERC20Beacon = address(new UpgradeableBeacon(address(_ibcERC20Logic), address(this)));
        ibcERC20 = IBCERC20(
            address(
                new BeaconProxy(
                    address(ibcERC20Beacon),
                    abi.encodeCall(_ibcERC20Logic.initialize, (address(this), address(escrow), "full/denom/path/test"))
                )
            )
        );
    }

    function test_ERC20DefaultMetadata() public view {
        assertEq(ibcERC20.ics20(), address(this));
        assertEq(ibcERC20.escrow(), address(escrow));
        assertEq(ibcERC20.name(), "full/denom/path/test");
        assertEq(ibcERC20.symbol(), "full/denom/path/test");
        assertEq(ibcERC20.fullDenomPath(), "full/denom/path/test");
        assertEq(ibcERC20.decimals(), 18);
        assertEq(0, ibcERC20.totalSupply());
    }

    function testFuzz_success_Mint(uint256 amount) public {
        ibcERC20.mint(address(escrow), amount);
        assertEq(ibcERC20.balanceOf(address(escrow)), amount);
        assertEq(ibcERC20.totalSupply(), amount);
    }

    // Just to document the behaviour
    function test_MintZero() public {
        ibcERC20.mint(address(escrow), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), 0);
        assertEq(ibcERC20.totalSupply(), 0);
    }

    function testFuzz_failure_Mint(uint256 amount) public {
        // unauthorized mint
        address notICS20Transfer = makeAddr("notICS20Transfer");
        vm.expectRevert(abi.encodeWithSelector(IIBCERC20Errors.IBCERC20Unauthorized.selector, notICS20Transfer));
        vm.prank(notICS20Transfer);
        ibcERC20.mint(address(escrow), amount);
        assertEq(ibcERC20.balanceOf(notICS20Transfer), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), 0);
        assertEq(ibcERC20.totalSupply(), 0);

        // non-esrow mint
        address notEscrow = makeAddr("notEscrow");
        vm.expectRevert(abi.encodeWithSelector(IIBCERC20Errors.IBCERC20NotEscrow.selector, address(escrow), notEscrow));
        ibcERC20.mint(notEscrow, amount);
        assertEq(ibcERC20.balanceOf(notICS20Transfer), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), 0);
        assertEq(ibcERC20.totalSupply(), 0);
    }

    function testFuzz_success_Burn(uint256 startingAmount, uint256 burnAmount) public {
        burnAmount = bound(burnAmount, 0, startingAmount);
        ibcERC20.mint(address(escrow), startingAmount);
        assertEq(ibcERC20.balanceOf(address(escrow)), startingAmount);

        ibcERC20.burn(address(escrow), burnAmount);
        uint256 leftOver = startingAmount - burnAmount;
        assertEq(ibcERC20.balanceOf(address(escrow)), leftOver);
        assertEq(ibcERC20.totalSupply(), leftOver);

        if (leftOver != 0) {
            ibcERC20.burn(address(escrow), leftOver);
            assertEq(ibcERC20.balanceOf(address(escrow)), 0);
            assertEq(ibcERC20.totalSupply(), 0);
        }
    }

    function testFuzz_failure_Burn(uint256 startingAmount, uint256 burnAmount) public {
        burnAmount = bound(burnAmount, 0, startingAmount);
        ibcERC20.mint(address(escrow), startingAmount);
        assertEq(ibcERC20.balanceOf(address(escrow)), startingAmount);

        // unauthorized burn
        address notICS20Transfer = makeAddr("notICS20Transfer");
        vm.expectRevert(abi.encodeWithSelector(IIBCERC20Errors.IBCERC20Unauthorized.selector, notICS20Transfer));
        vm.prank(notICS20Transfer);
        ibcERC20.burn(address(escrow), burnAmount);
        assertEq(ibcERC20.balanceOf(notICS20Transfer), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), startingAmount);
        assertEq(ibcERC20.totalSupply(), startingAmount);

        // non-esrow burn
        address notEscrow = makeAddr("notEscrow");
        vm.expectRevert(abi.encodeWithSelector(IIBCERC20Errors.IBCERC20NotEscrow.selector, address(escrow), notEscrow));
        ibcERC20.burn(notEscrow, burnAmount);
        assertEq(ibcERC20.balanceOf(notICS20Transfer), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), startingAmount);
        assertEq(ibcERC20.totalSupply(), startingAmount);
    }

    // Just to document the behaviour
    function test_BurnZero() public {
        ibcERC20.burn(address(escrow), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), 0);
        assertEq(ibcERC20.totalSupply(), 0);

        ibcERC20.mint(address(escrow), 1000);
        ibcERC20.burn(address(escrow), 0);
        assertEq(ibcERC20.balanceOf(address(escrow)), 1000);
        assertEq(ibcERC20.totalSupply(), 1000);
    }

    function test_failure_Burn() public {
        // test burn with zero balance
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(escrow), 0, 1));
        ibcERC20.burn(address(escrow), 1);

        // mint some to test other cases
        ibcERC20.mint(address(escrow), 1000);

        // test burn with insufficient balance
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(escrow), 1000, 1001)
        );
        ibcERC20.burn(address(escrow), 1001);
    }
}
