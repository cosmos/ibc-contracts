// SPDX-License-Identifier: Apache-2.0

package main

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	test_cpi_proxy "github.com/cosmos/solidity-ibc-eureka/e2e/interchaintestv8/solana/go-anchor/testcpiproxy"
	test_gmp_app "github.com/cosmos/solidity-ibc-eureka/e2e/interchaintestv8/solana/go-anchor/testgmpapp"
	bin "github.com/gagliardetto/binary"
	"github.com/stretchr/testify/suite"
	googleproto "google.golang.org/protobuf/proto"

	solanago "github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/programs/system"
	"github.com/gagliardetto/solana-go/programs/token"
	"github.com/gagliardetto/solana-go/rpc"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	gmptypes "github.com/cosmos/ibc-go/v11/modules/apps/27-gmp/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"

	"github.com/cosmos/interchaintest/v11/ibc"
	"github.com/cosmos/interchaintest/v11/testutil"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"
	ics27_gmp "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics27gmp"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/gmphelpers"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
	solanatypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/solana"
)

// IbcEurekaSolanaGMPTestSuite is a test suite for Solana GMP tests
type IbcEurekaSolanaGMPTestSuite struct {
	IbcEurekaSolanaTestSuite
}

func TestWithIbcEurekaSolanaGMPTestSuite(t *testing.T) {
	s := &IbcEurekaSolanaGMPTestSuite{}
	suite.Run(t, s)
}

const (
	// GMP App
	DefaultIncrementAmount = uint64(5)
	DefaultDecrementAmount = uint64(2)
	GMPPortID              = testvalues.SolanaGMPPortID
	// SPL Token amounts (with 6 decimals)
	SPLTokenDecimals       = uint8(6)
	SPLTokenMintAmount     = uint64(10_000_000) // 10 tokens
	SPLTokenTransferAmount = uint64(1_000_000)  // 1 token
	// Test amounts
	CosmosTestAmount = int64(1000) // stake denom
	// Dummy target program ID for security tests
	DummyTargetProgramID = "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi"
)

// accountIdentifier mirrors the Rust Hashable struct for Borsh serialization
type accountIdentifier struct {
	ClientID string
	Sender   string
	Salt     []byte
}

// gmpAccountPDA derives GMP account PDA using AccountIdentifier hash
// This is a specialized PDA that uses SHA256 hashing and is not in the IDL.
// GMP accounts are stateless - no account storage, only PDA validation.
// See: packages/solana-ibc-types/src/ics27.rs - GMPAccount::new
func gmpAccountPDA(programID solanago.PublicKey, clientID string, sender string, salt []byte) (solanago.PublicKey, uint8) {
	// Borsh-encode the AccountIdentifier struct to match Rust implementation
	id := accountIdentifier{
		ClientID: clientID,
		Sender:   sender,
		Salt:     salt,
	}

	buf := new(bytes.Buffer)
	encoder := bin.NewBorshEncoder(buf)
	if err := encoder.Encode(id); err != nil {
		panic(fmt.Sprintf("failed to borsh encode account identifier: %v", err))
	}

	accountIDHash := sha256.Sum256(buf.Bytes())

	pda, bump, err := solanago.FindProgramAddress(
		[][]byte{
			[]byte("gmp_account"),
			accountIDHash[:],
		},
		programID,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to derive GMP account PDA: %v", err))
	}
	return pda, bump
}

// instructionToGMPPayload converts a solanago.Instruction into a GMPSolanaPayload
// protobuf message by extracting its data and account metas.
func instructionToGMPPayload(ix solanago.Instruction, prefundLamports uint64) (*solanatypes.GMPSolanaPayload, error) {
	data, err := ix.Data()
	if err != nil {
		return nil, err
	}
	accounts := make([]*solanatypes.SolanaAccountMeta, len(ix.Accounts()))
	for i, meta := range ix.Accounts() {
		accounts[i] = &solanatypes.SolanaAccountMeta{
			Pubkey:     meta.PublicKey.Bytes(),
			IsSigner:   meta.IsSigner,
			IsWritable: meta.IsWritable,
		}
	}
	return &solanatypes.GMPSolanaPayload{
		Data:            data,
		Accounts:        accounts,
		PrefundLamports: prefundLamports,
	}, nil
}

func (s *IbcEurekaSolanaTestSuite) initializeGMPCounterApp(ctx context.Context) solanago.PublicKey {
	s.Require().True(s.Run("Initialize GMP Counter App", func() {
		// Program already deployed, just initialize
		// Initialize GMP counter app state
		counterAppStatePDA, _ := solana.TestGmpApp.CounterAppStatePDA(s.GMPCounterProgramID)

		initInstruction, err := test_gmp_app.NewInitializeInstruction(
			s.SolanaRelayer.PublicKey(), // authority
			counterAppStatePDA,
			s.SolanaRelayer.PublicKey(), // payer
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initInstruction)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("GMP Counter app initialized")
	}))

	return s.GMPCounterProgramID
}

func (s *IbcEurekaSolanaTestSuite) initializeICS27GMP(ctx context.Context) solanago.PublicKey {
	s.Require().True(s.Run("Initialize ICS27 GMP Program", func() {
		// Program already deployed, just initialize

		// Find GMP app state PDA (using standard pattern with port_id)
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		// Initialize ICS27 GMP app
		gmpProgramDataPDA, _ := solana.Ics27Gmp.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		initInstruction, err := ics27_gmp.NewInitializeInstruction(
			access_manager.ProgramID,    // access_manager program ID
			gmpAppStatePDA,              // app state account
			s.SolanaRelayer.PublicKey(), // payer
			solanago.SystemProgramID,    // system program
			gmpProgramDataPDA,
			solana.DeployerPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initInstruction)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err)

		s.T().Logf("ICS27 GMP program initialized at: %s", s.ICS27GMPProgramID)
		s.T().Logf("GMP app state PDA: %s", gmpAppStatePDA)
		s.T().Logf("GMP port ID: %s (using proper GMP port)", GMPPortID)
	}))

	// Register GMP app with ICS26 router
	s.Require().True(s.Run("Register ICS27 GMP with Router", func() {
		routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
		ibcAppAccount, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(GMPPortID))

		registerInstruction, err := ics26_router.NewAddIbcAppInstruction(
			GMPPortID,
			routerStateAccount,
			accessControlAccount,
			ibcAppAccount,
			s.ICS27GMPProgramID,
			s.SolanaRelayer.PublicKey(),
			s.SolanaRelayer.PublicKey(),
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), registerInstruction)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("ICS27 GMP registered with router on port: %s (using proper GMP port)", GMPPortID)
	}))

	return s.ICS27GMPProgramID
}

// Test_GMPCounterFromCosmos tests sending a counter increment call from Cosmos to Solana
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPCounterFromCosmos() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]

	// Create a second Cosmos user for multi-user testing
	var cosmosUser1 ibc.Wallet
	s.Require().True(s.Run("Create Second Cosmos User", func() {
		cosmosUser1 = s.CreateAndFundCosmosUser(ctx, simd)
		s.Cosmos.Users = append(s.Cosmos.Users, cosmosUser1)
		s.T().Logf("Created second Cosmos user: %s", cosmosUser1.FormattedAddress())
	}))

	// ICS27 GMP program is already deployed and initialized in SetupSuite
	ics27GMPProgramID := ics27_gmp.ProgramID
	s.Require().True(s.Run("Verify ICS27 GMP Program", func() {
	}))

	// Initialize GMP counter app (already deployed)
	var gmpCounterProgramID solanago.PublicKey
	s.Require().True(s.Run("Initialize GMP Counter App", func() {
		gmpCounterProgramID = s.initializeGMPCounterApp(ctx)
		s.T().Logf("GMP Counter app initialized at %s", gmpCounterProgramID)
	}))

	_ = ics27GMPProgramID // Use the GMP program ID for future packet flow

	// Setup user identities and helper functions
	var getCounterValue func(cosmosUserAddress string) uint64
	var sendGMPIncrement func(cosmosUser ibc.Wallet, amount uint64) []byte
	var relayGMPPacket func(cosmosGMPTxHash []byte, userLabel string) solanago.Signature

	s.Require().True(s.Run("Setup User Identities and Helpers", func() {
		// We don't need separate Solana user keys - the GMP account PDAs are the identities
		// The user counter PDAs are derived from the GMP account PDAs

		// Helper to get counter value for a Cosmos user
		// This derives the GMP account PDA, then the user counter PDA from that
		getCounterValue = func(cosmosUserAddress string) uint64 {
			// Derive GMP account PDA for this Cosmos user (no storage, just PDA validation)
			salt := []byte{} // Empty salt for this test

			ics27AccountPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosUserAddress, salt)

			// Derive user counter PDA from GMP account PDA
			userCounterPDA, _ := solana.TestGmpApp.UserCounterWithAccountSeedPDA(gmpCounterProgramID, ics27AccountPDA.Bytes())

			// Use confirmed commitment to match relay transaction confirmation level
			account, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, userCounterPDA, &rpc.GetAccountInfoOpts{
				Commitment: rpc.CommitmentConfirmed,
			})
			if err != nil || account.Value == nil {
				return 0 // Account doesn't exist yet
			}

			data := account.Value.Data.GetBinary()
			if len(data) >= 48 {
				return binary.LittleEndian.Uint64(data[40:48])
			}
			return 0
		}

		// Helper to send GMP increment from a Cosmos user
		sendGMPIncrement = func(cosmosUser ibc.Wallet, amount uint64) []byte {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			simd := s.Cosmos.Chains[0]

			// Derive the GMP account PDA for this Cosmos user
			// This PDA is the authority that signs for the counter operations (stateless, no storage)
			cosmosAddress := cosmosUser.FormattedAddress()
			salt := []byte{} // Empty salt for this test

			ics27AccountPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosAddress, salt)

			// Create the raw instruction data (just discriminator + amount, no user pubkey)
			incrementInstructionData := []byte{}
			incrementInstructionData = append(incrementInstructionData, test_gmp_app.Instruction_Increment[:]...)
			amountBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(amountBytes, amount)
			incrementInstructionData = append(incrementInstructionData, amountBytes...)

			// Derive required account addresses
			// 1. Counter app_state PDA
			counterAppStateAddress, _ := solana.TestGmpApp.CounterAppStatePDA(gmpCounterProgramID)

			// 2. User counter PDA - derived from the GMP account PDA (stateless identity)
			userCounterAddress, _ := solana.TestGmpApp.UserCounterWithAccountSeedPDA(gmpCounterProgramID, ics27AccountPDA.Bytes())

			// Create GMPSolanaPayload protobuf message
			solanaInstruction := &solanatypes.GMPSolanaPayload{
				Data: incrementInstructionData,
				Accounts: []*solanatypes.SolanaAccountMeta{
					{Pubkey: counterAppStateAddress.Bytes(), IsSigner: false, IsWritable: true},    // [0] counter app_state
					{Pubkey: userCounterAddress.Bytes(), IsSigner: false, IsWritable: true},        // [1] user_counter
					{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: false},           // [2] user_authority (GMP account PDA signs via invoke_signed)
					{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: true},            // [3] payer (GMP PDA, pre-funded by relayer)
					{Pubkey: solanago.SystemProgramID.Bytes(), IsSigner: false, IsWritable: false}, // [4] system_program
				},
				PrefundLamports: 5_000_000, // rent for UserCounter + GMP PDA rent-exempt minimum
			}

			payload, err := gmphelpers.MarshalGMPSolanaPayload(solanaInstruction, encodingType.String())
			if err != nil {
				return nil
			}

			// Send GMP call using proper gmptypes.MsgSendCall
			resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
				SourceClient:     CosmosClientID,
				Sender:           cosmosUser.FormattedAddress(),
				Receiver:         gmpCounterProgramID.String(),
				Salt:             []byte{},
				Payload:          payload,
				TimeoutTimestamp: timeout,
				Memo:             "increment counter via GMP",
				Encoding:         encodingType.String(),
			})
			if err != nil {
				return nil
			}

			cosmosGMPTxHashBytes, err := hex.DecodeString(resp.TxHash)
			if err != nil {
				return nil
			}

			s.T().Logf("GMP packet sent from %s: %s (increment by %d)", cosmosUser.FormattedAddress(), resp.TxHash, amount)
			return cosmosGMPTxHashBytes
		}

		// Helper to relay and execute a GMP packet
		relayGMPPacket = func(cosmosGMPTxHash []byte, userLabel string) solanago.Signature {
			var solanaRelayTxSig solanago.Signature

			simd := s.Cosmos.Chains[0]

			// First, update the Solana client to the latest height
			updateResp, err := s.ProofApiClient.UpdateClient(context.Background(), &proofapitypes.UpdateClientRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				DstClientId: SolanaClientID,
			})
			s.Require().NoError(err, "Proof API UpdateClient failed")
			s.Require().NotEmpty(updateResp.Tx, "Proof API UpdateClient should return transaction")

			s.Solana.Chain.SubmitChunkedUpdateClient(ctx, s.T(), s.Require(), updateResp, s.SolanaRelayer)

			// Now retrieve and relay the GMP packet
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosGMPTxHash},
				SrcClientId: CosmosClientID,
				DstClientId: SolanaClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

			// Execute on Solana using chunked submission
			solanaRelayTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
			s.Require().NoError(err)
			s.T().Logf("%s: GMP execution completed on Solana", userLabel)

			return solanaRelayTxSig
		}

		s.T().Logf("Setup complete - User0 key: %s, User1 key: %s", s.Cosmos.Users[0].FormattedAddress(), s.Cosmos.Users[1].FormattedAddress())
	}))

	// Check initial counter states
	var initialCounterUser0, initialCounterUser1 uint64
	s.Require().True(s.Run("Check Initial Counter States", func() {
		initialCounterUser0 = getCounterValue(s.Cosmos.Users[0].FormattedAddress())
		initialCounterUser1 = getCounterValue(s.Cosmos.Users[1].FormattedAddress())
		s.T().Logf("Initial counter for user0: %d", initialCounterUser0)
		s.T().Logf("Initial counter for user1: %d", initialCounterUser1)
	}))

	// Send increment from User 0
	var cosmosGMPTxHashUser0 []byte
	s.Require().True(s.Run("User0: Send GMP increment call from Cosmos", func() {
		cosmosGMPTxHashUser0 = sendGMPIncrement(s.Cosmos.Users[0], DefaultIncrementAmount)
		s.Require().NotEmpty(cosmosGMPTxHashUser0)
	}))

	// Relay User 0's increment
	var solanaRelayTxSigUser0 solanago.Signature
	s.Require().True(s.Run("User0: Relay and execute GMP packet on Solana", func() {
		solanaRelayTxSigUser0 = relayGMPPacket(cosmosGMPTxHashUser0, "User0")
	}))

	s.Require().True(s.Run("User0: Verify counter was incremented", func() {
		newCounter := getCounterValue(s.Cosmos.Users[0].FormattedAddress())
		expectedCounter := initialCounterUser0 + DefaultIncrementAmount
		s.Require().Equal(expectedCounter, newCounter)
		s.T().Logf("User0: Counter successfully incremented from %d to %d", initialCounterUser0, newCounter)
	}))

	// User 0 increments again (to test that existing account works correctly)
	var cosmosGMPTxHashUser0Second []byte
	s.Require().True(s.Run("User0: Send second GMP increment call from Cosmos", func() {
		cosmosGMPTxHashUser0Second = sendGMPIncrement(s.Cosmos.Users[0], 7) // Increment by 7 for variety
		s.Require().NotEmpty(cosmosGMPTxHashUser0Second)
	}))

	var solanaRelayTxSigUser0Second solanago.Signature
	s.Require().True(s.Run("User0: Relay and execute second GMP packet on Solana", func() {
		solanaRelayTxSigUser0Second = relayGMPPacket(cosmosGMPTxHashUser0Second, "User0 (second)")
	}))

	var afterSecondIncrement uint64
	s.Require().True(s.Run("User0: Verify counter was incremented again", func() {
		afterSecondIncrement = getCounterValue(s.Cosmos.Users[0].FormattedAddress())
		expectedCounter := initialCounterUser0 + DefaultIncrementAmount + 7
		s.Require().Equal(expectedCounter, afterSecondIncrement)
		s.T().Logf("User0: Counter successfully incremented from %d to %d (second increment by 7)", initialCounterUser0+DefaultIncrementAmount, afterSecondIncrement)
	}))

	// Now send increment from User 1
	var cosmosGMPTxHashUser1 []byte
	s.Require().True(s.Run("User1: Send GMP increment call from Cosmos", func() {
		cosmosGMPTxHashUser1 = sendGMPIncrement(s.Cosmos.Users[1], 3) // Increment by 3 for variety
		s.Require().NotEmpty(cosmosGMPTxHashUser1)
	}))

	// Relay User 1's increment
	var solanaRelayTxSigUser1 solanago.Signature
	s.Require().True(s.Run("User1: Relay and execute GMP packet on Solana", func() {
		solanaRelayTxSigUser1 = relayGMPPacket(cosmosGMPTxHashUser1, "User1")
	}))

	s.Require().True(s.Run("User1: Verify counter was incremented", func() {
		newCounter := getCounterValue(s.Cosmos.Users[1].FormattedAddress())
		expectedCounter := initialCounterUser1 + 3 // We incremented by 3
		s.Require().Equal(expectedCounter, newCounter)
		s.T().Logf("User1: Counter successfully incremented from %d to %d", initialCounterUser1, newCounter)
	}))

	s.Require().True(s.Run("Verify final counter states for both users", func() {
		finalCounterUser0 := getCounterValue(s.Cosmos.Users[0].FormattedAddress())
		finalCounterUser1 := getCounterValue(s.Cosmos.Users[1].FormattedAddress())

		// User 0 should have: initial + DefaultIncrementAmount (5) + 7
		expectedFinalUser0 := initialCounterUser0 + DefaultIncrementAmount + 7
		s.Require().Equal(expectedFinalUser0, finalCounterUser0)

		// User 1 should have: initial + 3
		expectedFinalUser1 := initialCounterUser1 + 3
		s.Require().Equal(expectedFinalUser1, finalCounterUser1)

		s.T().Logf("Final counter states - User0: %d (expected: %d, incremented twice), User1: %d (expected: %d)",
			finalCounterUser0, expectedFinalUser0, finalCounterUser1, expectedFinalUser1)
	}))

	s.Require().True(s.Run("Verify acknowledgments on Solana and relay to Cosmos", func() {
		simd := s.Cosmos.Chains[0]

		// Helper to verify ack content on Solana and relay to Cosmos
		// The WriteAcknowledgementEvent on Solana contains the protobuf-encoded GmpAcknowledgement
		// with the counter value returned by the GMP counter app
		verifyAndRelayAck := func(solanaRelayTxSig solanago.Signature, label string, expectedCounterValue uint64) {
			// First verify the ack content on Solana
			s.Require().True(s.Run(fmt.Sprintf("Verify %s ack on Solana", label), func() {
				events, err := s.Solana.Chain.GetWriteAcknowledgementEvents(ctx, solanaRelayTxSig)
				s.Require().NoError(err, "Failed to get WriteAcknowledgementEvents")
				s.Require().Len(events, 1, "Should have exactly one WriteAcknowledgementEvent")

				event := events[0]
				s.Require().Len(event.Acknowledgements, 1, "Should have exactly one ack (one payload)")

				ackBytes := event.Acknowledgements[0]

				ack, err := gmphelpers.UnmarshalAck(ackBytes, encodingType.String())
				s.Require().NoError(err, "Failed to unmarshal GMP acknowledgement")

				// Extract counter value (u64 little-endian)
				s.Require().Len(ack.Result, 8, "Result should be 8 bytes (u64)")
				actualCounter := binary.LittleEndian.Uint64(ack.Result)
				s.Require().Equal(expectedCounterValue, actualCounter,
					"Counter in ack should be %d, got %d", expectedCounterValue, actualCounter)
				s.T().Logf("%s ack verified on Solana: counter=%d", label, actualCounter)
			}))

			// Then relay the ack to Cosmos
			var ackRelayTxBodyBz []byte
			s.Require().True(s.Run(fmt.Sprintf("Retrieve %s ack relay tx", label), func() {
				resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
					SrcChain:    testvalues.SolanaChainID,
					DstChain:    simd.Config().ChainID,
					SourceTxIds: [][]byte{[]byte(solanaRelayTxSig.String())},
					SrcClientId: SolanaClientID,
					DstClientId: CosmosClientID,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				ackRelayTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run(fmt.Sprintf("Broadcast %s ack to Cosmos", label), func() {
				relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], CosmosDefaultGasLimit, ackRelayTxBodyBz)
				s.Require().Equal(uint32(0), relayTxResult.Code, "Ack relay tx should succeed")
				s.T().Logf("%s ack relayed to Cosmos: %s (code: %d, gas: %d)",
					label, relayTxResult.TxHash, relayTxResult.Code, relayTxResult.GasUsed)

				// Verify acknowledge_packet event was emitted
				_, err := cosmos.GetEventValue(relayTxResult.Events, channeltypesv2.EventTypeAcknowledgePacket, channeltypesv2.AttributeKeySequence)
				s.Require().NoError(err, "acknowledge_packet event should be emitted")
			}))
		}

		// User0 first increment: 0 + 5 = 5
		verifyAndRelayAck(solanaRelayTxSigUser0, "User0 first", initialCounterUser0+DefaultIncrementAmount)
		// User0 second increment: 5 + 7 = 12
		verifyAndRelayAck(solanaRelayTxSigUser0Second, "User0 second", initialCounterUser0+DefaultIncrementAmount+7)
		// User1 first increment: 0 + 3 = 3
		verifyAndRelayAck(solanaRelayTxSigUser1, "User1", initialCounterUser1+3)

		s.T().Logf("All GMP acknowledgments verified and relayed successfully")
	}))
}

