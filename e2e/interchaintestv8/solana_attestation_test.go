// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"os"
	"strconv"
	"strings"
	"testing"
	"time"

	test_ibc_app "github.com/cosmos/solidity-ibc-eureka/e2e/interchaintestv8/solana/go-anchor/testibcapp"
	bin "github.com/gagliardetto/binary"
	"github.com/stretchr/testify/suite"
	"google.golang.org/protobuf/proto"

	solanago "github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"

	"github.com/cosmos/interchaintest/v11/testutil"

	ics26router "github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	attestation "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/attestation"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	cosmosutils "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	attestortypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/attestor"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

const (
	// Number of Cosmos attestors for Cosmos → Solana direction.
	// Max supported is 11 due to Solana transaction size limit (1232 bytes).
	numCosmosAttestors = 11

	// Number of Solana attestors for Solana → Cosmos direction.
	numSolanaAttestors = 2
)

// IbcSolanaAttestationTestSuite is a comprehensive test suite for Solana attestation flows.
// It tests both directions:
// - Cosmos → Solana: using attestation-light-client with Cosmos attestors
// - Solana → Cosmos: using Solana attestors with mock/wasm client on Cosmos
type IbcSolanaAttestationTestSuite struct {
	e2esuite.TestSuite

	SolanaUser           *solanago.Wallet
	Ics26RouterProgramID solanago.PublicKey
	TestAppProgramID     solanago.PublicKey

	// Light clients on Solana
	Ics07TendermintProgramID        solanago.PublicKey
	AttestationLightClientProgramID solanago.PublicKey

	// AttestationClientID is the IBC client ID for the attestation light client (must be 8-64 chars)
	AttestationClientID string

	SolanaAltAddress string

	// Cosmos attestors (for Cosmos → Solana direction)
	CosmosAttestorContainers []*attestor.AttestorContainer
	CosmosAttestorEndpoints  []string
	CosmosAttestorAddresses  []string

	// Solana attestors (for Solana → Cosmos direction)
	SolanaAttestorContainers []*attestor.AttestorContainer
	SolanaAttestorEndpoints  []string
	SolanaAttestorAddresses  []string
	SolanaAttestorClient     attestortypes.AttestationServiceClient

	ProofApiClient  proofapitypes.ProofApiServiceClient
	ProofApiProcess *os.Process
}

func TestWithIbcSolanaAttestationTestSuite(t *testing.T) {
	suite.Run(t, new(IbcSolanaAttestationTestSuite))
}

func (s *IbcSolanaAttestationTestSuite) TearDownSuite() {
	ctx := context.Background()
	attestor.CleanupContainers(ctx, s.T(), s.CosmosAttestorContainers)
	attestor.CleanupContainers(ctx, s.T(), s.SolanaAttestorContainers)

	if s.ProofApiProcess != nil {
		s.T().Logf("Cleaning up proof API process (PID: %d)", s.ProofApiProcess.Pid)
		if err := s.ProofApiProcess.Kill(); err != nil {
			s.T().Logf("Failed to kill proof API process: %v", err)
		}
	}
}

