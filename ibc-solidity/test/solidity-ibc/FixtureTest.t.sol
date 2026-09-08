// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,gas-custom-errors

import { Test } from "forge-std/Test.sol";
import { IICS26RouterMsgs } from "../../contracts/msgs/IICS26RouterMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { ICS26Router } from "../../contracts/ICS26Router.sol";
import { SP1ICS07Tendermint } from "../../contracts/light-clients/sp1-ics07/SP1ICS07Tendermint.sol";
import { ICS20Transfer } from "../../contracts/ICS20Transfer.sol";
import { IICS07TendermintMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IICS07TendermintMsgs.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { SP1Verifier as SP1VerifierPlonk } from "@sp1-contracts/v6.1.0/SP1VerifierPlonk.sol";
import { SP1Verifier as SP1VerifierGroth16 } from "@sp1-contracts/v6.1.0/SP1VerifierGroth16.sol";
import { IBCERC20 } from "../../contracts/utils/IBCERC20.sol";
import { Escrow } from "../../contracts/utils/Escrow.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { DeployAccessManagerWithRoles } from "../../scripts/deployments/DeployAccessManagerWithRoles.sol";

abstract contract FixtureTest is Test, IICS07TendermintMsgs, DeployAccessManagerWithRoles {
    ICS26Router public ics26Router;
    SP1ICS07Tendermint public sp1ICS07Tendermint;
    ICS20Transfer public ics20Transfer;
    AccessManager public accessManager;

    string public customClientId = "cosmoshub-1";
    string public counterpartyId = "08-wasm-0";
    bytes[] public merklePrefix = [bytes("ibc"), bytes("")];
    bytes[] public singleSuccessAck = [ICS20Lib.SUCCESSFUL_ACKNOWLEDGEMENT_JSON];

    string internal constant FIXTURE_DIR = "/test/solidity-ibc/fixtures/";

    using stdJson for string;

    struct SP1ICS07GenesisFixtureJson {
        bytes trustedClientState;
        bytes32 trustedConsensusStateHash;
        bytes32 updateClientVkey;
        bytes32 membershipVkey;
        bytes32 ucAndMembershipVkey;
        bytes32 misbehaviourVkey;
    }

    struct Fixture {
        SP1ICS07GenesisFixtureJson genesisFixture;
        bytes msg;
        address erc20Address;
        uint256 timestamp;
        IICS26RouterMsgs.Packet packet;
    }

    function setUp() public {
        // Warp to a timestamp before the fixture's timestamp to ensure the test starts in a clean state.
        vm.warp(1);

        // ============ Step 1: Deploy the logic contracts ==============
        address escrowLogic = address(new Escrow());
        address ibcERC20Logic = address(new IBCERC20());
        ICS26Router ics26RouterLogic = new ICS26Router();
        ICS20Transfer ics20TransferLogic = new ICS20Transfer();

        // ============== Step 2: Deploy ERC1967 Proxies ==============
        accessManager = new AccessManager(address(this));

        ERC1967Proxy routerProxy = new ERC1967Proxy(
            address(ics26RouterLogic), abi.encodeCall(ICS26Router.initialize, (address(accessManager)))
        );

        ERC1967Proxy transferProxy = new ERC1967Proxy(
            address(ics20TransferLogic),
            abi.encodeCall(
                ICS20Transfer.initialize,
                (address(routerProxy), escrowLogic, ibcERC20Logic, address(0), address(accessManager))
            )
        );

        // ============== Step 3: Wire up the contracts ==============
        ics26Router = ICS26Router(address(routerProxy));
        ics20Transfer = ICS20Transfer(address(transferProxy));

        accessManagerSetTargetRoles(accessManager, address(routerProxy), address(transferProxy), address(0), true);

        accessManager.grantRole(IBCRolesLib.ID_CUSTOMIZER_ROLE, address(this), 0);
    }

    function loadInitialFixture(string memory fixtureFileName) internal returns (Fixture memory) {
        Fixture memory fixture = loadFixture(fixtureFileName);

        ClientState memory trustedClientState = abi.decode(fixture.genesisFixture.trustedClientState, (ClientState));

        address verifier;
        if (trustedClientState.zkAlgorithm == SupportedZkAlgorithm.Plonk) {
            verifier = address(new SP1VerifierPlonk());
        } else if (trustedClientState.zkAlgorithm == SupportedZkAlgorithm.Groth16) {
            verifier = address(new SP1VerifierGroth16());
        } else {
            revert("Unsupported zk algorithm");
        }

        SP1ICS07Tendermint ics07Tendermint = new SP1ICS07Tendermint(
            fixture.genesisFixture.updateClientVkey,
            fixture.genesisFixture.membershipVkey,
            fixture.genesisFixture.ucAndMembershipVkey,
            fixture.genesisFixture.misbehaviourVkey,
            verifier,
            fixture.genesisFixture.trustedClientState,
            fixture.genesisFixture.trustedConsensusStateHash,
            address(ics26Router)
        );

        ics26Router.addClient(
            customClientId, IICS02ClientMsgs.CounterpartyInfo(counterpartyId, merklePrefix), address(ics07Tendermint)
        );
        ics26Router.addIBCApp(ICS20Lib.DEFAULT_PORT_ID, address(ics20Transfer));

        // Deploy ERC20 to the expected address from the fixture
        deployCodeTo("TestERC20.sol:TestERC20", fixture.erc20Address);

        return fixture;
    }

    function loadFixture(string memory fixtureFileName) internal returns (Fixture memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, FIXTURE_DIR, fixtureFileName);
        string memory json = vm.readFile(path);

        bytes memory sp1GenesisBz = json.readBytes(".sp1GenesisFixture");
        string memory sp1GenesisJSON = string(sp1GenesisBz);
        SP1ICS07GenesisFixtureJson memory genesisFixture;
        genesisFixture.trustedClientState = sp1GenesisJSON.readBytes(".trustedClientState");
        genesisFixture.trustedConsensusStateHash = sp1GenesisJSON.readBytes32(".trustedConsensusStateHash");
        genesisFixture.updateClientVkey = sp1GenesisJSON.readBytes32(".updateClientVkey");
        genesisFixture.membershipVkey = sp1GenesisJSON.readBytes32(".membershipVkey");
        genesisFixture.ucAndMembershipVkey = sp1GenesisJSON.readBytes32(".ucAndMembershipVkey");
        genesisFixture.misbehaviourVkey = sp1GenesisJSON.readBytes32(".misbehaviourVkey");

        bytes memory packetBz = json.readBytes(".packet");
        IICS26RouterMsgs.Packet memory packet = abi.decode(packetBz, (IICS26RouterMsgs.Packet));

        bytes memory msgBz = json.readBytes(".msg");
        address erc20Address = json.readAddress(".erc20Address");
        uint256 timestamp = json.readUint(".timestamp");

        vm.warp(timestamp);

        return Fixture({
            genesisFixture: genesisFixture, msg: msgBz, erc20Address: erc20Address, timestamp: timestamp, packet: packet
        });
    }
}
