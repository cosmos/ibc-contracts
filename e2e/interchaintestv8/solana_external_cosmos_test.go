// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	bin "github.com/gagliardetto/binary"
	"github.com/stretchr/testify/suite"

	solanago "github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

// ExternalCosmosTestSuite tests Solana light client with real Cosmos chain
type ExternalCosmosTestSuite struct {
	suite.Suite

	// External Cosmos configuration from environment
	ExternalCosmosRPC     string
	ExternalCosmosChainID string

	// Solana components
	SolanaChain    solana.Solana
	SolanaUser     *solanago.Wallet
	SolanaRPCConn  *rpc.Client
	SolanaLocalnet chainconfig.SolanaLocalnetChain

	// Relayer components
	ProofApiClient  proofapitypes.ProofApiServiceClient
	ProofApiProcess *os.Process

	// ALT configuration
	SolanaAltAddress string
}

func TestExternalCosmos(t *testing.T) {
	rpcURL := os.Getenv("EXTERNAL_COSMOS_RPC_URL")
	chainID := os.Getenv("EXTERNAL_COSMOS_CHAIN_ID")

	if rpcURL == "" || chainID == "" {
		t.Skip("Skipping external Cosmos tests: EXTERNAL_COSMOS_RPC_URL and EXTERNAL_COSMOS_CHAIN_ID not set")
	}

	suite.Run(t, new(ExternalCosmosTestSuite))
}