func (s *IbcSolanaAttestationTestSuite) SetupSuite(ctx context.Context) {
	var err error

	err = os.Chdir("../..")
	s.Require().NoError(err)

	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetType_None)
	os.Setenv(testvalues.EnvKeySolanaTestnetType, testvalues.SolanaTestnetType_Localnet)
	os.Setenv(testvalues.EnvKeyEthLcOnCosmos, testvalues.EthLCOnCosmosTypeAttestorNative)

	s.TestSuite.SetupSuite(ctx)

	err = s.Solana.Chain.WaitForClusterReady(ctx, 30*time.Second)
	s.Require().NoError(err)

	s.SolanaUser = solanago.NewWallet()

	_, err = s.Solana.Chain.FundUserWithRetry(ctx, s.SolanaUser.PublicKey(), testvalues.InitialSolBalance, 5)
	s.Require().NoError(err)

	const deployerFunding = 100 * testvalues.InitialSolBalance
	_, err = s.Solana.Chain.FundUserWithRetry(ctx, solana.DeployerPubkey, deployerFunding, 5)
	s.Require().NoError(err)

	s.T().Log("Deploying programs...")
	const keypairDir = "solana-keypairs/localnet"
	const deployerPath = keypairDir + "/deployer_wallet.json"

	deployTasks := []e2esuite.ParallelTaskWithResult[solanago.PublicKey]{
		{
			Name: "Deploy ICS26 Router",
			Run: func() (solanago.PublicKey, error) {
				keypairPath := fmt.Sprintf("%s/ics26_router-keypair.json", keypairDir)
				return s.Solana.Chain.DeploySolanaProgramAsync(ctx, "ics26_router", keypairPath, deployerPath)
			},
		},
		{
			Name: "Deploy ICS07 Tendermint",
			Run: func() (solanago.PublicKey, error) {
				keypairPath := fmt.Sprintf("%s/ics07_tendermint-keypair.json", keypairDir)
				return s.Solana.Chain.DeploySolanaProgramAsync(ctx, "ics07_tendermint", keypairPath, deployerPath)
			},
		},
		{
			Name: "Deploy Attestation Light Client",
			Run: func() (solanago.PublicKey, error) {
				keypairPath := fmt.Sprintf("%s/attestation-keypair.json", keypairDir)
				return s.Solana.Chain.DeploySolanaProgramAsync(ctx, "attestation", keypairPath, deployerPath)
			},
		},
		{
			Name: "Deploy Test App",
			Run: func() (solanago.PublicKey, error) {
				keypairPath := fmt.Sprintf("%s/test_ibc_app-keypair.json", keypairDir)
				return s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_ibc_app", keypairPath, deployerPath)
			},
		},
		{
			Name: "Deploy Access Manager",
			Run: func() (solanago.PublicKey, error) {
				keypairPath := fmt.Sprintf("%s/access_manager-keypair.json", keypairDir)
				return s.Solana.Chain.DeploySolanaProgramAsync(ctx, "access_manager", keypairPath, deployerPath)
			},
		},
	}

	deployResults, err := e2esuite.RunParallelTasksWithResults(deployTasks...)
	s.Require().NoError(err)

	s.Ics26RouterProgramID = deployResults["Deploy ICS26 Router"]
	ics26_router.ProgramID = s.Ics26RouterProgramID
	s.Ics07TendermintProgramID = deployResults["Deploy ICS07 Tendermint"]
	ics07_tendermint.ProgramID = s.Ics07TendermintProgramID
	s.AttestationLightClientProgramID = deployResults["Deploy Attestation Light Client"]
	attestation.ProgramID = s.AttestationLightClientProgramID
	s.TestAppProgramID = deployResults["Deploy Test App"]
	test_ibc_app.ProgramID = s.TestAppProgramID
	access_manager.ProgramID = deployResults["Deploy Access Manager"]

	s.T().Log("Initializing Access Manager...")
	deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
	s.Require().NoError(err)

	accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
	amProgramDataPDA, _ := solana.AccessManager.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
	initAccessManagerInstruction, err := access_manager.NewInitializeInstruction(
		s.SolanaUser.PublicKey(),
		accessControlAccount,
		s.SolanaUser.PublicKey(),
		solanago.SystemProgramID,
		solanago.SysVarInstructionsPubkey,
		amProgramDataPDA,
		solana.DeployerPubkey,
	)
	s.Require().NoError(err)

	tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initAccessManagerInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentFinalized, 30, s.SolanaUser, deployerWallet)
	s.Require().NoError(err)
	s.T().Log("Access Manager initialized")

	s.T().Log("Granting RELAYER_ROLE to SolanaUser...")
	const RELAYER_ROLE = uint64(1)
	grantRelayerRoleInstruction, err := access_manager.NewGrantRoleInstruction(
		RELAYER_ROLE,
		s.SolanaUser.PublicKey(),
		accessControlAccount,
		s.SolanaUser.PublicKey(),
		solanago.SysVarInstructionsPubkey,
	)
	s.Require().NoError(err)

	tx, err = s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), grantRelayerRoleInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentFinalized, 30, s.SolanaUser)
	s.Require().NoError(err)
	s.T().Log("RELAYER_ROLE granted")

	s.T().Log("Granting ID_CUSTOMIZER_ROLE to SolanaUser...")
	const ID_CUSTOMIZER_ROLE = uint64(6)
	grantIdCustomizerRoleInstruction, err := access_manager.NewGrantRoleInstruction(
		ID_CUSTOMIZER_ROLE,
		s.SolanaUser.PublicKey(),
		accessControlAccount,
		s.SolanaUser.PublicKey(),
		solanago.SysVarInstructionsPubkey,
	)
	s.Require().NoError(err)

	tx, err = s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), grantIdCustomizerRoleInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentFinalized, 30, s.SolanaUser)
	s.Require().NoError(err)
	s.T().Log("ID_CUSTOMIZER_ROLE granted")

	s.T().Log("Initializing ICS26 Router...")
	routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	routerProgramDataPDA, _ := solana.Ics26Router.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
	initInstruction, err := ics26_router.NewInitializeInstruction(
		access_manager.ProgramID,
		routerStateAccount,
		s.SolanaUser.PublicKey(),
		solanago.SystemProgramID,
		routerProgramDataPDA,
		solana.DeployerPubkey,
	)
	s.Require().NoError(err)

	tx, err = s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentFinalized, 30, s.SolanaUser, deployerWallet)
	s.Require().NoError(err)

	simd := s.Cosmos.Chains[0]
	cosmosChainID := simd.Config().ChainID

	// Set AttestationClientID to a valid IBC identifier (8-64 chars required)
	// Must use "attestations-N" format to match CLIENT_TYPE_ATTESTATION for proper PDA derivation
	s.AttestationClientID = testvalues.FirstAttestationsClientID

	// Create ALT for attestation client with attestation light client accounts
	altAddress := s.Solana.Chain.CreateIBCAddressLookupTableWithAttestation(
		ctx, s.T(), s.Require(), s.SolanaUser, cosmosChainID, transfertypes.PortID, s.AttestationClientID, s.AttestationClientID,
	)
	s.SolanaAltAddress = altAddress.String()

	s.T().Log("Starting Cosmos attestor service(s) for Cosmos → Solana direction...")
	cosmosAttestorResult := attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         numCosmosAttestors,
		KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		ChainType:            attestor.ChainTypeCosmos,
		AdapterURL:           simd.GetRPCAddress(),
		RouterAddress:        "",
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
	})
	s.CosmosAttestorContainers = cosmosAttestorResult.Containers
	s.CosmosAttestorEndpoints = cosmosAttestorResult.Endpoints
	s.CosmosAttestorAddresses = cosmosAttestorResult.Addresses
	s.T().Logf("Cosmos attestor addresses: %v", s.CosmosAttestorAddresses)

	s.T().Log("Starting Solana attestor service(s) for Solana → Cosmos direction...")
	solanaAttestorResult := attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         numSolanaAttestors,
		KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		ChainType:            attestor.ChainTypeSolana,
		AdapterURL:           attestor.TransformLocalhostToDockerHost(testvalues.SolanaLocalnetRPC),
		RouterAddress:        ics26_router.ProgramID.String(),
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
		EnableHostAccess:     true,
	})
	s.SolanaAttestorContainers = solanaAttestorResult.Containers
	s.SolanaAttestorEndpoints = solanaAttestorResult.Endpoints
	s.SolanaAttestorAddresses = solanaAttestorResult.Addresses
	if len(s.SolanaAttestorEndpoints) > 0 {
		serverAddr := s.SolanaAttestorEndpoints[0][len("http://"):]
		s.SolanaAttestorClient, err = attestor.GetAttestationServiceClient(serverAddr)
		s.Require().NoError(err)
	}
	s.T().Logf("Solana attestor endpoints: %v", s.SolanaAttestorEndpoints)
	s.T().Logf("Solana attestor addresses: %v", s.SolanaAttestorAddresses)

	s.T().Log("Initializing Attestation Light Client on Solana...")
	s.initializeAttestationLightClient(ctx)

	s.T().Log("Starting proofapi...")
	config := proofapi.NewConfigBuilder().
		SolanaToCosmosAttested(proofapi.SolanaToCosmosAttestedParams{
			SolanaChainID:     testvalues.SolanaChainID,
			CosmosChainID:     simd.Config().ChainID,
			SolanaRPC:         testvalues.SolanaLocalnetRPC,
			TmRPC:             simd.GetHostRPCAddress(),
			ICS26ProgramID:    ics26_router.ProgramID.String(),
			SignerAddress:     s.Cosmos.Users[0].FormattedAddress(),
			AttestorEndpoints: s.SolanaAttestorEndpoints,
			AttestorTimeout:   30000,
			QuorumThreshold:   testvalues.DefaultMinRequiredSigs,
		}).
		CosmosToSolanaAttested(proofapi.CosmosToSolanaAttestedParams{
			CosmosChainID:     simd.Config().ChainID,
			SolanaChainID:     testvalues.SolanaChainID,
			SolanaRPC:         testvalues.SolanaLocalnetRPC,
			TmRPC:             simd.GetHostRPCAddress(),
			ICS26ProgramID:    ics26_router.ProgramID.String(),
			FeePayer:          s.SolanaUser.PublicKey().String(),
			ALTAddress:        s.SolanaAltAddress,
			AttestorEndpoints: s.CosmosAttestorEndpoints,
		}).
		Build()

	err = config.GenerateConfigFile(testvalues.ProofAPIConfigFilePath)
	s.Require().NoError(err)

	s.ProofApiProcess, err = proofapi.StartProofAPI(testvalues.ProofAPIConfigFilePath)
	s.Require().NoError(err)

	s.T().Cleanup(func() {
		os.Remove(testvalues.ProofAPIConfigFilePath)
	})

	s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
	s.Require().NoError(err)

	s.T().Log("Creating attestor client on Cosmos...")
	currentFinalizedSlot, err := s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
	s.Require().NoError(err)
	solanaTimestamp, err := s.Solana.Chain.RPCClient.GetBlockTime(ctx, currentFinalizedSlot)
	s.Require().NoError(err)

	s.Require().NotEmpty(s.SolanaAttestorAddresses, "No Solana attestor addresses available")

	clientResp, err := s.ProofApiClient.CreateClient(ctx, &proofapitypes.CreateClientRequest{
		SrcChain: testvalues.SolanaChainID,
		DstChain: simd.Config().ChainID,
		Parameters: map[string]string{
			testvalues.ParameterKey_AttestorAddresses: strings.Join(s.SolanaAttestorAddresses, ","),
			testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(numSolanaAttestors),
			testvalues.ParameterKey_height:            strconv.FormatUint(currentFinalizedSlot, 10),
			testvalues.ParameterKey_timestamp:         strconv.FormatInt(int64(*solanaTimestamp), 10),
		},
	})
	s.Require().NoError(err)

	txResp := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], CosmosCreateClientGasLimit, clientResp.Tx)
	s.T().Logf("Attestor client created on Cosmos - tx: %s", txResp.TxHash)

	s.T().Log("Adding attestation client to Router on Solana...")
	routerStateAccount, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	clientAccount, _ := solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID))

	counterpartyInfo := ics26_router.SolanaIbcTypesRouterCounterpartyInfo{
		ClientId:     CosmosClientID,
		MerklePrefix: [][]byte{[]byte("")}, // Single element for attestation light client compatibility
	}

	addClientInstruction, err := ics26_router.NewAddClientInstruction(
		s.AttestationClientID,
		counterpartyInfo,
		s.SolanaUser.PublicKey(),
		routerStateAccount,
		accessControlAccount,
		clientAccount,
		attestation.ProgramID,
		solanago.SystemProgramID,
		solanago.SysVarInstructionsPubkey,
	)
	s.Require().NoError(err)

	tx, err = s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), addClientInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentFinalized, s.SolanaUser)
	s.Require().NoError(err)
	s.T().Log("Attestation client added to router")

	s.T().Log("Registering counterparty on Cosmos...")
	merklePathPrefix := [][]byte{[]byte("")}
	_, err = s.BroadcastMessages(ctx, simd, s.Cosmos.Users[0], CosmosDefaultGasLimit, &clienttypesv2.MsgRegisterCounterparty{
		ClientId:                 CosmosClientID,
		CounterpartyMerklePrefix: merklePathPrefix,
		CounterpartyClientId:     s.AttestationClientID,
		Signer:                   s.Cosmos.Users[0].FormattedAddress(),
	})
	s.Require().NoError(err)
	s.T().Log("Counterparty registered on Cosmos")

	s.T().Log("Initializing Test IBC App...")
	appStateAccount, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)

	initAppInstruction, err := test_ibc_app.NewInitializeInstruction(
		s.SolanaUser.PublicKey(),
		appStateAccount,
		s.SolanaUser.PublicKey(),
		solanago.SystemProgramID,
	)
	s.Require().NoError(err)

	appTx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initAppInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, appTx, rpc.CommitmentFinalized, s.SolanaUser)
	s.Require().NoError(err)
	s.T().Log("Test IBC App initialized successfully")

	s.T().Log("Registering Test App with Router...")
	routerStateAccount, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	ibcAppAccount, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))

	registerInstruction, err := ics26_router.NewAddIbcAppInstruction(
		transfertypes.PortID,
		routerStateAccount,
		accessControlAccount,
		ibcAppAccount,
		s.TestAppProgramID,
		s.SolanaUser.PublicKey(),
		s.SolanaUser.PublicKey(),
		solanago.SystemProgramID,
		solanago.SysVarInstructionsPubkey,
	)
	s.Require().NoError(err)

	tx, err = s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), registerInstruction)
	s.Require().NoError(err)

	_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentFinalized, s.SolanaUser)
	s.Require().NoError(err)
	s.T().Log("Test app registered with router")
}

