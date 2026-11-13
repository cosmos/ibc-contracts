// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";
import { VmSafe } from "forge-std/Vm.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { Initializable } from "@openzeppelin-contracts/proxy/utils/Initializable.sol";
import { ERC1967Utils } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Utils.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { ICS26Router } from "@ibc-contracts/ICS26Router.sol";
import { IICS02ClientMsgs } from "@ibc-contracts/msgs/IICS02ClientMsgs.sol";

import { BesuQBFTZKLightClient } from "../../../contracts/src/BesuQBFTZKLightClient.sol";
import { BesuZKTypes } from "../../../contracts/src/BesuZKTypes.sol";
import { DeployLightClient } from "../../../contracts/script/DeployLightClient.s.sol";
import { ProfileJson } from "../../../contracts/script/ProfileJson.sol";
import { RegisterLightClient } from "../../../contracts/script/RegisterLightClient.s.sol";
import { LightClientProxy } from "../../../contracts/test/utils/LightClientProxy.sol";

contract ProfileLoader {
    function load(string memory path) external view returns (string memory) {
        return ProfileJson.load(path);
    }
}

/// @dev The fixture profile names no real chain: the scripts are chain agnostic, and every
/// chain-specific value lives in `config/`.
contract DeployLightClientProfileTest is Test {
    uint256 private constant DEPLOYER_KEY = 0xA11CE;
    uint256 private constant CHAIN_ID = 1111;
    uint256 private constant ANCHOR_TIMESTAMP = 1_000_000;
    uint256 private constant TRUSTING_PERIOD = 1_209_600;
    address private constant SOURCE_ROUTER = 0x2222222222222222222222222222222222222222;
    address private constant GATEWAY = 0x3333333333333333333333333333333333333333;

    DeployLightClient private deployScript;
    RegisterLightClient private registerScript;
    ICS26Router private router;
    AccessManager private manager;
    string private profile;

    function setUp() public {
        deployScript = new DeployLightClient();
        registerScript = new RegisterLightClient();
        vm.setEnv("DEPLOYER_PRIVATE_KEY", vm.toString(DEPLOYER_KEY));
        vm.chainId(CHAIN_ID);
        vm.warp(ANCHOR_TIMESTAMP + 60);
        manager = new AccessManager(address(this));
        router = _newRouter(address(manager));

        profile = string.concat(
            '{"name":"destination-from-source","chainId":1111,',
            '"deployer":"',
            vm.toString(vm.addr(DEPLOYER_KEY)),
            '",',
            '"localIbcRouter":"',
            vm.toString(address(router)),
            '",',
            '"sp1VerifierGateway":"0x3333333333333333333333333333333333333333",',
            '"upgradeAuthority":"',
            vm.toString(address(manager)),
            '",',
            '"sp1Groth16Selector":"0x12345678","proofMode":"groth16",',
            '"source":{"name":"source","chainId":2222,',
            '"ibcRouter":"0x2222222222222222222222222222222222222222","validatorMode":"header-qbft"},',
            '"vkeys":{',
            '"updateClient":"0x007aa78cbcbcdc2e78276048d6869d7ca3660dbe971b0eee71c19af557f05c37",',
            '"membership":"0x0003935e54dd412b774d87c51fa284adab23a44a684e60ad6e4b994f594a54eb",',
            '"nonMembership":"0x00bcf499be696412b24b6a16d395c31018c302bb6bd8129feb17a5a375ae43a6"},',
            '"bootstrap":{"height":7,',
            '"stateRoot":"0x4444444444444444444444444444444444444444444444444444444444444444",',
            '"blockTimestamp":1000000,',
            '"validatorSetHash":"0x5555555555555555555555555555555555555555555555555555555555555555"},',
            '"trustingPeriodSeconds":1209600,"maxClockDriftSeconds":20,',
            '"registration":{"clientId":"client-0","counterpartyClientId":"client-3"}}'
        );
    }

    // --- DeployLightClient ---------------------------------------------------------------------

    function test_deploysWhatTheProfileSays() public {
        BesuQBFTZKLightClient client = _deploy(profile);

        assertEq(client.LOCAL_IBC_ROUTER(), address(router));
        assertEq(client.SOURCE_IBC_ROUTER(), SOURCE_ROUTER);
        assertEq(client.SP1_VERIFIER(), GATEWAY);
        assertEq(
            client.UPDATE_CLIENT_VKEY(),
            0x007aa78cbcbcdc2e78276048d6869d7ca3660dbe971b0eee71c19af557f05c37
        );
        assertEq(
            client.MEMBERSHIP_VKEY(),
            0x0003935e54dd412b774d87c51fa284adab23a44a684e60ad6e4b994f594a54eb
        );
        assertEq(
            client.NON_MEMBERSHIP_VKEY(),
            0x00bcf499be696412b24b6a16d395c31018c302bb6bd8129feb17a5a375ae43a6
        );
        assertEq(client.latestHeight(), 7);
        assertEq(client.trustingPeriodSeconds(), TRUSTING_PERIOD);
        assertEq(client.maxClockDriftSeconds(), 20);
        assertEq(client.authority(), address(manager));
        (bool exists, BesuZKTypes.ConsensusState memory anchor) = client.getConsensusState(7);
        assertTrue(exists);
        assertEq(
            anchor.stateRoot,
            bytes32(0x4444444444444444444444444444444444444444444444444444444444444444)
        );
        assertEq(anchor.blockTimestamp, ANCHOR_TIMESTAMP);
        assertEq(
            anchor.validatorSetHash,
            bytes32(0x5555555555555555555555555555555555555555555555555555555555555555)
        );
    }

    function test_deploysAProxyInitializedInItsConstructor() public {
        (BesuQBFTZKLightClient client, address implementation) = deployScript.deploy(profile);

        assertEq(
            address(uint160(uint256(vm.load(address(client), ERC1967Utils.IMPLEMENTATION_SLOT)))),
            implementation
        );
        assertEq(BesuQBFTZKLightClient(implementation).LOCAL_IBC_ROUTER(), address(0));
        BesuQBFTZKLightClient.InitParams memory p = deployScript.parseParams(profile);
        vm.expectRevert(Initializable.InvalidInitialization.selector);
        client.initialize(address(manager), p);
        vm.expectRevert(Initializable.InvalidInitialization.selector);
        BesuQBFTZKLightClient(implementation).initialize(address(manager), p);
    }

    function test_deploymentDoesNotRequireTheRouterAuthority() public {
        address other = address(new AccessManager(address(this)));
        string memory json = vm.replace(profile, vm.toString(address(manager)), vm.toString(other));
        assertEq(_deploy(json).authority(), other);
    }

    function test_refusesARouterAuthorityWithoutCode() public {
        address eoa = vm.addr(0xE0A);
        string memory json = vm.replace(
            vm.replace(
                profile, vm.toString(address(router)), vm.toString(address(_newRouter(eoa)))
            ),
            vm.toString(address(manager)),
            vm.toString(eoa)
        );
        vm.expectRevert(
            abi.encodeWithSelector(DeployLightClient.NoCodeAtUpgradeAuthority.selector, eoa)
        );
        deployScript.deploy(json);
    }

    function test_nullUpgradeAuthorityIsNamed() public {
        string memory json = vm.replace(
            profile,
            string.concat('"upgradeAuthority":"', vm.toString(address(manager)), '"'),
            '"upgradeAuthority":null'
        );
        vm.expectRevert(
            abi.encodeWithSelector(ProfileJson.ProfileFieldMissing.selector, ".upgradeAuthority")
        );
        deployScript.deploy(json);
    }

    function test_nullFieldIsNamed() public {
        string memory json = vm.replace(
            profile,
            '"stateRoot":"0x4444444444444444444444444444444444444444444444444444444444444444"',
            '"stateRoot":null'
        );
        vm.expectRevert(
            abi.encodeWithSelector(ProfileJson.ProfileFieldMissing.selector, ".bootstrap.stateRoot")
        );
        deployScript.deploy(json);
    }

    function test_absentFieldIsNamed() public {
        string memory json = vm.replace(
            profile, '"sp1VerifierGateway":"0x3333333333333333333333333333333333333333",', ""
        );
        vm.expectRevert(
            abi.encodeWithSelector(ProfileJson.ProfileFieldMissing.selector, ".sp1VerifierGateway")
        );
        deployScript.deploy(json);
    }

    function test_nullChainIdIsNamed() public {
        string memory json = vm.replace(profile, '"chainId":1111', '"chainId":null');
        vm.expectRevert(
            abi.encodeWithSelector(ProfileJson.ProfileFieldMissing.selector, ".chainId")
        );
        deployScript.deploy(json);
    }

    function test_refusesTheWrongChain() public {
        vm.chainId(2222);
        vm.expectRevert(
            abi.encodeWithSelector(ProfileJson.ChainIdMismatch.selector, CHAIN_ID, 2222)
        );
        deployScript.deploy(profile);
    }

    function test_refusesAKeyThatIsNotTheDeployer() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                ProfileJson.DeployerMismatch.selector, vm.addr(0xB0B), vm.addr(DEPLOYER_KEY)
            )
        );
        deployScript.deploy(_otherDeployer());
    }

    function test_refusesAnAnchorAtTheEndOfTheTrustingPeriod() public {
        vm.warp(ANCHOR_TIMESTAMP + TRUSTING_PERIOD);
        vm.expectRevert(
            abi.encodeWithSelector(
                DeployLightClient.BootstrapExpired.selector,
                ANCHOR_TIMESTAMP + TRUSTING_PERIOD,
                ANCHOR_TIMESTAMP + TRUSTING_PERIOD
            )
        );
        deployScript.deploy(profile);
    }

    function test_acceptsAnAnchorOneSecondBeforeExpiry() public {
        vm.warp(ANCHOR_TIMESTAMP + TRUSTING_PERIOD - 1);
        _deploy(profile);
    }

    function test_zeroTrustingPeriodNeverExpires() public {
        string memory json =
            vm.replace(profile, '"trustingPeriodSeconds":1209600', '"trustingPeriodSeconds":0');
        vm.warp(ANCHOR_TIMESTAMP + 100 * TRUSTING_PERIOD);
        assertEq(_deploy(json).trustingPeriodSeconds(), 0);
    }

    function test_refusesAValueAboveUint64() public {
        string memory json = vm.replace(
            profile, '"maxClockDriftSeconds":20', '"maxClockDriftSeconds":"18446744073709551616"'
        );
        vm.expectRevert(
            abi.encodeWithSelector(
                DeployLightClient.ProfileValueDoesNotFitUint64.selector,
                ".maxClockDriftSeconds",
                uint256(type(uint64).max) + 1
            )
        );
        deployScript.deploy(json);
    }

    function test_readsAnIntegerWrittenAsADecimalString() public {
        string memory json = vm.replace(
            profile, '"trustingPeriodSeconds":1209600', '"trustingPeriodSeconds":"1209600"'
        );
        assertEq(_deploy(json).trustingPeriodSeconds(), TRUSTING_PERIOD);
    }

    // --- RegisterLightClient -------------------------------------------------------------------

    function test_registersWithTheProfilesIds() public {
        address client = address(_deploy(profile));

        assertEq(registerScript.register(profile, client), "client-0");
        assertEq(address(router.getClient("client-0")), client);
        IICS02ClientMsgs.CounterpartyInfo memory counterparty = router.getCounterparty("client-0");
        assertEq(counterparty.clientId, "client-3");
        assertEq(counterparty.merklePrefix.length, 1);
        assertEq(counterparty.merklePrefix[0], bytes(""));
    }

    function test_registersWithACustomProfileId() public {
        string memory customProfile =
            vm.replace(profile, '"clientId":"client-0"', '"clientId":"zk-41001-41002"');
        router.grantClientIdCustomizerRole(vm.addr(DEPLOYER_KEY));
        address client = address(_deploy(customProfile));

        assertEq(registerScript.register(customProfile, client), "zk-41001-41002");
        assertEq(address(router.getClient("zk-41001-41002")), client);
        IICS02ClientMsgs.CounterpartyInfo memory counterparty =
            router.getCounterparty("zk-41001-41002");
        assertEq(counterparty.clientId, "client-3");
        assertEq(counterparty.merklePrefix.length, 1);
        assertEq(counterparty.merklePrefix[0], bytes(""));
    }

    function test_refusesWhenTheRouterWouldAssignAnotherId() public {
        registerScript.register(profile, address(_deploy(profile)));
        address second = address(_deploy(profile));

        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.ClientIdUnavailable.selector, "client-0", "client-1"
            )
        );
        registerScript.register(profile, second);
    }

    function test_refusesAClientForAnotherRouter() public {
        address client = address(_deploy(profile));
        string memory json = vm.replace(
            profile,
            vm.toString(address(router)),
            vm.toString(address(_newRouter(address(manager))))
        );

        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".localIbcRouter"
            )
        );
        registerScript.register(json, client);
    }

    function test_refusesAClientWithAnotherAnchor() public {
        address client = address(_deploy(profile));
        string memory json = vm.replace(
            profile,
            "0x4444444444444444444444444444444444444444444444444444444444444444",
            "0x4444444444444444444444444444444444444444444444444444444444444445"
        );
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".bootstrap.stateRoot"
            )
        );
        registerScript.register(json, client);

        json = vm.replace(profile, '"bootstrap":{"height":7,', '"bootstrap":{"height":8,');
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".bootstrap.height"
            )
        );
        registerScript.register(json, client);
    }

    function test_refusesAClientWithOtherParameters() public {
        address client = address(_deploy(profile));
        string memory json =
            vm.replace(profile, '"maxClockDriftSeconds":20', '"maxClockDriftSeconds":21');
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".maxClockDriftSeconds"
            )
        );
        registerScript.register(json, client);

        json = vm.replace(profile, '"trustingPeriodSeconds":1209600', '"trustingPeriodSeconds":0');
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector,
                ".trustingPeriodSeconds"
            )
        );
        registerScript.register(json, client);
    }

    function test_refusesAClientWithOtherProgramKeys() public {
        address client = address(_deploy(profile));
        string memory json = vm.replace(
            profile,
            "0x0003935e54dd412b774d87c51fa284adab23a44a684e60ad6e4b994f594a54eb",
            "0x0003935e54dd412b774d87c51fa284adab23a44a684e60ad6e4b994f594a54ec"
        );

        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".vkeys.membership"
            )
        );
        registerScript.register(json, client);
    }

    function test_refusesAClientWithAnotherUpgradeAuthority() public {
        address client = address(
            LightClientProxy.deploy(
                address(new AccessManager(address(this))), deployScript.parseParams(profile)
            )
        );
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".upgradeAuthority"
            )
        );
        registerScript.register(profile, client);
    }

    function test_refusesABareImplementation() public {
        (, address implementation) = deployScript.deploy(profile);
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.LightClientDoesNotMatchProfile.selector, ".localIbcRouter"
            )
        );
        registerScript.register(profile, implementation);
    }

    function test_refusesAnAddressWithoutCode() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                RegisterLightClient.NoCodeAtLightClient.selector, address(0xC0DE)
            )
        );
        registerScript.register(profile, address(0xC0DE));
    }

    function test_refusesAnEmptyCounterpartyClientId() public {
        address client = address(_deploy(profile));
        string memory json =
            vm.replace(profile, '"counterpartyClientId":"client-3"', '"counterpartyClientId":""');

        vm.expectRevert(
            abi.encodeWithSelector(
                ProfileJson.ProfileFieldMissing.selector, ".registration.counterpartyClientId"
            )
        );
        registerScript.register(json, client);
    }

    function test_registrationRefusesTheWrongChain() public {
        address client = address(_deploy(profile));
        vm.chainId(2222);
        vm.expectRevert(
            abi.encodeWithSelector(ProfileJson.ChainIdMismatch.selector, CHAIN_ID, 2222)
        );
        registerScript.register(profile, client);
    }

    function test_registrationRefusesAKeyThatIsNotTheDeployer() public {
        address client = address(_deploy(profile));
        vm.expectRevert(
            abi.encodeWithSelector(
                ProfileJson.DeployerMismatch.selector, vm.addr(0xB0B), vm.addr(DEPLOYER_KEY)
            )
        );
        registerScript.register(_otherDeployer(), client);
    }

    // --- Profiles ------------------------------------------------------------------------------

    function test_refusesToReadOutsideConfig() public {
        ProfileLoader loader = new ProfileLoader();
        vm.expectRevert();
        loader.load("../../release/expected-vkeys.json");
    }

    /// @dev Every committed profile, whatever its chains. A missing key is a typo; a value still to
    /// be supplied is `null`.
    function test_committedProfilesCarryEveryField() public view {
        string[25] memory fields = [
            ".name",
            ".description",
            ".notes",
            ".chainId",
            ".deployer",
            ".localIbcRouter",
            ".sp1VerifierGateway",
            ".upgradeAuthority",
            ".sp1Groth16Selector",
            ".proofMode",
            ".source.name",
            ".source.chainId",
            ".source.ibcRouter",
            ".source.validatorMode",
            ".vkeys.updateClient",
            ".vkeys.membership",
            ".vkeys.nonMembership",
            ".bootstrap.height",
            ".bootstrap.stateRoot",
            ".bootstrap.blockTimestamp",
            ".bootstrap.validatorSetHash",
            ".trustingPeriodSeconds",
            ".maxClockDriftSeconds",
            ".registration.clientId",
            ".registration.counterpartyClientId"
        ];
        uint256 profiles;
        VmSafe.DirEntry[] memory entries = vm.readDir("../../config");
        for (uint256 i; i < entries.length; ++i) {
            string memory path = entries[i].path;
            string[] memory parts = vm.split(path, "/");
            string memory fileName = parts[parts.length - 1];
            if (entries[i].isDir || !_endsWith(fileName, ".json")) continue;
            ++profiles;
            string memory json = ProfileJson.load(path);
            assertEq(
                string.concat(vm.parseJsonString(json, ".name"), ".json"),
                fileName,
                "name must match the file"
            );
            for (uint256 j; j < fields.length; ++j) {
                assertTrue(vm.keyExistsJson(json, fields[j]), string.concat(path, fields[j]));
            }
        }
        assertGt(profiles, 0, "no profiles found in ../../config");
    }

    function _deploy(string memory json) private returns (BesuQBFTZKLightClient client) {
        (client,) = deployScript.deploy(json);
    }

    function _newRouter(address authority) private returns (ICS26Router) {
        return ICS26Router(
            address(
                new ERC1967Proxy(
                    address(new ICS26Router()), abi.encodeCall(ICS26Router.initialize, (authority))
                )
            )
        );
    }

    function _otherDeployer() private view returns (string memory) {
        return vm.replace(profile, vm.toString(vm.addr(DEPLOYER_KEY)), vm.toString(vm.addr(0xB0B)));
    }

    function _endsWith(string memory value, string memory suffix) private pure returns (bool) {
        bytes memory v = bytes(value);
        bytes memory s = bytes(suffix);
        if (v.length < s.length) return false;
        for (uint256 i; i < s.length; ++i) {
            if (v[v.length - s.length + i] != s[i]) return false;
        }
        return true;
    }
}