// Test_GMPSPLTokenTransfer tests transferring SPL tokens via GMP from Cosmos to Solana
// This demonstrates the SPL token transfer example from the ADR where:
// 1. A Cosmos user controls an ICS27 Account PDA on Solana
// 2. The ICS27 PDA owns SPL token accounts
// 3. Through GMP, the Cosmos user sends cross-chain calls to transfer tokens
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPSPLTokenTransferFromCosmos() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUser := s.Cosmos.Users[0]

	// Setup SPL token infrastructure
	var tokenMint solanago.PublicKey
	var ics27AccountPDA solanago.PublicKey
	var sourceTokenAccount solanago.PublicKey
	var destTokenAccount solanago.PublicKey
	var recipientWallet *solanago.Wallet

	s.Require().True(s.Run("Setup SPL Token Infrastructure", func() {
		s.Require().True(s.Run("Create Test SPL Token Mint", func() {
			var err error
			tokenMint, err = s.Solana.Chain.CreateSPLTokenMint(ctx, s.SolanaRelayer, 6)
			s.Require().NoError(err)
			s.T().Logf("Created test SPL token mint: %s (6 decimals)", tokenMint.String())
		}))

		s.Require().True(s.Run("Derive ICS27 Account PDA", func() {
			ics27AccountPDA, _ = gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosUser.FormattedAddress(), []byte{})
			s.T().Logf("ICS27 Account PDA for Cosmos user: %s", ics27AccountPDA.String())
		}))

		s.Require().True(s.Run("Create Token Accounts", func() {
			var err error

			// Create source token account (owned by ICS27 PDA)
			sourceTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, ics27AccountPDA)
			s.Require().NoError(err)
			s.T().Logf("Created source token account (owned by ICS27 PDA): %s", sourceTokenAccount.String())

			// Create recipient wallet and destination token account
			recipientWallet, err = s.Solana.Chain.CreateAndFundWallet()
			s.Require().NoError(err)

			destTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, recipientWallet.PublicKey())
			s.Require().NoError(err)
			s.T().Logf("Created destination token account (owned by recipient): %s", destTokenAccount.String())
		}))

		s.Require().True(s.Run("Mint Tokens to ICS27 PDA", func() {
			// Mint 10 tokens (10,000,000 with 6 decimals)
			mintAmount := SPLTokenMintAmount
			err := s.Solana.Chain.MintTokensTo(ctx, s.SolanaRelayer, tokenMint, sourceTokenAccount, mintAmount)
			s.Require().NoError(err)

			balance, err := s.Solana.Chain.GetTokenBalance(ctx, sourceTokenAccount)
			s.Require().NoError(err)
			s.Require().Equal(mintAmount, balance)
			s.T().Logf("Minted %d tokens to ICS27 PDA's token account", mintAmount)
		}))
	}))

	// Execute SPL token transfer via GMP
	var cosmosGMPTxHash []byte
	transferAmount := SPLTokenTransferAmount // 1 token (1,000,000 with 6 decimals)

	s.Require().True(s.Run("Send GMP SPL Token Transfer from Cosmos", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		// Build SPL transfer instruction
		splTransferInstruction := token.NewTransferInstruction(
			transferAmount,
			sourceTokenAccount,
			destTokenAccount,
			ics27AccountPDA, // Authority - will be signed by GMP program via invoke_signed
			[]solanago.PublicKey{},
		).Build()

		// Get instruction data
		instructionData, err := splTransferInstruction.Data()
		s.Require().NoError(err)

		// Create GMPSolanaPayload protobuf
		// SPL Transfer requires exactly 3 accounts: source, destination, authority
		solanaInstruction := &solanatypes.GMPSolanaPayload{
			Data: instructionData,
			Accounts: []*solanatypes.SolanaAccountMeta{
				{Pubkey: sourceTokenAccount.Bytes(), IsSigner: false, IsWritable: true}, // [0] source
				{Pubkey: destTokenAccount.Bytes(), IsSigner: false, IsWritable: true},   // [1] destination
				{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: false},    // [2] authority (GMP PDA signs via invoke_signed)
			},
		}

		payload, err := gmphelpers.MarshalGMPSolanaPayload(solanaInstruction, encodingType.String())
		s.Require().NoError(err)

		// Send GMP call
		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosUser.FormattedAddress(),
			Receiver:         token.ProgramID.String(),
			Salt:             []byte{},
			Payload:          payload,
			TimeoutTimestamp: timeout,
			Memo:             fmt.Sprintf("SPL token transfer: %d tokens", transferAmount),
			Encoding:         encodingType.String(),
		})
		s.Require().NoError(err)

		cosmosGMPTxHashBytes, err := hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		cosmosGMPTxHash = cosmosGMPTxHashBytes

		s.T().Logf("GMP SPL transfer packet sent from Cosmos: %s", resp.TxHash)
	}))

	// Record initial balances
	var initialSourceBalance uint64
	var initialDestBalance uint64
	s.Require().True(s.Run("Record Initial Token Balances", func() {
		var err error
		initialSourceBalance, err = s.Solana.Chain.GetTokenBalance(ctx, sourceTokenAccount)
		s.Require().NoError(err)

		initialDestBalance, err = s.Solana.Chain.GetTokenBalance(ctx, destTokenAccount)
		s.Require().NoError(err)

		s.T().Logf("Initial balances - Source: %d, Dest: %d", initialSourceBalance, initialDestBalance)
	}))

	// Relay and execute on Solana
	var solanaRelayTxSig solanago.Signature
	s.Require().True(s.Run("Relay and Execute SPL Transfer on Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		solanaRelayTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("SPL transfer executed on Solana: %s", solanaRelayTxSig)
	}))

	// Verify transfer completed
	s.Require().True(s.Run("Verify SPL Token Transfer", func() {
		finalSourceBalance, err := s.Solana.Chain.GetTokenBalance(ctx, sourceTokenAccount)
		s.Require().NoError(err)

		finalDestBalance, err := s.Solana.Chain.GetTokenBalance(ctx, destTokenAccount)
		s.Require().NoError(err)

		expectedSourceBalance := initialSourceBalance - transferAmount
		expectedDestBalance := initialDestBalance + transferAmount

		s.Require().Equal(expectedSourceBalance, finalSourceBalance,
			"Source balance should decrease by transfer amount")
		s.Require().Equal(expectedDestBalance, finalDestBalance,
			"Destination balance should increase by transfer amount")

		s.T().Logf("Transfer verified!")
		s.T().Logf("  Source: %d → %d (-%d)", initialSourceBalance, finalSourceBalance, transferAmount)
		s.T().Logf("  Dest:   %d → %d (+%d)", initialDestBalance, finalDestBalance, transferAmount)
	}))

	// Verify acknowledgment on Solana and relay to Cosmos
	s.Require().True(s.Run("Verify and Relay Acknowledgment", func() {
		// Verify ack on Solana
		s.Require().True(s.Run("Verify ack on Solana", func() {
			events, err := s.Solana.Chain.GetWriteAcknowledgementEvents(ctx, solanaRelayTxSig)
			s.Require().NoError(err, "Failed to get WriteAcknowledgementEvents")
			s.Require().Len(events, 1, "Should have exactly one WriteAcknowledgementEvent")

			event := events[0]
			s.Require().Len(event.Acknowledgements, 1, "Should have exactly one ack")

			// SPL Token program doesn't return data, so ack uses empty_success sentinel
			ackBytes := event.Acknowledgements[0]
			s.T().Logf("SPL transfer ack bytes: %v (len=%d)", ackBytes, len(ackBytes))

			ack, err := gmphelpers.UnmarshalAck(ackBytes, encodingType.String())
			s.Require().NoError(err, "Failed to unmarshal GMP acknowledgement")

			// SPL Token program returns no data; GMP uses [0] sentinel for protobuf, empty bytes for ABI
			if encodingType.String() == testvalues.Ics27AbiEncoding {
				s.Require().Empty(ack.Result, "SPL transfer ack result should be empty for ABI encoding")
			} else {
				s.Require().Equal([]byte{0}, ack.Result, "SPL transfer ack result should be success sentinel")
			}
			s.T().Logf("SPL transfer ack verified on Solana: success (no return data)")
		}))

		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve acknowledgment relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaRelayTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			ackRelayTxBodyBz = resp.Tx
		}))

		s.Require().True(s.Run("Broadcast ack to Cosmos", func() {
			relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, cosmosUser, CosmosDefaultGasLimit, ackRelayTxBodyBz)
			s.Require().Equal(uint32(0), relayTxResult.Code, "Ack relay tx should succeed")
			s.T().Logf("SPL transfer ack relayed to Cosmos: %s (code: %d, gas: %d)",
				relayTxResult.TxHash, relayTxResult.Code, relayTxResult.GasUsed)

			_, err := cosmos.GetEventValue(relayTxResult.Events, channeltypesv2.EventTypeAcknowledgePacket, channeltypesv2.AttributeKeySequence)
			s.Require().NoError(err, "acknowledge_packet event should be emitted")
		}))

		s.T().Logf("✓ SPL token transfer via GMP completed successfully")
		s.T().Logf("  Cosmos user %s controlled Solana ICS27 PDA %s",
			cosmosUser.FormattedAddress(), ics27AccountPDA.String())
		s.T().Logf("  Transferred %d tokens from ICS27 PDA to recipient", transferAmount)
	}))
}