// initializeAttestationLightClient initializes the attestation light client with Cosmos attestor addresses
func (s *IbcSolanaAttestationTestSuite) initializeAttestationLightClient(ctx context.Context) {
	var attestorAddresses [][20]uint8
	for _, addr := range s.CosmosAttestorAddresses {
		addrHex := addr
		if len(addrHex) >= 2 && addrHex[:2] == "0x" {
			addrHex = addrHex[2:]
		}

		addrBytes, err := hex.DecodeString(addrHex)
		s.Require().NoError(err)
		s.Require().Len(addrBytes, 20, "Attestor address must be 20 bytes")

		var addrArray [20]uint8
		copy(addrArray[:], addrBytes)
		attestorAddresses = append(attestorAddresses, addrArray)
	}

	minRequiredSigs := uint8(numCosmosAttestors)

	clientStatePDA, _ := solana.Attestation.ClientPDA(attestation.ProgramID)
	appStatePDA, _ := solana.Attestation.AppStatePDA(attestation.ProgramID)

	attestationProgramDataPDA, _ := solana.Attestation.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
	initInstruction, err := attestation.NewInitializeInstruction(
		attestorAddresses,
		minRequiredSigs,
		access_manager.ProgramID,
		clientStatePDA,
		appStatePDA,
		s.SolanaUser.PublicKey(),
		solanago.SystemProgramID,
		attestationProgramDataPDA,
		solana.DeployerPubkey,
	)
	s.Require().NoError(err)

	deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
	s.Require().NoError(err)

	tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initInstruction)
	s.Require().NoError(err)

	sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentFinalized, 30, s.SolanaUser, deployerWallet)
	s.Require().NoError(err)
	s.T().Logf("Attestation Light Client initialized - tx: %s", sig)
}

func (s *IbcSolanaAttestationTestSuite) Test_Attestation_Deploy() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	s.Require().True(s.Run("Verify attestation light client on Solana", func() {
		clientStatePDA, _ := solana.Attestation.ClientPDA(attestation.ProgramID)

		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, clientStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentFinalized,
		})
		s.Require().NoError(err)

		clientState, err := attestation.ParseAccount_AttestationTypesClientState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		s.Require().Equal(uint8(numCosmosAttestors), clientState.MinRequiredSigs)
		s.Require().Equal(uint64(0), clientState.LatestHeight)
		s.Require().False(clientState.IsFrozen)
		s.Require().Len(clientState.AttestorAddresses, len(s.CosmosAttestorAddresses))

		s.T().Logf("Attestation LC: latestHeight=%d, attestors=%d",
			clientState.LatestHeight, len(clientState.AttestorAddresses))
	}))

	s.Require().True(s.Run("Verify Solana attestor is running", func() {
		slot, err := s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
		s.Require().NoError(err)

		resp, err := attestor.GetStateAttestation(ctx, s.SolanaAttestorClient, slot)
		s.Require().NoError(err)
		s.Require().NotNil(resp.GetAttestation())

		attestation := resp.GetAttestation()
		s.Require().NotEmpty(attestation.GetSignature())
		s.Require().Equal(slot, attestation.GetHeight())
		s.T().Logf("Solana attestor working - attested slot: %d", slot)
	}))

	s.Require().True(s.Run("Verify proof API connection", func() {
		simd := s.Cosmos.Chains[0]

		resp, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: simd.Config().ChainID,
			DstChain: testvalues.SolanaChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp)
		s.T().Logf("Proof API: %s <-> %s", resp.SourceChain.ChainId, resp.TargetChain.ChainId)
	}))
}

