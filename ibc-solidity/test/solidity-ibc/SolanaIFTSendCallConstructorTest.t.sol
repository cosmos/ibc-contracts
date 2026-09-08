// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,gas-small-strings,no-inline-assembly,gas-increment-by-one

import { Test } from "forge-std/Test.sol";

import { SolanaIFTSendCallConstructor } from "../../contracts/utils/SolanaIFTSendCallConstructor.sol";
import { IIFTSendCallConstructor } from "../../contracts/interfaces/IIFTSendCallConstructor.sol";
import { ISolanaGMPMsgs } from "../../contracts/utils/SolanaIFTSendCallConstructor.sol";
import { SafeCast } from "@openzeppelin-contracts/utils/math/SafeCast.sol";
import { IERC165 } from "@openzeppelin-contracts/utils/introspection/IERC165.sol";

contract SolanaIFTSendCallConstructorTest is Test {
    SolanaIFTSendCallConstructor public constructor_;

    bytes32 internal constant APP_STATE = keccak256("APP_STATE");
    bytes32 internal constant APP_MINT_STATE = keccak256("APP_MINT_STATE");
    bytes32 internal constant IFT_BRIDGE = keccak256("IFT_BRIDGE");
    bytes32 internal constant MINT = keccak256("MINT");
    bytes32 internal constant MINT_AUTHORITY = keccak256("MINT_AUTHORITY");
    bytes32 internal constant GMP_ACCOUNT = keccak256("GMP_ACCOUNT");

    // Receiver: "0x" + wallet(64 hex) + ata(64 hex) = 130 chars
    bytes32 internal constant WALLET = 0x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9;
    bytes32 internal constant ATA = 0x8c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859;

    string internal constant VALID_RECEIVER =
        "0x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859";

    // Well-known Solana program IDs
    bytes32 internal constant TOKEN_PROGRAM = 0x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9;
    bytes32 internal constant ASSOCIATED_TOKEN_PROGRAM =
        0x8c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859;
    bytes32 internal constant SYSTEM_PROGRAM = bytes32(0);

    uint256 internal constant PACKED_ACCOUNT_SIZE = 34;

    /// @dev Anchor discriminator for `ift_mint`: sha256("global:ift_mint")[0:8]
    bytes8 internal constant IFT_MINT_DISCRIMINATOR = 0x9cc9d99072afaa53;

    function setUp() public {
        constructor_ =
            new SolanaIFTSendCallConstructor(APP_STATE, APP_MINT_STATE, IFT_BRIDGE, MINT, MINT_AUTHORITY, GMP_ACCOUNT);
    }

    function test_immutables() public view {
        assertEq(constructor_.APP_STATE(), APP_STATE);
        assertEq(constructor_.APP_MINT_STATE(), APP_MINT_STATE);
        assertEq(constructor_.IFT_BRIDGE(), IFT_BRIDGE);
        assertEq(constructor_.MINT(), MINT);
        assertEq(constructor_.MINT_AUTHORITY(), MINT_AUTHORITY);
        assertEq(constructor_.GMP_ACCOUNT(), GMP_ACCOUNT);
    }

    function testFuzz_constructMintCall_success(uint64 amount) public view {
        vm.assume(amount > 0);
        bytes memory result = constructor_.constructMintCall(VALID_RECEIVER, uint256(amount));

        ISolanaGMPMsgs.GMPSolanaPayload memory payload = abi.decode(result, (ISolanaGMPMsgs.GMPSolanaPayload));

        // 12 accounts * 34 bytes each
        assertEq(payload.packedAccounts.length, 12 * PACKED_ACCOUNT_SIZE);
        // discriminator(8) + wallet(32) + amount_le(8) = 48
        assertEq(payload.instructionData.length, 48);
        // Lamports to pre-fund GMP PDA for ATA creation rent
        assertEq(payload.prefundLamports, 3_000_000);

        bytes memory packedAccounts = payload.packedAccounts;
        _assertAccount(packedAccounts, 0, APP_STATE, false, false);
        _assertAccount(packedAccounts, 1, APP_MINT_STATE, false, true);
        _assertAccount(packedAccounts, 2, IFT_BRIDGE, false, false);
        _assertAccount(packedAccounts, 3, MINT, false, true);
        _assertAccount(packedAccounts, 4, MINT_AUTHORITY, false, false);
        _assertAccount(packedAccounts, 5, ATA, false, true); // receiver token account
        _assertAccount(packedAccounts, 6, WALLET, false, false); // receiver owner
        _assertAccount(packedAccounts, 7, GMP_ACCOUNT, true, false); // gmp_account signer
        _assertAccount(packedAccounts, 8, GMP_ACCOUNT, true, true); // payer (signer + writable)
        _assertAccount(packedAccounts, 9, TOKEN_PROGRAM, false, false);
        _assertAccount(packedAccounts, 10, ASSOCIATED_TOKEN_PROGRAM, false, false);
        _assertAccount(packedAccounts, 11, SYSTEM_PROGRAM, false, false);
    }

    function testFuzz_constructMintCall_instructionData(uint64 amount) public view {
        vm.assume(amount > 0);
        bytes memory result = constructor_.constructMintCall(VALID_RECEIVER, uint256(amount));
        ISolanaGMPMsgs.GMPSolanaPayload memory payload = abi.decode(result, (ISolanaGMPMsgs.GMPSolanaPayload));
        bytes memory instructionData = payload.instructionData;

        bytes8 discriminator;
        assembly {
            discriminator := mload(add(instructionData, 32))
        }
        assertEq(discriminator, IFT_MINT_DISCRIMINATOR);

        bytes32 walletInData;
        assembly {
            walletInData := mload(add(instructionData, 40))
        }
        assertEq(walletInData, WALLET);

        assertEq(_decodeLittleEndianU64(instructionData, 40), uint256(amount));
    }

    struct ConstructMintCallRevertCase {
        string name;
        string receiver;
        uint256 amount;
        bytes expectedRevert;
    }

    function test_constructMintCall_reverts() public {
        string memory invalidHexReceiver =
            "0xGGddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a98c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859";
        string memory tooShortReceiver = "0xaabbccdd";
        string memory tooLongReceiver = string.concat(VALID_RECEIVER, "aabbccdd");
        uint256 overflowAmount = uint256(type(uint64).max) + 1;

        ConstructMintCallRevertCase[] memory cases = new ConstructMintCallRevertCase[](5);
        cases[0] = ConstructMintCallRevertCase({
            name: "invalid hex receiver",
            receiver: invalidHexReceiver,
            amount: 1000,
            expectedRevert: abi.encodeWithSelector(
                SolanaIFTSendCallConstructor.SolanaIFTInvalidReceiver.selector, invalidHexReceiver
            )
        });
        cases[1] = ConstructMintCallRevertCase({
            name: "receiver too short",
            receiver: tooShortReceiver,
            amount: 1000,
            expectedRevert: abi.encodeWithSelector(
                SolanaIFTSendCallConstructor.SolanaIFTInvalidReceiver.selector, tooShortReceiver
            )
        });
        cases[2] = ConstructMintCallRevertCase({
            name: "receiver too long",
            receiver: tooLongReceiver,
            amount: 1000,
            expectedRevert: abi.encodeWithSelector(
                SolanaIFTSendCallConstructor.SolanaIFTInvalidReceiver.selector, tooLongReceiver
            )
        });
        cases[3] = ConstructMintCallRevertCase({
            name: "zero amount",
            receiver: VALID_RECEIVER,
            amount: 0,
            expectedRevert: abi.encodeWithSelector(SolanaIFTSendCallConstructor.SolanaIFTZeroAmount.selector)
        });
        cases[4] = ConstructMintCallRevertCase({
            name: "amount overflow",
            receiver: VALID_RECEIVER,
            amount: overflowAmount,
            expectedRevert: abi.encodeWithSelector(SafeCast.SafeCastOverflowedUintDowncast.selector, 64, overflowAmount)
        });

        for (uint256 i = 0; i < cases.length; ++i) {
            ConstructMintCallRevertCase memory tc = cases[i];
            vm.expectRevert(tc.expectedRevert);
            constructor_.constructMintCall(tc.receiver, tc.amount);
        }
    }

    function test_supportsInterface() public view {
        assertTrue(constructor_.supportsInterface(type(IIFTSendCallConstructor).interfaceId));
        assertTrue(constructor_.supportsInterface(type(IERC165).interfaceId));
    }

    function _decodeLittleEndianU64(bytes memory data, uint256 offset) private pure returns (uint256) {
        uint64 decoded;
        for (uint256 i = 0; i < 8; i++) {
            decoded |= uint64(uint8(data[offset + i])) << uint64(i * 8);
        }
        return uint256(decoded);
    }

    function _assertAccount(
        bytes memory packed,
        uint256 index,
        bytes32 expectedPubkey,
        bool expectedSigner,
        bool expectedWritable
    )
        private
        pure
    {
        uint256 offset = index * PACKED_ACCOUNT_SIZE;

        bytes32 pubkey;
        assembly {
            pubkey := mload(add(add(packed, 32), offset))
        }
        assertEq(pubkey, expectedPubkey);

        uint8 isSigner = uint8(packed[offset + 32]);
        uint8 isWritable = uint8(packed[offset + 33]);
        assertEq(isSigner, expectedSigner ? 1 : 0);
        assertEq(isWritable, expectedWritable ? 1 : 0);
    }
}