func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPSendCallFromSolana() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]

	testAmount := sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(CosmosTestAmount)))
	testCosmosUser := s.CreateAndFundCosmosUserWithBalance(ctx, simd, testAmount[0].Amount.Int64())

	var computedAddress sdk.AccAddress
	s.Require().True(s.Run("Fund pre-computed ICS27 address on Cosmos", func() {
		solanaUserAddress := s.SolanaRelayer.PublicKey().String()

		// Use CosmosClientID (08-wasm-0) - the dest_client on Cosmos
		// The GMP keeper creates accounts using NewAccountIdentifier(destClient, sender, salt)
		res, err := e2esuite.GRPCQuery[gmptypes.QueryAccountAddressResponse](ctx, simd, &gmptypes.QueryAccountAddressRequest{
			ClientId: CosmosClientID,
			Sender:   solanaUserAddress,
			Salt:     "",
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(res.AccountAddress)

		computedAddress, err = sdk.AccAddressFromBech32(res.AccountAddress)
		s.Require().NoError(err)

		s.T().Logf("ICS27 account on Cosmos: %s", computedAddress.String())

		_, err = s.BroadcastMessages(ctx, simd, testCosmosUser, CosmosDefaultGasLimit, &banktypes.MsgSend{
			FromAddress: testCosmosUser.FormattedAddress(),
			ToAddress:   computedAddress.String(),
			Amount:      testAmount,
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify initial balance on Cosmos", func() {
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: computedAddress.String(),
			Denom:   simd.Config().Denom,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		s.Require().Equal(testAmount[0].Amount.Int64(), resp.Balance.Amount.Int64())
	}))

	var solanaPacketTxHash string
	var sequence uint64
	s.Require().True(s.Run("Send call from Solana", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		var payload []byte
		s.Require().True(s.Run("Prepare GMP payload", func() {
			msgSend := &banktypes.MsgSend{
				FromAddress: computedAddress.String(),
				ToAddress:   testCosmosUser.FormattedAddress(),
				Amount:      testAmount,
			}

			var err error
			payload, err = gmphelpers.NewPayload_FromProto([]proto.Message{msgSend})
			s.Require().NoError(err)
			s.T().Logf("Encoded GMP payload (%d bytes)", len(payload))
		}))

		var gmpAppStatePDA, routerStatePDA, clientPDA, ibcAppPDA, lightClientStatePDA, consensusStatePDA solanago.PublicKey
		s.Require().True(s.Run("Derive required PDAs", func() {
			gmpAppStatePDA, _ = solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)
			routerStatePDA, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
			clientPDA, _ = solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))
			ibcAppPDA, _ = solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(GMPPortID))
			lightClientStatePDA, _ = solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
			consensusStatePDA = s.deriveIcs07ConsensusStatePDA(ctx, lightClientStatePDA)

			s.T().Logf("Derived PDAs: gmpAppState=%s, routerState=%s, client=%s",
				gmpAppStatePDA.String(), routerStatePDA.String(), clientPDA.String())
		}))

		var packetCommitmentPDA solanago.PublicKey
		s.Require().True(s.Run("Choose sequence and derive packet commitment PDA", func() {
			sequence = 1
			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, sequence)
			packetCommitmentPDA, _ = solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), sequenceBytes)
			s.T().Logf("Using sequence: %d", sequence)
		}))

		var sendCallInstruction solanago.Instruction
		s.Require().True(s.Run("Build send_call instruction", func() {
			var err error
			sendCallInstruction, err = ics27_gmp.NewSendCallInstruction(
				ics27_gmp.Ics27GmpStateSendCallMsg{
					SourceClient:     SolanaClientID,
					Sequence:         sequence,
					TimeoutTimestamp: timeout,
					Receiver:         "", // Target program on Cosmos (empty for native modules)
					Salt:             []byte{},
					Payload:          payload,
					Memo:             "send from Solana to Cosmos",
					Encoding:         encodingType.String(),
				},
				gmpAppStatePDA,
				s.SolanaRelayer.PublicKey(),
				s.SolanaRelayer.PublicKey(),
				ics26_router.ProgramID,
				routerStatePDA,
				packetCommitmentPDA,
				ibcAppPDA,
				clientPDA,
				ics07_tendermint.ProgramID,
				lightClientStatePDA,
				solanago.SysVarInstructionsPubkey,
				consensusStatePDA,
				solanago.SystemProgramID,
			)
			s.Require().NoError(err)
			s.T().Log("Built send_call instruction")
		}))

		s.Require().True(s.Run("Broadcast transaction", func() {
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(
				s.SolanaRelayer.PublicKey(),
				sendCallInstruction,
			)
			s.Require().NoError(err)

			sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
			s.Require().NoError(err)
			s.Require().NotEmpty(sig)

			solanaPacketTxHash = sig.String()
			s.T().Logf("Send call transaction: %s", solanaPacketTxHash)
		}))
	}))

	var ackTxHash []byte
	var ackBytes []byte
	s.Require().True(s.Run("Receive packet in Cosmos", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			txHashBytes := []byte(solanaPacketTxHash)

			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{txHashBytes},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			recvRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx to Cosmos", func() {
			receipt := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 2_000_000, recvRelayTx)
			s.T().Logf("Recv packet tx result: code=%d, log=%s, gas=%d", receipt.Code, receipt.RawLog, receipt.GasUsed)

			s.Require().Equal(uint32(0), receipt.Code, "Tx should succeed")
			s.Require().NotEmpty(receipt.TxHash)

			var err error
			ackTxHash, err = hex.DecodeString(receipt.TxHash)
			s.Require().NoError(err)

			ackHex, err := cosmos.GetEventValue(receipt.Events, channeltypesv2.EventTypeWriteAck, channeltypesv2.AttributeKeyEncodedAckHex)
			s.Require().NoError(err, "Failed to get acknowledgement from write_acknowledgement event")
			ackBytes, err = hex.DecodeString(ackHex)
			s.Require().NoError(err, "Failed to decode acknowledgement hex")
		}))

		s.Require().True(s.Run("Verify balance changed on Cosmos", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: computedAddress.String(),
				Denom:   simd.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Zero(resp.Balance.Amount.Int64())

			resp, err = e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: testCosmosUser.FormattedAddress(),
				Denom:   simd.Config().Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			s.Require().Equal(testAmount[0].Amount.Int64(), resp.Balance.Amount.Int64())
		}))
	}))

	var gmpResultPDA solanago.PublicKey
	s.Require().True(s.Run("Acknowledge packet in Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{ackTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		gmpResultPDA, _ = solana.GMPCallResultPDA(ics27_gmp.ProgramID, SolanaClientID, sequence)

		sig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Acknowledgement transaction broadcasted: %s", sig)

		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, sequence)
	}))

	s.Require().True(s.Run("Verify GMP call result PDA", func() {
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, gmpResultPDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value, "GMP result account should exist")
		s.Require().Equal(ics27_gmp.ProgramID, accountInfo.Value.Owner, "Account should be owned by ics27_gmp program")

		data := accountInfo.Value.Data.GetBinary()
		result, err := solana.DecodeGMPCallResultAccount(data)
		s.Require().NoError(err, "Failed to decode GMP call result account")

		s.Require().Equal(uint8(0), result.Version, "Version should be 0 (V1)")
		s.Require().Equal(s.SolanaRelayer.PublicKey(), result.Sender, "Sender should match")
		s.Require().Equal(sequence, result.Sequence, "Sequence should match")
		s.Require().Equal(SolanaClientID, result.SourceClient, "Source client should match")
		s.Require().Equal(CosmosClientID, result.DestClient, "Dest client should match")
		s.Require().Equal(solana.CallResultStatusAcknowledgement, result.Status, "Status should be Acknowledgement")
		var ibcAck channeltypesv2.Acknowledgement
		err = proto.Unmarshal(ackBytes, &ibcAck)
		s.Require().NoError(err, "Failed to unmarshal IBC Acknowledgement")
		s.Require().Len(ibcAck.AppAcknowledgements, 1, "Should have exactly one app acknowledgement")
		expectedCommitment := solana.IBCAckCommitment(ibcAck.AppAcknowledgements[0])
		s.Require().Equal(expectedCommitment, result.AckCommitment, "AckCommitment should match IBC commitment")
		s.Require().True(result.ResultTimestamp > 0, "Result timestamp should be set")
		s.Require().True(result.Bump > 0, "Bump should be non-zero")

		s.T().Logf("GMP result account validated: sender=%s, sequence=%d, status=Acknowledgement, ack_commitment=%x",
			result.Sender, result.Sequence, result.AckCommitment)
	}))
}

// Test_GMPRelayLatency_SolanaToCosmosPacket sends a Solana-to-Cosmos packet,
// waits for finality, then checks the relay completes quickly.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPRelayLatency_SolanaToCosmosPacket() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]

	testAmount := sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(CosmosTestAmount)))
	testCosmosUser := s.CreateAndFundCosmosUserWithBalance(ctx, simd, testAmount[0].Amount.Int64())

	var computedAddress sdk.AccAddress
	s.Require().True(s.Run("Fund pre-computed ICS27 address on Cosmos", func() {
		solanaUserAddress := s.SolanaRelayer.PublicKey().String()

		res, err := e2esuite.GRPCQuery[gmptypes.QueryAccountAddressResponse](ctx, simd, &gmptypes.QueryAccountAddressRequest{
			ClientId: CosmosClientID,
			Sender:   solanaUserAddress,
			Salt:     "",
		})
		s.Require().NoError(err)

		computedAddress, err = sdk.AccAddressFromBech32(res.AccountAddress)
		s.Require().NoError(err)

		_, err = s.BroadcastMessages(ctx, simd, testCosmosUser, CosmosDefaultGasLimit, &banktypes.MsgSend{
			FromAddress: testCosmosUser.FormattedAddress(),
			ToAddress:   computedAddress.String(),
			Amount:      testAmount,
		})
		s.Require().NoError(err)
	}))

	var solanaPacketTxSig solanago.Signature
	s.Require().True(s.Run("Send call from Solana", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		msgSend := &banktypes.MsgSend{
			FromAddress: computedAddress.String(),
			ToAddress:   testCosmosUser.FormattedAddress(),
			Amount:      testAmount,
		}
		payload, err := gmphelpers.NewPayload_FromProto([]proto.Message{msgSend})
		s.Require().NoError(err)

		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)
		routerStatePDA, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
		clientPDA, _ := solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))
		ibcAppPDA, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(GMPPortID))
		lightClientStatePDA, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, lightClientStatePDA)

		sequence := uint64(1)
		sequenceBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(sequenceBytes, sequence)
		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), sequenceBytes)

		sendCallInstruction, err := ics27_gmp.NewSendCallInstruction(
			ics27_gmp.Ics27GmpStateSendCallMsg{
				SourceClient:     SolanaClientID,
				Sequence:         sequence,
				TimeoutTimestamp: timeout,
				Receiver:         "",
				Salt:             []byte{},
				Payload:          payload,
				Memo:             "relay latency test",
				Encoding:         encodingType.String(),
			},
			gmpAppStatePDA,
			s.SolanaRelayer.PublicKey(),
			s.SolanaRelayer.PublicKey(),
			ics26_router.ProgramID,
			routerStatePDA,
			packetCommitmentPDA,
			ibcAppPDA,
			clientPDA,
			ics07_tendermint.ProgramID,
			lightClientStatePDA,
			solanago.SysVarInstructionsPubkey,
			consensusStatePDA,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			sendCallInstruction,
		)
		s.Require().NoError(err)

		solanaPacketTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Packet sent from Solana: %s", solanaPacketTxSig)
	}))

	s.Require().True(s.Run("Wait for Solana finality", func() {
		s.Require().NoError(s.Solana.Chain.WaitForTxStatus(solanaPacketTxSig, rpc.ConfirmationStatusFinalized))
	}))

	s.Require().True(s.Run("Relay packet and assert latency", func() {
		start := time.Now()

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    simd.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(solanaPacketTxSig.String())},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		relayElapsed := time.Since(start)
		s.T().Logf("RelayByTx completed in %s", relayElapsed)
		// Locally passes at ~3s; padded to 7s for CI.
		s.Require().Less(relayElapsed, 7*time.Second, "RelayByTx should complete within finalization window")

		receipt := s.MustBroadcastSdkTxBodyNoWait(ctx, simd, s.Cosmos.Users[0], 2_000_000, resp.Tx)
		s.Require().Equal(uint32(0), receipt.Code)
	}))
}

