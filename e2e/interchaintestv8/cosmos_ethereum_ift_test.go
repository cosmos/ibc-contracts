// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"math/big"
	"os"
	"strconv"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"

	clienttypes "github.com/cosmos/ibc-go/v11/modules/core/02-client/types"
	clienttypesv2 "github.com/cosmos/ibc-go/v11/modules/core/02-client/v2/types"
	ibcexported "github.com/cosmos/ibc-go/v11/modules/core/exported"

	interchaintest "github.com/cosmos/interchaintest/v11"
	interchaintestcosmos "github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"
	evmift "github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ift"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/attestor"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/chainconfig"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/cosmos"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	proofapi "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/proofapi"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
	ifttypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/ift"
	tokenfactorytypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/wfchain/tokenfactory"
)

// CosmosEthereumIFTTestSuite tests IFT transfers between wfchain and Ethereum
type CosmosEthereumIFTTestSuite struct {
	e2esuite.TestSuite

	// Cosmos chain
	Wfchain *interchaintestcosmos.CosmosChain

	// Ethereum contracts
	contractAddresses ethereum.DeployedContracts

	// Keys
	ethDeployer         *ecdsa.PrivateKey
	ethUser             *ecdsa.PrivateKey
	EthRelayerSubmitter *ecdsa.PrivateKey

	// Cosmos wallets
	CosmosRelayerSubmitter ibc.Wallet
	CosmosUser             ibc.Wallet

	// Relayer
	ProofApiClient proofapitypes.ProofApiServiceClient

	// Prover config
	prover    string
	proofType types.SupportedProofType

	// Fixture generators
	solidityFixtureGenerator *types.SolidityFixtureGenerator
	wasmFixtureGenerator     *types.WasmFixtureGenerator

	// Attestor infrastructure (for native attestor light client)
	ethAttestorResult    attestor.SetupResult
	ethAttestorAddresses []string
}

func TestWithCosmosEthereumIFTTestSuite(t *testing.T) {
	suite.Run(t, new(CosmosEthereumIFTTestSuite))
}