func (s *IbcSolanaAttestationTestSuite) Test_Attestation_SolanaAttestorVerifyPacketCommitment() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	var solanaTxSig solanago.Signature
	var sequence uint64
	var packetCommitmentPDA solanago.PublicKey
	var slot uint64

	simd := s.Cosmos.Chains[0]

	s.Require().True(s.Run("Update attestation client on Solana", func() {
		resp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			DstClientId: s.AttestationClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		var solanaUpdateClient proofapitypes.SolanaUpdateClient
		err = proto.Unmarshal(resp.Tx, &solanaUpdateClient)
		s.Require().NoError(err)
		s.Require().NotEmpty(solanaUpdateClient.AssemblyTx)

		unsignedSolanaTx, err := solanago.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.AssemblyTx))
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, unsignedSolanaTx, rpc.CommitmentFinalized, s.SolanaUser)
		s.Require().NoError(err)
		s.T().Logf("Attestation client updated - tx: %s", sig)
	}))

	s.Require().True(s.Run("Send packet on Solana", func() {
		appState, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)
		routerState, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
		ibcApp, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))
		client, _ := solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID))

		sequence = 1

		sequenceBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(sequenceBytes, sequence)
		packetCommitmentPDA, _ = solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID), sequenceBytes)

		timeoutTimestamp := uint64(time.Now().Add(1 * time.Hour).Unix())
		packetMsg := test_ibc_app.TestIbcAppInstructionsSendPacketSendPacketMsg{
			SourceClient:     s.AttestationClientID,
			SourcePort:       transfertypes.PortID,
			DestPort:         transfertypes.PortID,
			Version:          transfertypes.V1,
			Encoding:         "application/json",
			PacketData:       []byte(`{"test":"data"}`),
			TimeoutTimestamp: timeoutTimestamp,
			Sequence:         sequence,
		}

		attestationClientStatePDA, _ := solana.Attestation.ClientPDA(attestation.ProgramID)
		attestationConsensusStatePDA := s.deriveAttestationConsensusStatePDA(ctx, attestationClientStatePDA)
		sendPacketInstruction, err := test_ibc_app.NewSendPacketInstruction(
			packetMsg,
			appState,
			s.SolanaUser.PublicKey(),
			routerState,
			ibcApp,
			packetCommitmentPDA,
			client,
			attestation.ProgramID,
			attestationClientStatePDA,
			attestationConsensusStatePDA,
			ics26_router.ProgramID,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		computeBudgetInstruction := solana.NewComputeBudgetInstruction(DefaultComputeUnits)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaUser.PublicKey(),
			computeBudgetInstruction,
			sendPacketInstruction,
		)
		s.Require().NoError(err)

		solanaTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentFinalized, s.SolanaUser)
		s.Require().NoError(err)

		slot, err = s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
		s.Require().NoError(err)

		s.T().Logf("Sent packet - tx: %s, sequence: %d, slot: %d", solanaTxSig, sequence, slot)
	}))

	var event *solana.SendPacketEvent
	s.Require().True(s.Run("Parse event and verify sequence", func() {
		var err error
		event, err = solana.GetSendPacketEventFromTransaction(ctx, s.Solana.Chain.RPCClient, solanaTxSig)
		s.Require().NoError(err)
		s.Require().NotNil(event)
		s.Require().Equal(s.AttestationClientID, event.ClientID)
		s.Require().Equal(sequence, event.Sequence)
	}))

	var onChainCommitment []byte
	s.Require().True(s.Run("Query commitment on chain", func() {
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, packetCommitmentPDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentFinalized,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)

		data := accountInfo.Value.Data.GetBinary()
		s.Require().GreaterOrEqual(len(data), 40)

		onChainCommitment = data[8:40]
		s.Require().NotEmpty(onChainCommitment)
	}))

	s.Require().True(s.Run("Call Solana attestor and verify commitment", func() {
		abiPacket := convertSolanaPacketToABI(event.Packet)
		packetBytes, err := types.AbiEncodePacket(abiPacket)
		s.Require().NoError(err)

		resp, err := attestor.GetPacketAttestation(ctx, s.SolanaAttestorClient, [][]byte{packetBytes}, slot)
		s.Require().NoError(err)
		s.Require().NotNil(resp.GetAttestation())

		attestation := resp.GetAttestation()
		s.Require().NotEmpty(attestation.GetSignature())
		s.Require().Equal(slot, attestation.GetHeight())

		attestedData := attestation.GetAttestedData()

		packetAttestation, err := types.AbiDecodePacketAttestation(attestedData)
		s.Require().NoError(err)
		s.Require().NotNil(packetAttestation)
		s.Require().Equal(slot, packetAttestation.Height)
		s.Require().Len(packetAttestation.Packets, 1)

		attestorCommitment := packetAttestation.Packets[0].Commitment[:]
		s.Require().Equal(onChainCommitment, attestorCommitment)

		s.T().Log("Solana attestor packet commitment verification successful")
	}))
}

func (s *IbcSolanaAttestationTestSuite) Test_Attestation_CosmosToSolanaTransfer() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	simd := s.Cosmos.Chains[0]

	var cosmosRelayPacketTxHash []byte
	var timeout uint64
	var encodedPayload []byte

	s.Require().True(s.Run("Send ICS20 transfer from Cosmos to Solana", func() {
		cosmosUserWallet := s.Cosmos.Users[0]
		cosmosUserAddress := cosmosUserWallet.FormattedAddress()
		solanaUserAddress := s.SolanaUser.PublicKey().String()
		transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(TestTransferAmount))

		timeout = uint64(time.Now().Add(30 * time.Minute).Unix())

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   cosmosUserAddress,
			Receiver: solanaUserAddress,
			Memo:     "cosmos-to-solana-attestation-transfer",
		}
		var err error
		encodedPayload, err = transfertypes.MarshalPacketData(transferPayload, transfertypes.V1, transfertypes.EncodingProtobuf)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingProtobuf,
			Value:           encodedPayload,
		}
		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     CosmosClientID,
			TimeoutTimestamp: timeout,
			Payloads: []channeltypesv2.Payload{
				payload,
			},
			Signer: cosmosUserWallet.FormattedAddress(),
		}

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, CosmosDefaultGasLimit, &msgSendPacket)
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		cosmosPacketTxHashBytes, err := hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		cosmosRelayPacketTxHash = cosmosPacketTxHashBytes

		s.T().Logf("Cosmos packet sent: %s", resp.TxHash)
	}))

	s.Require().NoError(testutil.WaitForBlocks(ctx, 1, simd))

	var solanaRelayTxSig solanago.Signature

	s.Require().True(s.Run("Relay packet to Solana using attestation LC", func() {
		s.Require().True(s.Run("Update attestation client on Solana", func() {
			resp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				DstClientId: s.AttestationClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			// Unmarshal the protobuf-encoded SolanaUpdateClient
			var solanaUpdateClient proofapitypes.SolanaUpdateClient
			err = proto.Unmarshal(resp.Tx, &solanaUpdateClient)
			s.Require().NoError(err, "Failed to unmarshal SolanaUpdateClient")
			s.Require().NotEmpty(solanaUpdateClient.AssemblyTx, "AssemblyTx is empty")

			unsignedSolanaTx, err := solanago.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.AssemblyTx))
			s.Require().NoError(err)

			sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, unsignedSolanaTx, rpc.CommitmentFinalized, s.SolanaUser)
			s.Require().NoError(err)
			s.T().Logf("Attestation client updated - tx: %s", sig)
		}))

		s.Require().True(s.Run("Relay packet", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosRelayPacketTxHash},
				SrcClientId: CosmosClientID,
				DstClientId: s.AttestationClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			var err2 error
			solanaRelayTxSig, err2 = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaUser)
			s.Require().NoError(err2)
			s.T().Logf("Packet relayed to Solana - tx: %s", solanaRelayTxSig)
		}))
	}))

	s.Require().True(s.Run("Verify packet received on Solana", func() {
		testAppStateAccount, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)

		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, testAppStateAccount, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)

		appState, err := test_ibc_app.ParseAccount_TestIbcAppStateTestIbcAppState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		s.Require().Greater(appState.PacketsReceived, uint64(0))
		s.T().Logf("Solana dummy app received %d packets", appState.PacketsReceived)
	}))

	s.Require().True(s.Run("Verify ACK commitment via Solana attestor", func() {
		s.Require().NoError(s.Solana.Chain.WaitForTxStatus(solanaRelayTxSig, rpc.ConfirmationStatusFinalized))

		slot, err := s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
		s.Require().NoError(err)

		actualSequence := uint64(1)
		sequenceBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(sequenceBytes, actualSequence)

		packetAckPDA, _ := solana.Ics26Router.PacketAckWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID), sequenceBytes)

		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, packetAckPDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentFinalized,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)

		data := accountInfo.Value.Data.GetBinary()
		s.Require().GreaterOrEqual(len(data), 40)
		onChainAckCommitment := data[8:40]
		s.Require().NotEmpty(onChainAckCommitment)

		abiPacket := ics26router.IICS26RouterMsgsPacket{
			Sequence:         1,
			SourceClient:     CosmosClientID,
			DestClient:       s.AttestationClientID,
			TimeoutTimestamp: timeout,
			Payloads: []ics26router.IICS26RouterMsgsPayload{
				{
					SourcePort: transfertypes.PortID,
					DestPort:   transfertypes.PortID,
					Version:    transfertypes.V1,
					Encoding:   transfertypes.EncodingProtobuf,
					Value:      encodedPayload,
				},
			},
		}
		packetBytes, err := types.AbiEncodePacket(abiPacket)
		s.Require().NoError(err)

		resp, err := attestor.GetPacketAttestation(ctx, s.SolanaAttestorClient, [][]byte{packetBytes}, slot, attestortypes.CommitmentType_COMMITMENT_TYPE_ACK)
		s.Require().NoError(err)
		s.Require().NotNil(resp.GetAttestation())

		attestation := resp.GetAttestation()
		s.Require().NotEmpty(attestation.GetSignature())
		s.Require().Equal(slot, attestation.GetHeight())

		attestedData := attestation.GetAttestedData()
		packetAttestation, err := types.AbiDecodePacketAttestation(attestedData)
		s.Require().NoError(err)
		s.Require().NotNil(packetAttestation)
		s.Require().Len(packetAttestation.Packets, 1)

		attestorAckCommitment := packetAttestation.Packets[0].Commitment[:]
		s.Require().Equal(onChainAckCommitment, attestorAckCommitment)

		s.T().Log("Solana attestor ACK commitment verification successful")
	}))

	s.Require().True(s.Run("Relay acknowledgment back to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    testvalues.SolanaChainID,
			DstChain:    simd.Config().ChainID,
			SourceTxIds: [][]byte{[]byte(solanaRelayTxSig.String())},
			SrcClientId: s.AttestationClientID,
			DstClientId: CosmosClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], CosmosDefaultGasLimit, resp.Tx)
		s.T().Logf("ACK relayed to Cosmos - tx: %s", relayTxResult.TxHash)
	}))

	s.Require().True(s.Run("Verify balance on Cosmos", func() {
		cosmosUserAddress := s.Cosmos.Users[0].FormattedAddress()

		allBalancesResp, err := e2esuite.GRPCQuery[banktypes.QueryAllBalancesResponse](ctx, simd, &banktypes.QueryAllBalancesRequest{
			Address: cosmosUserAddress,
		})
		s.Require().NoError(err)
		s.T().Logf("Cosmos user balances:")
		for _, balance := range allBalancesResp.Balances {
			s.T().Logf("  - %s: %s", balance.Denom, balance.Amount.String())
		}
	}))
}

