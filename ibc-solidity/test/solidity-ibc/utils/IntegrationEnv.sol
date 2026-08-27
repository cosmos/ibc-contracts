// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,immutable-vars-naming

import {Test} from "forge-std/Test.sol";

import {ISignatureTransfer} from "@uniswap/permit2/src/interfaces/ISignatureTransfer.sol";
import {IERC20} from "@openzeppelin-contracts/token/ERC20/IERC20.sol";

import {TestERC20} from "../mocks/TestERC20.sol";
import {TestHelper} from "./TestHelper.sol";
import {DeployPermit2} from "@uniswap/permit2/test/utils/DeployPermit2.sol";
import {PermitSignature} from "./PermitSignature.sol";

contract IntegrationEnv is Test, DeployPermit2 {
    TestHelper private _th = new TestHelper();
    PermitSignature private _permitHelper = new PermitSignature();

    TestERC20 public immutable _erc20;
    ISignatureTransfer public immutable _permit2;

    mapping(address userAddress => uint256 userPrivateKey) private _userPrivateKeys;

    constructor() {
        // Set the default starting balance for the ERC20 token
        _permit2 = ISignatureTransfer(deployPermit2());
        _erc20 = new TestERC20();
    }

    function permit2() public view returns (address) {
        return address(_permit2);
    }

    function erc20() public view returns (TestERC20) {
        return TestERC20(_erc20);
    }

    /// @notice Creates a new user and funds the user with the default amount of tokens
    function createAndFundUser() public returns (address user) {
        user = createAndFundUser(_erc20);
        return user;
    }

    /// @notice Creates a new user and funds the user with the specified amount of tokens
    function createAndFundUser(uint256 amount) public returns (address) {
        return createAndFundUser(_erc20, amount);
    }

    /// @notice Creates a new user and funds them with the default amount of tokens from the specified token
    function createAndFundUser(TestERC20 token) public returns (address) {
        return createAndFundUser(token, _th.DEFAULT_ERC20_STARTING_BALANCE());
    }

    /// @notice Creates a new user and funds them with the specified amount of tokens from the specified token
    function createAndFundUser(TestERC20 token, uint256 amount) public returns (address) {
        address user = createUser();
        fundUser(token, user, amount);

        return user;
    }

    function createUser() public returns (address) {
        string memory keyName = _th.randomString();
        (address keyAddr, uint256 key) = makeAddrAndKey(keyName);
        _userPrivateKeys[keyAddr] = key;
        return keyAddr;
    }

    function fundUser(address user, uint256 amount) public {
        return fundUser(_erc20, user, amount);
    }

    function fundUser(TestERC20 token, address user, uint256 amount) public {
        token.mint(user, amount);
    }

    function getPermitAndSignature(address user, address spender, uint256 amount)
        public
        returns (ISignatureTransfer.PermitTransferFrom memory, bytes memory)
    {
        return getPermitAndSignature(user, spender, amount, address(_erc20));
    }

    function getPermitAndSignature(address user, address spender, uint256 amount, address token)
        public
        returns (ISignatureTransfer.PermitTransferFrom memory permit, bytes memory sig)
    {
        uint256 privateKey = _userPrivateKeys[user];
        require(privateKey != 0, "User not found");

        permit = ISignatureTransfer.PermitTransferFrom({
            permitted: ISignatureTransfer.TokenPermissions({token: token, amount: amount}),
            nonce: vm.randomUint(),
            deadline: block.timestamp + 100
        });
        sig = _permitHelper.getPermitTransferSignature(permit, privateKey, spender, _permit2.DOMAIN_SEPARATOR());

        vm.prank(user);
        IERC20(token).approve(address(_permit2), amount);

        return (permit, sig);
    }
}