func (s *CosmosEthereumIFTTestSuite) SetupSuite(ctx context.Context, proofType types.SupportedProofType) {
	// Use wfchain as the Cosmos chain
	chainconfig.DefaultChainSpecs = []*interchaintest.ChainSpec{
		chainconfig.WfchainChainSpec("wfchain-1", "wfchain-1"),
	}

	// Set up Ethereum chain with native attestor light client
	os.Setenv(testvalues.EnvKeyEthTestnetType, testvalues.EthTestnetTypeAnvil)
	os.Setenv(testvalues.EnvKeyEthLcOnCosmos, testvalues.EthLCOnCosmosTypeAttestorNative)

	s.TestSuite.SetupSuite(ctx)

	eth := s.Eth.Chains[0]
	s.Wfchain = s.Cosmos.Chains[0]
	s.proofType = proofType

	s.T().Logf("Setting up Cosmos-Ethereum IFT test suite with proof type: %s", proofType.String())

	s.Require().True(s.Run("Set up environment", func() {
		err := os.Chdir("../..")
		s.Require().NoError(err)

		s.ethUser, err = eth.CreateAndFundUser()
		s.Require().NoError(err)

		s.EthRelayerSubmitter, err = eth.CreateAndFundUser()
		s.Require().NoError(err)

		operatorKey, err := eth.CreateAndFundUser()
		s.Require().NoError(err)

		s.ethDeployer, err = eth.CreateAndFundUserFromKey(testvalues.E2EDeployerPrivateKeyHex)
		s.Require().NoError(err)

		s.CosmosRelayerSubmitter = s.CreateAndFundCosmosUser(ctx, s.Wfchain)
		s.CosmosUser = s.CreateAndFundCosmosUser(ctx, s.Wfchain)

		s.prover = os.Getenv(testvalues.EnvKeySp1Prover)
		switch s.prover {
		case "", testvalues.EnvValueSp1Prover_Mock:
			s.T().Logf("Using mock prover")
			s.prover = testvalues.EnvValueSp1Prover_Mock
			os.Setenv(testvalues.EnvKeySp1Prover, testvalues.EnvValueSp1Prover_Mock)
			os.Setenv(testvalues.EnvKeyVerifier, testvalues.EnvValueVerifier_Mock)
		case testvalues.EnvValueSp1Prover_Network:
			s.Require().NotEmpty(os.Getenv(testvalues.EnvKeyNetworkPrivateKey))
		default:
			s.Require().Fail("invalid prover type: %s", s.prover)
		}

		if os.Getenv(testvalues.EnvKeyRustLog) == "" {
			os.Setenv(testvalues.EnvKeyRustLog, testvalues.EnvValueRustLog_Info)
		}
		os.Setenv(testvalues.EnvKeyEthRPC, eth.RPC)
		os.Setenv(testvalues.EnvKeyTendermintRPC, s.Wfchain.GetHostRPCAddress())
		os.Setenv(testvalues.EnvKeySp1Prover, s.prover)
		os.Setenv(testvalues.EnvKeyOperatorPrivateKey, hex.EncodeToString(crypto.FromECDSA(operatorKey)))
	}))

	s.wasmFixtureGenerator = types.NewWasmFixtureGenerator(&s.Suite)
	s.solidityFixtureGenerator = types.NewSolidityFixtureGenerator()

	// Generate attestor keys first (before contract deployment)
	s.Require().True(s.Run("Generate attestor keys", func() {
		var err error
		s.ethAttestorAddresses, err = attestor.GenerateAttestorKeys(ctx, attestor.GenerateAttestorKeysParams{
			Client:               s.GetDockerClient(),
			NumKeys:              testvalues.NumAttestors,
			KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		})
		s.Require().NoError(err)
		s.T().Logf("Generated %d attestor keys: %v", len(s.ethAttestorAddresses), s.ethAttestorAddresses)
	}))

	s.Require().True(s.Run("Deploy IBC contracts", func() {
		// Set the ICA address for the CosmosIFTSendCallConstructor deployment
		os.Setenv(testvalues.EnvKeyIFTICAAddress, testvalues.DeterministicICAAddress)

		stdout, err := eth.ForgeScript(s.ethDeployer, testvalues.E2EDeployScriptPath)
		s.Require().NoError(err)

		s.contractAddresses, err = ethereum.GetEthContractsFromDeployOutput(string(stdout))
		s.Require().NoError(err)

		// Log the IFT address (used to determine testvalues.DeterministicIFTAddress)
		s.T().Logf("IFT contract address: %s", s.contractAddresses.Ift)
	}))

	// Start attestors after contract deployment (needs ICS26Router address)
	s.T().Log("Starting Eth attestors...")
	s.ethAttestorResult = attestor.SetupAttestors(ctx, s.T(), attestor.SetupParams{
		NumAttestors:         testvalues.NumAttestors,
		KeystorePathTemplate: testvalues.AttestorKeystorePathTemplate,
		ChainType:            attestor.ChainTypeEvm,
		AdapterURL:           eth.DockerRPC,
		RouterAddress:        s.contractAddresses.Ics26Router,
		DockerClient:         s.GetDockerClient(),
		NetworkID:            s.GetNetworkID(),
	})
	s.T().Logf("Started %d attestors with endpoints: %v, addresses: %v",
		len(s.ethAttestorResult.Endpoints), s.ethAttestorResult.Endpoints, s.ethAttestorResult.Addresses)

	// Verify attestors are reachable before starting proof API
	for i, endpoint := range s.ethAttestorResult.Endpoints {
		err := attestor.CheckAttestorHealth(ctx, endpoint)
		s.Require().NoError(err, "attestor %d at %s is not healthy", i, endpoint)
		s.T().Logf("Attestor %d at %s is healthy", i, endpoint)
	}

	var proofApiProcess *os.Process
	s.Require().True(s.Run("Start Proof API", func() {
		sp1Config := proofapi.SP1ProverConfig{
			Type:           s.prover,
			PrivateCluster: os.Getenv(testvalues.EnvKeyNetworkPrivateCluster) == testvalues.EnvValueSp1Prover_PrivateCluster,
		}

		config := proofapi.NewConfigBuilder().
			EthToCosmosAttested(proofapi.EthToCosmosAttestedParams{
				EthChainID:        eth.ChainID.String(),
				CosmosChainID:     s.Wfchain.Config().ChainID,
				TmRPC:             s.Wfchain.GetHostRPCAddress(),
				ICS26Address:      s.contractAddresses.Ics26Router,
				EthRPC:            eth.RPC,
				SignerAddress:     s.CosmosRelayerSubmitter.FormattedAddress(),
				AttestorEndpoints: s.ethAttestorResult.Endpoints,
				AttestorTimeout:   30000,
				QuorumThreshold:   testvalues.DefaultMinRequiredSigs,
			}).
			CosmosToEthSP1(proofapi.CosmosToEthSP1Params{
				CosmosChainID: s.Wfchain.Config().ChainID,
				EthChainID:    eth.ChainID.String(),
				TmRPC:         s.Wfchain.GetHostRPCAddress(),
				ICS26Address:  s.contractAddresses.Ics26Router,
				EthRPC:        eth.RPC,
				Prover:        sp1Config,
			}).
			Build()

		err := config.GenerateConfigFile(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		proofApiProcess, err = proofapi.StartProofAPI(testvalues.ProofAPIConfigFilePath)
		s.Require().NoError(err)

		s.T().Cleanup(func() {
			os.Remove(testvalues.ProofAPIConfigFilePath)
		})
	}))

	s.T().Cleanup(func() {
		if proofApiProcess != nil {
			_ = proofApiProcess.Kill()
		}
	})

	s.Require().True(s.Run("Create Proof API Client", func() {
		var err error
		s.ProofApiClient, err = proofapi.GetGRPCClient(proofapi.DefaultProofAPIGRPCAddress())
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Verify Proof API Info", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.Wfchain.Config().ChainID,
			DstChain: eth.ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
		s.Require().Equal(s.Wfchain.Config().ChainID, info.SourceChain.ChainId)
		s.Require().Equal(eth.ChainID.String(), info.TargetChain.ChainId)
	}))
}

func (s *CosmosEthereumIFTTestSuite) Test_Deploy() {
	ctx := context.Background()
	s.SetupSuite(ctx, types.ProofTypeGroth16)

	s.Require().True(s.Run("Verify Cosmos chain is running", func() {
		height, err := s.Wfchain.Height(ctx)
		s.Require().NoError(err)
		s.Require().Greater(height, int64(0))
		s.T().Logf("Wfchain height: %d", height)
	}))

	s.Require().True(s.Run("Verify Ethereum chain is running", func() {
		blockNum, err := s.Eth.Chains[0].RPCClient.BlockNumber(ctx)
		s.Require().NoError(err)
		s.Require().Greater(blockNum, uint64(0))
		s.T().Logf("Ethereum block: %d", blockNum)
	}))

	s.Require().True(s.Run("Verify Proof API Info Cosmos->Eth", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.Wfchain.Config().ChainID,
			DstChain: s.Eth.Chains[0].ChainID.String(),
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
	}))

	s.Require().True(s.Run("Verify Proof API Info Eth->Cosmos", func() {
		info, err := s.ProofApiClient.Info(ctx, &proofapitypes.InfoRequest{
			SrcChain: s.Eth.Chains[0].ChainID.String(),
			DstChain: s.Wfchain.Config().ChainID,
		})
		s.Require().NoError(err)
		s.Require().NotNil(info)
	}))

	s.Require().True(s.Run("Verify IFT module on Cosmos", func() {
		resp, err := e2esuite.GRPCQuery[ifttypes.QueryParamsResponse](ctx, s.Wfchain, &ifttypes.QueryParamsRequest{})
		s.Require().NoError(err)
		s.Require().NotNil(resp)
	}))

	s.Require().True(s.Run("Verify IFT contract on Ethereum", func() {
		s.Require().NotEmpty(s.contractAddresses.Ift)
		s.T().Logf("IFT contract: %s", s.contractAddresses.Ift)
	}))
}

