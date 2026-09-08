// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/binary"
	"encoding/hex"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	solanago "github.com/gagliardetto/solana-go"
	associatedtokenaccount "github.com/gagliardetto/solana-go/programs/associated-token-account"
	"github.com/gagliardetto/solana-go/programs/token"
	"github.com/gagliardetto/solana-go/rpc"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"

	gmptypes "github.com/cosmos/ibc-go/v11/modules/apps/27-gmp/types"

	interchaintest "github.com/cosmos/interchaintest/v11"
	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"
	ics27_gmp "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics27gmp"
	ift "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ift"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	cosmosutils "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
	ifttypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/ift"
	tokenfactorytypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/tokenfactory"
)

// IbcEurekaSolanaIFTTestSuite tests IFT functionality
type IbcEurekaSolanaIFTTestSuite struct {
	IbcEurekaSolanaTestSuite

	Wfchain         *cosmos.CosmosChain
	CosmosSubmitter ibc.Wallet

	IFTMintWallet        *solanago.Wallet // Mint keypair (IFT creates the mint during init)
	IFTMintAuthority     solanago.PublicKey
	IFTAppState          solanago.PublicKey
	IFTAppMintState      solanago.PublicKey
	IFTBridge            solanago.PublicKey
	SenderTokenAccount   solanago.PublicKey
	ReceiverTokenAccount solanago.PublicKey

	CosmosUser ibc.Wallet // Primary user for IFT operations

	GMPAppStatePDA      solanago.PublicKey
	RouterStatePDA      solanago.PublicKey
	IBCClientPDA        solanago.PublicKey
	GMPIBCAppPDA        solanago.PublicKey
	LightClientStatePDA solanago.PublicKey
}

// IFTMint returns the mint public key
func (s *IbcEurekaSolanaIFTTestSuite) IFTMint() solanago.PublicKey {
	return s.IFTMintWallet.PublicKey()
}

// IFTMintBytes returns the mint public key as bytes
func (s *IbcEurekaSolanaIFTTestSuite) IFTMintBytes() []byte {
	pk := s.IFTMintWallet.PublicKey()
	return pk[:]
}

const (
	IFTPortID                  = testvalues.SolanaGMPPortID // IFT uses GMP port for transport
	IFTTokenDecimals           = uint8(6)
	IFTMintAmount              = uint64(10_000_000) // 10 tokens with 6 decimals
	IFTTransferAmount          = uint64(1_000_000)  // 1 token with 6 decimals
	MockCosmosCounterpartyAddr = "cosmos1test123456789"

	IFT2022Name   = "IFT Test Token"
	IFT2022Symbol = "IFT"
	IFT2022Uri    = ""
)

func TestWithIbcEurekaSolanaIFTTestSuite(t *testing.T) {
	s := &IbcEurekaSolanaIFTTestSuite{}
	suite.Run(t, s)
}

func (s *IbcEurekaSolanaIFTTestSuite) SetupSuite(ctx context.Context) {
	chainconfig.DefaultChainSpecs = []*interchaintest.ChainSpec{
		chainconfig.WfchainChainSpec("wfchain-1", "wfchain-1"),
	}

	s.IbcEurekaSolanaTestSuite.SetupSuite(ctx)

	s.Wfchain = s.Cosmos.Chains[0]
	s.CosmosSubmitter = s.CreateAndFundCosmosUser(ctx, s.Wfchain)
	s.CosmosUser = s.Cosmos.Users[0]

	s.initializeICS27GMP(ctx)

	s.GMPAppStatePDA, _ = solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)
	s.RouterStatePDA, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	s.IBCClientPDA, _ = solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))
	s.GMPIBCAppPDA, _ = solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(GMPPortID))
	s.LightClientStatePDA, _ = solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
}

// Test_IFT_ExistingToken_SolanaToCosmosRoundtrip test: Solana → Cosmos → Solana
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_ExistingToken_SolanaToCosmosRoundtrip() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create SPL token with initial balance on Solana", func() {
		mint = s.initializeExistingToken(ctx, IFTMintAmount)
		s.T().Logf("SPL token initialized with IFT: %s", mint)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	s.Require().True(s.Run("Solana → Cosmos", func() {
		var solanaToCosmosSequence uint64
		var solanaTransferTxSig solanago.Signature
		s.Require().True(s.Run("Solana → Cosmos: Execute transfer", func() {
			solanaToCosmosSequence = 1
			seqBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)

			packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
			pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

			solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
			s.Require().NoError(err)

			transferMsg := ift.IftStateIftTransferMsg{
				ClientId:         SolanaClientID,
				Receiver:         s.CosmosUser.FormattedAddress(),
				Amount:           IFTTransferAmount,
				TimeoutTimestamp: uint64(solanaClockTime + 900),
				Sequence:         solanaToCosmosSequence,
			}

			consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
			transferIx, err := ift.NewIftTransferInstruction(
				transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, s.SenderTokenAccount,
				s.SolanaRelayer.PublicKey(), s.SolanaRelayer.PublicKey(),
				token.ProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
				ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
				s.GMPIBCAppPDA, s.IBCClientPDA,
				ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, pendingTransferPDA,
			)
			s.Require().NoError(err)

			computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
			s.Require().NoError(err)

			solanaTransferTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
			s.Require().NoError(err)
			s.T().Logf("Solana → Cosmos transfer tx: %s", solanaTransferTxSig)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Verify tokens burned on Solana", func() {
			balance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
			s.Require().NoError(err)
			s.Require().Equal(IFTMintAmount-IFTTransferAmount, balance)
		}))

		var cosmosRecvTxHash string
		s.Require().True(s.Run("Solana → Cosmos: Relay to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaTransferTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)

			receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
			cosmosRecvTxHash = receipt.TxHash
			s.T().Logf("Cosmos recv tx: %s", cosmosRecvTxHash)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Verify tokens minted on Cosmos", func() {
			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
			s.Require().NoError(err)
			s.Require().Equal(sdkmath.NewInt(int64(IFTTransferAmount)), balance)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Relay ack to Solana", func() {
			cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    s.Wfchain.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
				SrcClientId: CosmosClientID,
				DstClientId: SolanaClientID,
			})
			s.Require().NoError(err)

			_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Verify PendingTransfer closed", func() {
			s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
		}))
	}))

	s.Require().True(s.Run("Cosmos → Solana", func() {
		// Use a separate receiver wallet so the receiver pubkey differs from
		// the relayer/fee-payer. This avoids Solana's CPI account deduplication
		// merging signer flags, which would cause the GMP signer check to reject
		// the transaction.
		receiverWallet, err := s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)
		receiverATA, err := solana.AssociatedTokenAccountAddress(receiverWallet.PublicKey(), mint)
		s.Require().NoError(err)

		var cosmosToSolanaTxHash string
		s.Require().True(s.Run("Cosmos → Solana: Execute transfer", func() {
			timeout := uint64(time.Now().Add(15 * time.Minute).Unix())

			resp, err := s.BroadcastMessages(ctx, s.Wfchain, s.CosmosUser, 200_000, &ifttypes.MsgIFTTransfer{
				Signer:           s.CosmosUser.FormattedAddress(),
				Denom:            cosmosDenom,
				ClientId:         testvalues.FirstAttestationsClientID,
				Receiver:         receiverWallet.PublicKey().String(),
				Amount:           sdkmath.NewInt(int64(IFTTransferAmount)),
				TimeoutTimestamp: timeout,
			})
			s.Require().NoError(err)
			cosmosToSolanaTxHash = resp.TxHash
			s.T().Logf("Cosmos → Solana transfer tx: %s", cosmosToSolanaTxHash)
		}))

		s.Require().True(s.Run("Cosmos → Solana: Verify tokens burned on Cosmos", func() {
			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
			s.Require().NoError(err)
			s.Require().True(balance.IsZero())
		}))

		var solanaRecvTxSig solanago.Signature
		s.Require().True(s.Run("Cosmos → Solana: Relay to Solana", func() {
			cosmosIFTTxHashBytes, err := hex.DecodeString(cosmosToSolanaTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    s.Wfchain.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosIFTTxHashBytes},
				SrcClientId: CosmosClientID,
				DstClientId: SolanaClientID,
			})
			s.Require().NoError(err)

			solanaRecvTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
			s.Require().NoError(err)
			s.T().Logf("Solana recv tx: %s", solanaRecvTxSig)
		}))

		s.Require().True(s.Run("Cosmos → Solana: Verify tokens received on Solana", func() {
			balance, err := s.Solana.Chain.GetTokenBalance(ctx, receiverATA)
			s.Require().NoError(err)
			s.Require().Equal(IFTTransferAmount, balance)
		}))

		s.Require().True(s.Run("Cosmos → Solana: Relay ack to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaRecvTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
		}))
	}))
}