func (s *ExternalCosmosTestSuite) setupExternalCosmosTest(ctx context.Context) {
	var err error

	s.ExternalCosmosRPC = os.Getenv("EXTERNAL_COSMOS_RPC_URL")
	s.ExternalCosmosChainID = os.Getenv("EXTERNAL_COSMOS_CHAIN_ID")

	s.T().Logf("Using external Cosmos chain: %s at %s", s.ExternalCosmosChainID, s.ExternalCosmosRPC)

	err = os.Chdir("../..")
	s.Require().NoError(err)

	s.T().Log("Starting local Solana test validator...")

	s.SolanaLocalnet, err = chainconfig.StartLocalnet(ctx)
	s.Require().NoError(err, "Failed to start Solana test validator")

	s.T().Logf("Faucet wallet created: %s", s.SolanaLocalnet.Faucet.PublicKey())

	s.T().Cleanup(func() {
		if err := s.SolanaLocalnet.Destroy(); err != nil {
			s.T().Logf("Failed to destroy Solana localnet: %v", err)
		}
	})

	s.SolanaChain, err = solana.NewLocalnetSolana(s.SolanaLocalnet.Faucet)
	s.Require().NoError(err, "Failed to create Solana chain interface")

	balance, err := s.SolanaLocalnet.RPCClient.GetBalance(ctx, s.SolanaLocalnet.Faucet.PublicKey(), rpc.CommitmentConfirmed)
	if err != nil {
		s.T().Logf("Warning: Could not get faucet balance immediately after start: %v", err)
	} else {
		s.T().Logf("Faucet balance after validator start: %d lamports", balance.Value)
	}

	s.T().Log("Waiting for Solana cluster to be ready...")
	err = s.SolanaChain.WaitForClusterReady(ctx, 30*time.Second)
	s.Require().NoError(err, "Solana cluster failed to initialize")

	s.T().Log("Solana test validator started successfully")

	s.SolanaRPCConn = rpc.New(testvalues.SolanaLocalnetRPC)

	s.Require().True(s.Run("Setup Solana Environment", func() {
		s.SolanaUser = solanago.NewWallet()
		s.T().Logf("Created SolanaUser wallet: %s", s.SolanaUser.PublicKey())

		s.T().Log("Funding wallets...")
		const deployerFunding = 100 * testvalues.InitialSolBalance

		err := e2esuite.RunParallelTasks(
			e2esuite.ParallelTask{
				Name: "Fund SolanaUser",
				Run: func() error {
					_, err := s.SolanaChain.FundUserWithRetry(ctx, s.SolanaUser.PublicKey(), testvalues.InitialSolBalance, 5)
					return err
				},
			},
			e2esuite.ParallelTask{
				Name: "Fund Deployer",
				Run: func() error {
					_, err := s.SolanaChain.FundUserWithRetry(ctx, solana.DeployerPubkey, deployerFunding, 5)
					return err
				},
			},
		)
		s.Require().NoError(err, "Failed to fund wallets")

		s.T().Log("Deploying Solana programs...")
		const keypairDir = "solana-keypairs/localnet"
		const deployerPath = keypairDir + "/deployer_wallet.json"

		deployResults, err := e2esuite.RunParallelTasksWithResults(
			e2esuite.ParallelTaskWithResult[solanago.PublicKey]{
				Name: "Deploy Access Manager",
				Run: func() (solanago.PublicKey, error) {
					keypairPath := fmt.Sprintf("%s/access_manager-keypair.json", keypairDir)
					return s.SolanaChain.DeploySolanaProgramAsync(ctx, "access_manager", keypairPath, deployerPath)
				},
			},
			e2esuite.ParallelTaskWithResult[solanago.PublicKey]{
				Name: "Deploy ICS07 Tendermint",
				Run: func() (solanago.PublicKey, error) {
					keypairPath := fmt.Sprintf("%s/ics07_tendermint-keypair.json", keypairDir)
					return s.SolanaChain.DeploySolanaProgramAsync(ctx, "ics07_tendermint", keypairPath, deployerPath)
				},
			},
			e2esuite.ParallelTaskWithResult[solanago.PublicKey]{
				Name: "Deploy ICS26 Router",
				Run: func() (solanago.PublicKey, error) {
					keypairPath := fmt.Sprintf("%s/ics26_router-keypair.json", keypairDir)
					return s.SolanaChain.DeploySolanaProgramAsync(ctx, "ics26_router", keypairPath, deployerPath)
				},
			},
		)
		s.Require().NoError(err, "Program deployment failed")

		access_manager.ProgramID = deployResults["Deploy Access Manager"]
		ics07_tendermint.ProgramID = deployResults["Deploy ICS07 Tendermint"]
		ics26_router.ProgramID = deployResults["Deploy ICS26 Router"]

		s.T().Logf("Access Manager deployed at: %s", access_manager.ProgramID)
		s.T().Logf("ICS07 Tendermint deployed at: %s", ics07_tendermint.ProgramID)
		s.T().Logf("ICS26 Router deployed at: %s", ics26_router.ProgramID)
	}))

	s.Require().True(s.Run("Initialize Access Control", func() {
		const deployerPath = keypairDir + "/deployer_wallet.json"
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
		amProgramDataPDA, _ := solana.AccessManager.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
		initInstruction, err := access_manager.NewInitializeInstruction(
			s.SolanaUser.PublicKey(),
			accessControlAccount,
			s.SolanaUser.PublicKey(),
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
			amProgramDataPDA,
			solana.DeployerPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.SolanaChain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initInstruction)
		s.Require().NoError(err)
		_, err = s.SolanaChain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaUser, deployerWallet)
		s.Require().NoError(err)
		s.T().Log("Access control initialized")
	}))

	s.Require().True(s.Run("Grant RELAYER_ROLE to SolanaUser", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
		const RELAYER_ROLE = uint64(1)

		grantRelayerRoleInstruction, err := access_manager.NewGrantRoleInstruction(
			RELAYER_ROLE,
			s.SolanaUser.PublicKey(),
			accessControlAccount,
			s.SolanaUser.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.SolanaChain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), grantRelayerRoleInstruction)
		s.Require().NoError(err)
		_, err = s.SolanaChain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaUser)
		s.Require().NoError(err)
		s.T().Log("Granted RELAYER_ROLE to SolanaUser")
	}))

	s.Require().True(s.Run("Initialize ICS26 Router", func() {
		s.T().Log("Initializing ICS26 Router...")
		const deployerPath = keypairDir + "/deployer_wallet.json"
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

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

		tx, err := s.SolanaChain.NewTransactionFromInstructions(s.SolanaUser.PublicKey(), initInstruction)
		s.Require().NoError(err)

		_, err = s.SolanaChain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaUser, deployerWallet)
		s.Require().NoError(err)
		s.T().Log("ICS26 Router initialized successfully")
	}))

	s.Require().True(s.Run("Create Address Lookup Table", func() {
		s.T().Log("Creating Address Lookup Table for external Cosmos chain...")
		altAddress := s.SolanaChain.CreateIBCAddressLookupTable(
			ctx, s.T(), s.Require(), s.SolanaUser,
			s.ExternalCosmosChainID, "transfer", s.ExternalCosmosChainID,
		)
		s.SolanaAltAddress = altAddress.String()
		s.T().Logf("Created Address Lookup Table: %s", s.SolanaAltAddress)
	}))

	s.Require().True(s.Run("Start Proof API with External Cosmos", func() {
		s.T().Log("Starting proof API with external Cosmos configuration...")

		modules := []proofapi.ModuleConfig{
			{
				Name:     proofapi.ModuleCosmosToSolana,
				SrcChain: s.ExternalCosmosChainID,
				DstChain: testvalues.SolanaChainID,
				Config: proofapi.CosmosToSolanaModuleConfig{
					SourceRpcUrl:         s.ExternalCosmosRPC,
					TargetRpcUrl:         testvalues.SolanaLocalnetRPC,
					SolanaIcs26ProgramId: ics26_router.ProgramID.String(),
					SolanaFeePayer:       s.SolanaUser.PublicKey().String(),
					SolanaAltAddress:     &s.SolanaAltAddress,
				},
			},
		}
		config := proofapi.NewConfig(modules)

		configPath := "test-proofapi-config.json"
		err := config.GenerateConfigFile(configPath)
		s.Require().NoError(err, "Failed to generate proof API config")

		s.T().Cleanup(func() {
			os.Remove(configPath)
		})

		process, err := proofapi.StartProofAPI(configPath)
		s.Require().NoError(err, "Failed to start proof API")
		s.ProofApiProcess = process
		s.T().Logf("Proof API process started with PID: %d", process.Pid)

		s.T().Log("Waiting for proof API to initialize...")
		time.Sleep(5 * time.Second)

		s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
		s.Require().NoError(err, "Failed to create proof API client")

		s.T().Logf("Proof API started successfully with external Cosmos chain: %s", s.ExternalCosmosChainID)
	}))
}