func (s *IbcSolanaAttestationTestSuite) Test_Attestation_Roundtrip() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()
	solanaUserAddress := s.SolanaUser.PublicKey().String()
	transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(TestTransferAmount))

	var initialCosmosBalance int64
	var cosmosRelayPacketTxHash []byte
	var solanaRelayTxSig solanago.Signature
	var cosmosPacketSequence uint64

	s.Require().True(s.Run("Phase 1: Cosmos to Solana transfer", func() {
		s.Require().True(s.Run("Record initial Cosmos balance", func() {
			resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(resp.Balance)
			initialCosmosBalance = resp.Balance.Amount.Int64()
			s.T().Logf("Initial Cosmos balance: %d %s", initialCosmosBalance, transferCoin.Denom)
		}))

		s.Require().True(s.Run("Send ICS20 transfer from Cosmos", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   cosmosUserAddress,
				Receiver: solanaUserAddress,
				Memo:     "roundtrip-cosmos-to-solana",
			}
			encodedPayload, err := transfertypes.MarshalPacketData(transferPayload, transfertypes.V1, transfertypes.EncodingProtobuf)
			s.Require().NoError(err)

			payload := channeltypesv2.Payload{
				SourcePort:      transfertypes.PortID,
				DestinationPort: transfertypes.PortID,
				Version:         transfertypes.V1,
				Encoding:        transfertypes.EncodingProtobuf,
				Value:           encodedPayload,
			}
			msgSendPacket := channeltypesv2.MsgSendPacket{
				SourceClient:     CosmosClientID,
				TimeoutTimestamp: timeout,
				Payloads:         []channeltypesv2.Payload{payload},
				Signer:           cosmosUserAddress,
			}

			resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, CosmosDefaultGasLimit, &msgSendPacket)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			cosmosPacketTxHashBytes, err := hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			cosmosRelayPacketTxHash = cosmosPacketTxHashBytes
			cosmosPacketSequence = 1

			s.T().Logf("Cosmos → Solana packet sent: %s (sequence: %d)", resp.TxHash, cosmosPacketSequence)
		}))

		s.Require().True(s.Run("Verify packet commitment exists on Cosmos", func() {
			commitmentResp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
				ClientId: CosmosClientID,
				Sequence: cosmosPacketSequence,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(commitmentResp.Commitment)
			s.T().Logf("Cosmos packet commitment verified for sequence %d", cosmosPacketSequence)
		}))
	}))

	s.Require().NoError(testutil.WaitForBlocks(ctx, 1, simd))

	var initialPacketsReceived uint64

	s.Require().True(s.Run("Relay packet to Solana via attestation LC", func() {
		s.Require().True(s.Run("Record initial packets received on Solana", func() {
			testAppStateAccount, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)
			accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, testAppStateAccount, &rpc.GetAccountInfoOpts{
				Commitment: rpc.CommitmentConfirmed,
			})
			s.Require().NoError(err)
			if accountInfo.Value != nil {
				appState, err := test_ibc_app.ParseAccount_TestIbcAppStateTestIbcAppState(accountInfo.Value.Data.GetBinary())
				s.Require().NoError(err)
				initialPacketsReceived = appState.PacketsReceived
			}
			s.T().Logf("Initial packets received on Solana: %d", initialPacketsReceived)
		}))

		s.Require().True(s.Run("Update attestation client on Solana", func() {
			resp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				DstClientId: s.AttestationClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			// Unmarshal the protobuf-encoded SolanaUpdateClient
			var solanaUpdateClient proofapitypes.SolanaUpdateClient
			err = proto.Unmarshal(resp.Tx, &solanaUpdateClient)
			s.Require().NoError(err, "Failed to unmarshal SolanaUpdateClient")
			s.Require().NotEmpty(solanaUpdateClient.AssemblyTx, "AssemblyTx is empty")

			unsignedSolanaTx, err := solanago.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.AssemblyTx))
			s.Require().NoError(err)

			sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, unsignedSolanaTx, rpc.CommitmentFinalized, s.SolanaUser)
			s.Require().NoError(err)
			s.T().Logf("Attestation client updated - tx: %s", sig)
		}))

		s.Require().True(s.Run("Relay packet to Solana", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosRelayPacketTxHash},
				SrcClientId: CosmosClientID,
				DstClientId: s.AttestationClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			var err2 error
			solanaRelayTxSig, err2 = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaUser)
			s.Require().NoError(err2)
			s.T().Logf("Packet relayed to Solana - tx: %s", solanaRelayTxSig)
		}))

		s.Require().True(s.Run("Verify packet received on Solana", func() {
			testAppStateAccount, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)
			accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, testAppStateAccount, &rpc.GetAccountInfoOpts{
				Commitment: rpc.CommitmentConfirmed,
			})
			s.Require().NoError(err)
			s.Require().NotNil(accountInfo.Value)

			appState, err := test_ibc_app.ParseAccount_TestIbcAppStateTestIbcAppState(accountInfo.Value.Data.GetBinary())
			s.Require().NoError(err)

			s.Require().Greater(appState.PacketsReceived, initialPacketsReceived)
			s.T().Logf("Solana dummy app received %d packets (was %d)", appState.PacketsReceived, initialPacketsReceived)
		}))

		s.Require().True(s.Run("Relay ACK to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaRelayTxSig.String())},
				SrcClientId: s.AttestationClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, cosmosUserWallet, CosmosDefaultGasLimit, resp.Tx)
			s.T().Logf("ACK relayed to Cosmos - tx: %s", relayTxResult.TxHash)
		}))

		s.Require().True(s.Run("Verify packet commitment deleted on Cosmos", func() {
			_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
				ClientId: CosmosClientID,
				Sequence: cosmosPacketSequence,
			})
			s.Require().ErrorContains(err, "packet commitment hash not found")
			s.T().Logf("Cosmos packet commitment deleted for sequence %d", cosmosPacketSequence)
		}))
	}))

	var solanaSendTxSig solanago.Signature
	var sendPacketSlot uint64
	var sendPacketEvent *solana.SendPacketEvent
	var solanaSequence uint64

	s.Require().True(s.Run("Phase 2: Solana to Cosmos transfer", func() {
		routerState, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
		ibcApp, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))
		client, _ := solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID))
		appState, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)

		s.Require().True(s.Run("Send packet from Solana", func() {
			solanaSequence = 1

			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, solanaSequence)
			packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID), sequenceBytes)

			timeoutTimestamp := uint64(time.Now().Add(1 * time.Hour).Unix())
			packetMsg := test_ibc_app.TestIbcAppInstructionsSendPacketSendPacketMsg{
				SourceClient:     s.AttestationClientID,
				SourcePort:       transfertypes.PortID,
				DestPort:         transfertypes.PortID,
				Version:          transfertypes.V1,
				Encoding:         "application/json",
				PacketData:       []byte(fmt.Sprintf(`{"denom":"%s","amount":"%d","sender":"%s","receiver":"%s","memo":"roundtrip-solana-to-cosmos"}`, transferCoin.Denom, TestTransferAmount, solanaUserAddress, cosmosUserAddress)),
				TimeoutTimestamp: timeoutTimestamp,
				Sequence:         solanaSequence,
			}

			attestationClientStatePDA, _ := solana.Attestation.ClientPDA(attestation.ProgramID)
			attestationConsensusStatePDA := s.deriveAttestationConsensusStatePDA(ctx, attestationClientStatePDA)
			sendPacketInstruction, err := test_ibc_app.NewSendPacketInstruction(
				packetMsg,
				appState,
				s.SolanaUser.PublicKey(),
				routerState,
				ibcApp,
				packetCommitmentPDA,
				client,
				attestation.ProgramID,
				attestationClientStatePDA,
				attestationConsensusStatePDA,
				ics26_router.ProgramID,
				solanago.SystemProgramID,
			)
			s.Require().NoError(err)

			computeBudgetInstruction := solana.NewComputeBudgetInstruction(DefaultComputeUnits)
			tx, err := s.Solana.Chain.NewTransactionFromInstructions(
				s.SolanaUser.PublicKey(),
				computeBudgetInstruction,
				sendPacketInstruction,
			)
			s.Require().NoError(err)

			solanaSendTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentFinalized, s.SolanaUser)
			s.Require().NoError(err)

			sendPacketSlot, err = s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
			s.Require().NoError(err)

			s.T().Logf("Solana → Cosmos packet sent - tx: %s, sequence: %d, slot: %d", solanaSendTxSig, solanaSequence, sendPacketSlot)

			sendPacketEvent, err = solana.GetSendPacketEventFromTransaction(ctx, s.Solana.Chain.RPCClient, solanaSendTxSig)
			s.Require().NoError(err)
			s.Require().NotNil(sendPacketEvent)
		}))

		s.Require().True(s.Run("Verify packet commitment exists on Solana", func() {
			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, solanaSequence)
			packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID), sequenceBytes)

			accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, packetCommitmentPDA, &rpc.GetAccountInfoOpts{
				Commitment: rpc.CommitmentFinalized,
			})
			s.Require().NoError(err)
			s.Require().NotNil(accountInfo.Value)
			s.T().Logf("Solana packet commitment verified for sequence %d", solanaSequence)
		}))

		s.Require().True(s.Run("Verify Solana attestor can attest packet", func() {
			abiPacket := convertSolanaPacketToABI(sendPacketEvent.Packet)
			packetBytes, err := types.AbiEncodePacket(abiPacket)
			s.Require().NoError(err)

			resp, err := attestor.GetPacketAttestation(ctx, s.SolanaAttestorClient, [][]byte{packetBytes}, sendPacketSlot)
			s.Require().NoError(err)
			s.Require().NotNil(resp.GetAttestation())

			attestation := resp.GetAttestation()
			s.Require().NotEmpty(attestation.GetSignature())
			s.Require().Equal(sendPacketSlot, attestation.GetHeight())

			s.T().Log("Solana attestor packet verification successful")
		}))

		var cosmosRecvTxHash string

		s.Require().True(s.Run("Relay packet to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaSendTxSig.String())},
				SrcClientId: s.AttestationClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, cosmosUserWallet, CosmosDefaultGasLimit, resp.Tx)
			cosmosRecvTxHash = relayTxResult.TxHash
			s.T().Logf("Packet relayed to Cosmos - tx: %s", cosmosRecvTxHash)
		}))

		s.Require().True(s.Run("Verify packet receipt on Cosmos", func() {
			receiptResp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketReceiptResponse](ctx, simd, &channeltypesv2.QueryPacketReceiptRequest{
				ClientId: CosmosClientID,
				Sequence: sendPacketEvent.Sequence,
			})
			s.Require().NoError(err)
			s.Require().True(receiptResp.Received)
			s.T().Logf("Cosmos packet receipt verified for sequence %d", sendPacketEvent.Sequence)
		}))

		s.Require().True(s.Run("Relay ACK to Solana", func() {
			cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    simd.Config().ChainID,
				DstChain:    testvalues.SolanaChainID,
				SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
				SrcClientId: CosmosClientID,
				DstClientId: s.AttestationClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			ackTxSig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaUser)
			s.Require().NoError(err)
			s.T().Logf("ACK relayed to Solana - tx: %s", ackTxSig)
		}))

		s.Require().True(s.Run("Verify packet commitment deleted on Solana", func() {
			s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), s.AttestationClientID, solanaSequence)
			s.T().Logf("Solana packet commitment deleted for sequence %d", solanaSequence)
		}))
	}))

	s.Require().True(s.Run("Verify final state", func() {
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   transferCoin.Denom,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		finalBalance := resp.Balance.Amount.Int64()

		s.T().Logf("Final Cosmos balance: %d %s (initial: %d)", finalBalance, transferCoin.Denom, initialCosmosBalance)
		s.T().Log("Roundtrip complete: Cosmos → Solana → Cosmos")
	}))
}

