// SPDX-License-Identifier: Apache-2.0

package chainconfig

import (
	"context"
	"crypto/ecdsa"
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/kurtosis-tech/kurtosis/api/golang/core/lib/enclaves"
	"github.com/kurtosis-tech/kurtosis/api/golang/core/lib/services"
	"github.com/kurtosis-tech/kurtosis/api/golang/core/lib/starlark_run_config"
	"github.com/kurtosis-tech/kurtosis/api/golang/engine/lib/kurtosis_context"

	ethcommon "github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"

	"github.com/cosmos/interchaintest/v11/testutil"

	"github.com/lfdt-ibc/ibc-contracts/e2e/v8/ethereum"
	"github.com/lfdt-ibc/ibc-contracts/e2e/v8/testvalues"
)

const (
	// ethereumPackageId is the package ID used by Kurtosis to find the Ethereum package we use for the testnet
	ethereumPackageId = "github.com/ethpandaops/ethereum-package@5795b77e65235ad223d4d3b413b2739190bf60cc"

	faucetPrivateKey = "0x04b9f63ecf84210c5366c66d68fa1f5da1fa4f634fad6dfc86178e4d79ff9e59"
)

var (
	// KurtosisConfig sets up the default values for the eth testnet
	// It can be changed before calling SetupSuite to alter the testnet configuration
	KurtosisConfig = kurtosisNetworkParams{
		Participants: []kurtosisParticipant{
			{
				CLType:  "lodestar",
				CLImage: "chainsafe/lodestar:v1.50.1",
				ELType:  "geth",
				ELImage: "ethereum/client-go:v1.17.10",
				// The package omits --syncmode for archive nodes, and geth's default of snap
				// sync never completes on this single-node network
				ELExtraParams:  []string{"--gcmode=archive", "--syncmode=full"},
				ELLogLevel:     "info",
				ValidatorCount: 128,
				// Supernode required for Fulu testing
				Supernode: true,
			},
		},
		// We can change the preset dynamically before spinning up the testnet
		NetworkParams: kurtosisNetworkConfigParams{
			Preset:         "minimal",
			FuluForkEpoch:  0,
			GloasForkEpoch: 1,
		},
		WaitForFinalization: true,
		AdditionalServices:  []string{},
	}
	executionService = fmt.Sprintf("el-1-%s-%s", KurtosisConfig.Participants[0].ELType, KurtosisConfig.Participants[0].CLType)
	consensusService = fmt.Sprintf("cl-1-%s-%s", KurtosisConfig.Participants[0].CLType, KurtosisConfig.Participants[0].ELType)
)

// GetKurtosisPreset returns the Kurtosis preset to use.
// It retrieves the preset from the environment variable or falls back to the default.
func GetKurtosisPreset() string {
	preset := os.Getenv(testvalues.EnvKeyEthereumPosNetworkPreset)
	if preset == "" {
		return testvalues.EnvValueEthereumPosPreset_Minimal
	}
	return preset
}

type EthKurtosisChain struct {
	RPC             string
	BeaconApiClient ethereum.BeaconAPIClient
	Faucet          *ecdsa.PrivateKey

	kurtosisCtx *kurtosis_context.KurtosisContext
	enclaveCtx  *enclaves.EnclaveContext
}

// To see all the configuration options: github.com/ethpandaops/ethereum-package
type kurtosisNetworkParams struct {
	// Specification of the participants in the network
	Participants []kurtosisParticipant `json:"participants"`
	// Default configuration parameters for the network
	NetworkParams kurtosisNetworkConfigParams `json:"network_params"`
	// If set, the package will block until a finalized epoch has occurred.
	WaitForFinalization bool `json:"wait_for_finalization"`
	// Additional services to start in the network
	AdditionalServices []string `json:"additional_services"`
}

