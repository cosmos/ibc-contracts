// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,immutable-vars-naming

import { Vm } from "forge-std/Vm.sol";

import { ICS26Router } from "../../../contracts/ICS26Router.sol";
import { ICS20Transfer } from "../../../contracts/ICS20Transfer.sol";
import { ICS27GMP } from "../../../contracts/ICS27GMP.sol";
import { Escrow } from "../../../contracts/utils/Escrow.sol";
import { IBCERC20 } from "../../../contracts/utils/IBCERC20.sol";
import { ICS27Account } from "../../../contracts/utils/ICS27Account.sol";
import { IBCMulticall } from "../../../contracts/utils/IBCMulticall.sol";
import { IBCRolesLib } from "../../../contracts/utils/IBCRolesLib.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";

import { IbcImpl } from "../../solidity-ibc/utils/IbcImpl.sol";
import { LightClientDriverTest } from "../../solidity-ibc/integration/drivers/LightClientDriverTest.sol";

/// @notice The IBC deployment on Ethereum mainnet
library MainnetDeployment {
    address internal constant ICS26_ROUTER = 0x3aF134307D5Ee90faa2ba9Cdba14ba66414CF1A7;
    address internal constant ICS20_TRANSFER = 0xa348CfE719B63151F228e3C30EB424BA5a983012;
    address internal constant ICS27_GMP = 0xbebd14A66052d7dc6BDc05e7328E4fEC0a9e3B0e;
    address internal constant ACCESS_MANAGER = 0x3fa3f45acE1645614c80679AeEcE0A82A93c77Ec;
    address internal constant TIMELOCKED_ADMIN = 0xb3999B2D30dD8c9faEcE5A8a503fAe42b8b1b614;
    address internal constant RELAYER = 0xC4C09A23dDBd1fF0f313885265113F83622284C2;

    /// @notice Forks are rounded down to a multiple of this many blocks
    uint256 internal constant FORK_BLOCK_STEP = 32;

    Vm private constant VM = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));

    /// @notice Selects a fork of mainnet 32 to 63 blocks behind the tip
    /// @dev The tip may be unknown to some nodes behind a load balancer. Forks made within the same 32 blocks share
    /// the block, and with it Foundry's cache of mainnet state.
    /// @param rpcUrl The mainnet RPC endpoint
    /// @return forkId The identifier of the selected fork
    function selectFork(string memory rpcUrl) internal returns (uint256 forkId) {
        bytes memory tipBz = VM.rpc(rpcUrl, "eth_blockNumber", "[]");
        uint256 tip = uint256(bytes32(tipBz)) >> (256 - 8 * tipBz.length);
        return VM.createSelectFork(rpcUrl, tip - tip % FORK_BLOCK_STEP - FORK_BLOCK_STEP);
    }
}

/// @notice The mainnet deployment upgraded to the current contracts, administered by the deployer
/// @dev Relaying stays restricted to the relayer role, which the deployer and this contract are granted
contract MainnetIbcImpl is IbcImpl {
    constructor(address permit2) IbcImpl(permit2) { }

    /// @notice Upgrades the mainnet deployment the way a governance proposal would
    function _deploy(address permit2) internal override returns (Contracts memory contracts) {
        contracts = Contracts({
            accessManager: AccessManager(MainnetDeployment.ACCESS_MANAGER),
            ics26Router: ICS26Router(MainnetDeployment.ICS26_ROUTER),
            ics20Transfer: ICS20Transfer(MainnetDeployment.ICS20_TRANSFER),
            ics27Gmp: ICS27GMP(MainnetDeployment.ICS27_GMP),
            ibcMulticall: IBCMulticall(
                address(
                    new ERC1967Proxy(
                        address(new IBCMulticall()),
                        abi.encodeCall(IBCMulticall.initialize, (MainnetDeployment.ACCESS_MANAGER))
                    )
                )
            )
        });
        address ics26RouterLogic = address(new ICS26Router());
        address ics20TransferLogic = address(new ICS20Transfer());
        address ics27GmpLogic = address(new ICS27GMP());
        address escrowLogic = address(new Escrow());
        address ibcERC20Logic = address(new IBCERC20());
        address ics27AccountLogic = address(new ICS27Account());

        vm.startPrank(MainnetDeployment.TIMELOCKED_ADMIN);
        contracts.ics26Router.upgradeToAndCall(ics26RouterLogic, "");
        contracts.ics20Transfer.upgradeToAndCall(ics20TransferLogic, "");
        contracts.ics27Gmp.upgradeToAndCall(ics27GmpLogic, "");
        contracts.ics20Transfer.upgradeEscrowTo(escrowLogic);
        contracts.ics20Transfer.upgradeIBCERC20To(ibcERC20Logic);
        contracts.ics27Gmp.upgradeAccountTo(ics27AccountLogic);

        accessManagerSetTargetRoles(
            contracts.accessManager,
            address(contracts.ics26Router),
            address(contracts.ics20Transfer),
            address(contracts.ics27Gmp),
            false
        );
        accessManagerSetMulticallRoles(contracts.accessManager, address(contracts.ibcMulticall), false);

        contracts.accessManager.grantRole(IBCRolesLib.ADMIN_ROLE, msg.sender, 0);
        contracts.accessManager.grantRole(IBCRolesLib.RELAYER_ROLE, msg.sender, 0);
        contracts.accessManager.grantRole(IBCRolesLib.ID_CUSTOMIZER_ROLE, msg.sender, 0);
        contracts.accessManager.grantRole(IBCRolesLib.ERC20_CUSTOMIZER_ROLE, msg.sender, 0);
        contracts.accessManager.grantRole(IBCRolesLib.RELAYER_ROLE, address(this), 0);
        vm.stopPrank();

        require(contracts.ics20Transfer.getPermit2() == permit2, "mainnet uses another permit2");
    }
}

/// @notice Runs a suite on a mainnet fork, with the upgraded mainnet deployment as the chain at `slot`
abstract contract WithMainnetFork is LightClientDriverTest {
    uint256 private immutable _mainnetSlot;

    constructor(uint256 mainnetSlot) {
        _mainnetSlot = mainnetSlot;
    }

    function _beforeSetUp() internal override {
        MainnetDeployment.selectFork(vm.envString("ETH_RPC_URL"));
    }

    function _newIbcImpl(uint256 slot, address permit2) internal override returns (IbcImpl) {
        if (slot == _mainnetSlot) {
            return new MainnetIbcImpl(permit2);
        }
        return new IbcImpl(permit2);
    }
}
