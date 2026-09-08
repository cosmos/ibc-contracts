// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"os"
	"strconv"
	"testing"
	"time"

	test_cpi_proxy "github.com/cosmos/solidity-ibc-eureka/e2e/interchaintestv8/solana/go-anchor/testcpiproxy"
	test_gmp_app "github.com/cosmos/solidity-ibc-eureka/e2e/interchaintestv8/solana/go-anchor/testgmpapp"
	test_ibc_app "github.com/cosmos/solidity-ibc-eureka/e2e/interchaintestv8/solana/go-anchor/testibcapp"
	bin "github.com/gagliardetto/binary"
	"github.com/stretchr/testify/suite"

	solanago "github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"
	ibcclientutils "github.com/cosmos/ibc-go/v11/modules/core/02-client/client/utils"
	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"
	tmclient "github.com/cosmos/ibc-go/v11/modules/light-clients/07-tendermint"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	ics07_tendermint_patches "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07_tendermint_patches"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"
	ics27_gmp "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics27gmp"
	ift "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ift"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

const (
	// General
	DefaultTimeoutSeconds = 30
	SolanaClientID        = testvalues.CustomClientID
	CosmosClientID        = testvalues.FirstAttestationsClientID
	// Transfer App
	OneSolInLamports   = 1_000_000_000            // 1 SOL in lamports
	TestTransferAmount = OneSolInLamports / 1_000 // 0.001 SOL in lamports
	SolDenom           = "sol"
	TransferPortID     = transfertypes.PortID
	// Must exceed MIN_TIMEOUT_DURATION (10s) plus tx execution delay.
	SolanaOriginatedTimeoutSeconds = 15
	// Compute Units
	DefaultComputeUnits = uint32(400_000)
	// Cosmos Gas Limits
	CosmosDefaultGasLimit      = uint64(200_000)
	CosmosCreateClientGasLimit = uint64(20_000_000)
)

type IbcEurekaSolanaTestSuite struct {
	e2esuite.TestSuite

	SolanaRelayer *solanago.Wallet

	ProofApiClient        proofapitypes.ProofApiServiceClient
	ICS27GMPProgramID     solanago.PublicKey
	IFTProgramID          solanago.PublicKey
	GMPCounterProgramID   solanago.PublicKey
	TestAppProgramID      solanago.PublicKey
	TestCpiProxyProgramID solanago.PublicKey

	// ALT configuration - if set, will be used when starting the proof API
	SolanaAltAddress string
	ProofApiProcess  *os.Process

	// Signature threshold for skipping pre-verification (nil = use default 50)
	SkipPreVerifyThreshold *int
}

func TestWithIbcEurekaSolanaTestSuite(t *testing.T) {
	suite.Run(t, new(IbcEurekaSolanaTestSuite))
}

func (s *IbcEurekaSolanaTestSuite) TearDownSuite() {
	// Clean up proof API process if it's running
	if s.ProofApiProcess != nil {
		s.T().Logf("Cleaning up proof API process (PID: %d)", s.ProofApiProcess.Pid)
		err := s.ProofApiProcess.Kill()
		if err != nil {
			s.T().Logf("Failed to kill proof API process: %v", err)
		}
	}
}