func (s *ExternalCosmosTestSuite) TearDownSuite() {
	if s.ProofApiProcess != nil {
		s.T().Logf("Cleaning up proof API process (PID: %d)", s.ProofApiProcess.Pid)
		err := s.ProofApiProcess.Kill()
		if err != nil {
			s.T().Logf("Failed to kill proof API process: %v", err)
		}
	}
}

func (s *ExternalCosmosTestSuite) createClient() {
	ctx := context.Background()

	s.T().Logf("Creating Tendermint client for external Cosmos chain: %s", s.ExternalCosmosChainID)
	s.T().Logf("Using RPC endpoint: %s", s.ExternalCosmosRPC)

	createClientReq := &proofapitypes.CreateClientRequest{
		SrcChain: s.ExternalCosmosChainID,
		DstChain: testvalues.SolanaChainID,
		Parameters: map[string]string{
			"trust_level": "1/3",
		},
	}
	s.T().Logf("CreateClient request: SrcChain=%s, DstChain=%s", createClientReq.SrcChain, createClientReq.DstChain)

	resp, err := s.ProofApiClient.CreateClient(ctx, createClientReq)
	s.Require().NoError(err, "Failed to create client transaction")
	s.Require().NotEmpty(resp.Tx, "Proof API returned empty transaction")

	unsignedSolanaTx, err := solanago.TransactionFromDecoder(bin.NewBinDecoder(resp.Tx))
	s.Require().NoError(err, "Failed to decode transaction")

	sig, err := s.SolanaChain.SignAndBroadcastTxWithRetry(ctx, unsignedSolanaTx, rpc.CommitmentConfirmed, s.SolanaUser)
	s.Require().NoError(err, "Failed to broadcast create client transaction")

	s.T().Logf("Successfully created Tendermint client on Solana")
	s.T().Logf("Transaction signature: %s", sig)

	clientStateAccount, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)

	accountInfo, err := s.SolanaRPCConn.GetAccountInfoWithOpts(ctx, clientStateAccount, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	s.Require().NoError(err, "Failed to fetch client state account")

	clientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
	s.Require().NoError(err, "Failed to parse client state")

	s.Require().Equal(s.ExternalCosmosChainID, clientState.ChainId, "Chain ID mismatch")
}