// Test_GMPRelayLatency_CosmosToSolanaTimeout sends a Cosmos-to-Solana packet
// with a short timeout, lets it expire, then checks the timeout relay is fast.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPRelayLatency_CosmosToSolanaTimeout() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUser := s.Cosmos.Users[0]

	var tokenMint solanago.PublicKey
	var ics27AccountPDA solanago.PublicKey
	var sourceTokenAccount solanago.PublicKey

	const tokenAmount = uint64(1_000_000)

	s.Require().True(s.Run("Setup SPL Token Infrastructure", func() {
		s.Require().True(s.Run("Create Test SPL Token Mint", func() {
			var err error
			tokenMint, err = s.Solana.Chain.CreateSPLTokenMint(ctx, s.SolanaRelayer, SPLTokenDecimals)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Derive ICS27 Account PDA", func() {
			ics27AccountPDA, _ = gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosUser.FormattedAddress(), []byte{})
		}))

		s.Require().True(s.Run("Create and Fund Token Account", func() {
			var err error
			sourceTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, ics27AccountPDA)
			s.Require().NoError(err)
			err = s.Solana.Chain.MintTokensTo(ctx, s.SolanaRelayer, tokenMint, sourceTokenAccount, tokenAmount)
			s.Require().NoError(err)
		}))
	}))

	var cosmosGMPTxHash []byte
	packetTimeout := uint64(time.Now().Add(35 * time.Second).Unix())

	s.Require().True(s.Run("Send GMP call from Cosmos with short timeout", func() {
		timeout := packetTimeout

		recipientWallet, err := s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		destTokenAccount, err := s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, recipientWallet.PublicKey())
		s.Require().NoError(err)

		splTransferInstruction := token.NewTransferInstruction(
			tokenAmount,
			sourceTokenAccount,
			destTokenAccount,
			ics27AccountPDA,
			[]solanago.PublicKey{},
		).Build()

		instructionData, err := splTransferInstruction.Data()
		s.Require().NoError(err)

		solanaInstruction := &solanatypes.GMPSolanaPayload{
			Data: instructionData,
			Accounts: []*solanatypes.SolanaAccountMeta{
				{Pubkey: sourceTokenAccount.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: destTokenAccount.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: false},
			},
		}

		payload, err := proto.Marshal(solanaInstruction)
		s.Require().NoError(err)

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosUser.FormattedAddress(),
			Receiver:         token.ProgramID.String(),
			Salt:             []byte{},
			Payload:          payload,
			TimeoutTimestamp: timeout,
			Memo:             "timeout latency test from Cosmos",
			Encoding:         encodingType.String(),
		})
		s.Require().NoError(err)

		cosmosGMPTxHashBytes, err := hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		cosmosGMPTxHash = cosmosGMPTxHashBytes
		s.T().Logf("Cosmos packet sent: %s", resp.TxHash)
	}))

	s.Require().True(s.Run("Wait for finalized block past timeout", func() {
		err := testutil.WaitForCondition(60*time.Second, 1*time.Second, func() (bool, error) {
			slot, err := s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
			if err != nil {
				return false, err
			}
			blockTime, err := s.Solana.Chain.RPCClient.GetBlockTime(ctx, slot)
			if err != nil {
				return false, err
			}
			return uint64(*blockTime) >= packetTimeout, nil
		})
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Relay timeout and assert latency", func() {
		start := time.Now()

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:     testvalues.SolanaChainID,
			DstChain:     simd.Config().ChainID,
			TimeoutTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId:  SolanaClientID,
			DstClientId:  CosmosClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		relayElapsed := time.Since(start)
		s.T().Logf("RelayByTx completed in %s", relayElapsed)
		// Locally passes at ~3s; padded to 7s for CI.
		s.Require().Less(relayElapsed, 7*time.Second, "RelayByTx should complete within finalization window")

		txResp, err := s.BroadcastSdkTxBodyNoWait(ctx, simd, cosmosUser, 500_000, resp.Tx)
		s.Require().NoError(err)
		s.Require().Equal(uint32(0), txResp.Code)
	}))
}

// Test_GMPTimeoutFromSolana tests GMP packet timeout when sent from Solana to Cosmos
//
// HIGH-LEVEL FLOW:
//
// 1. Setup Phase
//   - Deploy ICS27 GMP program on Solana
//   - Fund ICS27 account on Cosmos with test tokens
//
// 2. Send Packet (Solana → Cosmos)
//   - Solana sends IBC packet via send_call instruction
//   - Packet type: GMP call packet (ICS27 general message passing)
//   - Payload: Protobuf-encoded MsgSend (bank transfer from ICS27 account to test user)
//   - Timeout: 35 seconds from current time
//   - Packet commitment created on Solana (stores hash of packet data)
//
// 3. Retrieve Recv Transaction Before Timeout
//   - Retrieve the recv relay transaction from proof API (before timeout expires)
//   - This transaction will be used later to verify it fails after timeout
//
// 4. Timeout Expiry
//   - Packet expires on Cosmos (not processed in time)
//   - Wait 40 seconds for timeout to occur
//
// 5. Update Light Client
//
//   - Update Tendermint client on Solana
//
//   - Ensures client can verify latest Cosmos state for timeout proof
//
//     6. Relay Timeout (Cosmos → Solana)
//     NOTE: This test provides solanaPacketTxHash via TimeoutTxIds for explicit control.
//     In production, the relayer discovers timeouts automatically by:
//
//   - Monitoring SendPacket events and tracking pending packets
//
//   - Detecting when current_time > timeout_timestamp for unacknowledged packets
//
//   - Initiating timeout relay for expired packets
//
//     Relayer timeout workflow:
//     a) Query original send transaction on Solana using solanaPacketTxHash
//     b) Extract SendPacket event (sequence, payload, timeout, client IDs)
//     c) Query Cosmos state tree at path: ["ibc", destClient + 0x02 + sequence]
//     → Receipt does not exist in state (packet was never received)
//     (Path format: destination client ID + 0x02 (receipt discriminator) + sequence as big-endian u64)
//     (Note: 0x01=commitment, 0x02=receipt, 0x03=acknowledgement)
//     d) Determine timeout condition: current_time > timeout_timestamp AND receipt not found
//     e) Build absence proof (non-membership proof) showing packet receipt doesn't exist at trusted height
//     f) Return chunked Solana transactions:
//
//   - Membership proof chunks (proving Cosmos state tree at height H)
//
//   - Final on_timeout_packet instruction with assembled absence proof
//
//     7. Process Timeout on Solana
//     On-chain verification via on_timeout_packet instruction:
//     a) ICS26 router verifies timeout proof against Tendermint consensus state
//     b) Validate proof_height <= latest_client_height
//     c) Confirm current_time > packet.timeout_timestamp
//     d) Verify packet commitment exists and matches stored hash
//     e) Delete packet commitment from Solana state (cleanup)
//     f) Call ICS27 GMP app's on_timeout_packet via CPI
//     g) App performs application-specific timeout handling
//
// 8. Verify Timeout Effects
//   - Packet commitment deleted on Solana
//   - ICS27 account balance on Cosmos unchanged (MsgSend never executed)
//
// 9. Verify RecvPacket Fails After Timeout
//   - Attempt to broadcast the recv transaction that was retrieved before timeout
//   - Transaction should fail on Cosmos (packet already timed out)
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPTimeoutFromSolana() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]

	testAmount := sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(CosmosTestAmount)))
	testCosmosUser := s.CreateAndFundCosmosUserWithBalance(ctx, simd, testAmount[0].Amount.Int64())

	var computedAddress sdk.AccAddress
	s.Require().True(s.Run("Fund pre-computed ICS27 address on Cosmos", func() {
		solanaUserAddress := s.SolanaRelayer.PublicKey().String()

		res, err := e2esuite.GRPCQuery[gmptypes.QueryAccountAddressResponse](ctx, simd, &gmptypes.QueryAccountAddressRequest{
			ClientId: CosmosClientID,
			Sender:   solanaUserAddress,
			Salt:     "",
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(res.AccountAddress)

		computedAddress, err = sdk.AccAddressFromBech32(res.AccountAddress)
		s.Require().NoError(err)

		s.T().Logf("ICS27 account on Cosmos: %s", computedAddress.String())

		_, err = s.BroadcastMessages(ctx, simd, testCosmosUser, CosmosDefaultGasLimit, &banktypes.MsgSend{
			FromAddress: testCosmosUser.FormattedAddress(),
			ToAddress:   computedAddress.String(),
			Amount:      testAmount,
		})
		s.Require().NoError(err)
	}))

	var solanaPacketTxHash []byte
	var sequence uint64
	var timeoutTimestamp uint64

	s.Require().True(s.Run("Send call from Solana with short timeout", func() {
		var payload []byte
		s.Require().True(s.Run("Prepare GMP payload", func() {
			msgSend := &banktypes.MsgSend{
				FromAddress: computedAddress.String(),
				ToAddress:   testCosmosUser.FormattedAddress(),
				Amount:      testAmount,
			}

			var err error
			payload, err = gmphelpers.NewPayload_FromProto([]proto.Message{msgSend})
			s.Require().NoError(err)
			s.T().Logf("Encoded GMP payload (%d bytes)", len(payload))
		}))

		var gmpAppStatePDA, routerStatePDA, clientPDA, ibcAppPDA, lightClientStatePDA, consensusStatePDA solanago.PublicKey
		s.Require().True(s.Run("Derive required PDAs", func() {
			gmpAppStatePDA, _ = solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)
			routerStatePDA, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
			clientPDA, _ = solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))
			ibcAppPDA, _ = solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(GMPPortID))
			lightClientStatePDA, _ = solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
			consensusStatePDA = s.deriveIcs07ConsensusStatePDA(ctx, lightClientStatePDA)

			s.T().Logf("Derived PDAs: gmpAppState=%s, routerState=%s, client=%s",
				gmpAppStatePDA.String(), routerStatePDA.String(), clientPDA.String())
		}))

		var packetCommitmentPDA solanago.PublicKey
		s.Require().True(s.Run("Choose sequence and derive packet commitment PDA", func() {
			sequence = 2
			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, sequence)
			packetCommitmentPDA, _ = solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), sequenceBytes)
			s.T().Logf("Using sequence: %d", sequence)
		}))

		var sendCallInstruction solanago.Instruction
		s.Require().True(s.Run("Build send_call instruction", func() {
			solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
			s.Require().NoError(err)

			timeoutTimestamp = uint64(solanaClockTime + SolanaOriginatedTimeoutSeconds)

			s.T().Logf("Setting timeout to: %d (solana_clock=%d + %ds)", timeoutTimestamp, solanaClockTime, SolanaOriginatedTimeoutSeconds)

			sendCallInstruction, err = ics27_gmp.NewSendCallInstruction(
				ics27_gmp.Ics27GmpStateSendCallMsg{
					SourceClient:     SolanaClientID,
					Sequence:         sequence,
					TimeoutTimestamp: timeoutTimestamp,
					Receiver:         "", // Target program on Cosmos (empty for native modules)
					Salt:             []byte{},
					Payload:          payload,
					Memo:             "timeout test from Solana",
					Encoding:         encodingType.String(),
				},
				gmpAppStatePDA,
				s.SolanaRelayer.PublicKey(),
				s.SolanaRelayer.PublicKey(),
				ics26_router.ProgramID,
				routerStatePDA,
				packetCommitmentPDA,
				ibcAppPDA,
				clientPDA,
				ics07_tendermint.ProgramID,
				lightClientStatePDA,
				solanago.SysVarInstructionsPubkey,
				consensusStatePDA,
				solanago.SystemProgramID,
			)
			s.Require().NoError(err)
			s.T().Log("Built send_call instruction with short timeout")
		}))

		s.Require().True(s.Run("Broadcast transaction", func() {
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(
				s.SolanaRelayer.PublicKey(),
				sendCallInstruction,
			)
			s.Require().NoError(err)

			sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
			s.Require().NoError(err)
			s.Require().NotEmpty(sig)

			solanaPacketTxHash = []byte(sig.String())
			s.T().Logf("Send call transaction (will timeout): %s", sig)
		}))
	}))

	// Retrieve the recv relay tx before timeout - we'll try to use it after timeout
	var recvRelayTxBodyBz []byte
	s.Require().True(s.Run("Retrieve recv relay tx before timeout", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    simd.Config().ChainID,
			SourceTxIds: [][]byte{solanaPacketTxHash},
			SrcClientId: SolanaClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)
		recvRelayTxBodyBz = resp.Tx
		s.T().Log("Retrieved recv relay transaction before timeout")
	}))

	err := e2esuite.WaitForBlockTime(ctx, s.T(), &cosmos.Chain{Cosmos: simd}, timeoutTimestamp)
	s.Require().NoError(err)

	s.Require().True(s.Run("Relay timeout back to Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:     simd.Config().ChainID,
			DstChain:     testvalues.SolanaChainID,
			TimeoutTxIds: [][]byte{solanaPacketTxHash},
			SrcClientId:  CosmosClientID,
			DstClientId:  SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		gmpResultPDA, _ := solana.GMPCallResultPDA(ics27_gmp.ProgramID, SolanaClientID, sequence)

		sig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Timeout transaction broadcasted: %s", sig)

		s.T().Log("Timeout successfully processed on Solana")

		s.Require().True(s.Run("Verify timeout effects", func() {
			s.Require().True(s.Run("Verify packet commitment deleted on Solana", func() {
				s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, sequence)
				s.T().Logf("Packet commitment successfully deleted from Solana for sequence %d", sequence)
			}))

			s.Require().True(s.Run("Verify Cosmos account balance unchanged", func() {
				// The MsgSend from ICS27 account never executed, so balance should remain the same
				balanceResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
					Address: computedAddress.String(),
					Denom:   simd.Config().Denom,
				})
				s.Require().NoError(err)
				s.Require().Equal(testAmount[0].Amount.String(), balanceResp.Balance.Amount.String(),
					"ICS27 account balance should remain unchanged after timeout")
				s.T().Logf("ICS27 account balance: %s (unchanged)", balanceResp.Balance.Amount.String())
			}))

			s.Require().True(s.Run("Verify recvPacket fails on Cosmos after timeout", func() {
				_, err := s.BroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 2_000_000, recvRelayTxBodyBz)
				s.Require().Error(err)
			}))

			s.Require().True(s.Run("Verify GMP call result PDA shows timed out", func() {
				accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, gmpResultPDA, &rpc.GetAccountInfoOpts{
					Commitment: rpc.CommitmentConfirmed,
				})
				s.Require().NoError(err)
				s.Require().NotNil(accountInfo.Value, "GMP result account should exist after timeout")
				s.Require().Equal(ics27_gmp.ProgramID, accountInfo.Value.Owner, "Account should be owned by ics27_gmp program")

				data := accountInfo.Value.Data.GetBinary()
				result, err := solana.DecodeGMPCallResultAccount(data)
				s.Require().NoError(err, "Failed to decode GMP call result account")

				// Validate all fields
				s.Require().Equal(uint8(0), result.Version, "Version should be 0 (V1)")
				s.Require().Equal(s.SolanaRelayer.PublicKey(), result.Sender, "Sender should match")
				s.Require().Equal(sequence, result.Sequence, "Sequence should match")
				s.Require().Equal(SolanaClientID, result.SourceClient, "Source client should match")
				s.Require().Equal(CosmosClientID, result.DestClient, "Dest client should match")
				s.Require().Equal(solana.CallResultStatusTimeout, result.Status, "Status should be Timeout")
				s.Require().Equal([32]byte{}, result.AckCommitment, "AckCommitment should be zeros for timeout")
				s.Require().True(result.ResultTimestamp > 0, "Result timestamp should be set")
				s.Require().True(result.Bump > 0, "Bump should be non-zero")

				s.T().Logf("GMP result account validated: sender=%s, sequence=%d, status=Timeout",
					result.Sender, result.Sequence)
			}))
		}))
	}))
}