type iftTestContext struct {
	ethIFTAddress       ethcommon.Address
	tmClientID          string
	ethClientIDOnCosmos string
	ics26Address        ethcommon.Address
	sp1Ics07Address     ethcommon.Address
	cosmosDenom         string
}

func (s *CosmosEthereumIFTTestSuite) setupIFTInfrastructure(ctx context.Context, prover string, proofType types.SupportedProofType) iftTestContext {
	eth := s.Eth.Chains[0]
	tc := iftTestContext{
		tmClientID:          testvalues.CustomClientID,
		ethClientIDOnCosmos: testvalues.FirstAttestationsClientID,
		ics26Address:        ethcommon.HexToAddress(s.contractAddresses.Ics26Router),
	}

	s.Require().True(s.Run("Setup light clients", func() {
		s.Require().True(s.Run("Create Tendermint light client on Ethereum", func() {
			verifierAddress, err := s.contractAddresses.GetVerifierAddress(prover, proofType.String())
			s.Require().NoError(err)

			var createClientTxBodyBz []byte
			s.Require().True(s.Run("Retrieve create client tx", func() {
				resp, err := s.ProofApiClient.CreateClient(ctx, &proofapitypes.CreateClientRequest{
					SrcChain: s.Wfchain.Config().ChainID,
					DstChain: eth.ChainID.String(),
					Parameters: map[string]string{
						testvalues.ParameterKey_Sp1Verifier: verifierAddress,
						testvalues.ParameterKey_ZkAlgorithm: proofType.String(),
					},
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				createClientTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Broadcast create client tx", func() {
				receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, createClientTxBodyBz)
				s.Require().NoError(err)
				s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
				tc.sp1Ics07Address = receipt.ContractAddress
			}))
		}))

		s.Require().True(s.Run("Create Ethereum light client on Cosmos", func() {
			// Get current Ethereum block for attestor light client initialization
			currentBlockHeader, err := eth.RPCClient.HeaderByNumber(ctx, nil)
			s.Require().NoError(err)
			clientHeight := currentBlockHeader.Number.Int64()
			if clientHeight < 1 {
				clientHeight = 1
			}
			blockHeader, err := eth.RPCClient.HeaderByNumber(ctx, big.NewInt(clientHeight))
			s.Require().NoError(err)

			// Format attestor addresses for registration
			attestorAddresses := s.ethAttestorResult.Addresses[0]
			for i := 1; i < len(s.ethAttestorResult.Addresses); i++ {
				attestorAddresses += "," + s.ethAttestorResult.Addresses[i]
			}

			var createClientTxBodyBz []byte
			s.Require().True(s.Run("Retrieve create client tx", func() {
				parameters := map[string]string{
					testvalues.ParameterKey_AttestorAddresses: attestorAddresses,
					testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(testvalues.DefaultMinRequiredSigs),
					testvalues.ParameterKey_height:            strconv.FormatInt(clientHeight, 10),
					testvalues.ParameterKey_timestamp:         strconv.FormatUint(blockHeader.Time, 10),
				}

				resp, err := s.ProofApiClient.CreateClient(ctx, &proofapitypes.CreateClientRequest{
					SrcChain:   eth.ChainID.String(),
					DstChain:   s.Wfchain.Config().ChainID,
					Parameters: parameters,
				})
				s.Require().NoError(err)
				s.Require().NotEmpty(resp.Tx)
				createClientTxBodyBz = resp.Tx
			}))

			s.Require().True(s.Run("Broadcast relay tx", func() {
				resp := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 20_000_000, createClientTxBodyBz)
				clientId, err := cosmos.GetEventValue(resp.Events, clienttypes.EventTypeCreateClient, clienttypes.AttributeKeyClientID)
				s.Require().NoError(err)
				s.Require().Equal(tc.ethClientIDOnCosmos, clientId)
			}))
		}))

		s.Require().True(s.Run("Add client and counterparty on Ethereum", func() {
			ics26Contract, err := ics26router.NewContract(tc.ics26Address, eth.RPCClient)
			s.Require().NoError(err)

			counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
				ClientId:     tc.ethClientIDOnCosmos,
				MerklePrefix: [][]byte{[]byte(ibcexported.StoreKey), []byte("")},
			}

			txOpts, err := eth.GetTransactOpts(s.ethDeployer)
			s.Require().NoError(err)

			tx, err := ics26Contract.AddClient(txOpts, tc.tmClientID, counterpartyInfo, tc.sp1Ics07Address)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		}))

		s.Require().True(s.Run("Register counterparty on Cosmos", func() {
			merklePathPrefix := [][]byte{[]byte("")}

			_, err := s.BroadcastMessages(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
				ClientId:                 tc.ethClientIDOnCosmos,
				CounterpartyMerklePrefix: merklePathPrefix,
				CounterpartyClientId:     tc.tmClientID,
				Signer:                   s.CosmosRelayerSubmitter.FormattedAddress(),
			})
			s.Require().NoError(err)
		}))
	}))

	var cosmosIftConstructorAddr ethcommon.Address
	s.Require().True(s.Run("Setup IFT bridges", func() {
		s.Require().True(s.Run("Verify IFT contract on Ethereum", func() {
			s.Require().NotEmpty(s.contractAddresses.Ift)
			tc.ethIFTAddress = ethcommon.HexToAddress(s.contractAddresses.Ift)
		}))

		s.Require().True(s.Run("Create denom on Cosmos", func() {
			tc.cosmosDenom = s.createTokenFactoryDenom(ctx, s.CosmosRelayerSubmitter, testvalues.IFTTestDenom)
		}))

		s.Require().True(s.Run("Verify CosmosIFTSendCallConstructor deployment", func() {
			s.Require().NotEmpty(s.contractAddresses.CosmosIftConstructor, "CosmosIFTSendCallConstructor should be deployed by the deploy script")
			cosmosIftConstructorAddr = ethcommon.HexToAddress(s.contractAddresses.CosmosIftConstructor)
			s.T().Logf("Using CosmosIFTSendCallConstructor at: %s", cosmosIftConstructorAddr.Hex())
		}))

		s.Require().True(s.Run("Register IFT bridge on Ethereum", func() {
			cosmosIFTModuleAddr := s.getIFTModuleAddress()

			iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
			s.Require().NoError(err)

			txOpts, err := eth.GetTransactOpts(s.ethDeployer)
			s.Require().NoError(err)

			tx, err := iftContract.RegisterIFTBridge(txOpts, tc.tmClientID, cosmosIFTModuleAddr, cosmosIftConstructorAddr)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		}))

		s.Require().True(s.Run("Register IFT bridge on Cosmos", func() {
			s.registerIFTBridgeOnCosmos(
				ctx,
				s.CosmosRelayerSubmitter,
				tc.cosmosDenom,
				tc.ethClientIDOnCosmos,
				tc.ethIFTAddress.Hex(),
				testvalues.IFTSendCallConstructorEVM,
			)
		}))
	}))

	return tc
}