// Test_IFT_NewToken_CosmosToSolanaRoundtrip test: Cosmos → Solana → Cosmos
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_NewToken_CosmosToSolanaRoundtrip() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var cosmosDenom string
	s.Require().True(s.Run("Create and mint tokenfactory denom", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
		s.mintTokenFactory(ctx, s.CosmosSubmitter, cosmosDenom, sdkmath.NewInt(int64(IFTMintAmount)), s.CosmosUser.FormattedAddress())

		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().Equal(sdkmath.NewInt(int64(IFTMintAmount)), balance)
	}))

	// Use a separate receiver wallet so the receiver pubkey differs from
	// the relayer/fee-payer. This avoids Solana's CPI account deduplication
	// merging signer flags, which would cause the GMP signer check to reject
	// the transaction.
	receiverWallet, err := s.Solana.Chain.CreateAndFundWallet()
	s.Require().NoError(err)

	var mint solanago.PublicKey
	var receiverTokenAccount solanago.PublicKey
	s.Require().True(s.Run("Create IFT SPL token", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplToken(ctx, s.IFTMintWallet)

		mint = s.IFTMintWallet.PublicKey()
		ata, err := solana.AssociatedTokenAccountAddress(receiverWallet.PublicKey(), mint)
		s.Require().NoError(err)
		receiverTokenAccount = ata
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	s.Require().True(s.Run("Cosmos → Solana", func() {
		var cosmosToSolanaTxHash string
		s.Require().True(s.Run("Cosmos → Solana: Execute transfer", func() {
			timeout := uint64(time.Now().Add(15 * time.Minute).Unix())

			resp, err := s.BroadcastMessages(ctx, s.Wfchain, s.CosmosUser, 200_000, &ifttypes.MsgIFTTransfer{
				Signer:           s.CosmosUser.FormattedAddress(),
				Denom:            cosmosDenom,
				ClientId:         testvalues.FirstAttestationsClientID,
				Receiver:         receiverWallet.PublicKey().String(),
				Amount:           sdkmath.NewInt(int64(IFTTransferAmount)),
				TimeoutTimestamp: timeout,
			})
			s.Require().NoError(err)
			cosmosToSolanaTxHash = resp.TxHash
			s.T().Logf("Cosmos → Solana transfer tx: %s", cosmosToSolanaTxHash)
		}))

		s.Require().True(s.Run("Cosmos → Solana: Verify tokens burned on Cosmos", func() {
			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
			s.Require().NoError(err)
			expected := sdkmath.NewInt(int64(IFTMintAmount - IFTTransferAmount))
			s.Require().Equal(expected, balance)
		}))

		var solanaRecvTxSig solanago.Signature
		s.Require().True(s.Run("Cosmos → Solana: Relay to Solana", func() {
			cosmosIFTTxHashBytes, err := hex.DecodeString(cosmosToSolanaTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    s.Wfchain.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosIFTTxHashBytes},
				SrcClientId: CosmosClientID,
				DstClientId: SolanaClientID,
			})
			s.Require().NoError(err)

			solanaRecvTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
			s.Require().NoError(err)
			s.T().Logf("Solana recv tx: %s", solanaRecvTxSig)
		}))

		s.Require().True(s.Run("Cosmos → Solana: Verify tokens minted on Solana", func() {
			balance, err := s.Solana.Chain.GetTokenBalance(ctx, receiverTokenAccount)
			s.Require().NoError(err)
			s.Require().Equal(IFTTransferAmount, balance)
		}))

		s.Require().True(s.Run("Cosmos → Solana: Relay ack to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaRecvTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
		}))
	}))

	s.Require().True(s.Run("Solana → Cosmos", func() {
		var solanaToCosmosSequence uint64
		var solanaTransferTxSig solanago.Signature
		s.Require().True(s.Run("Solana → Cosmos: Execute transfer", func() {
			solanaToCosmosSequence = 1
			seqBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)

			packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
			pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

			solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
			s.Require().NoError(err)

			transferMsg := ift.IftStateIftTransferMsg{
				ClientId:         SolanaClientID,
				Receiver:         s.CosmosUser.FormattedAddress(),
				Amount:           IFTTransferAmount,
				TimeoutTimestamp: uint64(solanaClockTime + 900),
				Sequence:         solanaToCosmosSequence,
			}

			consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
			transferIx, err := ift.NewIftTransferInstruction(
				transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, receiverTokenAccount,
				receiverWallet.PublicKey(), s.SolanaRelayer.PublicKey(),
				token.ProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
				ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
				s.GMPIBCAppPDA, s.IBCClientPDA,
				ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, pendingTransferPDA,
			)
			s.Require().NoError(err)

			computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
			s.Require().NoError(err)

			solanaTransferTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer, receiverWallet)
			s.Require().NoError(err)
			s.T().Logf("Solana → Cosmos transfer tx: %s", solanaTransferTxSig)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Verify tokens burned on Solana", func() {
			balance, err := s.Solana.Chain.GetTokenBalance(ctx, receiverTokenAccount)
			s.Require().NoError(err)
			s.Require().Equal(uint64(0), balance)
		}))

		var cosmosRecvTxHash string
		s.Require().True(s.Run("Solana → Cosmos: Relay to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaTransferTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)

			receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
			cosmosRecvTxHash = receipt.TxHash
			s.T().Logf("Cosmos recv tx: %s", cosmosRecvTxHash)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Verify tokens restored on Cosmos", func() {
			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
			s.Require().NoError(err)
			s.Require().Equal(sdkmath.NewInt(int64(IFTMintAmount)), balance, "Balance should be restored after roundtrip")
		}))

		s.Require().True(s.Run("Solana → Cosmos: Relay ack to Solana", func() {
			cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    s.Wfchain.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
				SrcClientId: CosmosClientID,
				DstClientId: SolanaClientID,
			})
			s.Require().NoError(err)

			_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Solana → Cosmos: Verify PendingTransfer closed", func() {
			s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
		}))
	}))
}

func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_NewToken2022_SolanaToCosmos() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	tokenProgramID := solanago.Token2022ProgramID

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create IFT SPL token with Token 2022", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplTokenWithProgram(ctx, s.IFTMintWallet, tokenProgramID)
		mint = s.IFTMint()
		s.T().Logf("Token 2022 mint created: %s", mint)
	}))

	s.Require().True(s.Run("Admin mint tokens via Token 2022", func() {
		senderATA, err := solana.AssociatedTokenAccountAddressWithProgram(s.SolanaRelayer.PublicKey(), mint, tokenProgramID)
		s.Require().NoError(err)
		s.SenderTokenAccount = senderATA

		iftMintAuthorityPDA, _ := solana.Ift.IftMintAuthorityPDA(ift.ProgramID, mint[:])

		adminMintMsg := ift.IftStateAdminMintMsg{
			Receiver: s.SolanaRelayer.PublicKey(),
			Amount:   IFTMintAmount,
		}

		adminMintIx, err := ift.NewAdminMintInstruction(
			adminMintMsg,
			s.IFTAppState,
			s.IFTAppMintState,
			mint,
			iftMintAuthorityPDA,
			senderATA,
			s.SolanaRelayer.PublicKey(),
			s.SolanaRelayer.PublicKey(), // admin
			s.SolanaRelayer.PublicKey(), // payer
			tokenProgramID,
			associatedtokenaccount.ProgramID,
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), adminMintIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)

		balance, err := s.Solana.Chain.GetTokenBalance(ctx, senderATA)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount, balance)
		s.T().Logf("Admin minted %d tokens to %s via Token 2022", IFTMintAmount, senderATA)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	var solanaToCosmosSequence uint64
	var solanaTransferTxSig solanago.Signature
	s.Require().True(s.Run("Transfer: Solana → Cosmos (Token 2022)", func() {
		solanaToCosmosSequence = 1
		seqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)

		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
		pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		transferMsg := ift.IftStateIftTransferMsg{
			ClientId:         SolanaClientID,
			Receiver:         s.CosmosUser.FormattedAddress(),
			Amount:           IFTTransferAmount,
			TimeoutTimestamp: uint64(solanaClockTime + 900),
			Sequence:         solanaToCosmosSequence,
		}

		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
		transferIx, err := ift.NewIftTransferInstruction(
			transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, s.SenderTokenAccount,
			s.SolanaRelayer.PublicKey(), s.SolanaRelayer.PublicKey(),
			tokenProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
			ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
			s.GMPIBCAppPDA, s.IBCClientPDA,
			ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, pendingTransferPDA,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
		s.Require().NoError(err)

		solanaTransferTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Solana → Cosmos transfer tx (Token 2022): %s", solanaTransferTxSig)
	}))

	s.Require().True(s.Run("Verify tokens burned on Solana", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount-IFTTransferAmount, balance)
	}))

	var cosmosRecvTxHash string
	s.Require().True(s.Run("Relay to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(solanaTransferTxSig.String())},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)

		receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
		cosmosRecvTxHash = receipt.TxHash
		s.T().Logf("Cosmos recv tx: %s", cosmosRecvTxHash)
	}))

	s.Require().True(s.Run("Verify tokens minted on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().Equal(sdkmath.NewInt(int64(IFTTransferAmount)), balance)
	}))

	s.Require().True(s.Run("Relay ack to Solana", func() {
		cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify PendingTransfer closed after ack", func() {
		s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
	}))
}