// Test_GMPTimeoutFromCosmos tests GMP packet timeout when sent from Cosmos to Solana
//
// HIGH-LEVEL FLOW:
//
// 1. Setup Phase
//   - Create SPL token infrastructure on Solana
//   - Set up mint, ICS27 account PDA, and token accounts
//   - Fund source token account with 1M tokens (6 decimals)
//
// 2. Send Packet (Cosmos → Solana)
//   - Cosmos sends IBC packet via MsgSendCall
//   - Packet type: GMP call packet (ICS27 general message passing)
//   - Payload: Protobuf-encoded GMPSolanaPayload (SPL token transfer: 1M tokens from ICS27 account to recipient)
//   - Timeout: 35 seconds from current time
//   - Packet commitment created on Cosmos (stores hash of packet data)
//
// 3. Retrieve Recv Transactions Before Timeout
//   - Retrieve the recv relay transactions from proof API (before timeout expires)
//   - These transactions will be used later to verify they fail after timeout
//
// 4. Timeout Expiry
//
//   - Packet expires on Solana (not relayed/processed in time)
//
//   - Wait 40 seconds for timeout to occur
//
//     5. Relay Timeout (Solana → Cosmos)
//     NOTE: This test provides cosmosGMPTxHash via TimeoutTxIds for explicit control.
//     In production, the relayer discovers timeouts automatically by:
//
//   - Monitoring SendPacket events and tracking pending packets
//
//   - Detecting when current_time > timeout_timestamp for unacknowledged packets
//
//   - Initiating timeout relay for expired packets
//
//     Relayer timeout workflow:
//     a) Query original send transaction on Cosmos using cosmosGMPTxHash
//     b) Extract SendPacket event (sequence, payload, timeout, client IDs)
//     c) Query Solana state tree at path: ["ibc", destClient + 0x02 + sequence]
//     → Receipt does not exist in state (packet was never received)
//     (Path format: destination client ID + 0x02 (receipt discriminator) + sequence as big-endian u64)
//     (Note: 0x01=commitment, 0x02=receipt, 0x03=acknowledgement)
//     d) Determine timeout condition: current_time > timeout_timestamp AND receipt not found
//     e) Build absence proof (non-membership proof) showing packet receipt doesn't exist at trusted height
//     f) Update Wasm light client if needed for proof verification
//     g) Return single Cosmos transaction (MsgTimeout) with absence proof
//
//     6. Process Timeout on Cosmos
//     On-chain verification via MsgTimeout transaction:
//     a) x/gmp module receives MsgTimeout transaction
//     b) Wasm light client verifies absence proof against Solana consensus state
//     c) Validate proof_height <= latest_client_height
//     d) Confirm current_time > packet.timeout_timestamp
//     e) Verify packet commitment exists and matches hash(packet_data)
//     f) Delete packet commitment from Cosmos state (cleanup)
//     g) Call application's OnTimeoutPacket callback
//     h) App performs application-specific timeout handling (refunds, state reversion)
//     i) Emit TimeoutPacket event
//
// 7. Verify Timeout Effects
//   - Packet commitment deleted on Cosmos
//   - Source SPL token account still has all 1M tokens (transfer never executed)
//   - Destination SPL token account has 0 tokens (never received)
//
// 8. Verify RecvPacket Fails After Timeout
//   - Attempt to broadcast the recv transactions that were retrieved before timeout
//   - Transactions should fail on Solana (packet already timed out)
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPTimeoutFromCosmos() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUser := s.Cosmos.Users[0]

	// Create SPL token and accounts
	var tokenMint solanago.PublicKey
	var ics27AccountPDA solanago.PublicKey
	var sourceTokenAccount solanago.PublicKey

	const tokenAmount = uint64(1_000_000) // 1 token with 6 decimals

	s.Require().True(s.Run("Setup SPL Token Infrastructure", func() {
		s.Require().True(s.Run("Create Test SPL Token Mint", func() {
			var err error
			tokenMint, err = s.Solana.Chain.CreateSPLTokenMint(ctx, s.SolanaRelayer, SPLTokenDecimals)
			s.Require().NoError(err)
			s.T().Logf("Created test SPL token mint: %s", tokenMint.String())
		}))

		s.Require().True(s.Run("Derive ICS27 Account PDA", func() {
			ics27AccountPDA, _ = gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosUser.FormattedAddress(), []byte{})
			s.T().Logf("ICS27 Account PDA: %s", ics27AccountPDA.String())
		}))

		s.Require().True(s.Run("Create and Fund Token Account", func() {
			var err error
			sourceTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, ics27AccountPDA)
			s.Require().NoError(err)

			err = s.Solana.Chain.MintTokensTo(ctx, s.SolanaRelayer, tokenMint, sourceTokenAccount, tokenAmount)
			s.Require().NoError(err)
			s.T().Logf("Created and funded source token account: %s", sourceTokenAccount.String())
		}))
	}))

	var cosmosGMPTxHash []byte
	var recipientWallet *solanago.Wallet
	var destTokenAccount solanago.PublicKey
	var cosmosTimeoutTimestamp uint64

	s.Require().True(s.Run("Send GMP call from Cosmos with short timeout", func() {
		cosmosTimeoutTimestamp = uint64(time.Now().Add(20 * time.Second).Unix())

		// Build SPL transfer instruction
		var err error
		recipientWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		destTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, recipientWallet.PublicKey())
		s.Require().NoError(err)

		splTransferInstruction := token.NewTransferInstruction(
			tokenAmount,
			sourceTokenAccount,
			destTokenAccount,
			ics27AccountPDA,
			[]solanago.PublicKey{},
		).Build()

		instructionData, err := splTransferInstruction.Data()
		s.Require().NoError(err)

		// Create GMPSolanaPayload protobuf
		solanaInstruction := &solanatypes.GMPSolanaPayload{
			Data: instructionData,
			Accounts: []*solanatypes.SolanaAccountMeta{
				{Pubkey: sourceTokenAccount.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: destTokenAccount.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: false},
			},
		}

		payload, err := gmphelpers.MarshalGMPSolanaPayload(solanaInstruction, encodingType.String())
		s.Require().NoError(err)

		// Send GMP call with short timeout
		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosUser.FormattedAddress(),
			Receiver:         token.ProgramID.String(),
			Salt:             []byte{},
			Payload:          payload,
			TimeoutTimestamp: cosmosTimeoutTimestamp,
			Memo:             "timeout test from Cosmos",
			Encoding:         encodingType.String(),
		})
		s.Require().NoError(err)

		cosmosGMPTxHashBytes, err := hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		cosmosGMPTxHash = cosmosGMPTxHashBytes

		s.T().Logf("Send call transaction (will timeout): %s", resp.TxHash)
	}))

	// Retrieve the recv relay txs before timeout - we'll try to use them after timeout
	var recvRelayTxs *proofapitypes.RelayByTxResponse
	s.Require().True(s.Run("Retrieve recv relay txs before timeout", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		recvRelayTxs = resp
		s.T().Log("Retrieved recv relay transactions before timeout")

		// Submit UpdateClient only (not packets) so we can verify packets fail after timeout
		var batch proofapitypes.SolanaRelayPacketBatch
		err = googleproto.Unmarshal(resp.Tx, &batch)
		s.Require().NoError(err)
		if batch.UpdateClient != nil {
			updateClientBytes, err := googleproto.Marshal(batch.UpdateClient)
			s.Require().NoError(err)
			s.Solana.Chain.SubmitChunkedUpdateClient(ctx, s.T(), s.Require(), &proofapitypes.UpdateClientResponse{
				Tx: updateClientBytes,
			}, s.SolanaRelayer)
		}
	}))

	err := e2esuite.WaitForBlockTime(ctx, s.T(), &s.Solana.Chain, cosmosTimeoutTimestamp)
	s.Require().NoError(err)

	s.Require().True(s.Run("Relay timeout back to Cosmos", func() {
		s.Require().True(s.Run("Relay timeout transaction", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:     testvalues.SolanaChainID,
				DstChain:     simd.Config().ChainID,
				TimeoutTxIds: [][]byte{cosmosGMPTxHash},
				SrcClientId:  SolanaClientID,
				DstClientId:  CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

			txResp, err := s.BroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 500_000, resp.Tx)
			s.Require().NoError(err)
			s.T().Logf("Timeout transaction broadcasted: %s", txResp.TxHash)

			s.T().Log("Timeout successfully processed on Cosmos")
		}))

		s.Require().True(s.Run("Verify timeout effects", func() {
			s.Require().True(s.Run("Verify packet commitment deleted on Cosmos", func() {
				// First GMP packet from Cosmos should have sequence 1
				_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
					ClientId: CosmosClientID,
					Sequence: 1,
				})
				s.Require().ErrorContains(err, "packet commitment hash not found")
				s.T().Log("Packet commitment successfully deleted from Cosmos")
			}))

			s.Require().True(s.Run("Verify SPL token balances unchanged", func() {
				// Source account should still have all tokens (transfer never executed)
				sourceBalance, err := s.Solana.Chain.GetTokenBalance(ctx, sourceTokenAccount)
				s.Require().NoError(err)
				s.Require().Equal(tokenAmount, sourceBalance, "Source token account should retain all tokens after timeout")
				s.T().Logf("Source token account balance: %d (unchanged)", sourceBalance)

				// Destination account should have 0 tokens (never received)
				destBalance, err := s.Solana.Chain.GetTokenBalance(ctx, destTokenAccount)
				s.Require().NoError(err)
				s.Require().Equal(uint64(0), destBalance, "Destination token account should have 0 tokens after timeout")
				s.T().Logf("Destination token account balance: %d (no transfer occurred)", destBalance)
			}))

			s.Require().True(s.Run("Verify recvPacket fails on Solana after timeout", func() {
				_, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), recvRelayTxs, s.SolanaRelayer)
				s.Require().Error(err)
			}))
		}))
	}))
}