func (s *ExternalCosmosTestSuite) Test_ExternalCosmos_UpdateClient() {
	ctx := context.Background()
	s.setupExternalCosmosTest(ctx)

	s.createClient()

	s.T().Log("Waiting for new blocks on external Cosmos chain...")
	time.Sleep(10 * time.Second)

	s.T().Logf("Updating Tendermint client with new headers from %s", s.ExternalCosmosChainID)

	updateResp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
		SrcChain:    s.ExternalCosmosChainID,
		DstChain:    testvalues.SolanaChainID,
		DstClientId: s.ExternalCosmosChainID,
	})
	s.Require().NoError(err, "Failed to create update client transaction")
	s.Require().NotEmpty(updateResp.Tx, "Proof API returned empty transaction")

	s.T().Log("Submitting chunked update client transactions...")
	s.SolanaChain.SubmitChunkedUpdateClientSkipCleanup(ctx, s.T(), s.Require(), updateResp, s.SolanaUser)

	s.T().Logf("Successfully updated Tendermint client on Solana")

	clientStateAccount, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)

	accountInfo, err := s.SolanaRPCConn.GetAccountInfoWithOpts(ctx, clientStateAccount, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	s.Require().NoError(err, "Failed to fetch updated client state account")

	updatedClientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
	s.Require().NoError(err, "Failed to parse updated client state")

	s.T().Logf("Updated client state:")
	s.T().Logf("  Chain ID: %s", updatedClientState.ChainId)
	s.T().Logf("  Latest Height: %d-%d", updatedClientState.LatestHeight.RevisionNumber, updatedClientState.LatestHeight.RevisionHeight)
	s.T().Logf("  Frozen Height: %d-%d", updatedClientState.FrozenHeight.RevisionNumber, updatedClientState.FrozenHeight.RevisionHeight)

	s.Require().Greater(updatedClientState.LatestHeight.RevisionHeight, uint64(0), "Client height should have increased")
}

func (s *ExternalCosmosTestSuite) Test_ExternalCosmos_MultipleUpdates() {
	ctx := context.Background()
	s.setupExternalCosmosTest(ctx)

	s.createClient()

	numUpdates := 3
	for i := range numUpdates {
		s.T().Logf("Performing update %d/%d", i+1, numUpdates)

		// Wait for new blocks
		time.Sleep(7 * time.Second)

		updateResp, err := s.ProofApiClient.UpdateClient(ctx, &proofapitypes.UpdateClientRequest{
			SrcChain:    s.ExternalCosmosChainID,
			DstChain:    testvalues.SolanaChainID,
			DstClientId: s.ExternalCosmosChainID,
		})
		s.Require().NoError(err, "Failed to create update %d", i+1)

		s.SolanaChain.SubmitChunkedUpdateClientSkipCleanup(ctx, s.T(), s.Require(), updateResp, s.SolanaUser)

		clientStateAccount, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
		accountInfo, err := s.SolanaRPCConn.GetAccountInfoWithOpts(ctx, clientStateAccount, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)

		clientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		s.T().Logf("After update %d, height: %d-%d", i+1, clientState.LatestHeight.RevisionNumber, clientState.LatestHeight.RevisionHeight)
	}

	s.T().Logf("Successfully performed %d consecutive client updates", numUpdates)
}