type kurtosisParticipant struct {
	// The type of CL client that should be started
	CLType string `json:"cl_type"`
	// The Docker image that should be used for the CL client
	CLImage string `json:"cl_image"`
	// The type of EL client that should be started
	ELType string `json:"el_type"`
	// The Docker image that should be used for the EL client
	ELImage string `json:"el_image"`
	// A list of optional extra params that will be passed to the EL client container for modifying its behaviour
	ELExtraParams []string `json:"el_extra_params"`
	// The log level string that this participant's EL client should log at
	ELLogLevel string `json:"el_log_level"`
	// Count of the number of validators you want to run for a given participant
	ValidatorCount uint64 `json:"validator_count"`
	// Whether to act as a supernode for the network
	Supernode bool `json:"supernode"`
	// A list of optional extra params that will be passed to the CL client container
	CLExtraParams []string `json:"cl_extra_params,omitempty"`
	// A list of optional extra params that will be passed to the validator client container
	VCExtraParams []string `json:"vc_extra_params,omitempty"`
}

type kurtosisNetworkConfigParams struct {
	// Preset for the network. Options: "mainnet", "minimal"
	Preset string `json:"preset"`
	// Number of seconds per slot on the Beacon chain
	SecondsPerSlot uint64 `json:"seconds_per_slot,omitempty"`
	// Duration of a slot in milliseconds
	SlotDuration uint64 `json:"slot_duration_ms,omitempty"`
	// Fulu fork epoch
	FuluForkEpoch uint64 `json:"fulu_fork_epoch"`
	// Gloas fork epoch
	GloasForkEpoch uint64 `json:"gloas_fork_epoch"`
}

// SpinUpKurtosisPoS spins up a kurtosis enclave with Etheruem PoS testnet using github.com/ethpandaops/ethereum-package
func SpinUpKurtosisPoS(ctx context.Context) (EthKurtosisChain, error) {
	// Load dynamic configurations
	KurtosisConfig.NetworkParams.Preset = GetKurtosisPreset()
	if KurtosisConfig.NetworkParams.Preset == testvalues.EnvValueEthereumPosPreset_Minimal {
		// Speed up slots to 3 seconds for minimal preset. Going lower breaks light client data at
		// Gloas, because validators must sign the head at 25% of the slot and lodestar takes around
		// 500ms to publish a block. Signing the previous head leaves the sync aggregates empty.
		KurtosisConfig.NetworkParams.SlotDuration = 3000
		KurtosisConfig.NetworkParams.SecondsPerSlot = 3
		// The package only sets the slot duration in the chain config, which leaves the deprecated
		// SECONDS_PER_SLOT at its default. The relayer of earlier client versions still reads it.
		secondsPerSlotParams := []string{"--params.SECONDS_PER_SLOT=3"}
		KurtosisConfig.Participants[0].CLExtraParams = secondsPerSlotParams
		KurtosisConfig.Participants[0].VCExtraParams = secondsPerSlotParams
	}

	faucet, err := crypto.ToECDSA(ethcommon.FromHex(faucetPrivateKey))
	if err != nil {
		return EthKurtosisChain{}, err
	}

	kurtosisCtx, err := kurtosis_context.NewKurtosisContextFromLocalEngine()
	if err != nil {
		return EthKurtosisChain{}, err
	}

	enclaveName := "ethereum-pos-testnet"
	enclaves, err := kurtosisCtx.GetEnclaves(ctx)
	if err != nil {
		return EthKurtosisChain{}, err
	}

	if enclaveInfos, found := enclaves.GetEnclavesByName()[enclaveName]; found {
		for _, enclaveInfo := range enclaveInfos {
			err = kurtosisCtx.DestroyEnclave(ctx, enclaveInfo.EnclaveUuid)
			if err != nil {
				return EthKurtosisChain{}, err
			}
		}
	}
	enclaveCtx, err := kurtosisCtx.CreateEnclave(ctx, enclaveName)
	if err != nil {
		return EthKurtosisChain{}, err
	}

	networkParamsJson, err := json.Marshal(KurtosisConfig)
	if err != nil {
		return EthKurtosisChain{}, err
	}
	starlarkResp, err := enclaveCtx.RunStarlarkRemotePackageBlocking(ctx, ethereumPackageId, &starlark_run_config.StarlarkRunConfig{
		SerializedParams: string(networkParamsJson),
	})
	if err != nil {
		return EthKurtosisChain{}, err
	}
	fmt.Println(starlarkResp.RunOutput)

	// exeuctionCtx is the service context (kurtosis concept) for the execution node that allows us to get the public ports
	executionCtx, err := enclaveCtx.GetServiceContext(executionService)
	if err != nil {
		return EthKurtosisChain{}, err
	}
	rpcPortSpec := executionCtx.GetPublicPorts()["rpc"]
	rpc := fmt.Sprintf("http://localhost:%d", rpcPortSpec.GetNumber())
	fmt.Println("Local Execution RPC: ", rpc)

	// consensusCtx is the service context (kurtosis concept) for the consensus node that allows us to get the public ports
	consensusCtx, err := enclaveCtx.GetServiceContext(consensusService)
	if err != nil {
		return EthKurtosisChain{}, err
	}
	beaconPortSpec := consensusCtx.GetPublicPorts()["http"]
	beaconRPC := fmt.Sprintf("http://localhost:%d", beaconPortSpec.GetNumber())
	fmt.Println("Local Beacon RPC: ", beaconRPC)

	// Wait for the chain to finalize
	var beaconAPIClient ethereum.BeaconAPIClient
	err = testutil.WaitForCondition(30*time.Minute, 5*time.Second, func() (bool, error) {
		beaconAPIClient, err = ethereum.NewBeaconAPIClient(ctx, beaconRPC)
		if err != nil {
			return false, nil
		}

		finalizedBlocksResp, err := beaconAPIClient.GetFinalizedBlocks()
		fmt.Printf("Waiting for chain to finalize, finalizedBlockResp: %+v, err: %s\n", finalizedBlocksResp, err)
		if err != nil {
			return false, nil
		}
		if !finalizedBlocksResp.Finalized {
			return false, nil
		}

		header, err := beaconAPIClient.GetHeader(finalizedBlocksResp.Data.Message.Slot)
		if err != nil {
			return false, nil
		}
		bootstrap, err := beaconAPIClient.GetBootstrap(header.Root)
		if err != nil {
			return false, nil
		}

		return bootstrap.Data.Header.Beacon.Slot != 0, nil
	})
	if err != nil {
		return EthKurtosisChain{}, err
	}

	return EthKurtosisChain{
		RPC:             rpc,
		BeaconApiClient: beaconAPIClient,
		Faucet:          faucet,
		kurtosisCtx:     kurtosisCtx,
		enclaveCtx:      enclaveCtx,
	}, nil
}