// Test_GMPFailedExecutionFromCosmos verifies that CPI errors cause immediate transaction failure
//
// Test Scenario:
// 1. Cosmos chain sends a GMP packet to Solana requesting an SPL token transfer
// 2. The ICS27 account is funded with insufficient tokens (5 tokens)
// 3. The GMP payload requests a transfer of 10 tokens (more than available)
// 4. When the ICS27 GMP program attempts the transfer via CPI, it fails
// 5. The entire transaction aborts - no error acknowledgment is sent back to Cosmos
//
// Solana Architectural Constraint:
// Unlike IBC/EVM where execution errors can be caught and returned as error acknowledgments,
// Solana CPIs (Cross-Program Invocations) fail atomically. When invoke() or invoke_signed()
// fails, the entire transaction aborts immediately - by design to maintain atomicity.
//
// Technical Details:
// CPI errors cannot be handled in Solana programs - when invoke() or invoke_signed()
// fails, the entire transaction aborts immediately. This is by design to maintain
// transaction atomicity.
//
// Runtime Implementation:
// The error propagation happens at the VM/runtime level. When a child program returns
// an error, it propagates immediately via the ? operator in cpi_common():
// https://github.com/anza-xyz/agave/blob/6ba8c59466d18ef480680732c89fa076b15843f5/program-runtime/src/cpi.rs#L843
//
// Error propagation flow in process_instruction():
// https://github.com/anza-xyz/agave/blob/6ba8c59466d18ef480680732c89fa076b15843f5/program-runtime/src/invoke_context.rs#L488-L498
//
// Unit Test Proof:
// There's a test that proves CPI errors cause transaction abort even when the Result
// is ignored.
//
// Test setup (expects transaction to fail with Custom(42)):
// https://github.com/anza-xyz/agave/blob/6ba8c59466d18ef480680732c89fa076b15843f5/programs/sbf/tests/programs.rs#L1043-L1049
//
// Parent program IGNORES the invoke() result with "let _ = invoke(...)":
// https://github.com/anza-xyz/agave/blob/6ba8c59466d18ef480680732c89fa076b15843f5/programs/sbf/rust/invoke/src/lib.rs#L604
//
// Child program returns error Custom(42):
// https://github.com/anza-xyz/agave/blob/6ba8c59466d18ef480680732c89fa076b15843f5/programs/sbf/rust/invoked/src/lib.rs#L119
//
// The test confirms that even though the parent ignores the Result, the transaction
// aborts with the child's error. The parent program never gets to execute any code
// after the failed invoke() call - the abort happens at the runtime/VM level.
//
// This is fundamentally different from EVM's try/catch mechanism or Cosmos SDK's error returns.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPFailedExecutionFromCosmos() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUser := s.Cosmos.Users[0]

	// Setup SPL token infrastructure with insufficient balance
	var tokenMint solanago.PublicKey
	var ics27AccountPDA solanago.PublicKey
	var sourceTokenAccount solanago.PublicKey
	var destTokenAccount solanago.PublicKey
	var recipientWallet *solanago.Wallet

	const (
		insufficientAmount = uint64(5_000_000)  // 5 tokens with 6 decimals
		excessiveAmount    = uint64(10_000_000) // 10 tokens - MORE than available!
	)

	s.Require().True(s.Run("Setup SPL Token Infrastructure", func() {
		s.Require().True(s.Run("Create Test SPL Token Mint", func() {
			var err error
			tokenMint, err = s.Solana.Chain.CreateSPLTokenMint(ctx, s.SolanaRelayer, SPLTokenDecimals)
			s.Require().NoError(err)
			s.T().Logf("Created test SPL token mint: %s (decimals: %d)", tokenMint.String(), SPLTokenDecimals)
		}))

		s.Require().True(s.Run("Derive ICS27 Account PDA", func() {
			ics27AccountPDA, _ = gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosUser.FormattedAddress(), []byte{})
			s.T().Logf("ICS27 Account PDA for Cosmos user: %s", ics27AccountPDA.String())
		}))

		s.Require().True(s.Run("Create Token Accounts", func() {
			var err error

			// Create source token account (owned by ICS27 PDA)
			sourceTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, ics27AccountPDA)
			s.Require().NoError(err)
			s.T().Logf("Created source token account (owned by ICS27 PDA): %s", sourceTokenAccount.String())

			// Create recipient wallet and destination token account
			recipientWallet, err = s.Solana.Chain.CreateAndFundWallet()
			s.Require().NoError(err)

			destTokenAccount, err = s.Solana.Chain.CreateTokenAccount(ctx, s.SolanaRelayer, tokenMint, recipientWallet.PublicKey())
			s.Require().NoError(err)
			s.T().Logf("Created destination token account (owned by recipient): %s", destTokenAccount.String())
		}))

		s.Require().True(s.Run("Mint Insufficient Tokens to ICS27 PDA", func() {
			// CRITICAL: Mint ONLY 5 tokens (we'll try to transfer 10 later)
			err := s.Solana.Chain.MintTokensTo(ctx, s.SolanaRelayer, tokenMint, sourceTokenAccount, insufficientAmount)
			s.Require().NoError(err)

			balance, err := s.Solana.Chain.GetTokenBalance(ctx, sourceTokenAccount)
			s.Require().NoError(err)
			s.Require().Equal(insufficientAmount, balance)
			s.T().Logf("Minted %d tokens to ICS27 PDA (intentionally insufficient)", insufficientAmount)
		}))
	}))

	// Record initial state
	var initialSourceBalance, initialDestBalance uint64

	s.Require().True(s.Run("Record Initial State", func() {
		var err error

		initialSourceBalance, err = s.Solana.Chain.GetTokenBalance(ctx, sourceTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(insufficientAmount, initialSourceBalance)

		initialDestBalance, err = s.Solana.Chain.GetTokenBalance(ctx, destTokenAccount)
		s.Require().NoError(err)
		s.Require().Equal(uint64(0), initialDestBalance)

		s.T().Logf("Initial state - Source: %d tokens, Dest: %d tokens",
			initialSourceBalance, initialDestBalance)
	}))

	// Send GMP call that will fail (requesting more tokens than available)
	var cosmosGMPTxHash []byte

	s.Require().True(s.Run("Send GMP SPL Transfer (Will Fail)", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		// Build SPL transfer instruction requesting 10 tokens (but only 5 available!)
		splTransferInstruction := token.NewTransferInstruction(
			excessiveAmount, // Request 10 tokens
			sourceTokenAccount,
			destTokenAccount,
			ics27AccountPDA, // Authority (will be signed by GMP via invoke_signed)
			[]solanago.PublicKey{},
		).Build()

		instructionData, err := splTransferInstruction.Data()
		s.Require().NoError(err)

		// Create GMPSolanaPayload protobuf
		solanaInstruction := &solanatypes.GMPSolanaPayload{
			Data: instructionData,
			Accounts: []*solanatypes.SolanaAccountMeta{
				{Pubkey: sourceTokenAccount.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: destTokenAccount.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: false},
			},
		}

		payload, err := gmphelpers.MarshalGMPSolanaPayload(solanaInstruction, encodingType.String())
		s.Require().NoError(err)

		// Send GMP call
		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosUser.FormattedAddress(),
			Receiver:         token.ProgramID.String(),
			Salt:             []byte{},
			Payload:          payload,
			TimeoutTimestamp: timeout,
			Memo:             "SPL transfer that will fail (insufficient balance)",
			Encoding:         encodingType.String(),
		})
		s.Require().NoError(err)

		cosmosGMPTxHashBytes, err := hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		cosmosGMPTxHash = cosmosGMPTxHashBytes

		s.T().Logf("GMP packet sent (will fail on execution): %s", resp.TxHash)
		s.T().Logf("Requested transfer: %d tokens (available: %d tokens)", excessiveAmount, insufficientAmount)
	}))

	// Relay packet to Solana and execute (will fail due to CPI error)
	s.Require().True(s.Run("Relay and Execute on Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		// Transaction will fail due to CPI error (insufficient balance for SPL token transfer)
		// Expected error: SPL Token program InstructionError with Custom error code 1 (InsufficientFunds)
		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().Error(err)
		s.T().Logf("Received error: %v", err)
		// Expected Solana error format: map[InstructionError:[%!s(float64=2) map[Custom:%!s(float64=1)]]]
		// where instruction index 2 failed with Custom error code 1 (InsufficientFunds)
		s.Require().Contains(err.Error(), "map[InstructionError:[%!s(float64=2) map[Custom:%!s(float64=1)]]]")
	}))
}

// Test_GMPFailedExecutionFromSolana verifies that Cosmos can handle execution errors gracefully
//
// Test Scenario:
// 1. Solana sends a GMP packet to Cosmos requesting a bank transfer
// 2. The ICS27 account on Cosmos has insufficient balance (0 tokens)
// 3. The GMP payload requests a transfer of tokens (but none available)
// 4. Cosmos receives the packet and attempts execution
// 5. Cosmos catches the execution error and returns an error acknowledgment
// 6. The error ack is relayed back to Solana successfully
//
// Cosmos Architectural Behavior:
// Unlike Solana where CPI errors cause immediate transaction abort, Cosmos SDK applications
// can catch execution errors and return them as error acknowledgments. This allows the IBC
// protocol to complete successfully even when the application-level execution fails.
//
// When a Cosmos SDK message fails (e.g., insufficient balance, invalid recipient), the error
// is caught by the GMP keeper and encoded into an error acknowledgment that gets returned
// to the sending chain. The IBC packet lifecycle completes normally - the packet is received,
// an ack is written, and the sending chain can process the error ack.
//
// This is fundamentally different from Solana's atomic CPI behavior where errors propagate
// through the runtime and abort the entire transaction before any acknowledgment can be written.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPFailedExecutionFromSolana() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]

	// Create a test Cosmos user to receive the transfer (if it succeeds)
	testAmount := sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(CosmosTestAmount)))
	testCosmosUser := s.CreateAndFundCosmosUserWithBalance(ctx, simd, testAmount[0].Amount.Int64())

	var computedAddress sdk.AccAddress
	s.Require().True(s.Run("Compute ICS27 address on Cosmos (will have zero balance)", func() {
		solanaUserAddress := s.SolanaRelayer.PublicKey().String()

		res, err := e2esuite.GRPCQuery[gmptypes.QueryAccountAddressResponse](ctx, simd, &gmptypes.QueryAccountAddressRequest{
			ClientId: CosmosClientID,
			Sender:   solanaUserAddress,
			Salt:     "",
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(res.AccountAddress)

		computedAddress, err = sdk.AccAddressFromBech32(res.AccountAddress)
		s.Require().NoError(err)

		s.T().Logf("ICS27 account on Cosmos: %s (will have zero balance - execution will fail)", computedAddress.String())
	}))

	// Verify the ICS27 account has zero balance (or doesn't exist yet)
	s.Require().True(s.Run("Verify ICS27 account has insufficient balance", func() {
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: computedAddress.String(),
			Denom:   simd.Config().Denom,
		})
		s.Require().NoError(err)

		balance := int64(0)
		if resp.Balance != nil {
			balance = resp.Balance.Amount.Int64()
		}
		s.T().Logf("ICS27 account balance: %d (insufficient for transfer)", balance)
	}))

	var solanaPacketTxHash string
	var sequence uint64
	s.Require().True(s.Run("Send call from Solana (will fail on Cosmos execution)", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		var payload []byte
		s.Require().True(s.Run("Prepare GMP payload (bank transfer that will fail)", func() {
			// Try to send testAmount from ICS27 account (which has zero balance)
			msgSend := &banktypes.MsgSend{
				FromAddress: computedAddress.String(),
				ToAddress:   testCosmosUser.FormattedAddress(),
				Amount:      testAmount,
			}

			var err error
			payload, err = gmphelpers.NewPayload_FromProto([]proto.Message{msgSend})
			s.Require().NoError(err)
			s.T().Logf("Encoded GMP payload (%d bytes) - will fail due to insufficient balance", len(payload))
		}))

		var gmpAppStatePDA, routerStatePDA, clientPDA, ibcAppPDA, lightClientStatePDA, consensusStatePDA solanago.PublicKey
		s.Require().True(s.Run("Derive required PDAs", func() {
			gmpAppStatePDA, _ = solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)
			routerStatePDA, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
			clientPDA, _ = solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))
			ibcAppPDA, _ = solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(GMPPortID))
			lightClientStatePDA, _ = solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
			consensusStatePDA = s.deriveIcs07ConsensusStatePDA(ctx, lightClientStatePDA)
		}))

		var packetCommitmentPDA solanago.PublicKey
		s.Require().True(s.Run("Choose sequence and derive packet commitment PDA", func() {
			sequence = 3
			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, sequence)
			packetCommitmentPDA, _ = solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), sequenceBytes)
			s.T().Logf("Using sequence: %d", sequence)
		}))

		var sendCallInstruction solanago.Instruction
		s.Require().True(s.Run("Build send_call instruction", func() {
			var err error
			sendCallInstruction, err = ics27_gmp.NewSendCallInstruction(
				ics27_gmp.Ics27GmpStateSendCallMsg{
					SourceClient:     SolanaClientID,
					Sequence:         sequence,
					TimeoutTimestamp: timeout,
					Receiver:         "", // Target program on Cosmos (empty for native modules)
					Salt:             []byte{},
					Payload:          payload,
					Memo:             "send from Solana to Cosmos (will fail on execution)",
					Encoding:         encodingType.String(),
				},
				gmpAppStatePDA,
				s.SolanaRelayer.PublicKey(),
				s.SolanaRelayer.PublicKey(),
				ics26_router.ProgramID,
				routerStatePDA,
				packetCommitmentPDA,
				ibcAppPDA,
				clientPDA,
				ics07_tendermint.ProgramID,
				lightClientStatePDA,
				solanago.SysVarInstructionsPubkey,
				consensusStatePDA,
				solanago.SystemProgramID,
			)
			s.Require().NoError(err)
		}))

		s.Require().True(s.Run("Broadcast transaction", func() {
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(
				s.SolanaRelayer.PublicKey(),
				sendCallInstruction,
			)
			s.Require().NoError(err)

			sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
			s.Require().NoError(err)
			s.Require().NotEmpty(sig)

			solanaPacketTxHash = sig.String()
			s.T().Logf("Send call transaction: %s", solanaPacketTxHash)
		}))
	}))

	// Relay packet to Cosmos and execute (will return error ack)
	var cosmosRecvTxHash string
	s.Require().True(s.Run("Receive packet in Cosmos (execution will fail gracefully)", func() {
		var recvRelayTx []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			txHashBytes := []byte(solanaPacketTxHash)

			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{txHashBytes},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			recvRelayTx = resp.Tx
		}))

		s.Require().True(s.Run("Submit relay tx to Cosmos", func() {
			receipt := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 2_000_000, recvRelayTx)
			s.T().Logf("Recv packet tx result: code=%d, log=%s, gas=%d", receipt.Code, receipt.RawLog, receipt.GasUsed)

			// The IBC packet should be received successfully (code=0)
			// even though the application-level execution failed
			s.Require().Equal(uint32(0), receipt.Code, "Recv packet should succeed (IBC layer)")
			s.Require().NotEmpty(receipt.TxHash)

			cosmosRecvTxHash = receipt.TxHash
			s.T().Logf("Packet received on Cosmos, execution failed, error ack written: %s", cosmosRecvTxHash)
		}))
	}))

	// Relay error acknowledgment back to Solana
	s.Require().True(s.Run("Relay error acknowledgment to Solana", func() {
		cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		sig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Error acknowledgment successfully relayed to Solana: %s", sig)
	}))

	// Verify the packet commitment was deleted (ack processed)
	s.Require().True(s.Run("Verify packet commitment deleted on Solana", func() {
		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, sequence)
		s.T().Logf("Verified packet commitment deleted for sequence %d", sequence)
	}))
}