func (s *CosmosEthereumIFTTestSuite) Test_IFTTransfer_Roundtrip() {
	ctx := context.Background()
	s.SetupSuite(ctx, types.ProofTypeGroth16)

	eth := s.Eth.Chains[0]
	transferAmount := sdkmath.NewInt(1_000_000)

	tc := s.setupIFTInfrastructure(ctx, s.prover, s.proofType)

	s.Require().True(s.Run("Mint tokens to user on Cosmos", func() {
		s.mintTokensOnCosmos(ctx, s.CosmosRelayerSubmitter, tc.cosmosDenom, transferAmount, s.CosmosUser.FormattedAddress())
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount))
	}))

	ethReceiverAddr := crypto.PubkeyToAddress(s.ethUser.PublicKey)
	var cosmosRecvTxHash []byte

	cosmosSequence := uint64(1)
	s.Require().True(s.Run("Transfer Cosmos to Ethereum", func() {
		var cosmosSendTxHash string
		s.Require().True(s.Run("Execute IFT transfer", func() {
			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			cosmosSendTxHash = s.iftTransferFromCosmos(
				ctx,
				s.CosmosUser,
				tc.cosmosDenom,
				tc.ethClientIDOnCosmos,
				ethReceiverAddr.Hex(),
				transferAmount,
				timeout,
			)
			s.Require().NotEmpty(cosmosSendTxHash)

			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
			s.Require().NoError(err)
			s.Require().True(balance.IsZero())
		}))

		s.Require().True(s.Run("Verify pending transfer exists on Cosmos", func() {
			pending, err := s.queryPendingTransferOnCosmos(ctx, tc.cosmosDenom, tc.ethClientIDOnCosmos, cosmosSequence)
			s.Require().NoError(err)
			s.Require().Equal(s.CosmosUser.FormattedAddress(), pending.Sender)
			s.Require().True(pending.Amount.Equal(transferAmount))
		}))

		s.Require().True(s.Run("Relay packet to Ethereum", func() {
			sendTxHashBytes, err := hex.DecodeString(cosmosSendTxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    s.Wfchain.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{sendTxHashBytes},
				SrcClientId: tc.ethClientIDOnCosmos,
				DstClientId: tc.tmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &tc.ics26Address, resp.Tx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
			cosmosRecvTxHash = receipt.TxHash.Bytes()
		}))

		s.Require().True(s.Run("Verify balance on Ethereum", func() {
			iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
			s.Require().NoError(err)

			balance, err := iftContract.BalanceOf(nil, ethReceiverAddr)
			s.Require().NoError(err)
			s.Require().Equal(transferAmount.BigInt().String(), balance.String())
		}))

		s.Require().True(s.Run("Relay ack to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{cosmosRecvTxHash},
				SrcClientId: tc.tmClientID,
				DstClientId: tc.ethClientIDOnCosmos,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 2_000_000, resp.Tx)
		}))

		s.Require().True(s.Run("Verify pending transfer cleared on Cosmos", func() {
			_, err := s.queryPendingTransferOnCosmos(ctx, tc.cosmosDenom, tc.ethClientIDOnCosmos, cosmosSequence)
			s.Require().Error(err, "pending transfer should be cleared after ack")
		}))
	}))

	ethSequence := uint64(1)
	var ethSendTxHash []byte
	var cosmosRecvTxResponse *sdk.TxResponse
	s.Require().True(s.Run("Transfer Ethereum to Cosmos", func() {
		s.Require().True(s.Run("Execute IFT transfer from Ethereum", func() {
			iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
			s.Require().NoError(err)

			txOpts, err := eth.GetTransactOpts(s.ethUser)
			s.Require().NoError(err)

			timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
			tx, err := iftContract.IftTransfer(txOpts, tc.tmClientID, s.CosmosUser.FormattedAddress(), transferAmount.BigInt(), timeout)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
			ethSendTxHash = receipt.TxHash.Bytes()
		}))

		s.Require().True(s.Run("Verify tokens burned on Ethereum", func() {
			iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
			s.Require().NoError(err)

			balance, err := iftContract.BalanceOf(nil, ethReceiverAddr)
			s.Require().NoError(err)
			s.Require().Equal("0", balance.String(), "Ethereum sender should have 0 tokens after transfer")
		}))

		s.Require().True(s.Run("Relay packet to Cosmos", func() {
			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    eth.ChainID.String(),
				DstChain:    s.Wfchain.Config().ChainID,
				SourceTxIds: [][]byte{ethSendTxHash},
				SrcClientId: tc.tmClientID,
				DstClientId: tc.ethClientIDOnCosmos,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			cosmosRecvTxResponse = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 2_000_000, resp.Tx)
		}))

		s.Require().True(s.Run("Verify balance on Cosmos", func() {
			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
			s.Require().NoError(err)
			s.Require().True(balance.Equal(transferAmount), "Expected %s, got %s", transferAmount.String(), balance.String())
		}))

		s.Require().True(s.Run("Relay ack to Ethereum", func() {
			cosmosRecvTxHashBytes, err := hex.DecodeString(cosmosRecvTxResponse.TxHash)
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
				SrcChain:    s.Wfchain.Config().ChainID,
				DstChain:    eth.ChainID.String(),
				SourceTxIds: [][]byte{cosmosRecvTxHashBytes},
				SrcClientId: tc.ethClientIDOnCosmos,
				DstClientId: tc.tmClientID,
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &tc.ics26Address, resp.Tx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		}))

		s.Require().True(s.Run("Verify pending transfer cleared on Ethereum", func() {
			iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
			s.Require().NoError(err)

			_, err = iftContract.GetPendingTransfer(nil, tc.tmClientID, ethSequence)
			s.Require().Error(err, "getPendingTransfer should revert when transfer is cleared")
		}))
	}))

	s.Require().True(s.Run("Verify final balances", func() {
		s.Require().True(s.Run("Cosmos user has tokens back", func() {
			balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
			s.Require().NoError(err)
			s.Require().True(balance.Equal(transferAmount), "Cosmos user should have tokens back after roundtrip")
		}))

		s.Require().True(s.Run("Ethereum user has no tokens", func() {
			iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
			s.Require().NoError(err)

			balance, err := iftContract.BalanceOf(nil, ethReceiverAddr)
			s.Require().NoError(err)
			s.Require().Equal("0", balance.String(), "Ethereum user should have 0 tokens after roundtrip")
		}))
	}))
}