// Test_IFT_TwoConsecutiveTransfersWithBatchRelay sends two IFT transfers from Solana,
// batch relays both packets to Cosmos in a single relay call, then relays the ack back.
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_TwoConsecutiveTransfersWithBatchRelay() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create SPL token with initial balance on Solana", func() {
		mint = s.initializeExistingToken(ctx, IFTMintAmount)
		s.T().Logf("SPL token initialized with IFT: %s", mint)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	// Helper to build and send an IFT transfer, returning the sequence and tx signature.
	var sequenceCounter uint64
	sendIFTTransfer := func(label string) (uint64, solanago.Signature) {
		sequenceCounter++
		seq := sequenceCounter
		var txSig solanago.Signature

		s.Require().True(s.Run(label, func() {
			seqBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(seqBytes, seq)

			packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
			pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

			solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
			s.Require().NoError(err)

			transferMsg := ift.IftStateIftTransferMsg{
				ClientId:         SolanaClientID,
				Receiver:         s.CosmosUser.FormattedAddress(),
				Amount:           IFTTransferAmount,
				TimeoutTimestamp: uint64(solanaClockTime + 900),
				Sequence:         seq,
			}

			consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
			transferIx, err := ift.NewIftTransferInstruction(
				transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, s.SenderTokenAccount,
				s.SolanaRelayer.PublicKey(), s.SolanaRelayer.PublicKey(),
				token.ProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
				ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
				s.GMPIBCAppPDA, s.IBCClientPDA,
				ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, pendingTransferPDA,
			)
			s.Require().NoError(err)

			computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
			s.Require().NoError(err)

			txSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
			s.Require().NoError(err)
			s.T().Logf("%s tx: %s (seq: %d)", label, txSig, seq)
		}))

		return seq, txSig
	}

	// Transfer #1
	seq1, txSig1 := sendIFTTransfer("Transfer #1: Solana → Cosmos")

	// Transfer #2
	seq2, txSig2 := sendIFTTransfer("Transfer #2: Solana → Cosmos")

	s.Require().True(s.Run("Verify both pending transfers exist", func() {
		s.Solana.Chain.VerifyPendingTransferExists(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, seq1)
		s.Solana.Chain.VerifyPendingTransferExists(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, seq2)
	}))

	s.Require().True(s.Run("Verify tokens burned on Solana", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount-2*IFTTransferAmount, balance)
	}))

	// Batch relay both packets to Cosmos
	var cosmosRecvTxHash string
	s.Require().True(s.Run("Batch relay both packets to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(txSig1.String()), []byte(txSig2.String())},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)

		receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
		cosmosRecvTxHash = receipt.TxHash
		s.T().Logf("Cosmos recv tx (batch): %s", cosmosRecvTxHash)
	}))

	s.Require().True(s.Run("Verify tokens minted on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().Equal(sdkmath.NewInt(int64(2*IFTTransferAmount)), balance)
	}))

	s.Require().True(s.Run("Relay ack to Solana", func() {
		cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify both pending transfers closed and commitments deleted", func() {
		s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, seq1)
		s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, seq2)
		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, seq1)
		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, seq2)
	}))
}

// Test_IFT_CosmosToSolanaRoundtrip test: Cosmos → Solana → Cosmos
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_CosmosToSolanaRoundtrip() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	// Create and mint tokens on Cosmos
	var cosmosDenom string
	s.Require().True(s.Run("Create and mint tokenfactory denom", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
		s.mintTokenFactory(ctx, s.CosmosSubmitter, cosmosDenom, sdkmath.NewInt(int64(IFTMintAmount)), s.CosmosUser.FormattedAddress())

		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().Equal(sdkmath.NewInt(int64(IFTMintAmount)), balance)
	}))

	// Use a separate receiver wallet so the receiver pubkey differs from
	// the relayer/fee-payer. This avoids Solana's CPI account deduplication
	// merging signer flags, which would cause the GMP signer check to reject
	// the transaction.
	receiverWallet, err := s.Solana.Chain.CreateAndFundWallet()
	s.Require().NoError(err)

	var receiverTokenAccount solanago.PublicKey
	s.Require().True(s.Run("Create IFT SPL token", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplToken(ctx, s.IFTMintWallet)

		mint := s.IFTMintWallet.PublicKey()
		ata, err := solana.AssociatedTokenAccountAddress(receiverWallet.PublicKey(), mint)
		s.Require().NoError(err)
		receiverTokenAccount = ata
	}))

	mint := s.IFTMintWallet.PublicKey()
	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	// === Cosmos → Solana ===
	var cosmosToSolanaTxHash string
	s.Require().True(s.Run("Transfer: Cosmos → Solana", func() {
		timeout := uint64(time.Now().Add(15 * time.Minute).Unix())

		resp, err := s.BroadcastMessages(ctx, s.Wfchain, s.CosmosUser, 200_000, &ifttypes.MsgIFTTransfer{
			Signer:           s.CosmosUser.FormattedAddress(),
			Denom:            cosmosDenom,
			ClientId:         testvalues.FirstAttestationsClientID,
			Receiver:         receiverWallet.PublicKey().String(),
			Amount:           sdkmath.NewInt(int64(IFTTransferAmount)),
			TimeoutTimestamp: timeout,
		})
		s.Require().NoError(err)
		cosmosToSolanaTxHash = resp.TxHash
		s.T().Logf("Cosmos → Solana transfer tx: %s", cosmosToSolanaTxHash)
	}))

	s.Require().True(s.Run("Verify tokens burned on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		expected := sdkmath.NewInt(int64(IFTMintAmount - IFTTransferAmount))
		s.Require().Equal(expected, balance)
	}))

	var solanaRecvTxSig solanago.Signature
	s.Require().True(s.Run("Relay to Solana", func() {
		cosmosIFTTxHashBytes, err := hex.DecodeString(cosmosToSolanaTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosIFTTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)

		solanaRecvTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Solana recv tx: %s", solanaRecvTxSig)
	}))

	s.Require().True(s.Run("Verify tokens minted on Solana", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, receiverTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTTransferAmount, balance)
	}))

	s.Require().True(s.Run("Relay ack to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(solanaRecvTxSig.String())},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)
		_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
	}))

	// === Solana → Cosmos ===
	var solanaToCosmosSequence uint64
	var solanaTransferTxSig solanago.Signature
	s.Require().True(s.Run("Transfer: Solana → Cosmos", func() {
		solanaToCosmosSequence = 1
		seqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)

		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
		pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		transferMsg := ift.IftStateIftTransferMsg{
			ClientId:         SolanaClientID,
			Receiver:         s.CosmosUser.FormattedAddress(),
			Amount:           IFTTransferAmount,
			TimeoutTimestamp: uint64(solanaClockTime + 900),
			Sequence:         solanaToCosmosSequence,
		}

		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
		transferIx, err := ift.NewIftTransferInstruction(
			transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, receiverTokenAccount,
			receiverWallet.PublicKey(), s.SolanaRelayer.PublicKey(),
			token.ProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
			ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
			s.GMPIBCAppPDA, s.IBCClientPDA,
			ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, pendingTransferPDA,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
		s.Require().NoError(err)

		solanaTransferTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer, receiverWallet)
		s.Require().NoError(err)
		s.T().Logf("Solana → Cosmos transfer tx: %s", solanaTransferTxSig)
	}))

	s.Require().True(s.Run("Verify tokens burned on Solana", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, receiverTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(uint64(0), balance)
	}))

	var cosmosRecvTxHash string
	s.Require().True(s.Run("Relay to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(solanaTransferTxSig.String())},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)

		receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
		cosmosRecvTxHash = receipt.TxHash
		s.T().Logf("Cosmos recv tx: %s", cosmosRecvTxHash)
	}))

	s.Require().True(s.Run("Verify tokens restored on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().Equal(sdkmath.NewInt(int64(IFTMintAmount)), balance, "Balance should be restored after roundtrip")
	}))

	s.Require().True(s.Run("Relay ack to Solana", func() {
		cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify PendingTransfer closed", func() {
		s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
	}))
}