// Test_GMPCPISecurity verifies that all GMP IBC callbacks properly validate
// their callers through TWO security layers:
// 1. Direct call protection - Validates router_program account parameter
// 2. CPI protection - Validates calling instruction's program_id via instructions sysvar
//
// Callbacks Tested:
// 1. on_recv_packet - Should reject both unauthorized direct calls and CPIs
// 2. on_acknowledgement_packet - Should reject both unauthorized direct calls and CPIs
// 3. on_timeout_packet - Should reject both unauthorized direct calls and CPIs
//
// Attack Pattern 1 (Direct Call):
// 1. Build GMP instruction with test_cpi_proxy as router_program
// 2. Call it directly without CPI
// 3. GMP should check router_program account and reject
//
// Attack Pattern 2 (Unauthorized CPI):
// 1. E2E test builds a legitimate GMP instruction
// 2. Test wraps it in a proxy_cpi call from test_cpi_proxy
// 3. test_cpi_proxy forwards the CPI to GMP
// 4. GMP should check instructions sysvar and reject the call
//
// Security Property:
// Target programs MUST validate BOTH:
// 1. The router_program account parameter (basic account validation)
// 2. The calling instruction's program_id via instructions sysvar (CPI validation)
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPCPISecurity() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	testCpiProxyProgramID := s.TestCpiProxyProgramID
	s.T().Logf("Using test CPI proxy program: %s", testCpiProxyProgramID)

	// Helper function to create proxy CPI instruction from any GMP instruction
	createProxyInstruction := func(gmpInstruction solanago.Instruction) (*solanago.GenericInstruction, error) {
		instructionData, err := gmpInstruction.Data()
		if err != nil {
			return nil, err
		}

		accountMetas := make([]test_cpi_proxy.TestCpiProxyCpiAccountMeta, len(gmpInstruction.Accounts()))
		for i, acc := range gmpInstruction.Accounts() {
			accountMetas[i] = test_cpi_proxy.TestCpiProxyCpiAccountMeta{
				IsSigner:   acc.IsSigner,
				IsWritable: acc.IsWritable,
			}
		}

		proxyIx, err := test_cpi_proxy.NewProxyCpiInstruction(
			instructionData,
			accountMetas,
			ics27_gmp.ProgramID,
			s.SolanaRelayer.PublicKey(),
		)
		if err != nil {
			return nil, err
		}

		genericIx := proxyIx.(*solanago.GenericInstruction)
		for _, acc := range gmpInstruction.Accounts() {
			genericIx.AccountValues = append(genericIx.AccountValues, acc)
		}

		return genericIx, nil
	}

	// ========================================================================
	// Test 1A: on_recv_packet - Direct Call Attack
	// ========================================================================
	s.Require().True(s.Run("Test on_recv_packet - Should Reject Direct Call", func() {
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		mockPacketData := gmptypes.GMPPacketData{
			Sender:   "cosmos1test",
			Receiver: s.SolanaRelayer.PublicKey().String(),
			Salt:     []byte{},
			Payload:  []byte("test payload"),
			Memo:     "",
		}

		packetDataBytes, err := proto.Marshal(&mockPacketData)
		s.Require().NoError(err)

		mockMsg := ics27_gmp.SolanaIbcTypesAppMsgsOnRecvPacketMsg{
			SourceClient: "cosmos-1",
			DestClient:   SolanaClientID,
			Sequence:     1,
			Payload: ics27_gmp.SolanaIbcTypesAppMsgsPayload{
				SourcePort: GMPPortID,
				DestPort:   GMPPortID,
				Version:    testvalues.Ics27Version,
				Encoding:   encodingType.String(),
				Value:      packetDataBytes,
			},
			Relayer: s.SolanaRelayer.PublicKey(),
		}

		// Build instruction and call it directly (not via CPI)
		// This tests that validate_cpi_caller checks the instructions sysvar to detect direct calls
		dummyTargetProgram := solanago.MustPublicKeyFromBase58(DummyTargetProgramID)
		gmpIx, err := ics27_gmp.NewOnRecvPacketInstruction(
			mockMsg,
			gmpAppStatePDA,
			solanago.SysVarInstructionsPubkey,
			s.SolanaRelayer.PublicKey(),
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		// Derive GMP account PDA for remaining accounts
		gmpAcctPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, "cosmos1test", []byte{})

		// Add remaining accounts manually: gmp_account_pda and target_program
		if ix, ok := gmpIx.(*solanago.GenericInstruction); ok {
			ix.AccountValues = append(ix.AccountValues,
				solanago.Meta(gmpAcctPDA).WRITE(), // [0] gmp_account_pda (writable for CPI signer)
				solanago.Meta(dummyTargetProgram), // [1] target_program (readonly)
			)
		}

		s.T().Log("Attempting direct call to on_recv_packet (bypassing router)...")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), gmpIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.SolanaRelayer)

		// Should FAIL - validate_cpi_caller detects direct call via instructions sysvar
		s.Require().Error(err, "on_recv_packet should reject direct call")
		s.T().Logf("✓ on_recv_packet rejected direct call (tx: %s)", sig)
		s.T().Logf("  Error: %v", err)

		// Verify it failed with DirectCallNotAllowed error
		s.Require().Contains(err.Error(), "12020",
			"Should fail with error code 12020 (DirectCallNotAllowed)")

		s.T().Log("✓ on_recv_packet SECURE - detects direct calls via instructions sysvar")
	}))

	// ========================================================================
	// Test 1B: on_recv_packet - CPI Attack
	// ========================================================================
	s.Require().True(s.Run("Test on_recv_packet - Should Reject Unauthorized CPI", func() {
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		mockPacketData := gmptypes.GMPPacketData{
			Sender:   "cosmos1test",
			Receiver: s.SolanaRelayer.PublicKey().String(),
			Salt:     []byte{},
			Payload:  []byte("test payload"),
			Memo:     "",
		}

		packetDataBytes, err := proto.Marshal(&mockPacketData)
		s.Require().NoError(err)

		mockMsg := ics27_gmp.SolanaIbcTypesAppMsgsOnRecvPacketMsg{
			SourceClient: "cosmos-1",
			DestClient:   SolanaClientID,
			Sequence:     1,
			Payload: ics27_gmp.SolanaIbcTypesAppMsgsPayload{
				SourcePort: GMPPortID,
				DestPort:   GMPPortID,
				Version:    testvalues.Ics27Version,
				Encoding:   encodingType.String(),
				Value:      packetDataBytes,
			},
			Relayer: s.SolanaRelayer.PublicKey(),
		}

		// Build instruction
		dummyTargetProgram := solanago.MustPublicKeyFromBase58(DummyTargetProgramID)
		gmpIx, err := ics27_gmp.NewOnRecvPacketInstruction(
			mockMsg,
			gmpAppStatePDA,
			solanago.SysVarInstructionsPubkey,
			s.SolanaRelayer.PublicKey(),
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		// Derive GMP account PDA for remaining accounts
		gmpAcctPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, "cosmos1test", []byte{})

		// Add remaining accounts manually: gmp_account_pda and target_program
		if ix, ok := gmpIx.(*solanago.GenericInstruction); ok {
			ix.AccountValues = append(ix.AccountValues,
				solanago.Meta(gmpAcctPDA).WRITE(), // [0] gmp_account_pda (writable for CPI signer)
				solanago.Meta(dummyTargetProgram), // [1] target_program (readonly)
			)
		}

		// Wrap in proxy CPI
		proxyIx, err := createProxyInstruction(gmpIx)
		s.Require().NoError(err)

		s.T().Log("Attempting unauthorized CPI to on_recv_packet...")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proxyIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.SolanaRelayer)

		// Should FAIL - on_recv_packet has instructions sysvar validation
		s.Require().Error(err, "on_recv_packet should reject unauthorized CPI")
		s.T().Logf("✓ on_recv_packet rejected unauthorized CPI (tx: %s)", sig)
		s.T().Logf("  Error: %v", err)

		// Verify it failed with UnauthorizedRouter error
		s.Require().Contains(err.Error(), "12019",
			"Should fail with error code 12019 (UnauthorizedRouter)")

		s.T().Log("✓ on_recv_packet SECURE - validates CPI caller via instructions sysvar")
	}))

	// ========================================================================
	// Test 2A: on_ack_packet - Direct Call Attack
	// ========================================================================
	s.Require().True(s.Run("Test on_ack_packet - Should Reject Direct Call", func() {
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		mockPacketData := gmptypes.GMPPacketData{
			Sender:   "cosmos1test",
			Receiver: s.SolanaRelayer.PublicKey().String(),
			Salt:     []byte{},
			Payload:  []byte("test payload"),
			Memo:     "",
		}

		packetDataBytes, err := proto.Marshal(&mockPacketData)
		s.Require().NoError(err)

		mockMsg := ics27_gmp.SolanaIbcTypesAppMsgsOnAcknowledgementPacketMsg{
			SourceClient: "cosmos-1",
			DestClient:   SolanaClientID,
			Sequence:     1,
			Payload: ics27_gmp.SolanaIbcTypesAppMsgsPayload{
				SourcePort: GMPPortID,
				DestPort:   GMPPortID,
				Version:    testvalues.Ics27Version,
				Encoding:   encodingType.String(),
				Value:      packetDataBytes,
			},
			Acknowledgement: []byte("test ack"),
			Relayer:         s.SolanaRelayer.PublicKey(),
		}

		// Derive result_account PDA
		resultAccountPDA, _ := solana.GMPCallResultPDA(ics27_gmp.ProgramID, mockMsg.SourceClient, mockMsg.Sequence)

		// Build instruction, but call it directly (not via CPI)
		gmpIx, err := ics27_gmp.NewOnAcknowledgementPacketInstruction(
			mockMsg,
			gmpAppStatePDA,
			solanago.SysVarInstructionsPubkey, // instruction_sysvar
			s.SolanaRelayer.PublicKey(),       // payer
			solanago.SystemProgramID,          // system_program
			resultAccountPDA,                  // result_account (passed as remaining account by router)
		)
		s.Require().NoError(err)

		s.T().Log("Attempting direct call to on_ack_packet (bypassing router)...")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), gmpIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.SolanaRelayer)

		// Should FAIL - validate_cpi_caller detects direct call via instructions sysvar
		s.Require().Error(err, "on_ack_packet should reject direct call")
		s.T().Logf("✓ on_ack_packet rejected direct call (tx: %s)", sig)
		s.T().Logf("  Error: %v", err)

		// Verify it failed with DirectCallNotAllowed error
		s.Require().Contains(err.Error(), "12020",
			"Should fail with error code 12020 (DirectCallNotAllowed)")

		s.T().Log("✓ on_ack_packet SECURE - detects direct calls via instructions sysvar")
	}))

	// ========================================================================
	// Test 2B: on_ack_packet - CPI Attack
	// ========================================================================
	s.Require().True(s.Run("Test on_ack_packet - Check CPI Validation", func() {
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		mockPacketData := gmptypes.GMPPacketData{
			Sender:   "cosmos1test",
			Receiver: s.SolanaRelayer.PublicKey().String(),
			Salt:     []byte{},
			Payload:  []byte("test payload"),
			Memo:     "",
		}

		packetDataBytes, err := proto.Marshal(&mockPacketData)
		s.Require().NoError(err)

		mockMsg := ics27_gmp.SolanaIbcTypesAppMsgsOnAcknowledgementPacketMsg{
			SourceClient: "cosmos-1",
			DestClient:   SolanaClientID,
			Sequence:     1,
			Payload: ics27_gmp.SolanaIbcTypesAppMsgsPayload{
				SourcePort: GMPPortID,
				DestPort:   GMPPortID,
				Version:    testvalues.Ics27Version,
				Encoding:   encodingType.String(),
				Value:      packetDataBytes,
			},
			Acknowledgement: []byte("test ack"),
			Relayer:         s.SolanaRelayer.PublicKey(),
		}

		// Derive result_account PDA
		resultAccountPDA, _ := solana.GMPCallResultPDA(ics27_gmp.ProgramID, mockMsg.SourceClient, mockMsg.Sequence)

		// Build instruction
		gmpIx, err := ics27_gmp.NewOnAcknowledgementPacketInstruction(
			mockMsg,
			gmpAppStatePDA,
			solanago.SysVarInstructionsPubkey, // instruction_sysvar
			s.SolanaRelayer.PublicKey(),       // payer
			solanago.SystemProgramID,          // system_program
			resultAccountPDA,                  // result_account (passed as remaining account by router)
		)
		s.Require().NoError(err)

		// Wrap in proxy CPI
		proxyIx, err := createProxyInstruction(gmpIx)
		s.Require().NoError(err)

		s.T().Log("Attempting unauthorized CPI to on_ack_packet...")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proxyIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.SolanaRelayer)

		// Should FAIL - on_ack_packet has instructions sysvar validation
		s.Require().Error(err, "on_ack_packet should reject unauthorized CPI")
		s.T().Logf("✓ on_ack_packet rejected unauthorized CPI (tx: %s)", sig)
		s.T().Logf("  Error: %v", err)

		// Verify it failed with UnauthorizedRouter error
		s.Require().Contains(err.Error(), "12019",
			"Should fail with error code 12019 (UnauthorizedRouter)")

		s.T().Log("✓ on_ack_packet SECURE - validates CPI caller via instructions sysvar")
	}))

	// ========================================================================
	// Test 3A: on_timeout_packet - Direct Call Attack
	// ========================================================================
	s.Require().True(s.Run("Test on_timeout_packet - Should Reject Direct Call", func() {
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		mockPacketData := gmptypes.GMPPacketData{
			Sender:   "cosmos1test",
			Receiver: s.SolanaRelayer.PublicKey().String(),
			Salt:     []byte{},
			Payload:  []byte("test payload"),
			Memo:     "",
		}

		packetDataBytes, err := proto.Marshal(&mockPacketData)
		s.Require().NoError(err)

		mockMsg := ics27_gmp.SolanaIbcTypesAppMsgsOnTimeoutPacketMsg{
			SourceClient: "cosmos-1",
			DestClient:   SolanaClientID,
			Sequence:     1,
			Payload: ics27_gmp.SolanaIbcTypesAppMsgsPayload{
				SourcePort: GMPPortID,
				DestPort:   GMPPortID,
				Version:    testvalues.Ics27Version,
				Encoding:   encodingType.String(),
				Value:      packetDataBytes,
			},
			Relayer: s.SolanaRelayer.PublicKey(),
		}

		// Derive result_account PDA
		resultAccountPDA, _ := solana.GMPCallResultPDA(ics27_gmp.ProgramID, mockMsg.SourceClient, mockMsg.Sequence)

		// Build instruction, but call it directly (not via CPI)
		gmpIx, err := ics27_gmp.NewOnTimeoutPacketInstruction(
			mockMsg,
			gmpAppStatePDA,
			solanago.SysVarInstructionsPubkey, // instruction_sysvar
			s.SolanaRelayer.PublicKey(),       // payer
			solanago.SystemProgramID,          // system_program
			resultAccountPDA,                  // result_account (passed as remaining account by router)
		)
		s.Require().NoError(err)

		s.T().Log("Attempting direct call to on_timeout_packet (bypassing router)...")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), gmpIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.SolanaRelayer)

		// Should FAIL - validate_cpi_caller detects direct call via instructions sysvar
		s.Require().Error(err, "on_timeout_packet should reject direct call")
		s.T().Logf("✓ on_timeout_packet rejected direct call (tx: %s)", sig)
		s.T().Logf("  Error: %v", err)

		// Verify it failed with DirectCallNotAllowed error
		s.Require().Contains(err.Error(), "12020",
			"Should fail with error code 12020 (DirectCallNotAllowed)")

		s.T().Log("✓ on_timeout_packet SECURE - detects direct calls via instructions sysvar")
	}))

	// ========================================================================
	// Test 3B: on_timeout_packet - CPI Attack
	// ========================================================================
	s.Require().True(s.Run("Test on_timeout_packet - Check CPI Validation", func() {
		gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

		mockPacketData := gmptypes.GMPPacketData{
			Sender:   "cosmos1test",
			Receiver: s.SolanaRelayer.PublicKey().String(),
			Salt:     []byte{},
			Payload:  []byte("test payload"),
			Memo:     "",
		}

		packetDataBytes, err := proto.Marshal(&mockPacketData)
		s.Require().NoError(err)

		mockMsg := ics27_gmp.SolanaIbcTypesAppMsgsOnTimeoutPacketMsg{
			SourceClient: "cosmos-1",
			DestClient:   SolanaClientID,
			Sequence:     1,
			Payload: ics27_gmp.SolanaIbcTypesAppMsgsPayload{
				SourcePort: GMPPortID,
				DestPort:   GMPPortID,
				Version:    testvalues.Ics27Version,
				Encoding:   encodingType.String(),
				Value:      packetDataBytes,
			},
			Relayer: s.SolanaRelayer.PublicKey(),
		}

		// Derive result_account PDA
		resultAccountPDA, _ := solana.GMPCallResultPDA(ics27_gmp.ProgramID, mockMsg.SourceClient, mockMsg.Sequence)

		// Build instruction
		gmpIx, err := ics27_gmp.NewOnTimeoutPacketInstruction(
			mockMsg,
			gmpAppStatePDA,
			solanago.SysVarInstructionsPubkey, // instruction_sysvar
			s.SolanaRelayer.PublicKey(),       // payer
			solanago.SystemProgramID,          // system_program
			resultAccountPDA,                  // result_account (passed as remaining account by router)
		)
		s.Require().NoError(err)

		// Wrap in proxy CPI
		proxyIx, err := createProxyInstruction(gmpIx)
		s.Require().NoError(err)

		s.T().Log("Attempting unauthorized CPI to on_timeout_packet...")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proxyIx)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.SolanaRelayer)

		// Should FAIL - on_timeout_packet has instructions sysvar validation
		s.Require().Error(err, "on_timeout_packet should reject unauthorized CPI")
		s.T().Logf("✓ on_timeout_packet rejected unauthorized CPI (tx: %s)", sig)
		s.T().Logf("  Error: %v", err)

		// Verify it failed with UnauthorizedRouter error
		s.Require().Contains(err.Error(), "12019",
			"Should fail with error code 12019 (UnauthorizedRouter)")

		s.T().Log("✓ on_timeout_packet SECURE - validates CPI caller via instructions sysvar")
	}))
}