func (s *CosmosEthereumIFTTestSuite) Test_IFTTransfer_TimeoutCosmosToEthereum() {
	ctx := context.Background()
	s.SetupSuite(ctx, types.ProofTypeGroth16)

	eth := s.Eth.Chains[0]
	transferAmount := sdkmath.NewInt(1_000_000)

	tc := s.setupIFTInfrastructure(ctx, s.prover, s.proofType)

	ethUserAddr := crypto.PubkeyToAddress(s.ethUser.PublicKey)

	s.Require().True(s.Run("Mint tokens to user on Cosmos", func() {
		s.mintTokensOnCosmos(ctx, s.CosmosRelayerSubmitter, tc.cosmosDenom, transferAmount, s.CosmosUser.FormattedAddress())
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount))
	}))

	var sendTxHash string
	s.Require().True(s.Run("Send transfer with short timeout", func() {
		timeout := uint64(time.Now().Add(30 * time.Second).Unix())
		sendTxHash = s.iftTransferFromCosmos(
			ctx,
			s.CosmosUser,
			tc.cosmosDenom,
			tc.ethClientIDOnCosmos,
			ethUserAddr.Hex(),
			transferAmount,
			timeout,
		)
		s.Require().NotEmpty(sendTxHash)
	}))

	s.Require().True(s.Run("Verify balance burned on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero())
	}))

	s.Require().True(s.Run("Verify pending transfer exists", func() {
		pending, err := s.queryPendingTransferOnCosmos(ctx, tc.cosmosDenom, tc.ethClientIDOnCosmos, 1)
		s.Require().NoError(err)
		s.Require().Equal(s.CosmosUser.FormattedAddress(), pending.Sender)
		s.Require().True(pending.Amount.Equal(transferAmount))
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		s.T().Log("Waiting 35 seconds for timeout...")
		time.Sleep(35 * time.Second)
	}))

	s.Require().True(s.Run("Relay timeout packet to Cosmos", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:     eth.ChainID.String(),
			DstChain:     s.Wfchain.Config().ChainID,
			TimeoutTxIds: [][]byte{sendTxHashBytes},
			SrcClientId:  tc.tmClientID,
			DstClientId:  tc.ethClientIDOnCosmos,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 2_000_000, resp.Tx)
	}))

	s.Require().True(s.Run("Verify tokens refunded on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount), "Expected %s (refunded), got %s", transferAmount.String(), balance.String())
	}))

	s.Require().True(s.Run("Verify pending transfer cleared", func() {
		_, err := s.queryPendingTransferOnCosmos(ctx, tc.cosmosDenom, tc.ethClientIDOnCosmos, 1)
		s.Require().Error(err, "pending transfer should be cleared after timeout")
	}))

	s.Require().True(s.Run("Verify no balance on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal("0", balance.String(), "Ethereum should have no tokens")
	}))
}

func (s *CosmosEthereumIFTTestSuite) Test_IFTTransfer_TimeoutEthereumToCosmos() {
	ctx := context.Background()
	s.SetupSuite(ctx, types.ProofTypeGroth16)

	eth := s.Eth.Chains[0]
	transferAmount := sdkmath.NewInt(1_000_000)

	tc := s.setupIFTInfrastructure(ctx, s.prover, s.proofType)

	ethUserAddr := crypto.PubkeyToAddress(s.ethUser.PublicKey)

	s.Require().True(s.Run("Mint tokens on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		txOpts, err := eth.GetTransactOpts(s.ethDeployer)
		s.Require().NoError(err)

		tx, err := iftContract.Mint(txOpts, ethUserAddr, transferAmount.BigInt())
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount.BigInt().String(), balance.String())
	}))

	var sendTxHash []byte
	s.Require().True(s.Run("Send transfer with short timeout", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		txOpts, err := eth.GetTransactOpts(s.ethUser)
		s.Require().NoError(err)

		timeout := uint64(time.Now().Add(30 * time.Second).Unix())
		tx, err := iftContract.IftTransfer(txOpts, tc.tmClientID, s.CosmosUser.FormattedAddress(), transferAmount.BigInt(), timeout)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		sendTxHash = receipt.TxHash.Bytes()
	}))

	s.Require().True(s.Run("Verify tokens burned on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal("0", balance.String(), "Tokens should be burned")
	}))

	s.Require().True(s.Run("Verify pending transfer exists on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		pending, err := iftContract.GetPendingTransfer(nil, tc.tmClientID, 1)
		s.Require().NoError(err)
		s.Require().Equal(ethUserAddr, pending.Sender)
		s.Require().Equal(transferAmount.BigInt().String(), pending.Amount.String())
	}))

	s.Require().True(s.Run("Wait for timeout", func() {
		s.T().Log("Waiting 35 seconds for timeout...")
		time.Sleep(35 * time.Second)
	}))

	s.Require().True(s.Run("Relay timeout packet to Ethereum", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:     s.Wfchain.Config().ChainID,
			DstChain:     eth.ChainID.String(),
			TimeoutTxIds: [][]byte{sendTxHash},
			SrcClientId:  tc.ethClientIDOnCosmos,
			DstClientId:  tc.tmClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &tc.ics26Address, resp.Tx)
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	s.Require().True(s.Run("Verify tokens refunded on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount.BigInt().String(), balance.String(), "tokens should be refunded")
	}))

	s.Require().True(s.Run("Verify pending transfer cleared on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		_, err = iftContract.GetPendingTransfer(nil, tc.tmClientID, 1)
		s.Require().Error(err, "getPendingTransfer should revert when transfer is cleared")
	}))
}