// Test_IFT_ExistingToken2022_SolanaToCosmos tests initialize_existing_token with Token 2022
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_ExistingToken2022_SolanaToCosmos() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	tokenProgramID := solanago.Token2022ProgramID

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create existing Token 2022 and transfer to IFT", func() {
		mint = s.initializeExistingTokenWithProgram(ctx, IFTMintAmount, tokenProgramID)
		s.T().Logf("Token 2022 existing mint initialized with IFT: %s", mint)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	var solanaToCosmosSequence uint64
	var solanaTransferTxSig solanago.Signature
	s.Require().True(s.Run("Transfer: Solana → Cosmos (existing Token 2022)", func() {
		solanaToCosmosSequence = 1
		seqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)

		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
		pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		transferMsg := ift.IftStateIftTransferMsg{
			ClientId:         SolanaClientID,
			Receiver:         s.CosmosUser.FormattedAddress(),
			Amount:           IFTTransferAmount,
			TimeoutTimestamp: uint64(solanaClockTime + 900),
			Sequence:         solanaToCosmosSequence,
		}

		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
		transferIx, err := ift.NewIftTransferInstruction(
			transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, s.SenderTokenAccount,
			s.SolanaRelayer.PublicKey(), s.SolanaRelayer.PublicKey(),
			tokenProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
			ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
			s.GMPIBCAppPDA, s.IBCClientPDA,
			ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, pendingTransferPDA,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
		s.Require().NoError(err)

		solanaTransferTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Solana → Cosmos transfer tx (existing Token 2022): %s", solanaTransferTxSig)
	}))

	s.Require().True(s.Run("Verify tokens burned on Solana", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount-IFTTransferAmount, balance)
	}))

	var cosmosRecvTxHash string
	s.Require().True(s.Run("Relay to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(solanaTransferTxSig.String())},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)

		receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosUser, 2_000_000, resp.Tx)
		cosmosRecvTxHash = receipt.TxHash
		s.T().Logf("Cosmos recv tx: %s", cosmosRecvTxHash)
	}))

	s.Require().True(s.Run("Verify tokens minted on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().Equal(sdkmath.NewInt(int64(IFTTransferAmount)), balance)
	}))

	s.Require().True(s.Run("Relay ack to Solana", func() {
		cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify PendingTransfer closed after ack", func() {
		s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
	}))
}

// Test_IFT_NewToken_AdminSetupFlow tests IFT initialization creates the mint with correct authority
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_NewToken_AdminSetupFlow() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Create IFT SPL token (creates mint)", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplToken(ctx, s.IFTMintWallet)

		mint := s.IFTMint()
		s.T().Logf("SPL Token mint created by IFT: %s", mint.String())
	}))

	var expectedMintAuthority solanago.PublicKey
	s.Require().True(s.Run("Verify mint authority is IFT PDA", func() {
		mint := s.IFTMint()
		expectedMintAuthority, _ = solana.Ift.IftMintAuthorityPDA(ift.ProgramID, mint[:])
		s.Solana.Chain.VerifyMintAuthority(ctx, s.T(), s.Require(), mint, expectedMintAuthority)
		s.T().Logf("Mint authority is IFT PDA: %s", expectedMintAuthority.String())
	}))

	var bridgePDA solanago.PublicKey
	s.Require().True(s.Run("Register IFT Bridge", func() {
		s.registerSolanaIFTBridge(ctx, SolanaClientID, MockCosmosCounterpartyAddr, testvalues.IFTTestDenom)
		bridgePDA = s.IFTBridge
		s.T().Logf("IFT Bridge registered: %s", bridgePDA.String())
	}))

	s.Require().True(s.Run("Verify bridge is active", func() {
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, bridgePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value, "Bridge account should exist")
		s.Require().True(accountInfo.Value.Lamports > 0, "Bridge account should have lamports")
		s.T().Logf("Bridge account verified: %d lamports", accountInfo.Value.Lamports)
	}))
}

// Test_IFT_NewToken_RevokeMintAuthority tests that admin can revoke mint authority from IFT
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_NewToken_RevokeMintAuthority() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var iftMintAuthorityPDA solanago.PublicKey
	s.Require().True(s.Run("Create IFT SPL token", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplToken(ctx, s.IFTMintWallet)

		mint := s.IFTMint()
		iftMintAuthorityPDA, _ = solana.Ift.IftMintAuthorityPDA(ift.ProgramID, mint[:])
		s.Solana.Chain.VerifyMintAuthority(ctx, s.T(), s.Require(), mint, iftMintAuthorityPDA)
		s.T().Logf("IFT initialized - mint authority: %s", iftMintAuthorityPDA)
	}))

	var newAuthorityWallet *solanago.Wallet
	s.Require().True(s.Run("Create new wallet to receive mint authority", func() {
		var err error
		newAuthorityWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify app state exists before revoke", func() {
		s.Solana.Chain.VerifyIftAppStateExists(ctx, s.T(), s.Require(), ift.ProgramID)
	}))

	s.Require().True(s.Run("Revoke mint authority", func() {
		revokeIx, err := ift.NewRevokeMintAuthorityInstruction(
			s.IFTAppState,
			s.IFTAppMintState,
			s.IFTMint(),
			iftMintAuthorityPDA,
			newAuthorityWallet.PublicKey(),
			s.SolanaRelayer.PublicKey(), // admin
			token.ProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), revokeIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Revoke mint authority tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify mint authority transferred", func() {
		s.Solana.Chain.VerifyMintAuthority(ctx, s.T(), s.Require(), s.IFTMint(), newAuthorityWallet.PublicKey())
		s.T().Logf("Mint authority transferred to: %s", newAuthorityWallet.PublicKey())
	}))

	s.Require().True(s.Run("Verify IFT app state still exists", func() {
		s.Solana.Chain.VerifyIftAppStateExists(ctx, s.T(), s.Require(), ift.ProgramID)
	}))

	s.Require().True(s.Run("Verify mint state closed", func() {
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, s.IFTAppMintState, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		// Closed accounts may return "not found" error or nil Value depending on RPC client
		if err != nil {
			s.Require().Contains(err.Error(), "not found", "expected 'not found' for closed account, got: %v", err)
		} else {
			s.Require().Nil(accountInfo.Value, "IFTAppMintState should be closed after revoke")
		}
	}))

	s.Require().True(s.Run("Verify new authority can mint tokens", func() {
		mint := s.IFTMint()
		tokenAccount, err := s.Solana.Chain.CreateOrGetAssociatedTokenAccount(ctx, newAuthorityWallet, mint, newAuthorityWallet.PublicKey())
		s.Require().NoError(err)

		err = s.Solana.Chain.MintTokensTo(ctx, newAuthorityWallet, mint, tokenAccount, IFTMintAmount)
		s.Require().NoError(err)

		balance, err := s.Solana.Chain.GetTokenBalance(ctx, tokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount, balance)
		s.T().Logf("New authority minted %d tokens", IFTMintAmount)
	}))
}