func (s *IbcEurekaSolanaTestSuite) SetupSuite(ctx context.Context) {
	var err error

	err = os.Chdir("../..")
	s.Require().NoError(err)

	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetType_None)
	os.Setenv(testvalues.EnvKeySolanaTestnetType, testvalues.SolanaTestnetType_Localnet)
	s.TestSuite.SetupSuite(ctx)

	s.T().Log("Waiting for Solana cluster to be ready...")
	err = s.Solana.Chain.WaitForClusterReady(ctx, 30*time.Second)
	s.Require().NoError(err, "Solana cluster failed to initialize")

	simd := s.Cosmos.Chains[0]

	s.Require().True(s.Run("Deploy IBC core contracts", func() {
		solanaUser := solanago.NewWallet()
		s.T().Logf("Created SolanaRelayer wallet: %s", solanaUser.PublicKey())

		fundWallet := func(name string, pubkey solanago.PublicKey, amount uint64) e2esuite.ParallelTask {
			return e2esuite.ParallelTask{
				Name: fmt.Sprintf("Fund %s", name),
				Run: func() error {
					s.T().Logf("Funding %s...", name)
					_, err := s.Solana.Chain.FundUserWithRetry(ctx, pubkey, amount, 5)
					if err == nil {
						s.T().Logf("✓ %s funded: %s", name, pubkey)
					}
					return err
				},
			}
		}

		s.Require().True(s.Run("Fund wallets", func() {
			s.T().Log("Funding wallets in parallel...")
			// Fund single deployer with sufficient funds for all program deployments
			const deployerFunding = 100 * testvalues.InitialSolBalance
			err := e2esuite.RunParallelTasks(
				fundWallet("SolanaRelayer", solanaUser.PublicKey(), testvalues.InitialSolBalance),
				fundWallet("Deployer", solana.DeployerPubkey, deployerFunding),
			)
			s.Require().NoError(err, "Failed to fund wallets")
			s.SolanaRelayer = solanaUser
			s.T().Log("All wallets funded successfully")
		}))

		s.Require().True(s.Run("Deploy programs", func() {
			// Deploy ALL programs in parallel using single deployer
			s.T().Log("Deploying Solana programs in parallel...")

			const keypairDir = "solana-keypairs/localnet"
			const deployerPath = keypairDir + "/deployer_wallet.json"

			deploy := func(displayName, programName string) e2esuite.ParallelTaskWithResult[solanago.PublicKey] {
				return e2esuite.DeploySolanaProgramTask(ctx, s.T(), s.Solana.Chain, displayName, programName, keypairDir, deployerPath)
			}

			deployResults, err := e2esuite.RunParallelTasksWithResults(
				deploy("Deploy Access Manager", "access_manager"),
				deploy("Deploy ICS07 Tendermint", "ics07_tendermint"),
				deploy("Deploy ICS26 Router", "ics26_router"),
				deploy("Deploy ICS27 GMP", "ics27_gmp"),
				deploy("Deploy IFT", "ift"),
				deploy("Deploy GMP Counter App", "test_gmp_app"),
				deploy("Deploy Test IBC App", "test_ibc_app"),
				deploy("Deploy Test CPI Proxy", "test_cpi_proxy"),
			)
			s.Require().NoError(err, "Program deployment failed")

			access_manager.ProgramID = deployResults["Deploy Access Manager"]
			ics07_tendermint.ProgramID = deployResults["Deploy ICS07 Tendermint"]
			ics26_router.ProgramID = deployResults["Deploy ICS26 Router"]
			s.ICS27GMPProgramID = deployResults["Deploy ICS27 GMP"]
			ics27_gmp.ProgramID = s.ICS27GMPProgramID
			s.IFTProgramID = deployResults["Deploy IFT"]
			ift.ProgramID = s.IFTProgramID
			s.GMPCounterProgramID = deployResults["Deploy GMP Counter App"]
			test_gmp_app.ProgramID = s.GMPCounterProgramID
			s.TestAppProgramID = deployResults["Deploy Test IBC App"]
			test_ibc_app.ProgramID = s.TestAppProgramID
			s.TestCpiProxyProgramID = deployResults["Deploy Test CPI Proxy"]
			test_cpi_proxy.ProgramID = s.TestCpiProxyProgramID

			s.T().Log("All programs deployed successfully")
		}))
	}))

	s.Require().True(s.Run("Initialize Access Control", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
		amProgramDataPDA, _ := solana.AccessManager.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
		initInstruction, err := access_manager.NewInitializeInstruction(
			s.SolanaRelayer.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
			amProgramDataPDA,
			solana.DeployerPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initInstruction)
		s.Require().NoError(err)
		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err)
		s.T().Log("Access control initialized")
	}))

	s.Require().True(s.Run("Grant RELAYER_ROLE and ID_CUSTOMIZER_ROLE to SolanaRelayer", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
		const RELAYER_ROLE = uint64(1)
		const ID_CUSTOMIZER_ROLE = uint64(6)

		grantRelayerRoleInstruction, err := access_manager.NewGrantRoleInstruction(
			RELAYER_ROLE,
			s.SolanaRelayer.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		grantIdCustomizerRoleInstruction, err := access_manager.NewGrantRoleInstruction(
			ID_CUSTOMIZER_ROLE,
			s.SolanaRelayer.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), grantRelayerRoleInstruction, grantIdCustomizerRoleInstruction)
		s.Require().NoError(err)
		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Log("Granted RELAYER_ROLE and ID_CUSTOMIZER_ROLE to SolanaRelayer")
	}))

	s.Require().True(s.Run("Initialize ICS26 Router", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
		routerProgramDataPDA, _ := solana.Ics26Router.ProgramDataPDA(solanago.BPFLoaderUpgradeableProgramID)
		initInstruction, err := ics26_router.NewInitializeInstruction(
			access_manager.ProgramID,
			routerStateAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SystemProgramID,
			routerProgramDataPDA,
			solana.DeployerPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initInstruction)
		s.Require().NoError(err)
		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Create Address Lookup Table", func() {
		simd := s.Cosmos.Chains[0]
		cosmosChainID := simd.Config().ChainID
		altAddress := s.Solana.Chain.CreateIBCAddressLookupTable(ctx, s.T(), s.Require(), s.SolanaRelayer, cosmosChainID, GMPPortID, SolanaClientID)
		s.SolanaAltAddress = altAddress.String()
		s.T().Logf("Created Address Lookup Table: %s", s.SolanaAltAddress)
	}))

	attestorResult := attestor.SetupSolanaAttestors(ctx, s.T(), s.GetDockerClient(), s.GetNetworkID(), testvalues.SolanaLocalnetRPC, ics26_router.ProgramID.String())

	if len(attestorResult.Endpoints) > 0 {
		serverAddr := attestorResult.Endpoints[0][len("http://"):]
		_, err := attestor.GetAttestationServiceClient(serverAddr)
		s.Require().NoError(err)
	}

	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		config := proofapi.NewConfigBuilder().
			SolanaToCosmosAttested(proofapi.SolanaToCosmosAttestedParams{
				SolanaChainID:     testvalues.SolanaChainID,
				CosmosChainID:     simd.Config().ChainID,
				SolanaRPC:         testvalues.SolanaLocalnetRPC,
				TmRPC:             simd.GetHostRPCAddress(),
				ICS26ProgramID:    ics26_router.ProgramID.String(),
				SignerAddress:     s.Cosmos.Users[0].FormattedAddress(),
				AttestorEndpoints: attestorResult.Endpoints,
				AttestorTimeout:   30000,
				QuorumThreshold:   testvalues.DefaultMinRequiredSigs,
			}).
			CosmosToSolana(proofapi.CosmosToSolanaParams{
				CosmosChainID:          simd.Config().ChainID,
				SolanaChainID:          testvalues.SolanaChainID,
				SolanaRPC:              testvalues.SolanaLocalnetRPC,
				TmRPC:                  simd.GetHostRPCAddress(),
				ICS07ProgramID:         ics07_tendermint.ProgramID.String(),
				ICS26ProgramID:         ics26_router.ProgramID.String(),
				FeePayer:               s.SolanaRelayer.PublicKey().String(),
				ALTAddress:             s.SolanaAltAddress,
				IFTProgramIDs:          []string{ift.ProgramID.String()},
				SkipPreVerifyThreshold: s.SkipPreVerifyThreshold,
			}).
			Build()

		err := config.GenerateConfigFile(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		proofApiProcess, err = proofapi.StartProofAPI(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		if s.SolanaAltAddress != "" {
			s.T().Logf("Started proof API with ALT address: %s", s.SolanaAltAddress)
		}

		s.T().Cleanup(func() {
			os.Remove(testvalues.ProofAPIConfigFilePath)
		})
	}))

	s.T().Cleanup(func() {
		if proofApiProcess != nil {
			err := proofApiProcess.Kill()
			if err != nil {
				s.T().Logf("Failed to kill the proof API process: %v", err)
			}
		}
	})

	// Wait for proof API to be ready and create client
	s.Require().True(s.Run("Wait for Proof API and Create Client", func() {
		// Create proof API gRPC client
		var err error
		s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
		s.Require().NoError(err, "proof API must be running and accessible")
		s.T().Log("Proof API client created successfully")
	}))

	// Create clients and setup IBC infrastructure
	s.Require().True(s.Run("Setup IBC Clients", func() {
		s.T().Log("Creating IBC clients in parallel...")

		err := e2esuite.RunParallelTasks(
			e2esuite.ParallelTask{
				Name: "Create Tendermint client on Solana",
				Run: func() error {
					s.T().Log("Creating Tendermint Client on Solana...")

					deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
					if err != nil {
						return fmt.Errorf("failed to load deployer wallet: %w", err)
					}

					resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
						SrcChain:   simd.Config().ChainID,
						DstChain:   testvalues.SolanaChainID,
						Parameters: map[string]string{},
					})
					if err != nil {
						return fmt.Errorf("failed to create client tx: %w", err)
					}
					if len(resp.Tx) == 0 {
						return fmt.Errorf("proof API returned empty tx")
					}
					s.T().Logf("proof API created client transaction")

					unsignedSolanaTx, err := solanago.TransactionFromDecoder(bin.NewBinDecoder(resp.Tx))
					if err != nil {
						return fmt.Errorf("failed to decode tx: %w", err)
					}

					sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, unsignedSolanaTx, rpc.CommitmentConfirmed, s.SolanaRelayer, deployerWallet)
					if err != nil {
						return fmt.Errorf("failed to broadcast tx: %w", err)
					}

					s.T().Logf("✓ Tendermint client created on Solana - tx: %s", sig)
					return nil
				},
			},
			e2esuite.ParallelTask{
				Name: "Create attestor client on Cosmos",
				Run: func() error {
					currentFinalizedSlot, err := s.Solana.Chain.RPCClient.GetSlot(ctx, rpc.CommitmentFinalized)
					s.Require().NoError(err)
					solanaTimestamp, err := s.Solana.Chain.RPCClient.GetBlockTime(ctx, currentFinalizedSlot)
					s.Require().NoError(err)
					resp, err := s.ProofApiClient.CreateClient(context.Background(), &proofapitypes.CreateClientRequest{
						SrcChain: testvalues.SolanaChainID,
						DstChain: simd.Config().ChainID,
						Parameters: map[string]string{
							testvalues.ParameterKey_AttestorAddresses: attestorResult.Addresses[0],
							testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(testvalues.DefaultMinRequiredSigs),
							testvalues.ParameterKey_height:            strconv.FormatUint(currentFinalizedSlot, 10),
							testvalues.ParameterKey_timestamp:         strconv.FormatInt(int64(*solanaTimestamp), 10),
						},
					})
					if err != nil {
						return fmt.Errorf("failed to create client tx: %w", err)
					}
					if len(resp.Tx) == 0 {
						return fmt.Errorf("proof API returned empty tx")
					}

					txResp := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], CosmosCreateClientGasLimit, resp.Tx)
					s.T().Logf("✓ Attestor client created on Cosmos - tx: %s", txResp.TxHash)
					return nil
				},
			},
		)
		s.Require().NoError(err, "Failed to create IBC clients")
		s.T().Log("Both IBC clients created successfully")

		// Run final setup steps in parallel
		err = e2esuite.NewParallelExecutor().
			Add("Register counterparty on Cosmos", func() error {
				s.T().Log("Registering counterparty on Cosmos chain...")
				merklePathPrefix := [][]byte{[]byte("")}

				_, err := s.BroadcastMessages(ctx, simd, s.Cosmos.Users[0], CosmosDefaultGasLimit, &clienttypesv2.MsgRegisterCounterparty{
					ClientId:                 CosmosClientID,
					CounterpartyMerklePrefix: merklePathPrefix,
					CounterpartyClientId:     SolanaClientID,
					Signer:                   s.Cosmos.Users[0].FormattedAddress(),
				})
				if err != nil {
					return fmt.Errorf("failed to register counterparty: %w", err)
				}
				s.T().Log("Counterparty registered on Cosmos")
				return nil
			}).
			Add("Add Client to Router on Solana", func() error {
				s.T().Log("Adding client to Router on Solana...")
				routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
				accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
				clientAccount, _ := solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))

				counterpartyInfo := ics26_router.SolanaIbcTypesRouterCounterpartyInfo{
					ClientId:     CosmosClientID,
					MerklePrefix: [][]byte{[]byte(ibcexported.StoreKey), []byte("")},
				}

				addClientInstruction, err := ics26_router.NewAddClientInstruction(
					SolanaClientID,
					counterpartyInfo,
					s.SolanaRelayer.PublicKey(),
					routerStateAccount,
					accessControlAccount,
					clientAccount,
					ics07_tendermint.ProgramID,
					solanago.SystemProgramID,
					solanago.SysVarInstructionsPubkey,
				)
				if err != nil {
					return fmt.Errorf("failed to create add client instruction: %w", err)
				}

				tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), addClientInstruction)
				if err != nil {
					return fmt.Errorf("failed to create transaction: %w", err)
				}

				// Use confirmed commitment - proof API reads Solana state with confirmed commitment
				_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
				if err != nil {
					return fmt.Errorf("failed to broadcast tx: %w", err)
				}
				s.T().Logf("Client added to router")
				return nil
			}).
			Run()
		s.Require().NoError(err)
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_Deploy() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	simd := s.Cosmos.Chains[0]

	s.Require().True(s.Run("Verify ics07-svm-tendermint", func() {
		clientStateAccount, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)

		// Use confirmed commitment to match client creation confirmation level
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, clientStateAccount, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)

		clientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		s.Require().Equal(simd.Config().ChainID, clientState.ChainId)
		s.Require().Equal(testvalues.DefaultTrustLevel.Denominator, clientState.TrustLevelDenominator)
		s.Require().Equal(testvalues.DefaultTrustLevel.Numerator, clientState.TrustLevelNumerator)
		s.Require().Equal(uint64(testvalues.DefaultTrustPeriod), clientState.TrustingPeriod)
		s.Require().True(clientState.UnbondingPeriod > clientState.TrustingPeriod)
		s.Require().Equal(uint64(testvalues.DefaultMaxClockDrift), clientState.MaxClockDrift)
		s.Require().Equal(uint64(1), clientState.LatestHeight.RevisionNumber)
		s.Require().Equal(uint64(0), clientState.FrozenHeight.RevisionNumber)
		s.Require().Equal(uint64(0), clientState.FrozenHeight.RevisionHeight)
	}))

	s.Require().True(s.Run("Test Proof API Info", func() {
		if s.ProofApiClient == nil {
			s.T().Skip("Proof API client not available, skipping info test")
			return
		}

		resp, err := s.ProofApiClient.Info(context.Background(), &proofapitypes.InfoRequest{
			SrcChain: testvalues.SolanaChainID,
			DstChain: simd.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp)

		s.T().Logf("Proof API Info - Source Chain: %+v", resp.SourceChain)
		s.T().Logf("Proof API Info - Target Chain: %+v", resp.TargetChain)
		s.T().Logf("Proof API Info - Metadata: %+v", resp.Metadata)

		s.Require().NotNil(resp.SourceChain, "Source chain info must be present")
		s.Require().Equal(testvalues.SolanaChainID, resp.SourceChain.ChainId)

		s.Require().NotNil(resp.TargetChain, "Target chain info must be present")
		s.Require().Equal(simd.Config().ChainID, resp.TargetChain.ChainId)
	}))
}