// Test_IFTTransfer_FailedReceiveOnCosmos tests error acknowledgment when Cosmos receive fails.
// The test sends from Ethereum to an invalid Cosmos address. The IFT module on Cosmos fails
// to mint tokens because the receiver address is not a valid bech32 address. This generates
// an error ack that is relayed back to Ethereum, triggering a refund to the sender.
func (s *CosmosEthereumIFTTestSuite) Test_IFTTransfer_FailedReceiveOnCosmos() {
	ctx := context.Background()
	s.SetupSuite(ctx, types.ProofTypeGroth16)

	eth := s.Eth.Chains[0]
	transferAmount := sdkmath.NewInt(1_000_000)

	tc := s.setupIFTInfrastructure(ctx, s.prover, s.proofType)

	ethUserAddr := crypto.PubkeyToAddress(s.ethUser.PublicKey)
	invalidCosmosAddr := "wf1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

	s.Require().True(s.Run("Mint tokens on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		txOpts, err := eth.GetTransactOpts(s.ethDeployer)
		s.Require().NoError(err)

		tx, err := iftContract.Mint(txOpts, ethUserAddr, transferAmount.BigInt())
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount.BigInt().String(), balance.String())
	}))

	var sendTxHash []byte
	s.Require().True(s.Run("Send transfer to invalid Cosmos address", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		txOpts, err := eth.GetTransactOpts(s.ethUser)
		s.Require().NoError(err)

		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		tx, err := iftContract.IftTransfer(txOpts, tc.tmClientID, invalidCosmosAddr, transferAmount.BigInt(), timeout)
		s.Require().NoError(err)

		receipt, err := eth.GetTxReciept(ctx, tx.Hash())
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		sendTxHash = receipt.TxHash.Bytes()
	}))

	s.Require().True(s.Run("Verify tokens burned on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal("0", balance.String(), "Tokens should be burned")
	}))

	s.Require().True(s.Run("Verify pending transfer exists on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		pending, err := iftContract.GetPendingTransfer(nil, tc.tmClientID, 1)
		s.Require().NoError(err)
		s.Require().Equal(ethUserAddr, pending.Sender)
		s.Require().Equal(transferAmount.BigInt().String(), pending.Amount.String())
	}))

	var recvTxHash string
	s.Require().True(s.Run("Relay packet to Cosmos (execution fails)", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    eth.ChainID.String(),
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{sendTxHash},
			SrcClientId: tc.tmClientID,
			DstClientId: tc.ethClientIDOnCosmos,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		cosmosRecvTxResponse := s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 2_000_000, resp.Tx)
		recvTxHash = cosmosRecvTxResponse.TxHash
	}))

	s.Require().True(s.Run("Verify no balance minted on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), tc.cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero(), "Cosmos user should have no tokens")
	}))

	s.Require().True(s.Run("Relay error ack to Ethereum", func() {
		recvTxHashBytes, err := hex.DecodeString(recvTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    eth.ChainID.String(),
			SourceTxIds: [][]byte{recvTxHashBytes},
			SrcClientId: tc.ethClientIDOnCosmos,
			DstClientId: tc.tmClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &tc.ics26Address, resp.Tx)
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
	}))

	s.Require().True(s.Run("Verify tokens refunded on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		balance, err := iftContract.BalanceOf(nil, ethUserAddr)
		s.Require().NoError(err)
		s.Require().Equal(transferAmount.BigInt().String(), balance.String(), "tokens should be refunded")
	}))

	s.Require().True(s.Run("Verify pending transfer cleared on Ethereum", func() {
		iftContract, err := evmift.NewContract(tc.ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		_, err = iftContract.GetPendingTransfer(nil, tc.tmClientID, 1)
		s.Require().Error(err, "getPendingTransfer should revert when transfer is cleared")
	}))
}

