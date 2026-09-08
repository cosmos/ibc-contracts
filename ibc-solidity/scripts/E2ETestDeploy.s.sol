// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/*
    This script is used for end-to-end testing
*/

// solhint-disable custom-errors,gas-custom-errors,function-max-lines

import { stdJson } from "forge-std/StdJson.sol";
import { Script } from "forge-std/Script.sol";

import { IICS07TendermintMsgs } from "../contracts/light-clients/sp1-ics07/msgs/IICS07TendermintMsgs.sol";
import { ICS26Router } from "../contracts/ICS26Router.sol";
import { ICS20Transfer } from "../contracts/ICS20Transfer.sol";
import { ICS27GMP } from "../contracts/ICS27GMP.sol";
import { TestERC20 } from "../test/solidity-ibc/mocks/TestERC20.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { ICS20Lib } from "../contracts/utils/ICS20Lib.sol";
import { ICS27Lib } from "../contracts/utils/ICS27Lib.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { DeployAccessManagerWithRoles } from "./deployments/DeployAccessManagerWithRoles.sol";
import { IBCERC20 } from "../contracts/utils/IBCERC20.sol";
import { Escrow } from "../contracts/utils/Escrow.sol";
import { ICS27Account } from "../contracts/utils/ICS27Account.sol";
import { IFTOwnable } from "../contracts/utils/IFTOwnable.sol";
import { CosmosIFTSendCallConstructor } from "../contracts/utils/CosmosIFTSendCallConstructor.sol";

import { SP1Verifier as SP1VerifierPlonk } from "@sp1-contracts/v6.1.0/SP1VerifierPlonk.sol";
import { SP1Verifier as SP1VerifierGroth16 } from "@sp1-contracts/v6.1.0/SP1VerifierGroth16.sol";
import { SP1MockVerifier } from "@sp1-contracts/SP1MockVerifier.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";

/// @dev See the Solidity Scripting tutorial: https://getfoundry.sh/guides/scripting-with-solidity
contract E2ETestDeploy is Script, IICS07TendermintMsgs, DeployAccessManagerWithRoles {
    using stdJson for string;

    /// @dev Protobuf type URL for the wfchain IFT mint message
    string internal constant IFT_MINT_TYPE_URL = "/wfchain.ift.MsgIFTMint";
    /// @dev Token denomination on the Cosmos side
    string internal constant IFT_TEST_DENOM = "testift";
    /// @dev ERC20 token name for the test IFT contract
    string internal constant IFT_TOKEN_NAME = "Test IFT";
    /// @dev ERC20 token symbol for the test IFT contract
    string internal constant IFT_TOKEN_SYMBOL = "TIFT";

    struct DeployedContracts {
        address verifierPlonk;
        address verifierGroth16;
        address verifierMock;
        address ics26Router;
        address ics20Transfer;
        address ics27Gmp;
        address erc20;
        address ift;
        address cosmosIftConstructor;
        address solanaIftConstructor;
    }

    function run() public returns (string memory) {
        address e2eFaucet = vm.envAddress("E2E_FAUCET_ADDRESS");
        string memory iftIcaAddress = vm.envOr("IFT_ICA_ADDRESS", string(""));

        vm.startBroadcast();
        DeployedContracts memory d = _deploy(e2eFaucet, iftIcaAddress);
        vm.stopBroadcast();

        return _toJson(d);
    }

    function _deploy(address e2eFaucet, string memory iftIcaAddress) internal returns (DeployedContracts memory d) {
        // Deploy SP1 verifiers
        d.verifierPlonk = address(new SP1VerifierPlonk());
        d.verifierGroth16 = address(new SP1VerifierGroth16());
        d.verifierMock = address(new SP1MockVerifier());

        // Deploy IBC core
        AccessManager accessManager = new AccessManager(msg.sender);
        address routerLogic = address(new ICS26Router());
        address transferLogic = address(new ICS20Transfer());
        address gmpLogic = address(new ICS27GMP());

        d.ics26Router =
            address(new ERC1967Proxy(routerLogic, abi.encodeCall(ICS26Router.initialize, (address(accessManager)))));

        d.ics20Transfer = address(
            new ERC1967Proxy(
                transferLogic,
                abi.encodeCall(
                    ICS20Transfer.initialize,
                    (d.ics26Router, address(new Escrow()), address(new IBCERC20()), address(0), address(accessManager))
                )
            )
        );

        d.ics27Gmp = address(
            new ERC1967Proxy(
                gmpLogic,
                abi.encodeCall(
                    ICS27GMP.initialize, (d.ics26Router, address(new ICS27Account()), address(accessManager))
                )
            )
        );

        // Deploy IFT
        address iftLogic = address(new IFTOwnable());
        d.ift = address(
            new ERC1967Proxy(
                iftLogic,
                abi.encodeCall(IFTOwnable.initialize, (msg.sender, IFT_TOKEN_NAME, IFT_TOKEN_SYMBOL, d.ics27Gmp))
            )
        );

        // Deploy CosmosIFTSendCallConstructor if ICA address is provided
        if (bytes(iftIcaAddress).length > 0) {
            d.cosmosIftConstructor =
                address(new CosmosIFTSendCallConstructor(IFT_MINT_TYPE_URL, IFT_TEST_DENOM, iftIcaAddress));
        }

        // NOTE: SolanaIFTSendCallConstructor is deployed separately via DeploySolanaIFTConstructor.s.sol
        // after both the EVM IFT contract and Solana mint are known.

        // Wire up access control and apps
        accessManagerSetTargetRoles(accessManager, d.ics26Router, d.ics20Transfer, d.ics27Gmp, true);
        accessManagerSetRoles(
            accessManager, new address[](0), new address[](0), new address[](0), msg.sender, msg.sender, msg.sender
        );
        ICS26Router(d.ics26Router).addIBCApp(ICS20Lib.DEFAULT_PORT_ID, d.ics20Transfer);
        ICS26Router(d.ics26Router).addIBCApp(ICS27Lib.DEFAULT_PORT_ID, d.ics27Gmp);

        // Deploy and mint test ERC20
        TestERC20 erc20 = new TestERC20();
        erc20.mint(e2eFaucet, type(uint256).max);
        d.erc20 = address(erc20);
    }

    function _toJson(DeployedContracts memory d) internal returns (string memory) {
        string memory json = "json";
        json.serialize("verifierPlonk", Strings.toHexString(d.verifierPlonk));
        json.serialize("verifierGroth16", Strings.toHexString(d.verifierGroth16));
        json.serialize("verifierMock", Strings.toHexString(d.verifierMock));
        json.serialize("ics26Router", Strings.toHexString(d.ics26Router));
        json.serialize("ics20Transfer", Strings.toHexString(d.ics20Transfer));
        json.serialize("ics27Gmp", Strings.toHexString(d.ics27Gmp));
        json.serialize("erc20", Strings.toHexString(d.erc20));
        json.serialize("ift", Strings.toHexString(d.ift));
        json.serialize("cosmosIftConstructor", Strings.toHexString(d.cosmosIftConstructor));
        return json.serialize("solanaIftConstructor", Strings.toHexString(d.solanaIftConstructor));
    }
}