func convertSolanaPacketToABI(packet solana.SolanaPacket) ics26router.IICS26RouterMsgsPacket {
	payloads := make([]ics26router.IICS26RouterMsgsPayload, len(packet.Payloads))
	for i, p := range packet.Payloads {
		payloads[i] = ics26router.IICS26RouterMsgsPayload{
			SourcePort: p.SourcePort,
			DestPort:   p.DestPort,
			Version:    p.Version,
			Encoding:   p.Encoding,
			Value:      p.Value,
		}
	}

	return ics26router.IICS26RouterMsgsPacket{
		Sequence:         packet.Sequence,
		SourceClient:     packet.SourceClient,
		DestClient:       packet.DestClient,
		TimeoutTimestamp: uint64(packet.TimeoutTimestamp),
		Payloads:         payloads,
	}
}

// Test_Attestation_TimeoutFromSolana tests the CosmosToSolanaAttested timeout path.
// A packet is sent from Solana with a short timeout. After the timeout expires,
// the relayer proves non-membership on Cosmos and delivers the timeout to Solana
// via the attestation light client.
func (s *IbcSolanaAttestationTestSuite) Test_Attestation_TimeoutFromSolana() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	simd := s.Cosmos.Chains[0]
	solanaUserAddress := s.SolanaUser.PublicKey().String()
	cosmosUserAddress := s.Cosmos.Users[0].FormattedAddress()

	routerState, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	ibcApp, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))
	client, _ := solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID))
	appState, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)

	s.Require().True(s.Run("Update attestation client on Solana", func() {
		resp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			DstClientId: s.AttestationClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		var solanaUpdateClient proofapitypes.SolanaUpdateClient
		err = proto.Unmarshal(resp.Tx, &solanaUpdateClient)
		s.Require().NoError(err)
		s.Require().NotEmpty(solanaUpdateClient.AssemblyTx)

		unsignedSolanaTx, err := solanago.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.AssemblyTx))
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, unsignedSolanaTx, rpc.CommitmentFinalized, s.SolanaUser)
		s.Require().NoError(err)
		s.T().Logf("Attestation client updated - tx: %s", sig)
	}))

	var solanaSendTxSig solanago.Signature
	var solanaTimeoutTimestamp uint64
	solanaSequence := uint64(1)

	s.Require().True(s.Run("Send packet from Solana with short timeout", func() {
		sequenceBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(sequenceBytes, solanaSequence)
		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID), sequenceBytes)

		attestationClientStatePDA, _ := solana.Attestation.ClientPDA(attestation.ProgramID)
		attestationConsensusStatePDA := s.deriveAttestationConsensusStatePDA(ctx, attestationClientStatePDA)

		// Query clock as late as possible to minimize drift between query and execution
		solanaClockTime, err := s.Solana.Chain.GetSolanaClockTime(ctx)
		s.Require().NoError(err)

		solanaTimeoutTimestamp = uint64(solanaClockTime + SolanaOriginatedTimeoutSeconds)
		s.T().Logf("Setting timeout to %d (solana_clock=%d + %ds)", solanaTimeoutTimestamp, solanaClockTime, SolanaOriginatedTimeoutSeconds)

		packetMsg := test_ibc_app.TestIbcAppInstructionsSendPacketSendPacketMsg{
			SourceClient:     s.AttestationClientID,
			SourcePort:       transfertypes.PortID,
			DestPort:         transfertypes.PortID,
			Version:          transfertypes.V1,
			Encoding:         "application/json",
			PacketData:       []byte(fmt.Sprintf(`{"denom":"%s","amount":"%d","sender":"%s","receiver":"%s","memo":"timeout-test-from-solana"}`, simd.Config().Denom, TestTransferAmount, solanaUserAddress, cosmosUserAddress)),
			TimeoutTimestamp: solanaTimeoutTimestamp,
			Sequence:         solanaSequence,
		}

		sendPacketInstruction, err := test_ibc_app.NewSendPacketInstruction(
			packetMsg,
			appState,
			s.SolanaUser.PublicKey(),
			routerState,
			ibcApp,
			packetCommitmentPDA,
			client,
			attestation.ProgramID,
			attestationClientStatePDA,
			attestationConsensusStatePDA,
			ics26_router.ProgramID,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		computeBudgetInstruction := solana.NewComputeBudgetInstruction(DefaultComputeUnits)
		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaUser.PublicKey(),
			computeBudgetInstruction,
			sendPacketInstruction,
		)
		s.Require().NoError(err)

		solanaSendTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaUser)
		s.Require().NoError(err)
		s.T().Logf("Packet sent from Solana (will timeout): %s", solanaSendTxSig)
	}))

	s.Require().True(s.Run("Verify packet commitment exists on Solana", func() {
		sequenceBytes := make([]byte, 8)
		binary.LittleEndian.PutUint64(sequenceBytes, solanaSequence)
		packetCommitmentPDA, _ := solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(s.AttestationClientID), sequenceBytes)

		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, packetCommitmentPDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)
		s.T().Logf("Packet commitment verified for sequence %d", solanaSequence)
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		err := e2esuite.WaitForBlockTime(ctx, s.T(), &cosmosutils.Chain{Cosmos: simd}, solanaTimeoutTimestamp)
		s.Require().NoError(err)
		// Wait for attestor consensus state to catch up with chain clock
		time.Sleep(e2esuite.ProofStaleness)
	}))

	s.Require().True(s.Run("Relay timeout from Cosmos to Solana via attested path", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:     simd.Config().ChainID,
			DstChain:     testvalues.SolanaChainID,
			TimeoutTxIds: [][]byte{[]byte(solanaSendTxSig.String())},
			SrcClientId:  CosmosClientID,
			DstClientId:  s.AttestationClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		sig, err := s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaUser)
		s.Require().NoError(err)
		s.T().Logf("Timeout relayed to Solana: %s", sig)
	}))

	s.Require().True(s.Run("Verify packet commitment deleted on Solana", func() {
		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), s.AttestationClientID, solanaSequence)
		s.T().Logf("Packet commitment deleted for sequence %d", solanaSequence)
	}))
}