// Test_IFTTransfer_FailedReceiveOnEthereum tests error acknowledgment when Ethereum receive fails.
// The test intentionally skips registering the IFT bridge on Ethereum while registering it on Cosmos.
// When Cosmos sends an IFT transfer, Ethereum's IFT contract fails because no bridge is registered
// for the client ID. The ICS26 router catches this error and generates an error ack, which is
// relayed back to Cosmos to refund the sender.
func (s *CosmosEthereumIFTTestSuite) Test_IFTTransfer_FailedReceiveOnEthereum() {
	ctx := context.Background()
	s.SetupSuite(ctx, types.ProofTypeGroth16)

	eth := s.Eth.Chains[0]
	transferAmount := sdkmath.NewInt(1_000_000)

	tmClientID := testvalues.CustomClientID
	ethClientIDOnCosmos := testvalues.FirstAttestationsClientID
	ics26Address := ethcommon.HexToAddress(s.contractAddresses.Ics26Router)
	ethIFTAddress := ethcommon.HexToAddress(s.contractAddresses.Ift)

	var sp1Ics07Address ethcommon.Address

	s.Require().True(s.Run("Setup light clients", func() {
		s.Require().True(s.Run("Create Tendermint light client on Ethereum", func() {
			verifierAddress, err := s.contractAddresses.GetVerifierAddress(s.prover, s.proofType.String())
			s.Require().NoError(err)

			resp, err := s.ProofApiClient.CreateClient(ctx, &proofapitypes.CreateClientRequest{
				SrcChain: s.Wfchain.Config().ChainID,
				DstChain: eth.ChainID.String(),
				Parameters: map[string]string{
					testvalues.ParameterKey_Sp1Verifier: verifierAddress,
					testvalues.ParameterKey_ZkAlgorithm: s.proofType.String(),
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, nil, resp.Tx)
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
			sp1Ics07Address = receipt.ContractAddress
		}))

		s.Require().True(s.Run("Create Ethereum light client on Cosmos", func() {
			currentBlockHeader, err := eth.RPCClient.HeaderByNumber(ctx, nil)
			s.Require().NoError(err)
			clientHeight := currentBlockHeader.Number.Int64()
			if clientHeight < 1 {
				clientHeight = 1
			}
			blockHeader, err := eth.RPCClient.HeaderByNumber(ctx, big.NewInt(clientHeight))
			s.Require().NoError(err)

			attestorAddresses := s.ethAttestorResult.Addresses[0]
			for i := 1; i < len(s.ethAttestorResult.Addresses); i++ {
				attestorAddresses += "," + s.ethAttestorResult.Addresses[i]
			}

			resp, err := s.ProofApiClient.CreateClient(ctx, &proofapitypes.CreateClientRequest{
				SrcChain: eth.ChainID.String(),
				DstChain: s.Wfchain.Config().ChainID,
				Parameters: map[string]string{
					testvalues.ParameterKey_AttestorAddresses: attestorAddresses,
					testvalues.ParameterKey_MinRequiredSigs:   strconv.Itoa(testvalues.DefaultMinRequiredSigs),
					testvalues.ParameterKey_height:            strconv.FormatInt(clientHeight, 10),
					testvalues.ParameterKey_timestamp:         strconv.FormatUint(blockHeader.Time, 10),
				},
			})
			s.Require().NoError(err)
			s.Require().NotEmpty(resp.Tx)

			_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 20_000_000, resp.Tx)
		}))

		s.Require().True(s.Run("Add client and counterparty on Ethereum", func() {
			ics26Contract, err := ics26router.NewContract(ics26Address, eth.RPCClient)
			s.Require().NoError(err)

			counterpartyInfo := ics26router.IICS02ClientMsgsCounterpartyInfo{
				ClientId:     ethClientIDOnCosmos,
				MerklePrefix: [][]byte{[]byte(ibcexported.StoreKey), []byte("")},
			}

			txOpts, err := eth.GetTransactOpts(s.ethDeployer)
			s.Require().NoError(err)

			tx, err := ics26Contract.AddClient(txOpts, tmClientID, counterpartyInfo, sp1Ics07Address)
			s.Require().NoError(err)

			receipt, err := eth.GetTxReciept(ctx, tx.Hash())
			s.Require().NoError(err)
			s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		}))

		s.Require().True(s.Run("Register counterparty on Cosmos", func() {
			merklePathPrefix := [][]byte{[]byte("")}

			_, err := s.BroadcastMessages(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 200_000, &clienttypesv2.MsgRegisterCounterparty{
				ClientId:                 ethClientIDOnCosmos,
				CounterpartyMerklePrefix: merklePathPrefix,
				CounterpartyClientId:     tmClientID,
				Signer:                   s.CosmosRelayerSubmitter.FormattedAddress(),
			})
			s.Require().NoError(err)
		}))
	}))

	var cosmosDenom string
	s.Require().True(s.Run("Setup IFT bridge on Cosmos only", func() {
		s.Require().True(s.Run("Create denom on Cosmos", func() {
			cosmosDenom = s.createTokenFactoryDenom(ctx, s.CosmosRelayerSubmitter, testvalues.IFTTestDenom)
		}))

		s.Require().True(s.Run("Register IFT bridge on Cosmos", func() {
			s.registerIFTBridgeOnCosmos(
				ctx,
				s.CosmosRelayerSubmitter,
				cosmosDenom,
				ethClientIDOnCosmos,
				ethIFTAddress.Hex(),
				testvalues.IFTSendCallConstructorEVM,
			)
		}))
		// NOTE: Intentionally NOT registering the IFT bridge on Ethereum
	}))

	ethReceiverAddr := crypto.PubkeyToAddress(s.ethUser.PublicKey)

	s.Require().True(s.Run("Mint tokens to user on Cosmos", func() {
		s.mintTokensOnCosmos(ctx, s.CosmosRelayerSubmitter, cosmosDenom, transferAmount, s.CosmosUser.FormattedAddress())
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount))
	}))

	var sendTxHash string
	s.Require().True(s.Run("Send transfer from Cosmos to Ethereum", func() {
		timeout := uint64(time.Now().Add(30 * time.Minute).Unix())
		sendTxHash = s.iftTransferFromCosmos(
			ctx,
			s.CosmosUser,
			cosmosDenom,
			ethClientIDOnCosmos,
			ethReceiverAddr.Hex(),
			transferAmount,
			timeout,
		)
		s.Require().NotEmpty(sendTxHash)
	}))

	s.Require().True(s.Run("Verify balance burned on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.IsZero())
	}))

	s.Require().True(s.Run("Verify pending transfer exists on Cosmos", func() {
		pending, err := s.queryPendingTransferOnCosmos(ctx, cosmosDenom, ethClientIDOnCosmos, 1)
		s.Require().NoError(err)
		s.Require().Equal(s.CosmosUser.FormattedAddress(), pending.Sender)
		s.Require().True(pending.Amount.Equal(transferAmount))
	}))

	var recvTxHash []byte
	s.Require().True(s.Run("Relay packet to Ethereum (execution fails - no bridge registered)", func() {
		sendTxHashBytes, err := hex.DecodeString(sendTxHash)
		s.Require().NoError(err)

		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    s.Wfchain.Config().ChainID,
			DstChain:    eth.ChainID.String(),
			SourceTxIds: [][]byte{sendTxHashBytes},
			SrcClientId: ethClientIDOnCosmos,
			DstClientId: tmClientID,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		receipt, err := eth.BroadcastTx(ctx, s.EthRelayerSubmitter, 15_000_000, &ics26Address, resp.Tx)
		s.Require().NoError(err)
		s.Require().Equal(ethtypes.ReceiptStatusSuccessful, receipt.Status)
		recvTxHash = receipt.TxHash.Bytes()
	}))

	s.Require().True(s.Run("Verify no balance minted on Ethereum", func() {
		iftContract, err := evmift.NewContract(ethIFTAddress, eth.RPCClient)
		s.Require().NoError(err)

		balance, err := iftContract.BalanceOf(nil, ethReceiverAddr)
		s.Require().NoError(err)
		s.Require().Equal("0", balance.String(), "Ethereum should have no tokens")
	}))

	s.Require().True(s.Run("Relay error ack to Cosmos", func() {
		resp, err := s.ProofApiClient.RelayByTx(ctx, &proofapitypes.RelayByTxRequest{
			SrcChain:    eth.ChainID.String(),
			DstChain:    s.Wfchain.Config().ChainID,
			SourceTxIds: [][]byte{recvTxHash},
			SrcClientId: tmClientID,
			DstClientId: ethClientIDOnCosmos,
		})
		s.Require().NoError(err)
		s.Require().NotEmpty(resp.Tx)

		_ = s.MustBroadcastSdkTxBody(ctx, s.Wfchain, s.CosmosRelayerSubmitter, 2_000_000, resp.Tx)
	}))

	s.Require().True(s.Run("Verify tokens refunded on Cosmos", func() {
		balance, err := s.Wfchain.GetBalance(ctx, s.CosmosUser.FormattedAddress(), cosmosDenom)
		s.Require().NoError(err)
		s.Require().True(balance.Equal(transferAmount), "Expected %s (refunded), got %s", transferAmount.String(), balance.String())
	}))

	s.Require().True(s.Run("Verify pending transfer cleared on Cosmos", func() {
		_, err := s.queryPendingTransferOnCosmos(ctx, cosmosDenom, ethClientIDOnCosmos, 1)
		s.Require().Error(err, "pending transfer should be cleared after error ack")
	}))
}

