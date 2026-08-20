// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTSendCallConstructor } from "../interfaces/IIFTSendCallConstructor.sol";
import { ISolanaGMPMsgs } from "../msgs/ISolanaGMPMsgs.sol";

import { SafeCast } from "@openzeppelin-contracts/utils/math/SafeCast.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { ERC165 } from "@openzeppelin-contracts/utils/introspection/ERC165.sol";
import { IERC165 } from "@openzeppelin-contracts/utils/introspection/IERC165.sol";

/// @title Solana IFT Send Call Constructor
/// @notice Constructs ABI-encoded GmpSolanaPayload for minting IFT tokens on Solana.
/// @dev Stores 6 static PDAs as immutables (set at deployment, precomputed off-chain).
///      The `constructMintCall` builds the complete execution payload committed in the IBC packet,
///      eliminating the need for a relayer-controlled hint account.
contract SolanaIFTSendCallConstructor is IIFTSendCallConstructor, ERC165 {
    // -- Well-known Solana program IDs (constants) --

    /// @notice SPL Token program ID (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`)
    bytes32 private constant TOKEN_PROGRAM = 0x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9;
    /// @notice Associated Token Account program ID (`ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`)
    bytes32 private constant ASSOCIATED_TOKEN_PROGRAM =
        0x8c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859;
    /// @notice System program ID (`11111111111111111111111111111111`)
    bytes32 private constant SYSTEM_PROGRAM = bytes32(0);

    /// @notice Anchor discriminator for `ift_mint`: sha256("global:ift_mint")[0:8]
    bytes8 private constant IFT_MINT_DISCRIMINATOR = 0x9cc9d99072afaa53;

    /// @notice Lamports to pre-fund the GMP PDA for ATA creation rent (~2,039,280 lamports minimum)
    uint64 private constant PREFUND_LAMPORTS = 3_000_000;

    /// @notice Expected length of receiver: "0x" + 64 hex (wallet) + 64 hex (ATA) = 130 chars
    uint256 private constant SOLANA_RECEIVER_HEX_LENGTH = 130;

    // -- Static PDAs (set at deployment) --

    /// @notice IFT app state PDA
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public immutable APP_STATE;
    /// @notice IFT app mint state PDA
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public immutable APP_MINT_STATE;
    /// @notice IFT bridge PDA
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public immutable IFT_BRIDGE;
    /// @notice SPL token mint address
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public immutable MINT;
    /// @notice IFT mint authority PDA
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public immutable MINT_AUTHORITY;
    /// @notice GMP account PDA
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public immutable GMP_ACCOUNT;

    /// @notice Error thrown when the receiver address is invalid
    /// @param receiver The invalid receiver string
    error SolanaIFTInvalidReceiver(string receiver);

    /// @notice Error thrown when the amount is zero
    error SolanaIFTZeroAmount();

    /// @notice Initializes the constructor with 6 static Solana PDAs.
    /// @param _appState IFT app state PDA
    /// @param _appMintState IFT app mint state PDA
    /// @param _iftBridge IFT bridge PDA
    /// @param _mint SPL token mint address
    /// @param _mintAuthority IFT mint authority PDA
    /// @param _gmpAccount GMP account PDA
    constructor(
        bytes32 _appState,
        bytes32 _appMintState,
        bytes32 _iftBridge,
        bytes32 _mint,
        bytes32 _mintAuthority,
        bytes32 _gmpAccount
    ) {
        APP_STATE = _appState;
        APP_MINT_STATE = _appMintState;
        IFT_BRIDGE = _iftBridge;
        MINT = _mint;
        MINT_AUTHORITY = _mintAuthority;
        GMP_ACCOUNT = _gmpAccount;
    }

    /// @inheritdoc IIFTSendCallConstructor
    /// @dev Receiver format: "0x" + wallet_hex(64) + ata_hex(64) = 130 chars.
    function constructMintCall(string calldata receiver, uint256 amount) external view returns (bytes memory) {
        require(amount > 0, SolanaIFTZeroAmount());
        (bytes32 wallet, bytes32 ata) = _parseWalletAndAta(receiver);

        ISolanaGMPMsgs.GMPSolanaPayload memory payload = ISolanaGMPMsgs.GMPSolanaPayload({
            packedAccounts: _buildPackedAccounts(wallet, ata),
            instructionData: _buildInstructionData(wallet, amount),
            prefundLamports: PREFUND_LAMPORTS
        });

        return abi.encode(payload);
    }

    /// @inheritdoc ERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override(ERC165, IERC165) returns (bool) {
        return interfaceId == type(IIFTSendCallConstructor).interfaceId || super.supportsInterface(interfaceId);
    }

    /// @notice Parse wallet and ATA from the receiver hex string.
    /// @param receiver The receiver string in format "0x" + wallet_hex(64) + ata_hex(64)
    /// @return The receiver wallet pubkey
    /// @return The receiver associated token account
    function _parseWalletAndAta(string calldata receiver) private pure returns (bytes32, bytes32) {
        require(bytes(receiver).length == SOLANA_RECEIVER_HEX_LENGTH, SolanaIFTInvalidReceiver(receiver));
        require(bytes(receiver)[0] == "0" && bytes(receiver)[1] == "x", SolanaIFTInvalidReceiver(receiver));

        string memory walletHex = string(abi.encodePacked("0x", bytes(receiver)[2:66]));
        string memory ataHex = string(abi.encodePacked("0x", bytes(receiver)[66:130]));

        (bool successW, uint256 walletParsed) = Strings.tryParseHexUint(walletHex);
        require(successW, SolanaIFTInvalidReceiver(receiver));

        (bool successA, uint256 ataParsed) = Strings.tryParseHexUint(ataHex);
        require(successA, SolanaIFTInvalidReceiver(receiver));

        return (bytes32(walletParsed), bytes32(ataParsed));
    }

    /// @notice Build packed accounts for the GmpSolanaPayload.
    /// @param wallet The receiver wallet pubkey
    /// @param ata The receiver associated token account
    /// @return The packed account bytes (12 * 34 bytes)
    function _buildPackedAccounts(bytes32 wallet, bytes32 ata) private view returns (bytes memory) {
        return bytes.concat(
            _packAccount(APP_STATE, false, false),
            _packAccount(APP_MINT_STATE, false, true),
            _packAccount(IFT_BRIDGE, false, false),
            _packAccount(MINT, false, true),
            _packAccount(MINT_AUTHORITY, false, false),
            _packAccount(ata, false, true),
            _packAccount(wallet, false, false),
            // GMP PDA appears twice: as gmp_account (signer, read-only) and payer (signer, writable)
            _packAccount(GMP_ACCOUNT, true, false),
            _packAccount(GMP_ACCOUNT, true, true),
            _packAccount(TOKEN_PROGRAM, false, false),
            _packAccount(ASSOCIATED_TOKEN_PROGRAM, false, false),
            _packAccount(SYSTEM_PROGRAM, false, false)
        );
    }

    /// @notice Pack a single account entry: pubkey(32) + is_signer(1) + is_writable(1)
    /// @param pubkey The account pubkey
    /// @param isSigner Whether the account is a signer
    /// @param isWritable Whether the account is writable
    /// @return The packed 34-byte account entry
    function _packAccount(bytes32 pubkey, bool isSigner, bool isWritable) private pure returns (bytes memory) {
        return abi.encodePacked(pubkey, isSigner, isWritable);
    }

    /// @notice Build instruction data: discriminator(8) + wallet(32) + amount_le(8) = 48 bytes.
    /// @param wallet The receiver wallet pubkey
    /// @param amount The amount to mint
    /// @return The Borsh-encoded instruction data matching Anchor's IFTMintMsg
    function _buildInstructionData(bytes32 wallet, uint256 amount) private pure returns (bytes memory) {
        uint64 amountU64 = SafeCast.toUint64(amount);
        // Convert to little-endian for Borsh encoding expected by Solana
        return abi.encodePacked(IFT_MINT_DISCRIMINATOR, wallet, _reverseBytes8(bytes8(amountU64)));
    }

    /// @notice Reverse an 8-byte word.
    /// @param input The big-endian bytes.
    /// @return output The little-endian bytes.
    function _reverseBytes8(bytes8 input) private pure returns (bytes8 output) {
        for (uint256 i = 0; i < 8; ++i) {
            output |= (input >> (i * 8) & bytes8(uint64(0xff))) << ((7 - i) * 8);
        }
    }
}