// Test_IFT_ExistingToken_TimeoutRefund tests that tokens are refunded on timeout
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_ExistingToken_TimeoutRefund() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create SPL token with initial balance on Solana", func() {
		mint = s.initializeExistingToken(ctx, IFTMintAmount)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	var solanaPacketTxHash []byte
	var solanaToCosmosSequence uint64
	var timeoutTimestamp uint64
	s.Require().True(s.Run("Execute Transfer with Short Timeout", func() {
		solanaToCosmosSequence = 1

		seqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)
		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
		pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		timeoutTimestamp = uint64(solanaClockTime + SolanaOriginatedTimeoutSeconds)

		transferMsg := ift.IftStateIftTransferMsg{
			ClientId:         SolanaClientID,
			Receiver:         s.CosmosUser.FormattedAddress(),
			Amount:           IFTTransferAmount,
			TimeoutTimestamp: timeoutTimestamp,
			Sequence:         solanaToCosmosSequence,
		}

		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
		transferIx, err := ift.NewIftTransferInstruction(
			transferMsg,
			s.IFTAppState,
			s.IFTAppMintState,
			s.IFTBridge,
			mint,
			s.SenderTokenAccount,
			s.SolanaRelayer.PublicKey(),
			s.SolanaRelayer.PublicKey(),
			token.ProgramID,
			solanago.SystemProgramID,
			ics27_gmp.ProgramID,
			s.GMPAppStatePDA,
			ics26_router.ProgramID,
			s.RouterStatePDA,
			packetCommitmentPDA,
			s.GMPIBCAppPDA,
			s.IBCClientPDA,
			ics07_tendermint.ProgramID,
			s.LightClientStatePDA,
			solanago.SysVarInstructionsPubkey,
			consensusStatePDA,
			pendingTransferPDA,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)

		solanaPacketTxHash = []byte(sig.String())
		s.T().Logf("IFT transfer transaction (will timeout): %s", sig)
	}))

	s.Require().True(s.Run("Verify tokens burned after transfer", func() {
		burnedBalance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount-IFTTransferAmount, burnedBalance)
	}))

	s.Require().True(s.Run("Verify PendingTransfer PDA exists before timeout", func() {
		s.Solana.Chain.VerifyPendingTransferExists(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
	}))

	s.Require().True(s.Run("Wait for packet timeout", func() {
		err := e2esuite.WaitForBlockTime(ctx, s.T(), &cosmosutils.Chain{Cosmos: s.Wfchain}, timeoutTimestamp)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Relay timeout back to Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:     s.Wfchain.Config().ChainID,
			DstChain:     testvalues.SolanaChainID,
			TimeoutTxIds: [][]byte{solanaPacketTxHash},
			SrcClientId:  CosmosClientID,
			DstClientId:  SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		sig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Timeout transaction: %s", sig)
	}))

	s.Require().True(s.Run("Verify timeout effects", func() {
		s.Require().True(s.Run("Verify packet commitment deleted", func() {
			s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, solanaToCosmosSequence)
			s.T().Logf("Packet commitment successfully deleted for sequence %d", solanaToCosmosSequence)
		}))

		s.Require().True(s.Run("Verify tokens refunded to sender", func() {
			refundedBalance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
			s.Require().NoError(err)
			s.Require().Equal(IFTMintAmount, refundedBalance, "Tokens should be refunded to sender after timeout")
		}))

		s.Require().True(s.Run("Verify PendingTransfer PDA closed", func() {
			s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(),
				ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
		}))
	}))
}

// Test_IFT_ExistingToken_AckFailureRefund tests that tokens are refunded on acknowledgement failure
// Note: wfchain has IFT module but we unregister the bridge to trigger error ack
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_ExistingToken_AckFailureRefund() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create SPL token with initial balance on Solana", func() {
		mint = s.initializeExistingToken(ctx, IFTMintAmount)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	s.Require().True(s.Run("Unregister Cosmos bridge to trigger error ack", func() {
		govModuleAddr := authtypes.NewModuleAddress(govtypes.ModuleName).String()
		msg := &ifttypes.MsgRemoveIFTBridge{
			Signer:   govModuleAddr,
			Denom:    cosmosDenom,
			ClientId: testvalues.FirstAttestationsClientID,
		}
		err := s.ExecuteGovV1Proposal(ctx, msg, s.Wfchain, s.CosmosSubmitter)
		s.Require().NoError(err)
	}))

	var transferTxSig solanago.Signature
	var solanaToCosmosSequence uint64
	s.Require().True(s.Run("Execute Transfer", func() {
		solanaToCosmosSequence = 1

		seqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(seqBytes, solanaToCosmosSequence)
		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)
		pendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), seqBytes)

		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		timeoutTimestamp := uint64(solanaClockTime + 900) // 15 minutes

		transferMsg := ift.IftStateIftTransferMsg{
			ClientId:         SolanaClientID,
			Receiver:         s.CosmosUser.FormattedAddress(),
			Amount:           IFTTransferAmount,
			TimeoutTimestamp: timeoutTimestamp,
			Sequence:         solanaToCosmosSequence,
		}

		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
		transferIx, err := ift.NewIftTransferInstruction(
			transferMsg,
			s.IFTAppState,
			s.IFTAppMintState,
			s.IFTBridge,
			mint,
			s.SenderTokenAccount,
			s.SolanaRelayer.PublicKey(),
			s.SolanaRelayer.PublicKey(),
			token.ProgramID,
			solanago.SystemProgramID,
			ics27_gmp.ProgramID,
			s.GMPAppStatePDA,
			ics26_router.ProgramID,
			s.RouterStatePDA,
			packetCommitmentPDA,
			s.GMPIBCAppPDA,
			s.IBCClientPDA,
			ics07_tendermint.ProgramID,
			s.LightClientStatePDA,
			solanago.SysVarInstructionsPubkey,
			consensusStatePDA,
			pendingTransferPDA,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
		s.Require().NoError(err)

		transferTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("IFT transfer transaction: %s", transferTxSig.String())
	}))

	s.Require().True(s.Run("Verify tokens burned", func() {
		burnedBalance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount-IFTTransferAmount, burnedBalance, "Tokens should be burned after transfer")
	}))

	s.Require().True(s.Run("Verify PendingTransfer PDA exists before relay", func() {
		s.Solana.Chain.VerifyPendingTransferExists(ctx, s.T(), s.Require(), ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
	}))

	var cosmosRecvTxHash string
	s.Require().True(s.Run("Relay packet to wfchain (will fail - no IFT bridge registered)", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(transferTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			recvRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx to Cosmos", func() {
			receipt := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.Cosmos.Users[0], 2_000_000, recvRelayTx)
			s.Require().Equal(uint32(0), receipt.Code, "IBC layer should succeed even if app fails")
			cosmosRecvTxHash = receipt.TxHash
		}))
	}))

	s.Require().True(s.Run("Relay error ack to Solana", func() {
		cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		sig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Error ack relayed: %s", sig)
	}))

	s.Require().True(s.Run("Verify tokens refunded", func() {
		finalBalance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount, finalBalance, "Tokens should be refunded after error ack")
	}))

	s.Require().True(s.Run("Verify PendingTransfer PDA closed", func() {
		s.Solana.Chain.VerifyPendingTransferClosed(ctx, s.T(), s.Require(),
			ift.ProgramID, mint, SolanaClientID, solanaToCosmosSequence)
	}))
}

// Test_IFT_NewToken_AdminMint tests that admin can mint tokens to any account
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_NewToken_AdminMint() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	const adminMintAmount = uint64(5_000_000)

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create IFT SPL token", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplToken(ctx, s.IFTMintWallet)
		mint = s.IFTMint()
	}))

	var receiverWallet *solanago.Wallet
	var receiverATA solanago.PublicKey
	var iftMintAuthorityPDA solanago.PublicKey
	s.Require().True(s.Run("Prepare receiver wallet and derive PDAs", func() {
		var err error
		receiverWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		receiverATA, err = solana.AssociatedTokenAccountAddress(receiverWallet.PublicKey(), mint)
		s.Require().NoError(err)

		iftMintAuthorityPDA, _ = solana.Ift.IftMintAuthorityPDA(ift.ProgramID, mint[:])
	}))

	s.Require().True(s.Run("Admin mint tokens to receiver", func() {
		adminMintMsg := ift.IftStateAdminMintMsg{
			Receiver: receiverWallet.PublicKey(),
			Amount:   adminMintAmount,
		}

		adminMintIx, err := ift.NewAdminMintInstruction(
			adminMintMsg,
			s.IFTAppState,
			s.IFTAppMintState,
			mint,
			iftMintAuthorityPDA,
			receiverATA,
			receiverWallet.PublicKey(),
			s.SolanaRelayer.PublicKey(), // admin
			s.SolanaRelayer.PublicKey(), // payer
			token.ProgramID,
			associatedtokenaccount.ProgramID,
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), adminMintIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Admin mint tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify receiver balance", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, receiverATA)
		s.Require().NoError(err)
		s.Require().Equal(adminMintAmount, balance)
	}))
}