func (e EthKurtosisChain) Destroy(ctx context.Context) {
	if err := e.kurtosisCtx.DestroyEnclave(ctx, string(e.enclaveCtx.GetEnclaveUuid())); err != nil {
		panic(err)
	}
}

func (e EthKurtosisChain) DumpLogs(ctx context.Context) error {
	enclaveServices, err := e.enclaveCtx.GetServices()
	if err != nil {
		return err
	}

	userServices := make(map[services.ServiceUUID]bool)
	serviceIdToName := make(map[services.ServiceUUID]string)
	for serviceName, servicesUUID := range enclaveServices {
		userServices[servicesUUID] = true
		serviceIdToName[servicesUUID] = string(serviceName)

	}

	stream, cancelFunc, err := e.kurtosisCtx.GetServiceLogs(ctx, string(e.enclaveCtx.GetEnclaveUuid()), userServices, false, true, 0, nil)
	if err != nil {
		return err
	}

	// Dump the stream chan into stdout
	fmt.Println("Dumping kurtosis logs")
	for {
		select {
		case logs, ok := <-stream:
			if !ok {
				return nil
			}
			for serviceID, serviceLog := range logs.GetServiceLogsByServiceUuids() {
				if serviceIdToName[serviceID] != executionService {
					continue
				}
				for _, log := range serviceLog {
					fmt.Printf("Service %s logs: %s\n", serviceIdToName[serviceID], log)
				}
			}
		case <-ctx.Done():
			cancelFunc()
			return nil
		}
	}
}