// Test_Attestation_TimeoutFromCosmos tests the SolanaToCosmosAttested timeout path.
// A packet is sent from Cosmos with a short timeout. After the timeout expires,
// the relayer proves non-membership on Solana and delivers the timeout to Cosmos
// via the attestation light client.
func (s *IbcSolanaAttestationTestSuite) Test_Attestation_TimeoutFromCosmos() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	simd := s.Cosmos.Chains[0]
	cosmosUserWallet := s.Cosmos.Users[0]
	cosmosUserAddress := cosmosUserWallet.FormattedAddress()
	solanaUserAddress := s.SolanaUser.PublicKey().String()
	transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(TestTransferAmount))

	var initialCosmosBalance int64
	s.Require().True(s.Run("Record initial Cosmos balance", func() {
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   transferCoin.Denom,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		initialCosmosBalance = resp.Balance.Amount.Int64()
		s.T().Logf("Initial Cosmos balance: %d %s", initialCosmosBalance, transferCoin.Denom)
	}))

	var cosmosPacketTxHash []byte
	var cosmosTimeoutTimestamp uint64
	cosmosPacketSequence := uint64(1)

	s.Require().True(s.Run("Send ICS20 transfer from Cosmos with short timeout", func() {
		cosmosTimeoutTimestamp = uint64(time.Now().Add(15 * time.Second).Unix())
		s.T().Logf("Setting timeout to %d (now + 15s)", cosmosTimeoutTimestamp)

		transferPayload := transfertypes.FungibleTokenPacketData{
			Denom:    transferCoin.Denom,
			Amount:   transferCoin.Amount.String(),
			Sender:   cosmosUserAddress,
			Receiver: solanaUserAddress,
			Memo:     "timeout-test-from-cosmos",
		}
		encodedPayload, err := transfertypes.MarshalPacketData(transferPayload, transfertypes.V1, transfertypes.EncodingProtobuf)
		s.Require().NoError(err)

		payload := channeltypesv2.Payload{
			SourcePort:      transfertypes.PortID,
			DestinationPort: transfertypes.PortID,
			Version:         transfertypes.V1,
			Encoding:        transfertypes.EncodingProtobuf,
			Value:           encodedPayload,
		}
		msgSendPacket := channeltypesv2.MsgSendPacket{
			SourceClient:     CosmosClientID,
			TimeoutTimestamp: cosmosTimeoutTimestamp,
			Payloads:         []channeltypesv2.Payload{payload},
			Signer:           cosmosUserAddress,
		}

		resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, CosmosDefaultGasLimit, &msgSendPacket)
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.TxHash)

		cosmosPacketTxHash, err = hex.DecodeString(resp.TxHash)
		s.Require().NoError(err)
		s.T().Logf("Cosmos packet sent (will timeout): %s", resp.TxHash)
	}))

	s.Require().True(s.Run("Verify packet commitment exists on Cosmos", func() {
		commitmentResp, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
			ClientId: CosmosClientID,
			Sequence: cosmosPacketSequence,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(commitmentResp.Commitment)
		s.T().Logf("Cosmos packet commitment verified for sequence %d", cosmosPacketSequence)
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		err := e2esuite.WaitForBlockTime(ctx, s.T(), &s.Solana.Chain, cosmosTimeoutTimestamp)
		s.Require().NoError(err)
		// Wait for attestor consensus state to catch up with chain clock
		time.Sleep(e2esuite.ProofStaleness)
	}))

	s.Require().True(s.Run("Relay timeout from Solana to Cosmos via attested path", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:     testvalues.SolanaChainID,
			DstChain:     simd.Config().ChainID,
			TimeoutTxIds: [][]byte{cosmosPacketTxHash},
			SrcClientId:  s.AttestationClientID,
			DstClientId:  CosmosClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, cosmosUserWallet, CosmosDefaultGasLimit, resp.Tx)
		s.T().Logf("Timeout relayed to Cosmos: %s", relayTxResult.TxHash)
	}))

	s.Require().True(s.Run("Verify packet commitment deleted on Cosmos", func() {
		_, err := e2esuite.GRPCQuery[channeltypesv2.QueryPacketCommitmentResponse](ctx, simd, &channeltypesv2.QueryPacketCommitmentRequest{
			ClientId: CosmosClientID,
			Sequence: cosmosPacketSequence,
		})
		s.Require().ErrorContains(err, "packet commitment hash not found")
		s.T().Logf("Cosmos packet commitment deleted for sequence %d", cosmosPacketSequence)
	}))

	s.Require().True(s.Run("Verify Cosmos balance unchanged", func() {
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   transferCoin.Denom,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		s.Require().Equal(initialCosmosBalance, resp.Balance.Amount.Int64(),
			"Cosmos balance should be unchanged after timeout")
		s.T().Logf("Cosmos balance unchanged: %d %s", resp.Balance.Amount.Int64(), transferCoin.Denom)
	}))
}