// Test_IFT_TwoStepAdminTransfer tests the propose → verify unchanged → accept → verify changed flow,
// and also tests cancel_admin_proposal.
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_TwoStepAdminTransfer() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Create IFT SPL token", func() {
		s.IFTMintWallet = solanago.NewWallet()
		s.createIFTSplToken(ctx, s.IFTMintWallet)
	}))

	var newAdminWallet *solanago.Wallet
	s.Require().True(s.Run("Create new admin wallet", func() {
		var err error
		newAdminWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)
	}))

	readAppState := func() *ift.IftStateIftAppState {
		s.T().Helper()
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, s.IFTAppState, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)
		state, err := ift.ParseAccount_IftStateIftAppState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)
		return state
	}

	s.Require().True(s.Run("Verify initial admin is relayer", func() {
		state := readAppState()
		s.Require().Equal(s.SolanaRelayer.PublicKey(), state.Admin)
		s.Require().Nil(state.PendingAdmin)
	}))

	s.Require().True(s.Run("Propose new admin", func() {
		proposeIx, err := ift.NewProposeAdminInstruction(
			newAdminWallet.PublicKey(),
			s.IFTAppState,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Propose admin tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify admin not changed yet, pending admin set", func() {
		state := readAppState()
		s.Require().Equal(s.SolanaRelayer.PublicKey(), state.Admin, "Admin should not change after propose")
		s.Require().NotNil(state.PendingAdmin)
		s.Require().Equal(newAdminWallet.PublicKey(), *state.PendingAdmin)
	}))

	s.Require().True(s.Run("Accept admin", func() {
		acceptIx, err := ift.NewAcceptAdminInstruction(
			s.IFTAppState,
			newAdminWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(newAdminWallet.PublicKey(), acceptIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, newAdminWallet)
		s.Require().NoError(err)
		s.T().Logf("Accept admin tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify admin changed to new admin", func() {
		state := readAppState()
		s.Require().Equal(newAdminWallet.PublicKey(), state.Admin, "Admin should be the new admin")
		s.Require().Nil(state.PendingAdmin, "Pending admin should be cleared")
	}))

	// Transfer admin back and test cancel flow
	s.Require().True(s.Run("Propose original admin back", func() {
		proposeIx, err := ift.NewProposeAdminInstruction(
			s.SolanaRelayer.PublicKey(),
			s.IFTAppState,
			newAdminWallet.PublicKey(), // current admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(newAdminWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, newAdminWallet)
		s.Require().NoError(err)
		s.T().Logf("Propose original admin back tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify pending admin set to original", func() {
		state := readAppState()
		s.Require().Equal(newAdminWallet.PublicKey(), state.Admin, "Admin should still be new admin")
		s.Require().NotNil(state.PendingAdmin)
		s.Require().Equal(s.SolanaRelayer.PublicKey(), *state.PendingAdmin)
	}))

	s.Require().True(s.Run("Cancel admin proposal", func() {
		cancelIx, err := ift.NewCancelAdminProposalInstruction(
			s.IFTAppState,
			newAdminWallet.PublicKey(), // current admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(newAdminWallet.PublicKey(), cancelIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, newAdminWallet)
		s.Require().NoError(err)
		s.T().Logf("Cancel admin proposal tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify proposal cancelled", func() {
		state := readAppState()
		s.Require().Equal(newAdminWallet.PublicKey(), state.Admin, "Admin should still be new admin")
		s.Require().Nil(state.PendingAdmin, "Pending admin should be cleared after cancel")
	}))

	// After cancel, propose a different admin and verify it can be accepted
	var thirdAdminWallet *solanago.Wallet
	s.Require().True(s.Run("Create third admin wallet", func() {
		var err error
		thirdAdminWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Propose third admin after cancel", func() {
		proposeIx, err := ift.NewProposeAdminInstruction(
			thirdAdminWallet.PublicKey(),
			s.IFTAppState,
			newAdminWallet.PublicKey(), // current admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(newAdminWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, newAdminWallet)
		s.Require().NoError(err)
		s.T().Logf("Propose third admin tx: %s", sig)
	}))

	s.Require().True(s.Run("Accept third admin", func() {
		acceptIx, err := ift.NewAcceptAdminInstruction(
			s.IFTAppState,
			thirdAdminWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(thirdAdminWallet.PublicKey(), acceptIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, thirdAdminWallet)
		s.Require().NoError(err)
		s.T().Logf("Accept third admin tx: %s", sig)
	}))

	s.Require().True(s.Run("Verify third admin is now admin", func() {
		state := readAppState()
		s.Require().Equal(thirdAdminWallet.PublicKey(), state.Admin, "Admin should be the third admin")
		s.Require().Nil(state.PendingAdmin, "Pending admin should be cleared")
	}))

	// Edge case: unauthorized propose (non-admin tries to propose)
	s.Require().True(s.Run("Unauthorized propose is rejected", func() {
		unauthorizedWallet, err := s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		proposeIx, err := ift.NewProposeAdminInstruction(
			unauthorizedWallet.PublicKey(),
			s.IFTAppState,
			unauthorizedWallet.PublicKey(), // not the admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(unauthorizedWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, unauthorizedWallet)
		s.Require().Error(err, "Non-admin should not be able to propose")
	}))

	// Edge case: cancel with no pending proposal
	s.Require().True(s.Run("Cancel with no pending proposal is rejected", func() {
		cancelIx, err := ift.NewCancelAdminProposalInstruction(
			s.IFTAppState,
			thirdAdminWallet.PublicKey(), // current admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(thirdAdminWallet.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, thirdAdminWallet)
		s.Require().Error(err, "Cancel should fail when no pending proposal exists")
	}))

	// Set up a proposal so we can test more edge cases
	var fourthAdminWallet *solanago.Wallet
	s.Require().True(s.Run("Propose fourth admin for edge case tests", func() {
		var err error
		fourthAdminWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		proposeIx, err := ift.NewProposeAdminInstruction(
			fourthAdminWallet.PublicKey(),
			s.IFTAppState,
			thirdAdminWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(thirdAdminWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, thirdAdminWallet)
		s.Require().NoError(err)
	}))

	// Edge case: unauthorized accept (non-pending wallet tries to accept)
	s.Require().True(s.Run("Unauthorized accept is rejected", func() {
		acceptIx, err := ift.NewAcceptAdminInstruction(
			s.IFTAppState,
			thirdAdminWallet.PublicKey(), // current admin, not the pending admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(thirdAdminWallet.PublicKey(), acceptIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, thirdAdminWallet)
		s.Require().Error(err, "Non-pending admin should not be able to accept")
	}))

	// Edge case: propose while pending already exists
	s.Require().True(s.Run("Propose while pending exists is rejected", func() {
		anotherWallet, err := s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		proposeIx, err := ift.NewProposeAdminInstruction(
			anotherWallet.PublicKey(),
			s.IFTAppState,
			thirdAdminWallet.PublicKey(), // current admin
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(thirdAdminWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, thirdAdminWallet)
		s.Require().Error(err, "Propose should fail when a pending proposal already exists")
	}))

	// Edge case: cancel then stale accept
	s.Require().True(s.Run("Cancel the proposal", func() {
		cancelIx, err := ift.NewCancelAdminProposalInstruction(
			s.IFTAppState,
			thirdAdminWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(thirdAdminWallet.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, thirdAdminWallet)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Stale accept after cancel is rejected", func() {
		acceptIx, err := ift.NewAcceptAdminInstruction(
			s.IFTAppState,
			fourthAdminWallet.PublicKey(), // was the pending admin before cancel
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(fourthAdminWallet.PublicKey(), acceptIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, fourthAdminWallet)
		s.Require().Error(err, "Previously-pending admin should not be able to accept after cancel")
	}))

	// Verify state unchanged after all negative tests
	s.Require().True(s.Run("Verify admin unchanged after edge case tests", func() {
		state := readAppState()
		s.Require().Equal(thirdAdminWallet.PublicKey(), state.Admin, "Admin should still be the third admin")
		s.Require().Nil(state.PendingAdmin, "Pending admin should be nil")
	}))
}

// Test_IFT_ExistingToken_InvalidPendingTransfer tests that ift_transfer rejects a wrong pending_transfer PDA
func (s *IbcEurekaSolanaIFTTestSuite) Test_IFT_ExistingToken_InvalidPendingTransfer() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var mint solanago.PublicKey
	s.Require().True(s.Run("Create SPL token with initial balance on Solana", func() {
		mint = s.initializeExistingToken(ctx, IFTMintAmount)
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Create tokenfactory denom on Cosmos", func() {
		cosmosDenom = s.createTokenFactoryDenom(ctx, testvalues.IFTTestDenom)
	}))

	s.Require().True(s.Run("Register IFT Bridges", func() {
		s.registerCosmosIFTBridge(ctx, cosmosDenom, testvalues.FirstAttestationsClientID, ift.ProgramID.String(), SolanaClientID, ics27_gmp.ProgramID, mint)
		iftModuleAddr := s.getCosmosIFTModuleAddress()
		s.registerSolanaIFTBridge(ctx, SolanaClientID, iftModuleAddr, cosmosDenom)
	}))

	s.Require().True(s.Run("Transfer with wrong pending_transfer PDA fails", func() {
		var sequence uint64 = 1
		seqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(seqBytes, sequence)

		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), seqBytes)

		// Derive pending transfer PDA with WRONG sequence to trigger InvalidPendingTransfer
		wrongSeqBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(wrongSeqBytes, 999_999)
		wrongPendingTransferPDA, _ := solana.Ift.PendingTransferPDA(ift.ProgramID, mint[:], []byte(SolanaClientID), wrongSeqBytes)

		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		transferMsg := ift.IftStateIftTransferMsg{
			ClientId:         SolanaClientID,
			Receiver:         s.CosmosUser.FormattedAddress(),
			Amount:           IFTTransferAmount,
			TimeoutTimestamp: uint64(solanaClockTime + 900),
			Sequence:         sequence,
		}

		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, s.LightClientStatePDA)
		transferIx, err := ift.NewIftTransferInstruction(
			transferMsg, s.IFTAppState, s.IFTAppMintState, s.IFTBridge, mint, s.SenderTokenAccount,
			s.SolanaRelayer.PublicKey(), s.SolanaRelayer.PublicKey(),
			token.ProgramID, solanago.SystemProgramID, ics27_gmp.ProgramID, s.GMPAppStatePDA,
			ics26_router.ProgramID, s.RouterStatePDA, packetCommitmentPDA,
			s.GMPIBCAppPDA, s.IBCClientPDA,
			ics07_tendermint.ProgramID, s.LightClientStatePDA, solanago.SysVarInstructionsPubkey, consensusStatePDA, wrongPendingTransferPDA,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), computeBudgetIx, transferIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().Error(err, "Transaction should fail with InvalidPendingTransfer")
		s.T().Logf("Transfer correctly rejected with wrong pending_transfer PDA: %v", err)
	}))

	s.Require().True(s.Run("Verify tokens not burned (transaction rolled back)", func() {
		balance, err := s.Solana.Chain.GetTokenBalance(ctx, s.SenderTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(IFTMintAmount, balance, "Tokens should not be burned when transaction fails")
	}))
}

// createTokenFactoryDenom creates a tokenfactory denom and returns the subdenom
func (s *IbcEurekaSolanaIFTTestSuite) createTokenFactoryDenom(ctx context.Context, subdenom string) string {
	msg := &tokenfactorytypes.MsgCreateDenom{
		Sender: s.CosmosSubmitter.FormattedAddress(),
		Denom:  subdenom,
	}
	_, err := s.BroadcastMessages(ctx, s.Wfchain, s.CosmosSubmitter, 200_000, msg)
	s.Require().NoError(err)
	return subdenom
}

// mintTokenFactory mints tokenfactory tokens to a recipient
func (s *IbcEurekaSolanaIFTTestSuite) mintTokenFactory(ctx context.Context, user ibc.Wallet, denom string, amount sdkmath.Int, recipient string) {
	msg := &tokenfactorytypes.MsgMint{
		From:    user.FormattedAddress(),
		Address: recipient,
		Amount:  sdk.Coin{Denom: denom, Amount: amount},
	}
	_, err := s.BroadcastMessages(ctx, s.Wfchain, user, 200_000, msg)
	s.Require().NoError(err)
}

func (s *IbcEurekaSolanaIFTTestSuite) registerCosmosIFTBridge(ctx context.Context, denom, clientId, counterpartyIftAddr, counterpartyClientId string, gmpProgramID, mint solanago.PublicKey) {
	govModuleAddr, err := s.Wfchain.AuthQueryModuleAddress(ctx, govtypes.ModuleName)
	s.Require().NoError(err)

	// counterpartyClientId is the client on Solana that tracks Cosmos - needed for gmp_account_pda derivation
	constructor := testvalues.BuildSolanaIFTConstructor(gmpProgramID.String(), mint.String())
	s.T().Logf("IFT constructor: %s", constructor)

	msg := &ifttypes.MsgRegisterIFTBridge{
		Signer:                 govModuleAddr,
		Denom:                  denom,
		ClientId:               clientId,
		CounterpartyIftAddress: counterpartyIftAddr,
		IftSendCallConstructor: constructor,
	}
	err = s.ExecuteGovV1Proposal(ctx, msg, s.Wfchain, s.CosmosSubmitter)
	s.Require().NoError(err)
}

// registerSolanaIFTBridge registers an IFT bridge for the Cosmos counterparty
func (s *IbcEurekaSolanaIFTTestSuite) registerSolanaIFTBridge(ctx context.Context, clientID, counterpartyAddress, counterpartyDenom string) {
	s.Require().True(s.Run("Register IFT Bridge", func() {
		bridgePDA, _ := solana.Ift.IftBridgePDA(ift.ProgramID, s.IFTMintBytes(), []byte(clientID))
		s.IFTBridge = bridgePDA

		// Query the ICA address on Cosmos for the IFT program.
		// GMP's `send_call_cpi` uses the calling program ID as the sender.
		iftProgramAddress := ift.ProgramID.String()
		res, err := e2esuite.GRPCQuery[gmptypes.QueryAccountAddressResponse](ctx, s.Wfchain, &gmptypes.QueryAccountAddressRequest{
			ClientId: CosmosClientID, // The wasm client on Cosmos (dest client)
			Sender:   iftProgramAddress,
			Salt:     "",
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(res.AccountAddress)
		cosmosIcaAddress := res.AccountAddress
		s.T().Logf("Computed Cosmos ICA address: %s (for IFT program: %s)", cosmosIcaAddress, iftProgramAddress)

		registerMsg := ift.IftStateRegisterIftBridgeMsg{
			ClientId:               clientID,
			CounterpartyIftAddress: counterpartyAddress,
			ChainOptions: &ift.IftStateChainOptions_Cosmos{
				Denom:      counterpartyDenom,
				TypeUrl:    "/wfchain.ift.MsgIFTMint", // Type URL for Cosmos MsgIFTMint
				IcaAddress: cosmosIcaAddress,
			},
		}

		registerIx, err := ift.NewRegisterIftBridgeInstruction(
			registerMsg,
			s.IFTAppState,
			s.IFTAppMintState,
			bridgePDA,
			s.SolanaRelayer.PublicKey(), // admin
			s.SolanaRelayer.PublicKey(), // payer
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), registerIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)

		s.T().Logf("IFT Bridge registered for client %s", clientID)
		s.T().Logf("  Bridge PDA: %s", bridgePDA)
		s.T().Logf("  Counterparty: %s (Cosmos)", counterpartyAddress)
		s.T().Logf("  Counterparty Denom: %s", counterpartyDenom)
		s.T().Logf("  Cosmos ICA Address: %s", cosmosIcaAddress)
	}))
}

func (s *IbcEurekaSolanaIFTTestSuite) getCosmosIFTModuleAddress() string {
	iftAddr := authtypes.NewModuleAddress(testvalues.IFTModuleName)
	bech32Addr, err := sdk.Bech32ifyAddressBytes(s.Wfchain.Config().Bech32Prefix, iftAddr)
	s.Require().NoError(err)
	return bech32Addr
}

// createIFTSplToken creates a new SPL token for IFT
func (s *IbcEurekaSolanaIFTTestSuite) createIFTSplToken(ctx context.Context, mintWallet *solanago.Wallet) {
	s.createIFTSplTokenWithProgram(ctx, mintWallet, token.ProgramID)
}

func (s *IbcEurekaSolanaIFTTestSuite) createIFTSplTokenWithProgram(ctx context.Context, mintWallet *solanago.Wallet, tokenProgramID solanago.PublicKey) {
	mint := mintWallet.PublicKey()
	appStatePDA, _ := solana.Ift.IftAppStatePDA(ift.ProgramID)
	appMintStatePDA, _ := solana.Ift.IftAppMintStatePDA(ift.ProgramID, mint[:])
	mintAuthorityPDA, _ := solana.Ift.IftMintAuthorityPDA(ift.ProgramID, mint[:])

	s.IFTAppState = appStatePDA
	s.IFTAppMintState = appMintStatePDA
	s.IFTMintAuthority = mintAuthorityPDA

	// Initialize global app state (idempotent - will fail silently if already initialized)
	iftProgramDataPDA, _ := solana.Ift.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
	deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
	s.Require().NoError(err)

	globalInitIx, err := ift.NewInitializeInstruction(
		s.SolanaRelayer.PublicKey(), // admin
		appStatePDA,
		s.SolanaRelayer.PublicKey(),
		solanago.SystemProgramID,
		iftProgramDataPDA,
		solana.DeployerPubkey,
	)
	s.Require().NoError(err)

	globalInitTx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), globalInitIx)
	s.Require().NoError(err)

	// Ignore error - may already be initialized
	_, _ = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, globalInitTx, rpc.CommitmentConfirmed, s.SolanaRelayer, deployerWallet)

	var createTokenParams ift.IftStateCreateTokenParams
	if tokenProgramID == solanago.Token2022ProgramID {
		createTokenParams = &ift.IftStateCreateTokenParams_Token2022{
			Decimals: IFTTokenDecimals,
			Name:     IFT2022Name,
			Symbol:   IFT2022Symbol,
			Uri:      IFT2022Uri,
		}
	} else {
		createTokenParams = &ift.IftStateCreateTokenParams_SplToken{Decimals: IFTTokenDecimals}
	}
	createTokenIx, err := ift.NewCreateAndInitializeSplTokenInstruction(
		createTokenParams,
		appStatePDA,
		appMintStatePDA,
		mint,
		mintAuthorityPDA,
		s.SolanaRelayer.PublicKey(),
		s.SolanaRelayer.PublicKey(),
		tokenProgramID,
		solanago.SystemProgramID,
		solanago.SysVarInstructionsPubkey,
	)
	s.Require().NoError(err)

	tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), createTokenIx)
	s.Require().NoError(err)

	// Both the payer and mint must sign (mint is created during init)
	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer, mintWallet)
	s.Require().NoError(err)
}

// initializeExistingToken creates an SPL token with relayer as authority, mints tokens,
// then transfers mint authority to IFT. Uses legacy SPL Token program.
func (s *IbcEurekaSolanaIFTTestSuite) initializeExistingToken(ctx context.Context, amount uint64) solanago.PublicKey {
	return s.initializeExistingTokenWithProgram(ctx, amount, token.ProgramID)
}

// initializeExistingTokenWithProgram creates a token with the given program, mints tokens,
// then transfers mint authority to IFT via initialize_existing_token.
func (s *IbcEurekaSolanaIFTTestSuite) initializeExistingTokenWithProgram(ctx context.Context, amount uint64, tokenProgramID solanago.PublicKey) solanago.PublicKey {
	mintWallet := solanago.NewWallet()
	mint := mintWallet.PublicKey()

	// Base mint size is 82 bytes for both SPL Token and Token 2022 (without extensions)
	const mintAccountSize = 82
	rentExemption, err := s.Solana.Chain.RPCClient.GetMinimumBalanceForRentExemption(ctx, mintAccountSize, "confirmed")
	s.Require().NoError(err)

	// Build CreateAccount instruction
	lamportsBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(lamportsBytes, rentExemption)
	spaceBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(spaceBytes, mintAccountSize)

	createAccountData := append([]byte{0, 0, 0, 0}, lamportsBytes...)
	createAccountData = append(createAccountData, spaceBytes...)
	createAccountData = append(createAccountData, tokenProgramID[:]...)

	createAccountIx := solanago.NewInstruction(
		solanago.SystemProgramID,
		solanago.AccountMetaSlice{
			solanago.NewAccountMeta(s.SolanaRelayer.PublicKey(), true, true),
			solanago.NewAccountMeta(mint, true, true),
		},
		createAccountData,
	)

	// InitializeMint2: [discriminator=20, decimals, mint_authority(32), has_freeze=1, freeze_authority(32)]
	// Instruction format is identical for both SPL Token and Token 2022.
	relayerPK := s.SolanaRelayer.PublicKey()
	initMintData := []byte{20, IFTTokenDecimals}
	initMintData = append(initMintData, relayerPK[:]...)
	initMintData = append(initMintData, 1)
	initMintData = append(initMintData, relayerPK[:]...)

	initMintIx := solanago.NewInstruction(
		tokenProgramID,
		solanago.AccountMetaSlice{
			solanago.NewAccountMeta(mint, true, false),
		},
		initMintData,
	)

	tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), createAccountIx, initMintIx)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer, mintWallet)
	s.Require().NoError(err)
	s.T().Logf("Created token mint: %s (program: %s)", mint, tokenProgramID)

	s.IFTMintWallet = mintWallet

	// Create ATA for sender
	senderATA, err := s.Solana.Chain.CreateOrGetAssociatedTokenAccountWithProgram(ctx, s.SolanaRelayer, mint, s.SolanaRelayer.PublicKey(), tokenProgramID)
	s.Require().NoError(err)
	s.SenderTokenAccount = senderATA

	// MintTo: [discriminator=7, amount(8)]
	// Instruction format is identical for both SPL Token and Token 2022.
	mintToData := []byte{7}
	amountBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(amountBytes, amount)
	mintToData = append(mintToData, amountBytes...)

	mintToIx := solanago.NewInstruction(
		tokenProgramID,
		solanago.AccountMetaSlice{
			solanago.NewAccountMeta(mint, true, false),
			solanago.NewAccountMeta(senderATA, true, false),
			solanago.NewAccountMeta(s.SolanaRelayer.PublicKey(), false, true),
		},
		mintToData,
	)

	tx2, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), mintToIx)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx2, rpc.CommitmentConfirmed, s.SolanaRelayer)
	s.Require().NoError(err)
	s.T().Logf("Minted %d tokens to %s", amount, senderATA)

	// Derive IFT PDAs
	appStatePDA, _ := solana.Ift.IftAppStatePDA(ift.ProgramID)
	appMintStatePDA, _ := solana.Ift.IftAppMintStatePDA(ift.ProgramID, mint[:])
	mintAuthorityPDA, _ := solana.Ift.IftMintAuthorityPDA(ift.ProgramID, mint[:])

	s.IFTAppState = appStatePDA
	s.IFTAppMintState = appMintStatePDA
	s.IFTMintAuthority = mintAuthorityPDA

	// Initialize global app state (idempotent)
	iftProgramDataPDA, _ := solana.Ift.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
	deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
	s.Require().NoError(err)

	globalInitIx, err := ift.NewInitializeInstruction(
		s.SolanaRelayer.PublicKey(),
		appStatePDA,
		s.SolanaRelayer.PublicKey(),
		solanago.SystemProgramID,
		iftProgramDataPDA,
		solana.DeployerPubkey,
	)
	s.Require().NoError(err)

	globalInitTx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), globalInitIx)
	s.Require().NoError(err)
	_, _ = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, globalInitTx, rpc.CommitmentConfirmed, s.SolanaRelayer, deployerWallet)

	// Transfer mint authority to IFT via initialize_existing_token
	initExistingIx, err := ift.NewInitializeExistingTokenInstruction(
		appStatePDA,
		appMintStatePDA,
		mint,
		mintAuthorityPDA,
		s.SolanaRelayer.PublicKey(),
		s.SolanaRelayer.PublicKey(),
		tokenProgramID,
		solanago.SystemProgramID,
	)
	s.Require().NoError(err)

	tx3, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initExistingIx)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx3, rpc.CommitmentConfirmed, s.SolanaRelayer)
	s.Require().NoError(err)
	s.T().Logf("Transferred mint authority to IFT PDA: %s", mintAuthorityPDA)

	return mint
}