// Helper functions

func (s *CosmosEthereumIFTTestSuite) getIFTModuleAddress() string {
	iftAddr := authtypes.NewModuleAddress(testvalues.IFTModuleName)
	bech32Addr, err := sdk.Bech32ifyAddressBytes(s.Wfchain.Config().Bech32Prefix, iftAddr)
	s.Require().NoError(err)
	return bech32Addr
}

func (s *CosmosEthereumIFTTestSuite) createTokenFactoryDenom(ctx context.Context, user ibc.Wallet, denom string) string {
	msg := &tokenfactorytypes.MsgCreateDenom{
		Sender: user.FormattedAddress(),
		Denom:  denom,
	}

	_, err := s.BroadcastMessages(ctx, s.Wfchain, user, 200_000, msg)
	s.Require().NoError(err)

	return denom
}

func (s *CosmosEthereumIFTTestSuite) mintTokensOnCosmos(ctx context.Context, user ibc.Wallet, denom string, amount sdkmath.Int, recipient string) {
	msg := &tokenfactorytypes.MsgMint{
		From:    user.FormattedAddress(),
		Address: recipient,
		Amount:  sdk.Coin{Denom: denom, Amount: amount},
	}

	_, err := s.BroadcastMessages(ctx, s.Wfchain, user, 200_000, msg)
	s.Require().NoError(err)
}

func (s *CosmosEthereumIFTTestSuite) registerIFTBridgeOnCosmos(ctx context.Context, user ibc.Wallet, denom, clientId, counterpartyIftAddr, constructor string) {
	govModuleAddr, err := s.Wfchain.AuthQueryModuleAddress(ctx, govtypes.ModuleName)
	s.Require().NoError(err)

	msg := &ifttypes.MsgRegisterIFTBridge{
		Signer:                 govModuleAddr,
		Denom:                  denom,
		ClientId:               clientId,
		CounterpartyIftAddress: counterpartyIftAddr,
		IftSendCallConstructor: constructor,
	}

	err = s.ExecuteGovV1Proposal(ctx, msg, s.Wfchain, user)
	s.Require().NoError(err)
}

func (s *CosmosEthereumIFTTestSuite) iftTransferFromCosmos(ctx context.Context, user ibc.Wallet, denom, clientId, receiver string, amount sdkmath.Int, timeoutTimestamp uint64) string {
	msg := &ifttypes.MsgIFTTransfer{
		Signer:           user.FormattedAddress(),
		Denom:            denom,
		ClientId:         clientId,
		Receiver:         receiver,
		Amount:           amount,
		TimeoutTimestamp: timeoutTimestamp,
	}

	resp, err := s.BroadcastMessages(ctx, s.Wfchain, user, 200_000, msg)
	s.Require().NoError(err)

	return resp.TxHash
}

func (s *CosmosEthereumIFTTestSuite) queryPendingTransferOnCosmos(ctx context.Context, denom, clientId string, sequence uint64) (*ifttypes.PendingTransfer, error) {
	resp, err := e2esuite.GRPCQuery[ifttypes.QueryPendingTransferResponse](ctx, s.Wfchain, &ifttypes.QueryPendingTransferRequest{
		Denom:    denom,
		ClientId: clientId,
		Sequence: sequence,
	})
	if err != nil {
		return nil, err
	}
	return &resp.PendingTransfer, nil
}