// Test_GMPPrefundedPDANotBlocked proves GMP account PDAs are immune to the
// pre-funding DoS attack that affects create_account-based PDAs (e.g. packet
// commitment PDA vulnerability). An adversary transfers lamports to the GMP
// PDA before the relayer submits the packet; execution still succeeds because
// GMP uses invoke_signed (signing only), not create_account.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPPrefundedPDANotBlocked() {
	ctx := context.Background()
	encodingType := types.GetEnvEncodingType()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]

	var gmpCounterProgramID solanago.PublicKey
	s.Require().True(s.Run("Initialize GMP Counter App", func() {
		gmpCounterProgramID = s.initializeGMPCounterApp(ctx)
	}))

	cosmosUser := s.Cosmos.Users[0]
	cosmosAddress := cosmosUser.FormattedAddress()
	salt := []byte{}

	// Derive the GMP account PDA that the adversary will pre-fund
	gmpAcctPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosAddress, salt)
	s.T().Logf("GMP account PDA to pre-fund: %s", gmpAcctPDA)

	// Step 1: Adversary pre-funds the GMP PDA via system transfer
	s.Require().True(s.Run("Adversary pre-funds GMP PDA", func() {
		adversaryLamports := uint64(50_000_000) // 0.05 SOL

		transferIx := system.NewTransferInstruction(
			adversaryLamports,
			s.SolanaRelayer.PublicKey(), // using relayer as adversary for convenience
			gmpAcctPDA,
		).Build()

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(), transferIx,
		)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(
			ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer,
		)
		s.Require().NoError(err)
		s.T().Logf("Adversary transfer to GMP PDA confirmed: %s", sig)

		// Verify the PDA now holds lamports
		acct, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, gmpAcctPDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(acct.Value, "GMP PDA should exist after transfer")
		s.Require().GreaterOrEqual(acct.Value.Lamports, adversaryLamports,
			"GMP PDA should hold at least the transferred lamports")
		s.T().Logf("GMP PDA balance after adversary transfer: %d lamports", acct.Value.Lamports)
	}))

	// Step 2: Send GMP counter increment from Cosmos (same as normal flow)
	var cosmosGMPTxHash []byte
	s.Require().True(s.Run("Send GMP increment from Cosmos", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

		ics27AccountPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosAddress, salt)
		counterAppStateAddress, _ := solana.TestGmpApp.CounterAppStatePDA(gmpCounterProgramID)
		userCounterAddress, _ := solana.TestGmpApp.UserCounterWithAccountSeedPDA(gmpCounterProgramID, ics27AccountPDA.Bytes())

		incrementInstructionData := []byte{}
		incrementInstructionData = append(incrementInstructionData, test_gmp_app.Instruction_Increment[:]...)
		amountBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(amountBytes, DefaultIncrementAmount)
		incrementInstructionData = append(incrementInstructionData, amountBytes...)

		solanaInstruction := &solanatypes.GMPSolanaPayload{
			Data: incrementInstructionData,
			Accounts: []*solanatypes.SolanaAccountMeta{
				{Pubkey: counterAppStateAddress.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: userCounterAddress.Bytes(), IsSigner: false, IsWritable: true},
				{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: false},
				{Pubkey: ics27AccountPDA.Bytes(), IsSigner: true, IsWritable: true},
				{Pubkey: solanago.SystemProgramID.Bytes(), IsSigner: false, IsWritable: false},
			},
			PrefundLamports: 5_000_000,
		}

		payload, err := gmphelpers.MarshalGMPSolanaPayload(solanaInstruction, encodingType.String())
		s.Require().NoError(err)

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosAddress,
			Receiver:         gmpCounterProgramID.String(),
			Salt:             salt,
			Payload:          payload,
			TimeoutTimestamp: timeout,
			Memo:             "increment counter via GMP (pre-funded PDA test)",
			Encoding:         encodingType.String(),
		})
		s.Require().NoError(err)

		cosmosGMPTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		s.T().Logf("GMP packet sent: %s", resp.TxHash)
	}))

	// Step 3: Relay and execute the packet on Solana (should succeed despite pre-funding)
	var solanaRelayTxSig solanago.Signature
	s.Require().True(s.Run("Relay GMP packet to Solana (pre-funded PDA)", func() {
		updateResp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(updateResp.Tx)

		s.Solana.Chain.SubmitChunkedUpdateClient(ctx, s.T(), s.Require(), updateResp, s.SolanaRelayer)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		solanaRelayTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("GMP execution completed on Solana despite pre-funded PDA: %s", solanaRelayTxSig)
	}))

	// Step 4: Verify the counter was incremented
	s.Require().True(s.Run("Verify counter incremented despite pre-funded PDA", func() {
		ics27AccountPDA, _ := gmpAccountPDA(ics27_gmp.ProgramID, SolanaClientID, cosmosAddress, salt)
		userCounterPDA, _ := solana.TestGmpApp.UserCounterWithAccountSeedPDA(gmpCounterProgramID, ics27AccountPDA.Bytes())

		account, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, userCounterPDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(account.Value, "User counter should exist")

		data := account.Value.Data.GetBinary()
		s.Require().GreaterOrEqual(len(data), 48, "User counter data should be at least 48 bytes")
		counterValue := binary.LittleEndian.Uint64(data[40:48])
		s.Require().Equal(DefaultIncrementAmount, counterValue,
			"Counter should be %d despite adversary pre-funding the GMP PDA", DefaultIncrementAmount)
		s.T().Logf("Counter value: %d (pre-funded PDA did not block execution)", counterValue)
	}))

	// Step 5: Verify acknowledgement
	s.Require().True(s.Run("Verify acknowledgement on Solana", func() {
		events, err := s.Solana.Chain.GetWriteAcknowledgementEvents(ctx, solanaRelayTxSig)
		s.Require().NoError(err)
		s.Require().Len(events, 1)

		event := events[0]
		s.Require().Len(event.Acknowledgements, 1)

		ack, err := gmphelpers.UnmarshalAck(event.Acknowledgements[0], encodingType.String())
		s.Require().NoError(err)

		s.Require().Len(ack.Result, 8)
		actualCounter := binary.LittleEndian.Uint64(ack.Result)
		s.Require().Equal(DefaultIncrementAmount, actualCounter,
			"Ack counter should be %d", DefaultIncrementAmount)
		s.T().Logf("Ack verified: counter=%d (pre-funding DoS attack mitigated)", actualCounter)
	}))
}

// Test_GMPSignerExploit_RelayerPubkey verifies that the GMP program rejects
// a counter increment payload where the relayer's pubkey claims is_signer=true.
// The relayer IS a signer on the outer Solana transaction (fee payer), so
// without the fix Solana's account merging would propagate signer status into
// the target CPI.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPSignerExploit_RelayerPubkey() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUser := s.Cosmos.Users[0]

	var gmpCounterProgramID solanago.PublicKey
	s.Require().True(s.Run("Initialize GMP Counter App", func() {
		gmpCounterProgramID = s.initializeGMPCounterApp(ctx)
	}))

	var cosmosGMPTxHash []byte
	s.Require().True(s.Run("Send exploit payload from Cosmos", func() {
		exploitPubkey := s.SolanaRelayer.PublicKey()
		counterAppState, _ := solana.TestGmpApp.CounterAppStatePDA(gmpCounterProgramID)
		userCounter, _ := solana.TestGmpApp.UserCounterWithAccountSeedPDA(gmpCounterProgramID, exploitPubkey.Bytes())

		// Use auto-generated instruction builder with the relayer pubkey as
		// user_authority and payer (instead of the legitimate GMP PDA)
		ix, err := test_gmp_app.NewIncrementInstruction(
			DefaultIncrementAmount,
			counterAppState, userCounter,
			exploitPubkey, // EXPLOIT: unauthorized signer
			exploitPubkey, // EXPLOIT: unauthorized payer
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		gmpPayload, err := instructionToGMPPayload(ix, 5_000_000)
		s.Require().NoError(err)

		payload, err := gmphelpers.MarshalGMPSolanaPayload(gmpPayload, testvalues.Ics27ProtobufEncoding)
		s.Require().NoError(err)

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosUser.FormattedAddress(),
			Receiver:         gmpCounterProgramID.String(),
			Payload:          payload,
			TimeoutTimestamp: uint64(time.Now().Add(30 * time.Minute).Unix()),
			Encoding:         testvalues.Ics27ProtobufEncoding,
		})
		s.Require().NoError(err)

		cosmosGMPTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Update Solana client", func() {
		updateResp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(updateResp.Tx)
		s.Solana.Chain.SubmitChunkedUpdateClient(ctx, s.T(), s.Require(), updateResp, s.SolanaRelayer)
	}))

	s.Require().True(s.Run("Relay and expect UnexpectedSigner", func() {
		relayResp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(relayResp.Tx)

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), relayResp, s.SolanaRelayer)
		s.Require().Error(err)
		s.Require().Contains(err.Error(), "12014", "Should fail with UnexpectedSigner (error code 12014)")
	}))
}

// Test_GMPSignerExploit_SOLTransfer verifies that the GMP program rejects a
// System Program Transfer payload that uses the relayer as the funding source
// (signer + writable). Without the fix, Solana's account merging would make
// the relayer a valid signer in the CPI, letting the System Program transfer
// SOL from the proof API to an attacker-controlled wallet.
func (s *IbcEurekaSolanaGMPTestSuite) Test_GMPSignerExploit_SOLTransfer() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUser := s.Cosmos.Users[0]

	var cosmosGMPTxHash []byte
	s.Require().True(s.Run("Send SOL transfer exploit from Cosmos", func() {
		attackerWallet := solanago.NewWallet()
		relayerPubkey := s.SolanaRelayer.PublicKey()

		stolenLamports := uint64(1_000_000) // 0.001 SOL
		transferIx := system.NewTransferInstruction(stolenLamports, relayerPubkey, attackerWallet.PublicKey()).Build()

		gmpPayload, err := instructionToGMPPayload(transferIx, 5_000_000)
		s.Require().NoError(err)

		payload, err := gmphelpers.MarshalGMPSolanaPayload(gmpPayload, testvalues.Ics27ProtobufEncoding)
		s.Require().NoError(err)

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUser, 2_000_000, &gmptypes.MsgSendCall{
			SourceClient:     CosmosClientID,
			Sender:           cosmosUser.FormattedAddress(),
			Receiver:         solanago.SystemProgramID.String(),
			Salt:             []byte("sol-transfer"),
			Payload:          payload,
			TimeoutTimestamp: uint64(time.Now().Add(30 * time.Minute).Unix()),
			Encoding:         testvalues.Ics27ProtobufEncoding,
		})
		s.Require().NoError(err)

		cosmosGMPTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Update Solana client", func() {
		updateResp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(updateResp.Tx)
		s.Solana.Chain.SubmitChunkedUpdateClient(ctx, s.T(), s.Require(), updateResp, s.SolanaRelayer)
	}))

	s.Require().True(s.Run("Relay and expect UnexpectedSigner", func() {
		relayResp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosGMPTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(relayResp.Tx)

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), relayResp, s.SolanaRelayer)
		s.Require().Error(err)
		s.Require().Contains(err.Error(), "12014", "Should fail with UnexpectedSigner (error code 12014)")
	}))
}