func (s *IbcEurekaSolanaTestSuite) setupTestApp(ctx context.Context) {
	s.Require().True(s.Run("Initialize Test IBC App", func() {
		appStateAccount, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)

		initInstruction, err := test_ibc_app.NewInitializeInstruction(
			s.SolanaRelayer.PublicKey(),
			appStateAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initInstruction)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Test app initialized at: %s", s.TestAppProgramID)
	}))

	s.Require().True(s.Run("Register Test App with Router", func() {
		routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		ibcAppAccount, _ := solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))

		registerInstruction, err := ics26_router.NewAddIbcAppInstruction(
			transfertypes.PortID,
			routerStateAccount,
			accessControlAccount,
			ibcAppAccount,
			s.TestAppProgramID,
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
		s.T().Logf("Test app registered with router on transfer port")
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_SolanaToCosmosTransfer_SendPacket() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.setupTestApp(ctx)

	simd := s.Cosmos.Chains[0]

	var solanaTxSig solanago.Signature
	var cosmosPacketRelayTxHash []byte
	var sentPacketSequence uint64

	s.Require().True(s.Run("Send ICS20 transfer using send_packet", func() {
		initialBalance := s.SolanaRelayer.PublicKey()
		balanceResp, err := s.Solana.Chain.RPCClient.GetBalance(ctx, initialBalance, "confirmed")
		s.Require().NoError(err)
		initialLamports := balanceResp.Value

		s.T().Logf("Initial SOL balance: %d lamports", initialLamports)

		cosmosUserWallet := s.Cosmos.Users[0]
		receiver := cosmosUserWallet.FormattedAddress()

		transferData := transfertypes.NewFungibleTokenPacketData(
			SolDenom,                              // denom
			fmt.Sprintf("%d", TestTransferAmount), // amount as string
			s.SolanaRelayer.PublicKey().String(),  // sender
			receiver,                              // receiver
			"Test via send_packet",                // memo
		)
		packetData := transferData.GetBytes()

		var appState, routerState, ibcApp, client, packetCommitment solanago.PublicKey
		sentPacketSequence = 1
		s.Require().True(s.Run("Prepare accounts", func() {
			appState, _ = solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)
			routerState, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
			ibcApp, _ = solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))
			client, _ = solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))

			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, sentPacketSequence)
			packetCommitment, _ = solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), sequenceBytes)
		}))

		packetMsg := test_ibc_app.TestIbcAppInstructionsSendPacketSendPacketMsg{
			SourceClient:     SolanaClientID,
			SourcePort:       transfertypes.PortID,
			DestPort:         transfertypes.PortID,
			Version:          transfertypes.V1,
			Encoding:         "application/json",
			PacketData:       packetData,
			TimeoutTimestamp: uint64(time.Now().Unix() + 3600),
			Sequence:         sentPacketSequence,
		}

		lightClientStatePDA, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, lightClientStatePDA)
		sendPacketInstruction, err := test_ibc_app.NewSendPacketInstruction(
			packetMsg,
			appState,
			s.SolanaRelayer.PublicKey(),
			routerState,
			ibcApp,
			packetCommitment,
			client,
			ics07_tendermint.ProgramID,
			lightClientStatePDA,
			consensusStatePDA,
			ics26_router.ProgramID,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), sendPacketInstruction)
		s.Require().NoError(err)

		solanaTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("send_packet transaction: %s", solanaTxSig)
		s.T().Logf("Sent ICS20 transfer packet with %d bytes of data", len(packetData))

		finalBalance, err := s.Solana.Chain.RPCClient.GetBalance(ctx, s.SolanaRelayer.PublicKey(), "confirmed")
		s.Require().NoError(err)
		s.T().Logf("Final SOL balance: %d lamports (change: %d lamports for fees)", finalBalance.Value, initialLamports-finalBalance.Value)
		s.T().Logf("Note: send_packet sends IBC transfer data without local escrow - tokens should be minted on destination")

		s.T().Logf("Solana packet transaction %s ready for relaying", solanaTxSig)
	}))

	s.Require().True(s.Run("Relay acknowledgment back to Cosmos", func() {
		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			ackRelayTxBodyBz = resp.Tx
			s.T().Logf("Retrieved acknowledgment relay transaction with %d bytes", len(ackRelayTxBodyBz))
		}))

		s.Require().True(s.Run("Broadcast acknowledgment relay tx on Cosmos", func() {
			relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 200_000, ackRelayTxBodyBz)
			s.T().Logf("Acknowledgment relay transaction: %s (code: %d, gas: %d)",
				relayTxResult.TxHash, relayTxResult.Code, relayTxResult.GasUsed)

			txResp, err := simd.GetTransaction(relayTxResult.TxHash)
			s.Require().NoError(err)
			s.T().Logf("Transaction events count: %d", len(txResp.Events))

			cosmosPacketRelayTxHashBytes, err := hex.DecodeString(relayTxResult.TxHash)
			s.Require().NoError(err)
			cosmosPacketRelayTxHash = cosmosPacketRelayTxHashBytes
		}))
	}))

	var denomOnCosmos transfertypes.Denom
	s.Require().True(s.Run("Verify balances on Cosmos chain", func() {
		cosmosUserAddress := s.Cosmos.Users[0].FormattedAddress()

		denomOnCosmos = getSolDenomOnCosmos()

		allBalancesResp, err := e2esuite.GRPCQuery[banktypes.QueryAllBalancesResponse](ctx, simd, &banktypes.QueryAllBalancesRequest{
			Address: cosmosUserAddress,
		})
		s.Require().NoError(err)
		s.T().Logf("All balances for user %s:", cosmosUserAddress)
		for _, balance := range allBalancesResp.Balances {
			s.T().Logf("  - %s: %s", balance.Denom, balance.Amount.String())
		}

		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   denomOnCosmos.IBCDenom(),
		})
		s.Require().NoError(err, "Balances query failed")
		s.Require().NotNil(resp.Balance, "Balance should not be nil")
		s.T().Logf("Current balance for %s: %s %s", denomOnCosmos.IBCDenom(), resp.Balance.Amount.String(), resp.Balance.Denom)

		expectedAmount := sdkmath.NewInt(TestTransferAmount)
		s.Require().Equal(expectedAmount, resp.Balance.Amount)
		s.Require().Equal(denomOnCosmos.IBCDenom(), resp.Balance.Denom)
	}))

	s.Require().True(s.Run("Acknowledge packet on Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosPacketRelayTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)

		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, sentPacketSequence)
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_SolanaToCosmosTransfer_SendTransfer() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.setupTestApp(ctx)

	simd := s.Cosmos.Chains[0]

	var solanaTxSig solanago.Signature
	var cosmosRelayTxHash []byte
	var sentPacketSequence uint64
	s.Require().True(s.Run("Send SOL transfer from Solana", func() {
		initialBalance := s.SolanaRelayer.PublicKey()
		balanceResp, err := s.Solana.Chain.RPCClient.GetBalance(ctx, initialBalance, "confirmed")
		s.Require().NoError(err)
		initialLamports := balanceResp.Value

		s.T().Logf("Initial SOL balance: %d lamports", initialLamports)

		transferAmount := fmt.Sprintf("%d", TestTransferAmount)
		cosmosUserWallet := s.Cosmos.Users[0]
		receiver := cosmosUserWallet.FormattedAddress()
		memo := "Test transfer from Solana to Cosmos"

		var appState, routerState, ibcApp, client, packetCommitment, escrow, escrowState solanago.PublicKey
		sentPacketSequence = 1
		s.Require().True(s.Run("Prepare accounts", func() {
			appState, _ = solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)
			routerState, _ = solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
			ibcApp, _ = solana.Ics26Router.IbcAppWithArgSeedPDA(ics26_router.ProgramID, []byte(transfertypes.PortID))
			client, _ = solana.Ics26Router.ClientWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID))

			sequenceBytes := make([]byte, 8)
			binary.LittleEndian.PutUint64(sequenceBytes, sentPacketSequence)
			packetCommitment, _ = solana.Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(SolanaClientID), sequenceBytes)

			escrow, _ = solana.TestIbcApp.EscrowWithArgSeedPDA(s.TestAppProgramID, []byte(SolanaClientID))
			escrowState, _ = solana.TestIbcApp.EscrowStateWithArgSeedPDA(s.TestAppProgramID, []byte(SolanaClientID))
		}))

		timeoutTimestamp := uint64(time.Now().Unix() + 3600)

		transferMsg := test_ibc_app.TestIbcAppInstructionsSendTransferSendTransferMsg{
			Denom:            SolDenom,
			Amount:           transferAmount,
			Receiver:         receiver,
			SourceClient:     SolanaClientID,
			DestPort:         transfertypes.PortID,
			TimeoutTimestamp: timeoutTimestamp,
			Memo:             memo,
			Sequence:         sentPacketSequence,
		}

		lightClientStatePDA, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
		consensusStatePDA := s.deriveIcs07ConsensusStatePDA(ctx, lightClientStatePDA)
		sendTransferInstruction, err := test_ibc_app.NewSendTransferInstruction(
			transferMsg,
			appState,
			s.SolanaRelayer.PublicKey(),
			escrow,
			escrowState,
			routerState,
			ibcApp,
			packetCommitment,
			client,
			ics07_tendermint.ProgramID,
			lightClientStatePDA,
			consensusStatePDA,
			ics26_router.ProgramID,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		computeBudgetInstruction := solana.NewComputeBudgetInstruction(400000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			computeBudgetInstruction,
			sendTransferInstruction,
		)
		s.Require().NoError(err)

		solanaTxSig, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Transfer transaction sent: %s", solanaTxSig)

		finalLamports, balanceChanged := s.Solana.Chain.WaitForBalanceChange(ctx, s.SolanaRelayer.PublicKey(), initialLamports)
		s.Require().True(balanceChanged, "Balance should change after transfer")

		s.T().Logf("Final SOL balance: %d lamports", finalLamports)
		s.T().Logf("SOL transferred: %d lamports", initialLamports-finalLamports)

		s.Require().Less(finalLamports, initialLamports, "Balance should decrease after transfer")

		escrowBalance, balanceChanged := s.Solana.Chain.WaitForBalanceChange(ctx, escrow, 0)
		s.Require().True(balanceChanged, "Escrow account should receive SOL")

		s.T().Logf("Escrow account balance: %d lamports", escrowBalance)

		expectedAmount := uint64(TestTransferAmount)
		s.Require().Equal(escrowBalance, expectedAmount,
			"Escrow should contain exactly the transferred amount")

		s.T().Logf("Solana transaction %s ready for relaying to Cosmos", solanaTxSig)
	}))

	s.Require().True(s.Run("Relay transfer to Cosmos", func() {
		var relayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			relayTxBodyBz = resp.Tx
			s.T().Logf("Retrieved relay transaction with %d bytes", len(relayTxBodyBz))
		}))

		s.Require().True(s.Run("Broadcast relay tx on Cosmos", func() {
			relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 200_000, relayTxBodyBz)
			s.T().Logf("Relay transaction: %s (code: %d, gas: %d)",
				relayTxResult.TxHash, relayTxResult.Code, relayTxResult.GasUsed)

			cosmosRelayTxHashBytes, err := hex.DecodeString(relayTxResult.TxHash)
			s.Require().NoError(err)
			cosmosRelayTxHash = cosmosRelayTxHashBytes
		}))
	}))

	s.Require().True(s.Run("Verify transfer completion on Cosmos", func() {
		ibc_sol_denom := getSolDenomOnCosmos()

		cosmosUserAddress := s.Cosmos.Users[0].FormattedAddress()
		resp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
			Address: cosmosUserAddress,
			Denom:   ibc_sol_denom.IBCDenom(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(resp.Balance)
		s.Require().Equal(sdkmath.NewIntFromUint64(TestTransferAmount), resp.Balance.Amount)
		s.Require().Equal(ibc_sol_denom.IBCDenom(), resp.Balance.Denom)
		s.T().Logf("Verified IBC SOL balance on Cosmos: %s %s", resp.Balance.Amount.String(), resp.Balance.Denom)
	}))

	s.Require().True(s.Run("Acknowledge transfer on Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRelayTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		_, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)

		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, sentPacketSequence)
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_CosmosToSolanaTransfer() {
	s.runCosmosToSolanaTransfer(nil) // nil = use default threshold (optimized path)
}

// Test_CosmosToSolanaTransfer_WithPreVerify tests the full path with pre-verification
// and Address Lookup Table (ALT) by setting skip_pre_verify_threshold to 0.
func (s *IbcEurekaSolanaTestSuite) Test_CosmosToSolanaTransfer_WithPreVerify() {
	threshold := 0
	s.runCosmosToSolanaTransfer(&threshold) // 0 = force pre-verify + ALT path
}

func (s *IbcEurekaSolanaTestSuite) runCosmosToSolanaTransfer(skipPreVerifyThreshold *int) {
	ctx := context.Background()

	s.SkipPreVerifyThreshold = skipPreVerifyThreshold

	s.SetupSuite(ctx)
	s.setupTestApp(ctx)

	simd := s.Cosmos.Chains[0]

	var cosmosRelayPacketTxHash []byte
	var solanaRelayTxSig solanago.Signature

	s.Require().True(s.Run("Send ICS20 transfer from Cosmos to Solana", func() {
		cosmosUserWallet := s.Cosmos.Users[0]
		cosmosUserAddress := cosmosUserWallet.FormattedAddress()
		solanaUserAddress := s.SolanaRelayer.PublicKey().String()
		transferCoin := sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(TestTransferAmount))

		var initialBalance int64
		s.Require().True(s.Run("Verify balances on Cosmos before transfer", func() {
			initialResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(initialResp.Balance)
			initialBalance = initialResp.Balance.Amount.Int64()
			s.T().Logf("Initial Cosmos balance: %d %s", initialBalance, transferCoin.Denom)
		}))

		s.Require().True(s.Run("Send transfer packet from Cosmos", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())

			transferPayload := transfertypes.FungibleTokenPacketData{
				Denom:    transferCoin.Denom,
				Amount:   transferCoin.Amount.String(),
				Sender:   cosmosUserAddress,
				Receiver: solanaUserAddress,
				Memo:     "cosmos-to-solana-transfer",
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
				Payloads: []channeltypesv2.Payload{
					payload,
				},
				Signer: cosmosUserWallet.FormattedAddress(),
			}

			resp, err := s.BroadcastMessages(ctx, simd, cosmosUserWallet, 200_000, &msgSendPacket)
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.TxHash)

			cosmosPacketTxHashBytes, err := hex.DecodeString(resp.TxHash)
			s.Require().NoError(err)
			cosmosRelayPacketTxHash = cosmosPacketTxHashBytes

			s.T().Logf("Cosmos packet transaction sent: %s", resp.TxHash)
		}))

		s.Require().True(s.Run("Verify balances on Cosmos after transfer", func() {
			finalResp, err := e2esuite.GRPCQuery[banktypes.QueryBalanceResponse](ctx, simd, &banktypes.QueryBalanceRequest{
				Address: cosmosUserAddress,
				Denom:   transferCoin.Denom,
			})
			s.Require().NoError(err)
			s.Require().NotNil(finalResp.Balance)
			finalBalance := finalResp.Balance.Amount.Int64()
			s.T().Logf("Final Cosmos balance: %d %s (transferred: %d)", finalBalance, transferCoin.Denom, initialBalance-finalBalance)
			s.Require().Equal(initialBalance-TestTransferAmount, finalBalance, "Balance should decrease by transfer amount")
		}))
	}))

	s.Require().True(s.Run("Acknowledge packet on Solana", func() {
		resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
			SrcChain:    simd.Config().ChainID,
			DstChain:    testvalues.SolanaChainID,
			SourceTxIds: [][]byte{cosmosRelayPacketTxHash},
			SrcClientId: CosmosClientID,
			DstClientId: SolanaClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx, "Relay should return transaction")

		solanaRelayTxSig, err = s.Solana.Chain.SubmitChunkedRelayPackets(ctx, s.T(), resp, s.SolanaRelayer)
		s.Require().NoError(err)

		s.Solana.Chain.VerifyPacketCommitmentDeleted(ctx, s.T(), s.Require(), SolanaClientID, 1)
	}))

	s.Require().True(s.Run("Verify packet received on Solana", func() {
		// Check that the dummy app state was updated
		testAppStateAccount, _ := solana.TestIbcApp.AppStatePDA(s.TestAppProgramID)

		// Use confirmed commitment to match relay transaction confirmation level
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, testAppStateAccount, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)

		appState, err := test_ibc_app.ParseAccount_TestIbcAppStateTestIbcAppState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		s.Require().Greater(appState.PacketsReceived, uint64(0), "Test app should have received at least one packet")
		s.T().Logf("Solana dummy app has received %d packets total", appState.PacketsReceived)
	}))

	s.Require().True(s.Run("Verify balances on Solana", func() {
		s.T().Logf("SKIPPED: Solana balance verification not applicable for dummy IBC app")
		s.T().Logf("The dummy app only processes packets without actual token transfers")
	}))

	s.Require().True(s.Run("Relay acknowledgment back to Cosmos", func() {
		var ackRelayTxBodyBz []byte
		s.Require().True(s.Run("Retrieve relay tx", func() {
			resp, err := s.ProofApiClient.RelayByTx(context.Background(), &proofapitypes.RelayByTxRequest{
				SrcChain:    testvalues.SolanaChainID,
				DstChain:    simd.Config().ChainID,
				SourceTxIds: [][]byte{[]byte(solanaRelayTxSig.String())},
				SrcClientId: SolanaClientID,
				DstClientId: CosmosClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)
			s.Require().Empty(resp.Address)

			ackRelayTxBodyBz = resp.Tx
			s.T().Logf("Retrieved acknowledgment relay transaction with %d bytes", len(ackRelayTxBodyBz))
		}))

		s.Require().True(s.Run("Broadcast acknowledgment relay tx on Cosmos", func() {
			relayTxResult := s.MustBroadcastSdkTxBody(ctx, simd, s.Cosmos.Users[0], 200_000, ackRelayTxBodyBz)
			s.T().Logf("Acknowledgment relay transaction: %s (code: %d, gas: %d)",
				relayTxResult.TxHash, relayTxResult.Code, relayTxResult.GasUsed)
		}))
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_CleanupOrphanedChunks() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.setupTestApp(ctx)

	testClientID := SolanaClientID
	testSequence := uint64(99999)
	relayer := s.SolanaRelayer.PublicKey()

	payloadData0 := []byte("payload chunk 0 data for testing orphaned chunks cleanup")
	payloadData1 := []byte("payload chunk 1 data for testing orphaned chunks cleanup")
	proofData0 := []byte("proof chunk 0 data for testing orphaned chunks cleanup")
	proofData1 := []byte("proof chunk 1 data for testing orphaned chunks cleanup")

	// PayloadChunk PDA: [b"payload_chunk", relayer, client_id, sequence, payload_idx, chunk_idx]
	// ProofChunk PDA: [b"proof_chunk", relayer, client_id, sequence, chunk_idx]
	sequenceBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(sequenceBytes, testSequence)

	payloadChunk0PDA, _, _ := solanago.FindProgramAddress(
		[][]byte{
			[]byte("payload_chunk"),
			relayer.Bytes(),
			[]byte(testClientID),
			sequenceBytes,
			{0},
			{0},
		},
		ics26_router.ProgramID,
	)

	payloadChunk1PDA, _, _ := solanago.FindProgramAddress(
		[][]byte{
			[]byte("payload_chunk"),
			relayer.Bytes(),
			[]byte(testClientID),
			sequenceBytes,
			{0},
			{1},
		},
		ics26_router.ProgramID,
	)

	proofChunk0PDA, _, _ := solanago.FindProgramAddress(
		[][]byte{
			[]byte("proof_chunk"),
			relayer.Bytes(),
			[]byte(testClientID),
			sequenceBytes,
			{0},
		},
		ics26_router.ProgramID,
	)

	proofChunk1PDA, _, _ := solanago.FindProgramAddress(
		[][]byte{
			[]byte("proof_chunk"),
			relayer.Bytes(),
			[]byte(testClientID),
			sequenceBytes,
			{1},
		},
		ics26_router.ProgramID,
	)

	routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
	accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

	var initialRelayerBalance uint64

	s.Require().True(s.Run("Get initial relayer balance", func() {
		balanceResp, err := s.Solana.Chain.RPCClient.GetBalance(ctx, relayer, rpc.CommitmentConfirmed)
		s.Require().NoError(err)
		initialRelayerBalance = balanceResp.Value
		s.T().Logf("Initial relayer balance: %d lamports", initialRelayerBalance)
	}))

	s.Require().True(s.Run("Upload orphaned payload chunks", func() {
		uploadPayload0Msg := ics26_router.SolanaIbcTypesRouterMsgUploadChunk{
			ClientId:     testClientID,
			Sequence:     testSequence,
			PayloadIndex: 0,
			ChunkIndex:   0,
			ChunkData:    payloadData0,
		}

		uploadPayload0Instruction, err := ics26_router.NewUploadPayloadChunkInstruction(
			uploadPayload0Msg,
			routerStateAccount,
			accessControlAccount,
			payloadChunk0PDA,
			relayer,
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx0, err := s.Solana.Chain.NewTransactionFromInstructions(relayer, uploadPayload0Instruction)
		s.Require().NoError(err)

		sig0, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx0, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Uploaded payload chunk 0: %s", sig0)

		uploadPayload1Msg := ics26_router.SolanaIbcTypesRouterMsgUploadChunk{
			ClientId:     testClientID,
			Sequence:     testSequence,
			PayloadIndex: 0,
			ChunkIndex:   1,
			ChunkData:    payloadData1,
		}

		uploadPayload1Instruction, err := ics26_router.NewUploadPayloadChunkInstruction(
			uploadPayload1Msg,
			routerStateAccount,
			accessControlAccount,
			payloadChunk1PDA,
			relayer,
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx1, err := s.Solana.Chain.NewTransactionFromInstructions(relayer, uploadPayload1Instruction)
		s.Require().NoError(err)

		sig1, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx1, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Uploaded payload chunk 1: %s", sig1)
	}))

	s.Require().True(s.Run("Upload orphaned proof chunks", func() {
		uploadProof0Msg := ics26_router.SolanaIbcTypesRouterMsgUploadChunk{
			ClientId:     testClientID,
			Sequence:     testSequence,
			PayloadIndex: 0, // Not used for proof chunks
			ChunkIndex:   0,
			ChunkData:    proofData0,
		}

		uploadProof0Instruction, err := ics26_router.NewUploadProofChunkInstruction(
			uploadProof0Msg,
			routerStateAccount,
			accessControlAccount,
			proofChunk0PDA,
			relayer,
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx0, err := s.Solana.Chain.NewTransactionFromInstructions(relayer, uploadProof0Instruction)
		s.Require().NoError(err)

		sig0, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx0, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Uploaded proof chunk 0: %s", sig0)

		// Upload proof chunk 1
		uploadProof1Msg := ics26_router.SolanaIbcTypesRouterMsgUploadChunk{
			ClientId:     testClientID,
			Sequence:     testSequence,
			PayloadIndex: 0, // Not used for proof chunks
			ChunkIndex:   1,
			ChunkData:    proofData1,
		}

		uploadProof1Instruction, err := ics26_router.NewUploadProofChunkInstruction(
			uploadProof1Msg,
			routerStateAccount,
			accessControlAccount,
			proofChunk1PDA,
			relayer,
			solanago.SystemProgramID,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx1, err := s.Solana.Chain.NewTransactionFromInstructions(relayer, uploadProof1Instruction)
		s.Require().NoError(err)

		sig1, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx1, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Uploaded proof chunk 1: %s", sig1)
	}))

	s.Require().True(s.Run("Verify chunks exist on-chain", func() {
		// Verify payload chunk 0
		payload0Info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, payloadChunk0PDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(payload0Info.Value, "Payload chunk 0 should exist")
		s.Require().Greater(payload0Info.Value.Lamports, uint64(0), "Payload chunk 0 should have rent")
		s.T().Logf("Payload chunk 0 has %d lamports", payload0Info.Value.Lamports)

		// Verify payload chunk 1
		payload1Info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, payloadChunk1PDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(payload1Info.Value, "Payload chunk 1 should exist")
		s.Require().Greater(payload1Info.Value.Lamports, uint64(0), "Payload chunk 1 should have rent")
		s.T().Logf("Payload chunk 1 has %d lamports", payload1Info.Value.Lamports)

		// Verify proof chunk 0
		proof0Info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, proofChunk0PDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(proof0Info.Value, "Proof chunk 0 should exist")
		s.Require().Greater(proof0Info.Value.Lamports, uint64(0), "Proof chunk 0 should have rent")
		s.T().Logf("Proof chunk 0 has %d lamports", proof0Info.Value.Lamports)

		// Verify proof chunk 1
		proof1Info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, proofChunk1PDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(proof1Info.Value, "Proof chunk 1 should exist")
		s.Require().Greater(proof1Info.Value.Lamports, uint64(0), "Proof chunk 1 should have rent")
		s.T().Logf("Proof chunk 1 has %d lamports", proof1Info.Value.Lamports)
	}))

	s.Require().True(s.Run("Call cleanup_chunks instruction", func() {
		cleanupMsg := ics26_router.SolanaIbcTypesRouterMsgCleanupChunks{
			ClientId:         testClientID,
			Sequence:         testSequence,
			PayloadChunks:    []byte{2},
			TotalProofChunks: 2,
		}

		routerStateAccount, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		cleanupInstruction, err := ics26_router.NewCleanupChunksInstruction(
			cleanupMsg,
			routerStateAccount,
			accessControlAccount,
			relayer,
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		// Chunk accounts must be ordered: all payload chunks (by payload_idx, then chunk_idx), then all proof chunks
		genericInstruction := cleanupInstruction.(*solanago.GenericInstruction)
		genericInstruction.AccountValues = append(genericInstruction.AccountValues,
			solanago.NewAccountMeta(payloadChunk0PDA, true, false),
			solanago.NewAccountMeta(payloadChunk1PDA, true, false),
			solanago.NewAccountMeta(proofChunk0PDA, true, false),
			solanago.NewAccountMeta(proofChunk1PDA, true, false),
		)
		cleanupInstruction = genericInstruction

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(relayer, cleanupInstruction)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Cleanup chunks transaction: %s", sig)
	}))

	s.Require().True(s.Run("Verify chunks are deleted and rent returned", func() {
		finalBalanceResp, err := s.Solana.Chain.RPCClient.GetBalance(ctx, relayer, rpc.CommitmentConfirmed)
		s.Require().NoError(err)
		finalRelayerBalance := finalBalanceResp.Value
		s.T().Logf("Final relayer balance: %d lamports", finalRelayerBalance)

		// Relayer should have recovered most rent (minus transaction fees)
		s.Require().Greater(finalRelayerBalance, initialRelayerBalance-10_000_000,
			"Relayer should have recovered most rent (initial: %d, final: %d)",
			initialRelayerBalance, finalRelayerBalance)
		s.T().Logf("Relayer recovered approximately %d lamports in rent", finalRelayerBalance-initialRelayerBalance)

		// Accounts with 0 lamports may be garbage collected
		chunks := []struct {
			pda  solanago.PublicKey
			name string
		}{
			{payloadChunk0PDA, "Payload chunk 0"},
			{payloadChunk1PDA, "Payload chunk 1"},
			{proofChunk0PDA, "Proof chunk 0"},
			{proofChunk1PDA, "Proof chunk 1"},
		}

		for _, chunk := range chunks {
			info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, chunk.pda, &rpc.GetAccountInfoOpts{
				Commitment: rpc.CommitmentConfirmed,
			})
			if err == nil && info.Value != nil {
				s.Require().Equal(uint64(0), info.Value.Lamports, chunk.name+" should have 0 lamports")
				data := info.Value.Data.GetBinary()
				s.Require().Equal(make([]byte, len(data)), data, chunk.name+" data should be zeroed")
			}
		}
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_CleanupOrphanedTendermintHeaderChunks() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.setupTestApp(ctx)

	testHeight := uint64(99999)
	submitter := s.SolanaRelayer.PublicKey()

	clientStatePDA, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
	appStatePDA, _ := solana.Ics07Tendermint.AppStatePDA(ics07_tendermint.ProgramID)
	ics07AccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

	chunk0Data := []byte("header chunk 0 data for testing orphaned chunks cleanup")
	chunk1Data := []byte("header chunk 1 data for testing orphaned chunks cleanup")

	// HeaderChunk PDA: [b"header_chunk", submitter, target_height, chunk_index]
	heightBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(heightBytes, testHeight)

	chunk0PDA, _, _ := solanago.FindProgramAddress(
		[][]byte{
			[]byte("header_chunk"),
			submitter.Bytes(),
			heightBytes,
			{0},
		},
		ics07_tendermint.ProgramID,
	)

	chunk1PDA, _, _ := solanago.FindProgramAddress(
		[][]byte{
			[]byte("header_chunk"),
			submitter.Bytes(),
			heightBytes,
			{1},
		},
		ics07_tendermint.ProgramID,
	)

	var initialSubmitterBalance uint64

	s.Require().True(s.Run("Get initial submitter balance", func() {
		balanceResp, err := s.Solana.Chain.RPCClient.GetBalance(ctx, submitter, rpc.CommitmentConfirmed)
		s.Require().NoError(err)
		initialSubmitterBalance = balanceResp.Value
		s.T().Logf("Initial submitter balance: %d lamports", initialSubmitterBalance)
	}))

	s.Require().True(s.Run("Upload orphaned header chunks", func() {
		chunk0Params := ics07_tendermint.Ics07TendermintTypesUploadChunkParams{
			TargetHeight: testHeight,
			ChunkIndex:   0,
			ChunkData:    chunk0Data,
		}

		chunk0Instruction, err := ics07_tendermint.NewUploadHeaderChunkInstruction(
			chunk0Params,
			chunk0PDA,
			clientStatePDA,
			appStatePDA,
			ics07AccessManagerPDA,
			submitter,
			solanago.SysVarInstructionsPubkey,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		tx0, err := s.Solana.Chain.NewTransactionFromInstructions(submitter, chunk0Instruction)
		s.Require().NoError(err)

		sig0, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx0, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Uploaded header chunk 0: %s", sig0)

		chunk1Params := ics07_tendermint.Ics07TendermintTypesUploadChunkParams{
			TargetHeight: testHeight,
			ChunkIndex:   1,
			ChunkData:    chunk1Data,
		}

		chunk1Instruction, err := ics07_tendermint.NewUploadHeaderChunkInstruction(
			chunk1Params,
			chunk1PDA,
			clientStatePDA,
			appStatePDA,
			ics07AccessManagerPDA,
			submitter,
			solanago.SysVarInstructionsPubkey,
			solanago.SystemProgramID,
		)
		s.Require().NoError(err)

		tx1, err := s.Solana.Chain.NewTransactionFromInstructions(submitter, chunk1Instruction)
		s.Require().NoError(err)

		sig1, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx1, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Uploaded header chunk 1: %s", sig1)
	}))

	s.Require().True(s.Run("Verify chunks exist on-chain", func() {
		// Verify chunk 0
		chunk0Info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, chunk0PDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(chunk0Info.Value, "Header chunk 0 should exist")
		s.T().Logf("Header chunk 0 has %d lamports", chunk0Info.Value.Lamports)

		// Verify chunk 1
		chunk1Info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, chunk1PDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(chunk1Info.Value, "Header chunk 1 should exist")
		s.T().Logf("Header chunk 1 has %d lamports", chunk1Info.Value.Lamports)
	}))

	s.Require().True(s.Run("Call cleanup_chunks instruction", func() {
		// Use the patched version due to anchor-go bug with no-argument instructions
		// See packages/go-anchor/ics07_tendermint_patches/instructions.go for details
		cleanupInstruction, err := ics07_tendermint_patches.NewCleanupIncompleteUploadInstruction(
			submitter,
		)
		s.Require().NoError(err)

		// Chunk accounts must be ordered by index
		genericInstruction := cleanupInstruction.(*solanago.GenericInstruction)
		genericInstruction.AccountValues = append(genericInstruction.AccountValues,
			solanago.NewAccountMeta(chunk0PDA, true, false),
			solanago.NewAccountMeta(chunk1PDA, true, false),
		)
		cleanupInstruction = genericInstruction

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(submitter, cleanupInstruction)
		s.Require().NoError(err)

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
		s.T().Logf("Cleanup header chunks transaction: %s", sig)
	}))

	s.Require().True(s.Run("Verify chunks are deleted and rent returned", func() {
		finalBalanceResp, err := s.Solana.Chain.RPCClient.GetBalance(ctx, submitter, rpc.CommitmentConfirmed)
		s.Require().NoError(err)
		finalSubmitterBalance := finalBalanceResp.Value
		s.T().Logf("Final submitter balance: %d lamports", finalSubmitterBalance)

		// Submitter should have recovered most rent (minus transaction fees)
		s.Require().Greater(finalSubmitterBalance, initialSubmitterBalance-10_000_000,
			"Submitter should have recovered most rent (initial: %d, final: %d)",
			initialSubmitterBalance, finalSubmitterBalance)

		// Accounts with 0 lamports may be garbage collected
		chunks := []struct {
			pda  solanago.PublicKey
			name string
		}{
			{chunk0PDA, "Header chunk 0"},
			{chunk1PDA, "Header chunk 1"},
		}

		for _, chunk := range chunks {
			info, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, chunk.pda, &rpc.GetAccountInfoOpts{
				Commitment: rpc.CommitmentConfirmed,
			})
			if err == nil && info.Value != nil {
				s.Require().Equal(uint64(0), info.Value.Lamports, chunk.name+" should have 0 lamports")
				data := info.Value.Data.GetBinary()
				s.Require().Equal(make([]byte, len(data)), data, chunk.name+" data should be zeroed")
			}
		}
	}))
}

func (s *IbcEurekaSolanaTestSuite) Test_TendermintSubmitMisbehaviour_DoubleSign() {
	ctx := context.Background()
	s.SetupSuite(ctx)

	simd := s.Cosmos.Chains[0]

	var height clienttypes.Height
	var trustedHeader tmclient.Header
	s.Require().True(s.Run("Get trusted header", func() {
		var latestHeight int64
		var err error
		trustedHeader, latestHeight, err = ibcclientutils.QueryTendermintHeader(simd.Validators[0].CliContext())
		s.Require().NoError(err)
		s.Require().NotZero(latestHeight)

		height = clienttypes.NewHeight(clienttypes.ParseChainID(simd.Config().ChainID), uint64(latestHeight))

		clientStatePDA, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, clientStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)

		clientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		trustedHeight := clienttypes.NewHeight(clienttypes.ParseChainID(simd.Config().ChainID), clientState.LatestHeight.RevisionHeight)

		trustedHeader.TrustedHeight = trustedHeight
		trustedHeader.TrustedValidators = trustedHeader.ValidatorSet
	}))

	s.Require().True(s.Run("Valid misbehaviour - double sign", func() {
		newHeader := s.CreateTMClientHeader(
			ctx,
			simd,
			int64(height.RevisionHeight),
			trustedHeader.GetTime().Add(time.Minute),
			trustedHeader,
		)

		misbehaviour := &tmclient.Misbehaviour{
			Header1: &newHeader,
			Header2: &trustedHeader,
		}

		borshBytes, err := solana.MisbehaviourToBorsh(SolanaClientID, misbehaviour)
		s.Require().NoError(err)

		altAddress := solanago.MustPublicKeyFromBase58(s.SolanaAltAddress)
		altAccounts := s.Solana.Chain.CreateIBCAddressLookupTableAccounts(simd.Config().ChainID, GMPPortID, SolanaClientID, s.SolanaRelayer.PublicKey())
		addressTables := map[solanago.PublicKey]solanago.PublicKeySlice{
			altAddress: altAccounts,
		}

		s.Solana.Chain.SubmitChunkedMisbehaviour(
			ctx,
			s.T(),
			s.Require(),
			borshBytes,
			misbehaviour.Header1.TrustedHeight.RevisionHeight,
			misbehaviour.Header2.TrustedHeight.RevisionHeight,
			s.SolanaRelayer,
			addressTables,
		)

		clientStatePDA, _ := solana.Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, clientStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)

		clientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)

		isFrozen := clientState.FrozenHeight.RevisionHeight > 0 || clientState.FrozenHeight.RevisionNumber > 0
		s.Require().True(isFrozen, "Client should be frozen after misbehaviour submission")
	}))
}

// Helpers

// deriveIcs07ConsensusStatePDA fetches the client state to get the latest revision height,
// then derives the consensus state PDA for the ics07-tendermint light client.
func (s *IbcEurekaSolanaTestSuite) deriveIcs07ConsensusStatePDA(ctx context.Context, clientStatePDA solanago.PublicKey) solanago.PublicKey {
	accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, clientStatePDA, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	s.Require().NoError(err)

	clientState, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesClientState(accountInfo.Value.Data.GetBinary())
	s.Require().NoError(err)

	revisionHeightBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(revisionHeightBytes, clientState.LatestHeight.RevisionHeight)

	consensusStatePDA, _ := solana.Ics07Tendermint.ConsensusStateWithArgSeedPDA(
		ics07_tendermint.ProgramID,
		revisionHeightBytes,
	)
	return consensusStatePDA
}

func getSolDenomOnCosmos() transfertypes.Denom {
	return transfertypes.NewDenom(SolDenom, transfertypes.NewHop("transfer", CosmosClientID))
}