// Test_Attestation_AccessManagerTransfer tests propose/accept/cancel access manager
// transfer on the attestation light client program.
func (s *IbcSolanaAttestationTestSuite) Test_Attestation_AccessManagerTransfer() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	const keypairDir = "solana-keypairs/localnet"
	const deployerPath = keypairDir + "/deployer_wallet.json"

	// --- Deploy and initialize AM-B ---

	var amBProgramID solanago.PublicKey

	s.Require().True(s.Run("Deploy AM-B (test_access_manager)", func() {
		var err error
		amBKeypairPath := fmt.Sprintf("%s/test_access_manager-keypair.json", keypairDir)
		amBProgramID, err = s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_access_manager", amBKeypairPath, deployerPath)
		s.Require().NoError(err, "failed to deploy test_access_manager")
	}))

	s.Require().True(s.Run("Initialize AM-B with user as admin", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		amBProgramDataPDA, err := solana.GetProgramDataAddress(amBProgramID)
		s.Require().NoError(err)

		savedProgramID := access_manager.ProgramID
		access_manager.ProgramID = amBProgramID
		defer func() { access_manager.ProgramID = savedProgramID }()

		initIx, err := access_manager.NewInitializeInstruction(
			s.SolanaUser.PublicKey(),
			amBAccessManagerPDA,
			s.SolanaUser.PublicKey(),
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
			amBProgramDataPDA,
			solana.DeployerPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaUser, deployerWallet)
		s.Require().NoError(err, "failed to initialize AM-B")
	}))

	// --- Helper: read attestation app state ---

	appStatePDA, _ := solana.Attestation.AppStatePDA(attestation.ProgramID)

	readAppState := func() *attestation.AttestationTypesAppState {
		s.T().Helper()
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, appStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)
		state, err := attestation.ParseAccount_AttestationTypesAppState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)
		return state
	}

	amAAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

	// --- Verify initial state ---

	s.Require().True(s.Run("Verify initial state: AM-A is active, no pending", func() {
		state := readAppState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "Attestation should point to AM-A")
		s.Require().Nil(state.AmState.PendingAccessManager, "No pending transfer initially")
	}))

	// --- Propose transfer to AM-B ---

	s.Require().True(s.Run("Propose access manager transfer to AM-B", func() {
		proposeIx, err := attestation.NewProposeAccessManagerTransferInstruction(
			amBProgramID,
			appStatePDA,
			amAAccessManagerPDA,
			s.SolanaUser.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaUser)
		s.Require().NoError(err, "propose should succeed")
	}))

	s.Require().True(s.Run("Verify: pending set, AM unchanged", func() {
		state := readAppState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "AM should still be AM-A")
		s.Require().NotNil(state.AmState.PendingAccessManager, "Pending should be set")
		s.Require().Equal(amBProgramID, *state.AmState.PendingAccessManager, "Pending should be AM-B")
	}))

	// --- Accept transfer ---

	amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

	s.Require().True(s.Run("Accept access manager transfer (AM-B admin)", func() {
		acceptIx, err := attestation.NewAcceptAccessManagerTransferInstruction(
			appStatePDA,
			amBAccessManagerPDA,
			s.SolanaUser.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), acceptIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaUser)
		s.Require().NoError(err, "accept should succeed")
	}))

	s.Require().True(s.Run("Verify: AM is now AM-B, pending cleared", func() {
		state := readAppState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should now be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after accept")
	}))

	// --- Propose back to AM-A and cancel ---

	s.Require().True(s.Run("Propose transfer back to AM-A", func() {
		proposeIx, err := attestation.NewProposeAccessManagerTransferInstruction(
			access_manager.ProgramID,
			appStatePDA,
			amBAccessManagerPDA,
			s.SolanaUser.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaUser)
		s.Require().NoError(err, "propose back to AM-A should succeed")
	}))

	s.Require().True(s.Run("Cancel pending transfer", func() {
		cancelIx, err := attestation.NewCancelAccessManagerTransferInstruction(
			appStatePDA,
			amBAccessManagerPDA,
			s.SolanaUser.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaUser)
		s.Require().NoError(err, "cancel should succeed")
	}))

	s.Require().True(s.Run("Verify: pending cleared, AM still AM-B", func() {
		state := readAppState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should still be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after cancel")
	}))
}

// deriveAttestationConsensusStatePDA fetches the attestation client state to get the latest height,
// then derives the consensus state PDA.
func (s *IbcSolanaAttestationTestSuite) deriveAttestationConsensusStatePDA(ctx context.Context, clientStatePDA solanago.PublicKey) solanago.PublicKey {
	accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, clientStatePDA, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	s.Require().NoError(err)

	clientState, err := attestation.ParseAccount_AttestationTypesClientState(accountInfo.Value.Data.GetBinary())
	s.Require().NoError(err)

	heightBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(heightBytes, clientState.LatestHeight)

	consensusStatePDA, _ := solana.Attestation.ConsensusStateWithArgSeedPDA(
		attestation.ProgramID,
		heightBytes,
	)
	return consensusStatePDA
}
