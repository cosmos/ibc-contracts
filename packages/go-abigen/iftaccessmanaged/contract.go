// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package iftaccessmanaged

import (
	"context"
	"errors"
	"math/big"
	"strings"
	"time"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/event"
)

// Reference imports to suppress errors if they are not otherwise used.
var (
	_ = errors.New
	_ = big.NewInt
	_ = strings.NewReader
	_ = ethereum.NotFound
	_ = bind.Bind
	_ = common.Big1
	_ = types.BloomLookup
	_ = event.NewSubscription
	_ = abi.ConvertType
	_ = time.Tick
	_ = context.Background
)

// IIBCAppCallbacksOnAcknowledgementPacketCallback is an auto generated low-level Go binding around an user-defined struct.
type IIBCAppCallbacksOnAcknowledgementPacketCallback struct {
	SourceClient      string
	DestinationClient string
	Sequence          uint64
	Payload           IICS26RouterMsgsPayload
	Acknowledgement   []byte
	Relayer           common.Address
}

// IIBCAppCallbacksOnTimeoutPacketCallback is an auto generated low-level Go binding around an user-defined struct.
type IIBCAppCallbacksOnTimeoutPacketCallback struct {
	SourceClient      string
	DestinationClient string
	Sequence          uint64
	Payload           IICS26RouterMsgsPayload
	Relayer           common.Address
}

// IICS26RouterMsgsPayload is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsPayload struct {
	SourcePort string
	DestPort   string
	Version    string
	Encoding   string
	Value      []byte
}

// IIFTMsgsIFTBridge is an auto generated low-level Go binding around an user-defined struct.
type IIFTMsgsIFTBridge struct {
	ClientId               string
	CounterpartyIFTAddress string
	IftSendCallConstructor common.Address
}

// IIFTMsgsPendingTransfer is an auto generated low-level Go binding around an user-defined struct.
type IIFTMsgsPendingTransfer struct {
	Sender common.Address
	Amount *big.Int
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"allowance\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"approve\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"authority\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"balanceOf\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"burn\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"burnFrom\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"decimals\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.IFTBridge\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"contractIIFTSendCallConstructor\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimit\",\"inputs\":[],\"outputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimitAvailable\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPendingTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.PendingTransfer\",\"components\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ics27\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIICS27GMP\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"iftMint\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"authority_\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"erc20Name\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"erc20Symbol\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"ics27Gmp\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"isConsumingScheduledOp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"mint\",\"inputs\":[{\"name\":\"mintAddress\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"name\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"onAckPacket\",\"inputs\":[{\"name\":\"success\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnAcknowledgementPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onTimeoutPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnTimeoutPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"pause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"paused\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"registerIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"removeIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setAuthority\",\"inputs\":[{\"name\":\"newAuthority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setIFTRateLimit\",\"inputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"supportsInterface\",\"inputs\":[{\"name\":\"interfaceId\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"symbol\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalSupply\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferFrom\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"unpause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"event\",\"name\":\"Approval\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"AuthorityUpdated\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRegistered\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRemoved\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTMintReceived\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTRateLimitSet\",\"inputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"indexed\":false,\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"indexed\":false,\"internalType\":\"uint48\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferCompleted\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferInitiated\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"receiver\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferRefunded\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Paused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Transfer\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Unpaused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AccessManagedInvalidAuthority\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AccessManagedRequiredDelay\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"delay\",\"type\":\"uint32\",\"internalType\":\"uint32\"}]},{\"type\":\"error\",\"name\":\"AccessManagedUnauthorized\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ERC20InsufficientAllowance\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"allowance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InsufficientBalance\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidApprover\",\"inputs\":[{\"name\":\"approver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSpender\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"EnforcedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ExpectedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTBridgeNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTEmptyClientId\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyCounterpartyAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyReceiver\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTInvalidConstructorInterface\",\"inputs\":[{\"name\":\"callConstructor\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTInvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTOnlyICS27GMP\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTPendingTransferNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTTimeoutInPast\",\"inputs\":[{\"name\":\"timeout\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"currentTime\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTUnauthorizedMint\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTUnexpectedSalt\",\"inputs\":[{\"name\":\"salt\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"IFTZeroAddressConstructor\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTZeroAmount\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"RateLimitExceeded\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeCastOverflowedUintDowncast\",\"inputs\":[{\"name\":\"bits\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}]",
	Bin: "0x60a080604052346100c257306080525f516020614fe65f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b604051614f1f90816100c78239608051818181611f83015261204c0152f35b6001600160401b0319166001600160401b039081175f516020614fe65f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe6080806040526004361015610012575f80fd5b5f905f358060e01c91826301ffc9a7146134225750816306fdde03146132fc578163095ea7b3146132015781630a7244e714612e9257816318160ddd14612e5657816323b872dd14612e1e578163313ce56714612e035781633f4ba83a14612ada57816340c10f19146128b4578163419e41c61461255357816342966c68146125355781634f1ef28614611ffb57816352d1902d14611f68578163599deb4814611f225781635c975abb14611ee05781635e32b6b614611e6d578163613d25bb1461177f57816370a0823114611728578163711708b3146116a957816379cc6790146116785781637a9e5e4b146115e65781638456cb59146112ec5781638dfcd9ad146112985781638fb36037146112035781639226e0831461114157816395d89b4114611002578163997d99b814610f9e578163a50e31f114610f61578163a9059cbb14610f2f578163ad3cb1cc14610eca578163b8ce241814610b8b578163bf7e214f14610b45578163d638f98b146103f657508063d88a36fe14610312578063dd62ed3e146102985763e529a26d146101ac575f80fd5b346102955760206003193601126102955760043567ffffffffffffffff811161029357906101e16102749236906004016135eb565b91604080516101ef81613511565b606081526060602082015201526001600160a01b03610287610211848461388e565b6102586040519561022187613511565b61022a8361366a565b875284600261023b6001860161366a565b9460208a0195865201541695604088019687528751511515613a9e565b60405195869560208752516060602088015260808701906134c0565b9051601f198683030160408701526134c0565b91511660608301520390f35b505b80fd5b5034610295576040600319360112610295576102b26134e5565b6001600160a01b036102fc6102c56134fb565b926001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f52602052602060405f2054604051908152f35b50346102955760606003193601126102955760043567ffffffffffffffff8111610293576103449036906004016135eb565b919060243567ffffffffffffffff81116103f2576103669036906004016135eb565b610371929192613b51565b61038467ffffffffffffffff4216019267ffffffffffffffff84116103c55761039f949560443593336141ad565b807f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d80f35b6024857f4e487b710000000000000000000000000000000000000000000000000000000081526011600452fd5b8280fd5b82346102955760606003193601126102955760043567ffffffffffffffff8111610293576104289036906004016135eb565b60243567ffffffffffffffff8111610b41576104489036906004016135eb565b94906044356001600160a01b03811692838203610afa577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546001600160a01b0381169136600411610b3d578860408a809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb700961300000000000000000000000000000000000000000000000000000000845233602484015230604484015216606482015260648152610506608482613576565b828052826020525190875afa610b2a575b156109a1575b5050508315610979578615610951578215610929578561053c82614b7e565b91826108b7575b50501561088b576040519561055787613511565b6105623685876135b5565b875261056f3682846135b5565b966020810197885260408101848152610588868861388e565b915180519067ffffffffffffffff82116107b7576105a68454613619565b601f8111610851575b50602090601f83116001146107ef576105de92918b91836107e4575b50505f198260011b9260031b1c19161790565b82555b60018201985198895167ffffffffffffffff81116107b7576106038254613619565b601f8111610774575b506020601f82116001146106dd576106c6969594936001600160a01b0393610671848e9f957fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89f956002978997926106d25750505f198260011b9260031b1c19161790565b90555b5116920191167fffffffffffffffffffffffff00000000000000000000000000000000000000008254161790556106b86040519687966060885260608801916138e6565b9185830360208701526138e6565b9060408301520390a180f35b015190505f806105cb565b828b52808b209b601f1983168c5b81811061075c5750936001847fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89e9f6001600160a01b0397956106c69c9b9a99956002988a9810610744575b505050811b019055610674565b01515f1960f88460031b161c191690555f8080610737565b828401518f556001909e019d602093840193016106eb565b8181111561060c576107a990838c5260208c2090601f840160051c90602085106107af575b601f82910160051c039101613ae7565b8b61060c565b8d9150610799565b60248a7f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b015190508c806105cb565b848b52818b209190601f1984168c5b8181106108395750908460019594939210610821575b505050811b0182556105e1565b01515f1960f88460031b161c191690558b8080610814565b929360206001819287860151815501950193016107fe565b828111156105af5761088590858c5260208c2090601f850160051c90602086106107af57601f82910160051c039101613ae7565b8b6105af565b602485837f3730169b000000000000000000000000000000000000000000000000000000008252600452fd5b60209250816024917f01ffc9a70000000000000000000000000000000000000000000000000000000082527f56d981a700000000000000000000000000000000000000000000000000000000600452617530fa85511515601f3d111681610921575b508588610543565b905087610919565b6004867ff3a03283000000000000000000000000000000000000000000000000000000008152fd5b6004867f9624aeb8000000000000000000000000000000000000000000000000000000008152fd5b6004867fc8ca373e000000000000000000000000000000000000000000000000000000008152fd5b63ffffffff1615610afe577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055803b15610afa578660405180927f94c7d7ee00000000000000000000000000000000000000000000000000000000825233600483015260406024830152818381610a5460448201366138c6565b03925af18015610aef57908791610ad6575b50507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005587808061051d565b81610ae091613576565b610aeb578588610a66565b8580fd5b6040513d89823e3d90fd5b8680fd5b6024887f068ca9d800000000000000000000000000000000000000000000000000000000815233600452fd5b505087516020518060201c150290610517565b8880fd5b8380fd5b823461029557806003193601126102955760206001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416604051908152f35b9050346102935760206003193601126102935760043567ffffffffffffffff81116103f257610bbe9036906004016135eb565b917ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546001600160a01b0381169136600411610aeb5785604087809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb700961300000000000000000000000000000000000000000000000000000000845233602484015230604484015216606482015260648152610c66608482613576565b828052826020525190875afa610eb7575b15610d36575b50505081610cf58183610cb17f142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a846557751968261388e565b6001600160a01b03600260405192610cc884613511565b610cd18161366a565b8452610cdf6001820161366a565b6020850152015416604082015251511515613a9e565b836002610d02838561388e565b610d0b81613b02565b610d1760018201613b02565b0155610d306040519283926020845260208401916138e6565b0390a180f35b63ffffffff1615610e8b577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00558390803b15610293578160405180927f94c7d7ee00000000000000000000000000000000000000000000000000000000825233600483015260406024830152818381610deb60448201366138c6565b03925af18015610e8057610e6b575b50507ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0080547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1690557f142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a8465577515f80610c7d565b81610e7591613576565b6103f257825f610dfa565b6040513d84823e3d90fd5b6024857f068ca9d800000000000000000000000000000000000000000000000000000000815233600452fd5b505084516020518060201c150290610c77565b823461029557806003193601126102955750610f2b604051610eed604082613576565b600581527f352e302e3000000000000000000000000000000000000000000000000000000060208201526040519182916020835260208301906134c0565b0390f35b823461029557604060031936011261029557610f56610f4c6134e5565b6024359033613dc8565b602060405160018152f35b82346102955780600319360112610295576040610f84610f7f61478d565b614a3b565b9050610f91610f7f6147ec565b8351928352602083015250f35b82346102955780600319360112610295577f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00546040805179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316815260d09290921c602083015290f35b823461029557806003193601126102955760405190807f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04549061104482613619565b80855291600181169081156110fc5750600114611080575b610f2b8461106c81860382613576565b6040519182916020835260208301906134c0565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0481527f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa939250905b8082106110e25750909150810160200161106c8261105c565b9192600181602092548385880101520191019092916110c9565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208087019190915292151560051b8501909201925061106c915083905061105c565b82346102955760406003193601126102955760043567ffffffffffffffff8111610293576111739036906004016135eb565b9060243567ffffffffffffffff81168103610b41576111ec906040946020865161119c8161355a565b82815201526111ab8484613856565b67ffffffffffffffff82165f52602052845f209360018651956111cd8761355a565b6001600160a01b03815416875201549360208601948086521515613906565b6001600160a01b0383519251168252516020820152f35b82346102955780600319360112610295577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005460a01c60ff1615611290575060207f8fb36037000000000000000000000000000000000000000000000000000000005b7fffffffff0000000000000000000000000000000000000000000000000000000060405191168152f35b602090611266565b8234610295576040600319360112610295576004358015158103610293576024359067ffffffffffffffff82116103f25760c060031983360301126103f25761039f916112e3613b51565b60040190613964565b82346102955780600319360112610295577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054916001600160a01b03831690366004116103f25782604084809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb7009613000000000000000000000000000000000000000000000000000000008452336024840152306044840152166064820152606481526113a5608482613576565b828052826020525190865afa6115d3575b1561145b575b826113c5614865565b60017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff007fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005416177fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a2586020604051338152a180f35b63ffffffff16156115a757740100000000000000000000000000000000000000007fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff839416177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055803b156115a4578160405180927f94c7d7ee0000000000000000000000000000000000000000000000000000000082523360048301526040602483015281838161151060448201366138c6565b03925af18015610e805761158f575b50507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00558082806113bc565b8161159991613576565b61029557808261151f565b50fd5b6024827f068ca9d800000000000000000000000000000000000000000000000000000000815233600452fd5b505081516020518060201c1502906113b6565b8234610295576020600319360112610295576116006134e5565b6001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00541633036115a757803b1561164457611641906146ef565b80f35b7fc2f31e5e0000000000000000000000000000000000000000000000000000000082526001600160a01b0316600452602490fd5b8234610295576040600319360112610295576116416116956134e5565b602435906116a4823383613cac565b613f04565b82346102955760806003193601126102955760043567ffffffffffffffff8111610293576116db9036906004016135eb565b9060243567ffffffffffffffff8111610b41576116fc9036906004016135eb565b606435929167ffffffffffffffff84168403610aeb5761039f9461171e613b51565b60443593336141ad565b82346102955760206003193601126102955760406020916001600160a01b0361174f6134e5565b1681527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0083522054604051908152f35b8234610295576080600319360112610295576117996134e5565b9060243567ffffffffffffffff8111610293576117ba9036906004016135eb565b60449391933567ffffffffffffffff8111610b41576117dd9036906004016135eb565b94606435926001600160a01b038416809403610aeb577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549460ff8660401c16159567ffffffffffffffff811680159081611e65575b6001149081611e5b575b159081611e52575b50611e2a57916118bc6118d3959492848960017fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000006118cb9816177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0055611dd5575b506118af6149e4565b6118b76149e4565b6146ef565b6118c46149e4565b36916135b5565b9536916135b5565b936118dc6149e4565b6118e46149e4565b6118ec6149e4565b80519067ffffffffffffffff8211611da8576119287f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0354613619565b601f8111611d29575b50602090601f8311600114611c875761195f9291869183611c7c5750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03555b835167ffffffffffffffff8111611c4f576119bd7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454613619565b601f8111611bd0575b50602094601f8211600114611b2f57849582916119f794959692611b245750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04555b611a226149e4565b7fffffffffffffffffffffffff00000000000000000000000000000000000000007f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416177f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0055611a905780f35b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a180f35b0151905086806105cb565b601f198216957f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04865280862091865b888110611bb857508360019596979810611ba0575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0455611a1a565b01515f1960f88460031b161c19169055858080611b73565b91926020600181928685015181550194019201611b5e565b818111156119c6577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace048552611c41907f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa90601f840160051c9060208510611c4757601f82910160051c039101613ae7565b856119c6565b879150610799565b6024847f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b0151905087806105cb565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0386528186209190601f198416875b818110611d115750908460019594939210611cf9575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0355611982565b01515f1960f88460031b161c19169055868080611ccc565b92936020600181928786015181550195019301611cb6565b82811115611931577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace038652611d9a907f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab090601f850160051c9060208610611da057601f82910160051c039101613ae7565b86611931565b889150610799565b6024857f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00558b6118a6565b6004887ff92ee8a9000000000000000000000000000000000000000000000000000000008152fd5b9050158a611845565b303b15915061183d565b889150611833565b82346102955760206003193601126102955760043567ffffffffffffffff811161029357806004019060a060031982360301126103f257611edb604491611ed3611eca61039f95611ebc613b51565b611ec461401f565b806137f0565b94909201613841565b9236916135b5565b61407e565b8234610295578060031936011261029557602060ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f0330054166040519015158152f35b823461029557806003193601126102955760206001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051908152f35b82346102955780600319360112610295576001600160a01b037f0000000000000000000000000000000000000000000000000000000000000000163003611fd35760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b807fe07c8dba0000000000000000000000000000000000000000000000000000000060049252fd5b826040600319360112610295576120106134e5565b60243567ffffffffffffffff81116103f257366023820112156103f2576120419036906024816004013591016135b5565b926001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016803014908115612500575b506124d8577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546001600160a01b03811691366004116123385784604086809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb700961300000000000000000000000000000000000000000000000000000000845233602484015230604484015216606482015260648152612123608482613576565b828052826020525190875afa6124c5575b15612344575b5050506001600160a01b03811690604051937f52d1902d000000000000000000000000000000000000000000000000000000008552602085600481865afa8095859661230c575b506121b257602484847f4c9c8ce3000000000000000000000000000000000000000000000000000000008252600452fd5b9091847f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc81036122e15750823b156122b657807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b8480a28051156122845761228091614d44565b5080f35b50503461228e5780f35b807fb398979f0000000000000000000000000000000000000000000000000000000060049252fd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000008452600452602483fd5b7faa1d49a4000000000000000000000000000000000000000000000000000000008552600452602484fd5b9095506020813d60201161233c575b8161232860209383613576565b8101031261233857519486612181565b8480fd5b3d915061231b565b63ffffffff1615612499577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055803b156103f2578260405180927f94c7d7ee000000000000000000000000000000000000000000000000000000008252336004830152604060248301528183816123f760448201366138c6565b03925af1801561248e57908391612479575b50507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005583808061213a565b8161248391613576565b610293578184612409565b6040513d85823e3d90fd5b6024847f068ca9d800000000000000000000000000000000000000000000000000000000815233600452fd5b505083516020518060201c150290612134565b6004837fe07c8dba000000000000000000000000000000000000000000000000000000008152fd5b90506001600160a01b037f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416141585612077565b82346102955760206003193601126102955761164160043533613f04565b905034610293576040600319360112610293576004359079ffffffffffffffffffffffffffffffffffffffffffffffffffff821691828103610b41576024359165ffffffffffff83168303612338577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546001600160a01b0381169136600411610afa5786604088809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb700961300000000000000000000000000000000000000000000000000000000845233602484015230604484015216606482015260648152612649608482613576565b828052826020525190875afa6128a1575b1561272b575b5050507fab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a919261269561269061478d565b6148f2565b6126a06126906147ec565b79ffffffffffffffffffffffffffffffffffffffffffffffffffff90811660d084901b7fffffffffffff000000000000000000000000000000000000000000000000000016177f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00556040805192909116825265ffffffffffff90921660208201529081908101610d30565b63ffffffff1615612875577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00558490803b15610293578160405180927f94c7d7ee000000000000000000000000000000000000000000000000000000008252336004830152604060248301528183816127e060448201366138c6565b03925af18015610e8057612860575b50507ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0080547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1690557fab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a915f80612660565b8161286a91613576565b610b4157835f6127ef565b6024867f068ca9d800000000000000000000000000000000000000000000000000000000815233600452fd5b505085516020518060201c15029061265a565b905034610293576040600319360112610293576128cf6134e5565b907ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546001600160a01b03811691366004116123385784604086809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb700961300000000000000000000000000000000000000000000000000000000845233602484015230604484015216606482015260648152612977608482613576565b828052826020525190875afa612ac7575b1561299b575b8461164160243586613bc5565b63ffffffff1615612499577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00558290803b15610293578160405180927f94c7d7ee00000000000000000000000000000000000000000000000000000000825233600483015260406024830152818381612a5060448201366138c6565b03925af18015610e8057612ab2575b50507ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0080547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1690556116415f8061298e565b81612abc91613576565b61029357815f612a5f565b505083516020518060201c150290612988565b905034612dc0575f600319360112612dc0577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546001600160a01b0381169136600411612dc0575f60405f809382517fffffffff0000000000000000000000000000000000000000000000000000000060208201927fb700961300000000000000000000000000000000000000000000000000000000845233602484015230604484015216606482015260648152612b93608482613576565b828052826020525190875afa612df0575b15612c71575b5050507fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005460ff811615612c49577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00167fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa6020604051338152a180f35b6004827f8dfc202b000000000000000000000000000000000000000000000000000000008152fd5b63ffffffff1615612dc4577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055803b15612dc0575f60405180927f94c7d7ee00000000000000000000000000000000000000000000000000000000825233600483015260406024830152818381612d2460448201366138c6565b03925af18015612db557612da2575b507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00555f8080612baa565b612dae91505f90613576565b5f5f612d33565b6040513d5f823e3d90fd5b5f80fd5b7f068ca9d8000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b50505f516020518060201c150290612ba4565b34612dc0575f600319360112612dc057602060405160128152f35b34612dc0576060600319360112612dc057610f56612e3a6134e5565b612e426134fb565b60443591612e51833383613cac565b613dc8565b34612dc0575f600319360112612dc05760207f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254604051908152f35b34612dc0576040600319360112612dc057612eab6134e5565b60243590612eb7613b51565b60245f6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051928380927fe57d7de00000000000000000000000000000000000000000000000000000000082523360048301525afa908115612db5575f9161314c575b506020808251604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a01815203019020612fca604051612f7181613511565b612f7a8361366a565b81526001600160a01b036002612f926001860161366a565b94602084019586520154166040820152612fb281515115158551906137aa565b516020815191012083519081516020830120146137aa565b518051602082012060208301519081516020830120036130f8575050604081015180516130b65750612ffa614865565b61300b8361300661478d565b614c5c565b1561308e57816001600160a01b0361305e92613048867f3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae96613bc5565b51916040519384936040855260408501906134c0565b95602084015216930390a25f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7fa74c1c5f000000000000000000000000000000000000000000000000000000005f5260045ffd5b6130f4906040519182917f3ef1dc020000000000000000000000000000000000000000000000000000000083526020600484015260248301906134c0565b0390fd5b906130f461313a926040519384937ff39880b10000000000000000000000000000000000000000000000000000000085526040600486015260448501906134c0565b906003198483030160248501526134c0565b90503d805f833e61315d8183613576565b810190602081830312612dc05780519067ffffffffffffffff8211612dc05701606081830312612dc0576040519161319483613511565b815167ffffffffffffffff8111612dc057816131b1918401613728565b8352602082015167ffffffffffffffff8111612dc057816131d3918401613728565b6020840152604082015167ffffffffffffffff8111612dc0576131f69201613728565b604082015283612f25565b34612dc0576040600319360112612dc05761321a6134e5565b6024359033156132d0576001600160a01b03169081156132a457335f9081527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260409020825f526020528060405f20556040519081527f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560203392a3602060405160018152f35b7f94280d62000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7fe602df05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b34612dc0575f600319360112612dc0576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035461333b81613619565b80845290600181169081156133e05750600114613363575b610f2b8361106c81850382613576565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f9081527f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab0939250905b8082106133c65750909150810160200161106c613353565b9192600181602092548385880101520191019092916133ae565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208086019190915291151560051b8401909101915061106c9050613353565b34612dc0576020600319360112612dc057600435907fffffffff000000000000000000000000000000000000000000000000000000008216809203612dc057817fd3ce6f1b0000000000000000000000000000000000000000000000000000000060209314908115613496575b5015158152f35b7f01ffc9a7000000000000000000000000000000000000000000000000000000009150148361348f565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b600435906001600160a01b0382168203612dc057565b602435906001600160a01b0382168203612dc057565b6060810190811067ffffffffffffffff82111761352d57604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b6040810190811067ffffffffffffffff82111761352d57604052565b90601f601f19910116810190811067ffffffffffffffff82111761352d57604052565b67ffffffffffffffff811161352d57601f01601f191660200190565b9291926135c182613599565b916135cf6040519384613576565b829481845281830111612dc0578281602093845f960137010152565b9181601f84011215612dc05782359167ffffffffffffffff8311612dc05760208381860195010111612dc057565b90600182811c92168015613660575b602083101461363357565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f1691613628565b9060405191825f82549261367d84613619565b80845293600181169081156136e857506001146136a4575b506136a292500383613576565b565b90505f9291925260205f20905f915b8183106136cc5750509060206136a2928201015f613695565b60209193508060019154838589010152019101909184926136b3565b602093506136a29592507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0091501682840152151560051b8201015f613695565b81601f82011215612dc05760208151910161374282613599565b926137506040519485613576565b82845282820111612dc057815f926020928386015e8301015290565b60208091604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b156137b25750565b6130f4906040519182917f24a61b950000000000000000000000000000000000000000000000000000000083526020600484015260248301906134c0565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe181360301821215612dc0570180359067ffffffffffffffff8211612dc057602001918136038313612dc057565b3567ffffffffffffffff81168103612dc05790565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0181520301902090565b90601f19601f602093808452805f868601375f8582860101520116010190565b601f8260209493601f1993818652868601375f8582860101520116010190565b9290921561391357505050565b67ffffffffffffffff6139596040519485947fa4e158af0000000000000000000000000000000000000000000000000000000086526040600487015260448601916138e6565b911660248301520390fd5b61396c61401f565b15613a88577f5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc9286139a561399f83806137f0565b90613856565b604083019067ffffffffffffffff6139bc83613841565b165f52602052613a5260405f206001600160a01b03613a4a6001604051936139e38561355a565b838154168552015496613a136020850198808a52613a0183806137f0565b90613a0b8a613841565b921515613906565b613a2061399f82806137f0565b67ffffffffffffffff613a3288613841565b165f526020525f6001604082208281550155806137f0565b939094613841565b915116945167ffffffffffffffff613a776040519586956060875260608701916138e6565b9216602084015260408301520390a2565b6040611edb82611ed3611eca826136a2966137f0565b91909115613aaa575050565b6130f46040519283927f24a61b950000000000000000000000000000000000000000000000000000000084526020600485015260248401916138e6565b5f5b828110613af557505050565b5f82820155600101613ae9565b613b0c8154613619565b9081613b16575050565b81601f5f9311600114613b27575055565b81835260208320613b4491601f0160051c5f190190600101613ae7565b8082528160208120915555565b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c613b9d5760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b6001600160a01b0316908115613c80577fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef602082613c255f947f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02546148b8565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02558484527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260408420818154019055604051908152a3565b7fec442f05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b9190613ce8836001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b6001600160a01b0382165f5260205260405f2054925f198410613d0c575b50505050565b828410613d8b576001600160a01b038116156132d0576001600160a01b038216156132a457613d736001600160a01b03916001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f5260205260405f20910390555f808080613d06565b506001600160a01b0383917ffb8f41b2000000000000000000000000000000000000000000000000000000005f521660045260245260445260645ffd5b6001600160a01b0316908115613ed8576001600160a01b0316918215613c8057815f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054818110613ea657817fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92602092855f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0084520360405f2055845f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260405f20818154019055604051908152a3565b827fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b7f96c6fd1e000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b9091906001600160a01b03168015613ed857805f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054838110613fec576020845f94957fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef938587527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace008452036040862055807f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254037f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0255604051908152a3565b91507fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416330361405257565b7f9a47cf30000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b906140888261376c565b67ffffffffffffffff82165f5260205260405f206001604051916140ab8361355a565b6001600160a01b038154168352015492602082019380855215614165576140d0614865565b6140dd845161300661478d565b1561308e577fef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332926001600160a01b038361411f82613a77965116885190613bc5565b6141288461376c565b67ffffffffffffffff84165f526020525f60016040822082815501555116945167ffffffffffffffff6040519485946060865260608601906134c0565b613959908367ffffffffffffffff6040519384937fa4e158af0000000000000000000000000000000000000000000000000000000085526040600486015260448501906134c0565b9594929091939484156146c757851561469f5783156146775767ffffffffffffffff16934285111561463d576141e3818461388e565b92614249604051926141f484613511565b6141fd8661366a565b84526001600160a01b0360026142156001890161366a565b9760208701988952015416926040850193845284516020815191012061423c3684846135b5565b6020815191012014613a9e565b614251614865565b61425d856130066147ec565b1561308e576001600160a01b0390614275868a613f04565b5116925f60405180957f56d981a70000000000000000000000000000000000000000000000000000000082526040600483015281806142b8604482018d8a6138e6565b8a602483015203915afa938415612db5575f946145fd575b506001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054169582519151966040519260c0840184811067ffffffffffffffff82111761352d5760405283526020830197885260209788936040519361433c8686613576565b5f85526040820194855260608201988952608082019081526040516143618782613576565b5f815260a08301908152604051998a96879586957f7e5a65660000000000000000000000000000000000000000000000000000000087528a6004880152516024870160c0905260e487016143b4916134c0565b9051908681037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160448801526143ea916134c0565b9051908581037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc016064870152614420916134c0565b9051908481037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc016084860152614456916134c0565b915167ffffffffffffffff1660a484015251908281037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160c484015261449c916134c0565b03915a905f91f1928315612db5575f93614593575b50917fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a95614588926145759695946001600160a01b036040519a6144f48c61355a565b16998a8152600188820188815261450b845161376c565b67ffffffffffffffff88165f528a526001600160a01b038060405f20945116167fffffffffffffffffffffffff000000000000000000000000000000000000000084541617835551910155519567ffffffffffffffff60405198899860808a5260808a01906134c0565b94169087015285830360408701526138e6565b9060608301520390a2565b9091948094935081813d83116145f6575b6145ae8183613576565b81010312612dc057519067ffffffffffffffff82168203612dc057929391929091907fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a6144b1565b503d6145a4565b9093503d805f833e61460f8183613576565b8101602082820312612dc057815167ffffffffffffffff8111612dc0576146369201613728565b925f6142d0565b847f0b8a29e4000000000000000000000000000000000000000000000000000000005f5260045267ffffffffffffffff421660245260445ffd5b7fcc2a76e5000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f5de17177000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fc8ca373e000000000000000000000000000000000000000000000000000000005f5260045ffd5b60206001600160a01b037f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad9216807fffffffffffffffffffffffff00000000000000000000000000000000000000007ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055604051908152a1565b602060405161479d604082613576565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e5f603b820152601c81526147e6603c82613576565b51902090565b60206040516147fc604082613576565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e7f0100000000000000000000000000000000000000000000000000000000000000603b820152601c81526147e6603c82613576565b60ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300541661489057565b7fd93c0665000000000000000000000000000000000000000000000000000000005f5260045ffd5b919082018092116148c557565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b6136a29065ffffffffffff61490642614eca565b614930837f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00614b1e565b509279ffffffffffffffffffffffffffffffffffffffffffffffffffff6040519461495a8661355a565b16845290821660208085019182525f9283527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb01905260409091209251905190911660d01b7fffffffffffff00000000000000000000000000000000000000000000000000001679ffffffffffffffffffffffffffffffffffffffffffffffffffff91909116179055565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c1615614a1357565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b907f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb005479ffffffffffffffffffffffffffffffffffffffffffffffffffff8116925f527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb0160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff614ad642614eca565b9160d01c91160365ffffffffffff81116148c5578465ffffffffffff614b0b9360d01c60018082119118026001189216614dfe565b8103908111150291828103908111150290565b919091600181549179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316945f520160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff614ad642614eca565b7f01ffc9a7000000000000000000000000000000000000000000000000000000005f527f01ffc9a70000000000000000000000000000000000000000000000000000000060045260205f60248184617530fa5f511515601f3d111681614c54575b5015614c4f575f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fffffffff00000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681614c49575090565b90501590565b505f90565b90505f614bdf565b908015614d3d57614c8d827f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00614b1e565b9091908111614d3657614ca990614ca342614eca565b926148b8565b9179ffffffffffffffffffffffffffffffffffffffffffffffffffff8311614d0557614d00929165ffffffffffff9179ffffffffffffffffffffffffffffffffffffffffffffffffffff6040519461495a8661355a565b600190565b827f6dfcc650000000000000000000000000000000000000000000000000000000005f5260d060045260245260445ffd5b5050505f90565b5050600190565b905f8091602081519101845af48080614deb575b15614d785750506040513d81523d5f602083013e60203d82010160405290565b15614db2576001600160a01b03907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d15614dc3576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d151580614d585750813b1515614d58565b90915f198383099280830292838086109503948086039514614e8f5784831115614e775790829109815f0382168092046002816003021880820260020302808202600203028082026002030280820260020302808202600203028091026002030293600183805f03040190848311900302920304170290565b82634e487b715f52156003026011186020526024601cfd5b505080925015614e9d570490565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b65ffffffffffff8111614ee25765ffffffffffff1690565b7f6dfcc650000000000000000000000000000000000000000000000000000000005f52603060045260245260445ffdfea164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
}

// ContractABI is the input ABI used to generate the binding from.
// Deprecated: Use ContractMetaData.ABI instead.
var ContractABI = ContractMetaData.ABI

// ContractBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ContractMetaData.Bin instead.
var ContractBin = ContractMetaData.Bin

// DeployContract deploys a new Ethereum contract, binding an instance of Contract to it.
func DeployContract(auth *bind.TransactOpts, backend bind.ContractBackend) (common.Address, *types.Transaction, *Contract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ContractBin), backend)
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	return address, tx, &Contract{ContractCaller: ContractCaller{contract: contract}, ContractTransactor: ContractTransactor{contract: contract}, ContractFilterer: ContractFilterer{contract: contract}}, nil
}

// Contract is an auto generated Go binding around an Ethereum contract.
type Contract struct {
	ContractCaller     // Read-only binding to the contract
	ContractTransactor // Write-only binding to the contract
	ContractFilterer   // Log filterer for contract events
}

// ContractCaller is an auto generated read-only Go binding around an Ethereum contract.
type ContractCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ContractTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ContractFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ContractSession struct {
	Contract     *Contract         // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ContractCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ContractCallerSession struct {
	Contract *ContractCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts   // Call options to use throughout this session
}

// ContractTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ContractTransactorSession struct {
	Contract     *ContractTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// ContractRaw is an auto generated low-level Go binding around an Ethereum contract.
type ContractRaw struct {
	Contract *Contract // Generic contract binding to access the raw methods on
}

// ContractCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ContractCallerRaw struct {
	Contract *ContractCaller // Generic read-only contract binding to access the raw methods on
}

// ContractTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ContractTransactorRaw struct {
	Contract *ContractTransactor // Generic write-only contract binding to access the raw methods on
}

// NewContract creates a new instance of Contract, bound to a specific deployed contract.
func NewContract(address common.Address, backend bind.ContractBackend) (*Contract, error) {
	contract, err := bindContract(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &Contract{ContractCaller: ContractCaller{contract: contract}, ContractTransactor: ContractTransactor{contract: contract}, ContractFilterer: ContractFilterer{contract: contract}}, nil
}

// NewContractCaller creates a new read-only instance of Contract, bound to a specific deployed contract.
func NewContractCaller(address common.Address, caller bind.ContractCaller) (*ContractCaller, error) {
	contract, err := bindContract(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ContractCaller{contract: contract}, nil
}

// NewContractTransactor creates a new write-only instance of Contract, bound to a specific deployed contract.
func NewContractTransactor(address common.Address, transactor bind.ContractTransactor) (*ContractTransactor, error) {
	contract, err := bindContract(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ContractTransactor{contract: contract}, nil
}

// NewContractFilterer creates a new log filterer instance of Contract, bound to a specific deployed contract.
func NewContractFilterer(address common.Address, filterer bind.ContractFilterer) (*ContractFilterer, error) {
	contract, err := bindContract(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ContractFilterer{contract: contract}, nil
}

// bindContract binds a generic wrapper to an already deployed contract.
func bindContract(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Contract *ContractRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Contract.Contract.ContractCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Contract *ContractRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.Contract.ContractTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Contract *ContractRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Contract.Contract.ContractTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Contract *ContractCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Contract.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Contract *ContractTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Contract *ContractTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Contract.Contract.contract.Transact(opts, method, params...)
}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_Contract *ContractCaller) UPGRADEINTERFACEVERSION(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "UPGRADE_INTERFACE_VERSION")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_Contract *ContractSession) UPGRADEINTERFACEVERSION() (string, error) {
	return _Contract.Contract.UPGRADEINTERFACEVERSION(&_Contract.CallOpts)
}

// UPGRADEINTERFACEVERSION is a free data retrieval call binding the contract method 0xad3cb1cc.
//
// Solidity: function UPGRADE_INTERFACE_VERSION() view returns(string)
func (_Contract *ContractCallerSession) UPGRADEINTERFACEVERSION() (string, error) {
	return _Contract.Contract.UPGRADEINTERFACEVERSION(&_Contract.CallOpts)
}

// Allowance is a free data retrieval call binding the contract method 0xdd62ed3e.
//
// Solidity: function allowance(address owner, address spender) view returns(uint256)
func (_Contract *ContractCaller) Allowance(opts *bind.CallOpts, owner common.Address, spender common.Address) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "allowance", owner, spender)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Allowance is a free data retrieval call binding the contract method 0xdd62ed3e.
//
// Solidity: function allowance(address owner, address spender) view returns(uint256)
func (_Contract *ContractSession) Allowance(owner common.Address, spender common.Address) (*big.Int, error) {
	return _Contract.Contract.Allowance(&_Contract.CallOpts, owner, spender)
}

// Allowance is a free data retrieval call binding the contract method 0xdd62ed3e.
//
// Solidity: function allowance(address owner, address spender) view returns(uint256)
func (_Contract *ContractCallerSession) Allowance(owner common.Address, spender common.Address) (*big.Int, error) {
	return _Contract.Contract.Allowance(&_Contract.CallOpts, owner, spender)
}

// Authority is a free data retrieval call binding the contract method 0xbf7e214f.
//
// Solidity: function authority() view returns(address)
func (_Contract *ContractCaller) Authority(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "authority")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Authority is a free data retrieval call binding the contract method 0xbf7e214f.
//
// Solidity: function authority() view returns(address)
func (_Contract *ContractSession) Authority() (common.Address, error) {
	return _Contract.Contract.Authority(&_Contract.CallOpts)
}

// Authority is a free data retrieval call binding the contract method 0xbf7e214f.
//
// Solidity: function authority() view returns(address)
func (_Contract *ContractCallerSession) Authority() (common.Address, error) {
	return _Contract.Contract.Authority(&_Contract.CallOpts)
}

// BalanceOf is a free data retrieval call binding the contract method 0x70a08231.
//
// Solidity: function balanceOf(address account) view returns(uint256)
func (_Contract *ContractCaller) BalanceOf(opts *bind.CallOpts, account common.Address) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "balanceOf", account)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// BalanceOf is a free data retrieval call binding the contract method 0x70a08231.
//
// Solidity: function balanceOf(address account) view returns(uint256)
func (_Contract *ContractSession) BalanceOf(account common.Address) (*big.Int, error) {
	return _Contract.Contract.BalanceOf(&_Contract.CallOpts, account)
}

// BalanceOf is a free data retrieval call binding the contract method 0x70a08231.
//
// Solidity: function balanceOf(address account) view returns(uint256)
func (_Contract *ContractCallerSession) BalanceOf(account common.Address) (*big.Int, error) {
	return _Contract.Contract.BalanceOf(&_Contract.CallOpts, account)
}

// Decimals is a free data retrieval call binding the contract method 0x313ce567.
//
// Solidity: function decimals() view returns(uint8)
func (_Contract *ContractCaller) Decimals(opts *bind.CallOpts) (uint8, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "decimals")

	if err != nil {
		return *new(uint8), err
	}

	out0 := *abi.ConvertType(out[0], new(uint8)).(*uint8)

	return out0, err

}

// Decimals is a free data retrieval call binding the contract method 0x313ce567.
//
// Solidity: function decimals() view returns(uint8)
func (_Contract *ContractSession) Decimals() (uint8, error) {
	return _Contract.Contract.Decimals(&_Contract.CallOpts)
}

// Decimals is a free data retrieval call binding the contract method 0x313ce567.
//
// Solidity: function decimals() view returns(uint8)
func (_Contract *ContractCallerSession) Decimals() (uint8, error) {
	return _Contract.Contract.Decimals(&_Contract.CallOpts)
}

// GetIFTBridge is a free data retrieval call binding the contract method 0xe529a26d.
//
// Solidity: function getIFTBridge(string clientId) view returns((string,string,address))
func (_Contract *ContractCaller) GetIFTBridge(opts *bind.CallOpts, clientId string) (IIFTMsgsIFTBridge, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIFTBridge", clientId)

	if err != nil {
		return *new(IIFTMsgsIFTBridge), err
	}

	out0 := *abi.ConvertType(out[0], new(IIFTMsgsIFTBridge)).(*IIFTMsgsIFTBridge)

	return out0, err

}

// GetIFTBridge is a free data retrieval call binding the contract method 0xe529a26d.
//
// Solidity: function getIFTBridge(string clientId) view returns((string,string,address))
func (_Contract *ContractSession) GetIFTBridge(clientId string) (IIFTMsgsIFTBridge, error) {
	return _Contract.Contract.GetIFTBridge(&_Contract.CallOpts, clientId)
}

// GetIFTBridge is a free data retrieval call binding the contract method 0xe529a26d.
//
// Solidity: function getIFTBridge(string clientId) view returns((string,string,address))
func (_Contract *ContractCallerSession) GetIFTBridge(clientId string) (IIFTMsgsIFTBridge, error) {
	return _Contract.Contract.GetIFTBridge(&_Contract.CallOpts, clientId)
}

// GetIFTRateLimit is a free data retrieval call binding the contract method 0x997d99b8.
//
// Solidity: function getIFTRateLimit() view returns(uint208 capacity, uint48 window)
func (_Contract *ContractCaller) GetIFTRateLimit(opts *bind.CallOpts) (struct {
	Capacity *big.Int
	Window   *big.Int
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIFTRateLimit")

	outstruct := new(struct {
		Capacity *big.Int
		Window   *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Capacity = *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	outstruct.Window = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// GetIFTRateLimit is a free data retrieval call binding the contract method 0x997d99b8.
//
// Solidity: function getIFTRateLimit() view returns(uint208 capacity, uint48 window)
func (_Contract *ContractSession) GetIFTRateLimit() (struct {
	Capacity *big.Int
	Window   *big.Int
}, error) {
	return _Contract.Contract.GetIFTRateLimit(&_Contract.CallOpts)
}

// GetIFTRateLimit is a free data retrieval call binding the contract method 0x997d99b8.
//
// Solidity: function getIFTRateLimit() view returns(uint208 capacity, uint48 window)
func (_Contract *ContractCallerSession) GetIFTRateLimit() (struct {
	Capacity *big.Int
	Window   *big.Int
}, error) {
	return _Contract.Contract.GetIFTRateLimit(&_Contract.CallOpts)
}

// GetIFTRateLimitAvailable is a free data retrieval call binding the contract method 0xa50e31f1.
//
// Solidity: function getIFTRateLimitAvailable() view returns(uint256, uint256)
func (_Contract *ContractCaller) GetIFTRateLimitAvailable(opts *bind.CallOpts) (*big.Int, *big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIFTRateLimitAvailable")

	if err != nil {
		return *new(*big.Int), *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	out1 := *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)

	return out0, out1, err

}

// GetIFTRateLimitAvailable is a free data retrieval call binding the contract method 0xa50e31f1.
//
// Solidity: function getIFTRateLimitAvailable() view returns(uint256, uint256)
func (_Contract *ContractSession) GetIFTRateLimitAvailable() (*big.Int, *big.Int, error) {
	return _Contract.Contract.GetIFTRateLimitAvailable(&_Contract.CallOpts)
}

// GetIFTRateLimitAvailable is a free data retrieval call binding the contract method 0xa50e31f1.
//
// Solidity: function getIFTRateLimitAvailable() view returns(uint256, uint256)
func (_Contract *ContractCallerSession) GetIFTRateLimitAvailable() (*big.Int, *big.Int, error) {
	return _Contract.Contract.GetIFTRateLimitAvailable(&_Contract.CallOpts)
}

// GetPendingTransfer is a free data retrieval call binding the contract method 0x9226e083.
//
// Solidity: function getPendingTransfer(string clientId, uint64 sequence) view returns((address,uint256))
func (_Contract *ContractCaller) GetPendingTransfer(opts *bind.CallOpts, clientId string, sequence uint64) (IIFTMsgsPendingTransfer, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getPendingTransfer", clientId, sequence)

	if err != nil {
		return *new(IIFTMsgsPendingTransfer), err
	}

	out0 := *abi.ConvertType(out[0], new(IIFTMsgsPendingTransfer)).(*IIFTMsgsPendingTransfer)

	return out0, err

}

// GetPendingTransfer is a free data retrieval call binding the contract method 0x9226e083.
//
// Solidity: function getPendingTransfer(string clientId, uint64 sequence) view returns((address,uint256))
func (_Contract *ContractSession) GetPendingTransfer(clientId string, sequence uint64) (IIFTMsgsPendingTransfer, error) {
	return _Contract.Contract.GetPendingTransfer(&_Contract.CallOpts, clientId, sequence)
}

// GetPendingTransfer is a free data retrieval call binding the contract method 0x9226e083.
//
// Solidity: function getPendingTransfer(string clientId, uint64 sequence) view returns((address,uint256))
func (_Contract *ContractCallerSession) GetPendingTransfer(clientId string, sequence uint64) (IIFTMsgsPendingTransfer, error) {
	return _Contract.Contract.GetPendingTransfer(&_Contract.CallOpts, clientId, sequence)
}

// Ics27 is a free data retrieval call binding the contract method 0x599deb48.
//
// Solidity: function ics27() view returns(address)
func (_Contract *ContractCaller) Ics27(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "ics27")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Ics27 is a free data retrieval call binding the contract method 0x599deb48.
//
// Solidity: function ics27() view returns(address)
func (_Contract *ContractSession) Ics27() (common.Address, error) {
	return _Contract.Contract.Ics27(&_Contract.CallOpts)
}

// Ics27 is a free data retrieval call binding the contract method 0x599deb48.
//
// Solidity: function ics27() view returns(address)
func (_Contract *ContractCallerSession) Ics27() (common.Address, error) {
	return _Contract.Contract.Ics27(&_Contract.CallOpts)
}

// IsConsumingScheduledOp is a free data retrieval call binding the contract method 0x8fb36037.
//
// Solidity: function isConsumingScheduledOp() view returns(bytes4)
func (_Contract *ContractCaller) IsConsumingScheduledOp(opts *bind.CallOpts) ([4]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "isConsumingScheduledOp")

	if err != nil {
		return *new([4]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([4]byte)).(*[4]byte)

	return out0, err

}

// IsConsumingScheduledOp is a free data retrieval call binding the contract method 0x8fb36037.
//
// Solidity: function isConsumingScheduledOp() view returns(bytes4)
func (_Contract *ContractSession) IsConsumingScheduledOp() ([4]byte, error) {
	return _Contract.Contract.IsConsumingScheduledOp(&_Contract.CallOpts)
}

// IsConsumingScheduledOp is a free data retrieval call binding the contract method 0x8fb36037.
//
// Solidity: function isConsumingScheduledOp() view returns(bytes4)
func (_Contract *ContractCallerSession) IsConsumingScheduledOp() ([4]byte, error) {
	return _Contract.Contract.IsConsumingScheduledOp(&_Contract.CallOpts)
}

// Name is a free data retrieval call binding the contract method 0x06fdde03.
//
// Solidity: function name() view returns(string)
func (_Contract *ContractCaller) Name(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "name")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// Name is a free data retrieval call binding the contract method 0x06fdde03.
//
// Solidity: function name() view returns(string)
func (_Contract *ContractSession) Name() (string, error) {
	return _Contract.Contract.Name(&_Contract.CallOpts)
}

// Name is a free data retrieval call binding the contract method 0x06fdde03.
//
// Solidity: function name() view returns(string)
func (_Contract *ContractCallerSession) Name() (string, error) {
	return _Contract.Contract.Name(&_Contract.CallOpts)
}

// Paused is a free data retrieval call binding the contract method 0x5c975abb.
//
// Solidity: function paused() view returns(bool)
func (_Contract *ContractCaller) Paused(opts *bind.CallOpts) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "paused")

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// Paused is a free data retrieval call binding the contract method 0x5c975abb.
//
// Solidity: function paused() view returns(bool)
func (_Contract *ContractSession) Paused() (bool, error) {
	return _Contract.Contract.Paused(&_Contract.CallOpts)
}

// Paused is a free data retrieval call binding the contract method 0x5c975abb.
//
// Solidity: function paused() view returns(bool)
func (_Contract *ContractCallerSession) Paused() (bool, error) {
	return _Contract.Contract.Paused(&_Contract.CallOpts)
}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_Contract *ContractCaller) ProxiableUUID(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "proxiableUUID")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_Contract *ContractSession) ProxiableUUID() ([32]byte, error) {
	return _Contract.Contract.ProxiableUUID(&_Contract.CallOpts)
}

// ProxiableUUID is a free data retrieval call binding the contract method 0x52d1902d.
//
// Solidity: function proxiableUUID() view returns(bytes32)
func (_Contract *ContractCallerSession) ProxiableUUID() ([32]byte, error) {
	return _Contract.Contract.ProxiableUUID(&_Contract.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_Contract *ContractCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_Contract *ContractSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _Contract.Contract.SupportsInterface(&_Contract.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_Contract *ContractCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _Contract.Contract.SupportsInterface(&_Contract.CallOpts, interfaceId)
}

// Symbol is a free data retrieval call binding the contract method 0x95d89b41.
//
// Solidity: function symbol() view returns(string)
func (_Contract *ContractCaller) Symbol(opts *bind.CallOpts) (string, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "symbol")

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// Symbol is a free data retrieval call binding the contract method 0x95d89b41.
//
// Solidity: function symbol() view returns(string)
func (_Contract *ContractSession) Symbol() (string, error) {
	return _Contract.Contract.Symbol(&_Contract.CallOpts)
}

// Symbol is a free data retrieval call binding the contract method 0x95d89b41.
//
// Solidity: function symbol() view returns(string)
func (_Contract *ContractCallerSession) Symbol() (string, error) {
	return _Contract.Contract.Symbol(&_Contract.CallOpts)
}

// TotalSupply is a free data retrieval call binding the contract method 0x18160ddd.
//
// Solidity: function totalSupply() view returns(uint256)
func (_Contract *ContractCaller) TotalSupply(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "totalSupply")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// TotalSupply is a free data retrieval call binding the contract method 0x18160ddd.
//
// Solidity: function totalSupply() view returns(uint256)
func (_Contract *ContractSession) TotalSupply() (*big.Int, error) {
	return _Contract.Contract.TotalSupply(&_Contract.CallOpts)
}

// TotalSupply is a free data retrieval call binding the contract method 0x18160ddd.
//
// Solidity: function totalSupply() view returns(uint256)
func (_Contract *ContractCallerSession) TotalSupply() (*big.Int, error) {
	return _Contract.Contract.TotalSupply(&_Contract.CallOpts)
}

// Approve is a paid mutator transaction binding the contract method 0x095ea7b3.
//
// Solidity: function approve(address spender, uint256 value) returns(bool)
func (_Contract *ContractTransactor) Approve(opts *bind.TransactOpts, spender common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "approve", spender, value)
}

// Approve is a paid mutator transaction binding the contract method 0x095ea7b3.
//
// Solidity: function approve(address spender, uint256 value) returns(bool)
func (_Contract *ContractSession) Approve(spender common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Approve(&_Contract.TransactOpts, spender, value)
}

// Approve is a paid mutator transaction binding the contract method 0x095ea7b3.
//
// Solidity: function approve(address spender, uint256 value) returns(bool)
func (_Contract *ContractTransactorSession) Approve(spender common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Approve(&_Contract.TransactOpts, spender, value)
}

// Burn is a paid mutator transaction binding the contract method 0x42966c68.
//
// Solidity: function burn(uint256 value) returns()
func (_Contract *ContractTransactor) Burn(opts *bind.TransactOpts, value *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "burn", value)
}

// Burn is a paid mutator transaction binding the contract method 0x42966c68.
//
// Solidity: function burn(uint256 value) returns()
func (_Contract *ContractSession) Burn(value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Burn(&_Contract.TransactOpts, value)
}

// Burn is a paid mutator transaction binding the contract method 0x42966c68.
//
// Solidity: function burn(uint256 value) returns()
func (_Contract *ContractTransactorSession) Burn(value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Burn(&_Contract.TransactOpts, value)
}

// BurnFrom is a paid mutator transaction binding the contract method 0x79cc6790.
//
// Solidity: function burnFrom(address account, uint256 value) returns()
func (_Contract *ContractTransactor) BurnFrom(opts *bind.TransactOpts, account common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "burnFrom", account, value)
}

// BurnFrom is a paid mutator transaction binding the contract method 0x79cc6790.
//
// Solidity: function burnFrom(address account, uint256 value) returns()
func (_Contract *ContractSession) BurnFrom(account common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.BurnFrom(&_Contract.TransactOpts, account, value)
}

// BurnFrom is a paid mutator transaction binding the contract method 0x79cc6790.
//
// Solidity: function burnFrom(address account, uint256 value) returns()
func (_Contract *ContractTransactorSession) BurnFrom(account common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.BurnFrom(&_Contract.TransactOpts, account, value)
}

// IftMint is a paid mutator transaction binding the contract method 0x0a7244e7.
//
// Solidity: function iftMint(address receiver, uint256 amount) returns()
func (_Contract *ContractTransactor) IftMint(opts *bind.TransactOpts, receiver common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "iftMint", receiver, amount)
}

// IftMint is a paid mutator transaction binding the contract method 0x0a7244e7.
//
// Solidity: function iftMint(address receiver, uint256 amount) returns()
func (_Contract *ContractSession) IftMint(receiver common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.IftMint(&_Contract.TransactOpts, receiver, amount)
}

// IftMint is a paid mutator transaction binding the contract method 0x0a7244e7.
//
// Solidity: function iftMint(address receiver, uint256 amount) returns()
func (_Contract *ContractTransactorSession) IftMint(receiver common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.IftMint(&_Contract.TransactOpts, receiver, amount)
}

// IftTransfer is a paid mutator transaction binding the contract method 0x711708b3.
//
// Solidity: function iftTransfer(string clientId, string receiver, uint256 amount, uint64 timeoutTimestamp) returns()
func (_Contract *ContractTransactor) IftTransfer(opts *bind.TransactOpts, clientId string, receiver string, amount *big.Int, timeoutTimestamp uint64) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "iftTransfer", clientId, receiver, amount, timeoutTimestamp)
}

// IftTransfer is a paid mutator transaction binding the contract method 0x711708b3.
//
// Solidity: function iftTransfer(string clientId, string receiver, uint256 amount, uint64 timeoutTimestamp) returns()
func (_Contract *ContractSession) IftTransfer(clientId string, receiver string, amount *big.Int, timeoutTimestamp uint64) (*types.Transaction, error) {
	return _Contract.Contract.IftTransfer(&_Contract.TransactOpts, clientId, receiver, amount, timeoutTimestamp)
}

// IftTransfer is a paid mutator transaction binding the contract method 0x711708b3.
//
// Solidity: function iftTransfer(string clientId, string receiver, uint256 amount, uint64 timeoutTimestamp) returns()
func (_Contract *ContractTransactorSession) IftTransfer(clientId string, receiver string, amount *big.Int, timeoutTimestamp uint64) (*types.Transaction, error) {
	return _Contract.Contract.IftTransfer(&_Contract.TransactOpts, clientId, receiver, amount, timeoutTimestamp)
}

// IftTransfer0 is a paid mutator transaction binding the contract method 0xd88a36fe.
//
// Solidity: function iftTransfer(string clientId, string receiver, uint256 amount) returns()
func (_Contract *ContractTransactor) IftTransfer0(opts *bind.TransactOpts, clientId string, receiver string, amount *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "iftTransfer0", clientId, receiver, amount)
}

// IftTransfer0 is a paid mutator transaction binding the contract method 0xd88a36fe.
//
// Solidity: function iftTransfer(string clientId, string receiver, uint256 amount) returns()
func (_Contract *ContractSession) IftTransfer0(clientId string, receiver string, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.IftTransfer0(&_Contract.TransactOpts, clientId, receiver, amount)
}

// IftTransfer0 is a paid mutator transaction binding the contract method 0xd88a36fe.
//
// Solidity: function iftTransfer(string clientId, string receiver, uint256 amount) returns()
func (_Contract *ContractTransactorSession) IftTransfer0(clientId string, receiver string, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.IftTransfer0(&_Contract.TransactOpts, clientId, receiver, amount)
}

// Initialize is a paid mutator transaction binding the contract method 0x613d25bb.
//
// Solidity: function initialize(address authority_, string erc20Name, string erc20Symbol, address ics27Gmp) returns()
func (_Contract *ContractTransactor) Initialize(opts *bind.TransactOpts, authority_ common.Address, erc20Name string, erc20Symbol string, ics27Gmp common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "initialize", authority_, erc20Name, erc20Symbol, ics27Gmp)
}

// Initialize is a paid mutator transaction binding the contract method 0x613d25bb.
//
// Solidity: function initialize(address authority_, string erc20Name, string erc20Symbol, address ics27Gmp) returns()
func (_Contract *ContractSession) Initialize(authority_ common.Address, erc20Name string, erc20Symbol string, ics27Gmp common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, authority_, erc20Name, erc20Symbol, ics27Gmp)
}

// Initialize is a paid mutator transaction binding the contract method 0x613d25bb.
//
// Solidity: function initialize(address authority_, string erc20Name, string erc20Symbol, address ics27Gmp) returns()
func (_Contract *ContractTransactorSession) Initialize(authority_ common.Address, erc20Name string, erc20Symbol string, ics27Gmp common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, authority_, erc20Name, erc20Symbol, ics27Gmp)
}

// Mint is a paid mutator transaction binding the contract method 0x40c10f19.
//
// Solidity: function mint(address mintAddress, uint256 amount) returns()
func (_Contract *ContractTransactor) Mint(opts *bind.TransactOpts, mintAddress common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "mint", mintAddress, amount)
}

// Mint is a paid mutator transaction binding the contract method 0x40c10f19.
//
// Solidity: function mint(address mintAddress, uint256 amount) returns()
func (_Contract *ContractSession) Mint(mintAddress common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Mint(&_Contract.TransactOpts, mintAddress, amount)
}

// Mint is a paid mutator transaction binding the contract method 0x40c10f19.
//
// Solidity: function mint(address mintAddress, uint256 amount) returns()
func (_Contract *ContractTransactorSession) Mint(mintAddress common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Mint(&_Contract.TransactOpts, mintAddress, amount)
}

// OnAckPacket is a paid mutator transaction binding the contract method 0x8dfcd9ad.
//
// Solidity: function onAckPacket(bool success, (string,string,uint64,(string,string,string,string,bytes),bytes,address) msg_) returns()
func (_Contract *ContractTransactor) OnAckPacket(opts *bind.TransactOpts, success bool, msg_ IIBCAppCallbacksOnAcknowledgementPacketCallback) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "onAckPacket", success, msg_)
}

// OnAckPacket is a paid mutator transaction binding the contract method 0x8dfcd9ad.
//
// Solidity: function onAckPacket(bool success, (string,string,uint64,(string,string,string,string,bytes),bytes,address) msg_) returns()
func (_Contract *ContractSession) OnAckPacket(success bool, msg_ IIBCAppCallbacksOnAcknowledgementPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnAckPacket(&_Contract.TransactOpts, success, msg_)
}

// OnAckPacket is a paid mutator transaction binding the contract method 0x8dfcd9ad.
//
// Solidity: function onAckPacket(bool success, (string,string,uint64,(string,string,string,string,bytes),bytes,address) msg_) returns()
func (_Contract *ContractTransactorSession) OnAckPacket(success bool, msg_ IIBCAppCallbacksOnAcknowledgementPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnAckPacket(&_Contract.TransactOpts, success, msg_)
}

// OnTimeoutPacket is a paid mutator transaction binding the contract method 0x5e32b6b6.
//
// Solidity: function onTimeoutPacket((string,string,uint64,(string,string,string,string,bytes),address) msg_) returns()
func (_Contract *ContractTransactor) OnTimeoutPacket(opts *bind.TransactOpts, msg_ IIBCAppCallbacksOnTimeoutPacketCallback) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "onTimeoutPacket", msg_)
}

// OnTimeoutPacket is a paid mutator transaction binding the contract method 0x5e32b6b6.
//
// Solidity: function onTimeoutPacket((string,string,uint64,(string,string,string,string,bytes),address) msg_) returns()
func (_Contract *ContractSession) OnTimeoutPacket(msg_ IIBCAppCallbacksOnTimeoutPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnTimeoutPacket(&_Contract.TransactOpts, msg_)
}

// OnTimeoutPacket is a paid mutator transaction binding the contract method 0x5e32b6b6.
//
// Solidity: function onTimeoutPacket((string,string,uint64,(string,string,string,string,bytes),address) msg_) returns()
func (_Contract *ContractTransactorSession) OnTimeoutPacket(msg_ IIBCAppCallbacksOnTimeoutPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnTimeoutPacket(&_Contract.TransactOpts, msg_)
}

// Pause is a paid mutator transaction binding the contract method 0x8456cb59.
//
// Solidity: function pause() returns()
func (_Contract *ContractTransactor) Pause(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "pause")
}

// Pause is a paid mutator transaction binding the contract method 0x8456cb59.
//
// Solidity: function pause() returns()
func (_Contract *ContractSession) Pause() (*types.Transaction, error) {
	return _Contract.Contract.Pause(&_Contract.TransactOpts)
}

// Pause is a paid mutator transaction binding the contract method 0x8456cb59.
//
// Solidity: function pause() returns()
func (_Contract *ContractTransactorSession) Pause() (*types.Transaction, error) {
	return _Contract.Contract.Pause(&_Contract.TransactOpts)
}

// RegisterIFTBridge is a paid mutator transaction binding the contract method 0xd638f98b.
//
// Solidity: function registerIFTBridge(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor) returns()
func (_Contract *ContractTransactor) RegisterIFTBridge(opts *bind.TransactOpts, clientId string, counterpartyIFTAddress string, iftSendCallConstructor common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "registerIFTBridge", clientId, counterpartyIFTAddress, iftSendCallConstructor)
}

// RegisterIFTBridge is a paid mutator transaction binding the contract method 0xd638f98b.
//
// Solidity: function registerIFTBridge(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor) returns()
func (_Contract *ContractSession) RegisterIFTBridge(clientId string, counterpartyIFTAddress string, iftSendCallConstructor common.Address) (*types.Transaction, error) {
	return _Contract.Contract.RegisterIFTBridge(&_Contract.TransactOpts, clientId, counterpartyIFTAddress, iftSendCallConstructor)
}

// RegisterIFTBridge is a paid mutator transaction binding the contract method 0xd638f98b.
//
// Solidity: function registerIFTBridge(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor) returns()
func (_Contract *ContractTransactorSession) RegisterIFTBridge(clientId string, counterpartyIFTAddress string, iftSendCallConstructor common.Address) (*types.Transaction, error) {
	return _Contract.Contract.RegisterIFTBridge(&_Contract.TransactOpts, clientId, counterpartyIFTAddress, iftSendCallConstructor)
}

// RemoveIFTBridge is a paid mutator transaction binding the contract method 0xb8ce2418.
//
// Solidity: function removeIFTBridge(string clientId) returns()
func (_Contract *ContractTransactor) RemoveIFTBridge(opts *bind.TransactOpts, clientId string) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "removeIFTBridge", clientId)
}

// RemoveIFTBridge is a paid mutator transaction binding the contract method 0xb8ce2418.
//
// Solidity: function removeIFTBridge(string clientId) returns()
func (_Contract *ContractSession) RemoveIFTBridge(clientId string) (*types.Transaction, error) {
	return _Contract.Contract.RemoveIFTBridge(&_Contract.TransactOpts, clientId)
}

// RemoveIFTBridge is a paid mutator transaction binding the contract method 0xb8ce2418.
//
// Solidity: function removeIFTBridge(string clientId) returns()
func (_Contract *ContractTransactorSession) RemoveIFTBridge(clientId string) (*types.Transaction, error) {
	return _Contract.Contract.RemoveIFTBridge(&_Contract.TransactOpts, clientId)
}

// SetAuthority is a paid mutator transaction binding the contract method 0x7a9e5e4b.
//
// Solidity: function setAuthority(address newAuthority) returns()
func (_Contract *ContractTransactor) SetAuthority(opts *bind.TransactOpts, newAuthority common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "setAuthority", newAuthority)
}

// SetAuthority is a paid mutator transaction binding the contract method 0x7a9e5e4b.
//
// Solidity: function setAuthority(address newAuthority) returns()
func (_Contract *ContractSession) SetAuthority(newAuthority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SetAuthority(&_Contract.TransactOpts, newAuthority)
}

// SetAuthority is a paid mutator transaction binding the contract method 0x7a9e5e4b.
//
// Solidity: function setAuthority(address newAuthority) returns()
func (_Contract *ContractTransactorSession) SetAuthority(newAuthority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SetAuthority(&_Contract.TransactOpts, newAuthority)
}

// SetIFTRateLimit is a paid mutator transaction binding the contract method 0x419e41c6.
//
// Solidity: function setIFTRateLimit(uint208 capacity, uint48 window) returns()
func (_Contract *ContractTransactor) SetIFTRateLimit(opts *bind.TransactOpts, capacity *big.Int, window *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "setIFTRateLimit", capacity, window)
}

// SetIFTRateLimit is a paid mutator transaction binding the contract method 0x419e41c6.
//
// Solidity: function setIFTRateLimit(uint208 capacity, uint48 window) returns()
func (_Contract *ContractSession) SetIFTRateLimit(capacity *big.Int, window *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.SetIFTRateLimit(&_Contract.TransactOpts, capacity, window)
}

// SetIFTRateLimit is a paid mutator transaction binding the contract method 0x419e41c6.
//
// Solidity: function setIFTRateLimit(uint208 capacity, uint48 window) returns()
func (_Contract *ContractTransactorSession) SetIFTRateLimit(capacity *big.Int, window *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.SetIFTRateLimit(&_Contract.TransactOpts, capacity, window)
}

// Transfer is a paid mutator transaction binding the contract method 0xa9059cbb.
//
// Solidity: function transfer(address to, uint256 value) returns(bool)
func (_Contract *ContractTransactor) Transfer(opts *bind.TransactOpts, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "transfer", to, value)
}

// Transfer is a paid mutator transaction binding the contract method 0xa9059cbb.
//
// Solidity: function transfer(address to, uint256 value) returns(bool)
func (_Contract *ContractSession) Transfer(to common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Transfer(&_Contract.TransactOpts, to, value)
}

// Transfer is a paid mutator transaction binding the contract method 0xa9059cbb.
//
// Solidity: function transfer(address to, uint256 value) returns(bool)
func (_Contract *ContractTransactorSession) Transfer(to common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Transfer(&_Contract.TransactOpts, to, value)
}

// TransferFrom is a paid mutator transaction binding the contract method 0x23b872dd.
//
// Solidity: function transferFrom(address from, address to, uint256 value) returns(bool)
func (_Contract *ContractTransactor) TransferFrom(opts *bind.TransactOpts, from common.Address, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "transferFrom", from, to, value)
}

// TransferFrom is a paid mutator transaction binding the contract method 0x23b872dd.
//
// Solidity: function transferFrom(address from, address to, uint256 value) returns(bool)
func (_Contract *ContractSession) TransferFrom(from common.Address, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.TransferFrom(&_Contract.TransactOpts, from, to, value)
}

// TransferFrom is a paid mutator transaction binding the contract method 0x23b872dd.
//
// Solidity: function transferFrom(address from, address to, uint256 value) returns(bool)
func (_Contract *ContractTransactorSession) TransferFrom(from common.Address, to common.Address, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.TransferFrom(&_Contract.TransactOpts, from, to, value)
}

// Unpause is a paid mutator transaction binding the contract method 0x3f4ba83a.
//
// Solidity: function unpause() returns()
func (_Contract *ContractTransactor) Unpause(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "unpause")
}

// Unpause is a paid mutator transaction binding the contract method 0x3f4ba83a.
//
// Solidity: function unpause() returns()
func (_Contract *ContractSession) Unpause() (*types.Transaction, error) {
	return _Contract.Contract.Unpause(&_Contract.TransactOpts)
}

// Unpause is a paid mutator transaction binding the contract method 0x3f4ba83a.
//
// Solidity: function unpause() returns()
func (_Contract *ContractTransactorSession) Unpause() (*types.Transaction, error) {
	return _Contract.Contract.Unpause(&_Contract.TransactOpts)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_Contract *ContractTransactor) UpgradeToAndCall(opts *bind.TransactOpts, newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "upgradeToAndCall", newImplementation, data)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_Contract *ContractSession) UpgradeToAndCall(newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeToAndCall(&_Contract.TransactOpts, newImplementation, data)
}

// UpgradeToAndCall is a paid mutator transaction binding the contract method 0x4f1ef286.
//
// Solidity: function upgradeToAndCall(address newImplementation, bytes data) payable returns()
func (_Contract *ContractTransactorSession) UpgradeToAndCall(newImplementation common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeToAndCall(&_Contract.TransactOpts, newImplementation, data)
}

// ContractApprovalIterator is returned from FilterApproval and is used to iterate over the raw logs and unpacked data for Approval events raised by the Contract contract.
type ContractApprovalIterator struct {
	Event *ContractApproval // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractApprovalIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractApproval)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractApproval)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractApprovalIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractApprovalIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractApproval represents a Approval event raised by the Contract contract.
type ContractApproval struct {
	Owner   common.Address
	Spender common.Address
	Value   *big.Int
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterApproval is a free log retrieval operation binding the contract event 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925.
//
// Solidity: event Approval(address indexed owner, address indexed spender, uint256 value)
func (_Contract *ContractFilterer) FilterApproval(opts *bind.FilterOpts, owner []common.Address, spender []common.Address) (*ContractApprovalIterator, error) {

	var ownerRule []interface{}
	for _, ownerItem := range owner {
		ownerRule = append(ownerRule, ownerItem)
	}
	var spenderRule []interface{}
	for _, spenderItem := range spender {
		spenderRule = append(spenderRule, spenderItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Approval", ownerRule, spenderRule)
	if err != nil {
		return nil, err
	}
	return &ContractApprovalIterator{contract: _Contract.contract, event: "Approval", logs: logs, sub: sub}, nil
}

// WatchApproval is a free log subscription operation binding the contract event 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925.
//
// Solidity: event Approval(address indexed owner, address indexed spender, uint256 value)
func (_Contract *ContractFilterer) WatchApproval(opts *bind.WatchOpts, sink chan<- *ContractApproval, owner []common.Address, spender []common.Address) (event.Subscription, error) {

	var ownerRule []interface{}
	for _, ownerItem := range owner {
		ownerRule = append(ownerRule, ownerItem)
	}
	var spenderRule []interface{}
	for _, spenderItem := range spender {
		spenderRule = append(spenderRule, spenderItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Approval", ownerRule, spenderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractApproval)
				if err := _Contract.contract.UnpackLog(event, "Approval", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseApproval is a log parse operation binding the contract event 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925.
//
// Solidity: event Approval(address indexed owner, address indexed spender, uint256 value)
func (_Contract *ContractFilterer) ParseApproval(log types.Log) (*ContractApproval, error) {
	event := new(ContractApproval)
	if err := _Contract.contract.UnpackLog(event, "Approval", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractAuthorityUpdatedIterator is returned from FilterAuthorityUpdated and is used to iterate over the raw logs and unpacked data for AuthorityUpdated events raised by the Contract contract.
type ContractAuthorityUpdatedIterator struct {
	Event *ContractAuthorityUpdated // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractAuthorityUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractAuthorityUpdated)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractAuthorityUpdated)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractAuthorityUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractAuthorityUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractAuthorityUpdated represents a AuthorityUpdated event raised by the Contract contract.
type ContractAuthorityUpdated struct {
	Authority common.Address
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterAuthorityUpdated is a free log retrieval operation binding the contract event 0x2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad.
//
// Solidity: event AuthorityUpdated(address authority)
func (_Contract *ContractFilterer) FilterAuthorityUpdated(opts *bind.FilterOpts) (*ContractAuthorityUpdatedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "AuthorityUpdated")
	if err != nil {
		return nil, err
	}
	return &ContractAuthorityUpdatedIterator{contract: _Contract.contract, event: "AuthorityUpdated", logs: logs, sub: sub}, nil
}

// WatchAuthorityUpdated is a free log subscription operation binding the contract event 0x2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad.
//
// Solidity: event AuthorityUpdated(address authority)
func (_Contract *ContractFilterer) WatchAuthorityUpdated(opts *bind.WatchOpts, sink chan<- *ContractAuthorityUpdated) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "AuthorityUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractAuthorityUpdated)
				if err := _Contract.contract.UnpackLog(event, "AuthorityUpdated", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseAuthorityUpdated is a log parse operation binding the contract event 0x2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad.
//
// Solidity: event AuthorityUpdated(address authority)
func (_Contract *ContractFilterer) ParseAuthorityUpdated(log types.Log) (*ContractAuthorityUpdated, error) {
	event := new(ContractAuthorityUpdated)
	if err := _Contract.contract.UnpackLog(event, "AuthorityUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTBridgeRegisteredIterator is returned from FilterIFTBridgeRegistered and is used to iterate over the raw logs and unpacked data for IFTBridgeRegistered events raised by the Contract contract.
type ContractIFTBridgeRegisteredIterator struct {
	Event *ContractIFTBridgeRegistered // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTBridgeRegisteredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTBridgeRegistered)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTBridgeRegistered)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTBridgeRegisteredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTBridgeRegisteredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTBridgeRegistered represents a IFTBridgeRegistered event raised by the Contract contract.
type ContractIFTBridgeRegistered struct {
	ClientId               string
	CounterpartyIFTAddress string
	IftSendCallConstructor common.Address
	Raw                    types.Log // Blockchain specific contextual infos
}

// FilterIFTBridgeRegistered is a free log retrieval operation binding the contract event 0xa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb8.
//
// Solidity: event IFTBridgeRegistered(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor)
func (_Contract *ContractFilterer) FilterIFTBridgeRegistered(opts *bind.FilterOpts) (*ContractIFTBridgeRegisteredIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTBridgeRegistered")
	if err != nil {
		return nil, err
	}
	return &ContractIFTBridgeRegisteredIterator{contract: _Contract.contract, event: "IFTBridgeRegistered", logs: logs, sub: sub}, nil
}

// WatchIFTBridgeRegistered is a free log subscription operation binding the contract event 0xa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb8.
//
// Solidity: event IFTBridgeRegistered(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor)
func (_Contract *ContractFilterer) WatchIFTBridgeRegistered(opts *bind.WatchOpts, sink chan<- *ContractIFTBridgeRegistered) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTBridgeRegistered")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTBridgeRegistered)
				if err := _Contract.contract.UnpackLog(event, "IFTBridgeRegistered", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTBridgeRegistered is a log parse operation binding the contract event 0xa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb8.
//
// Solidity: event IFTBridgeRegistered(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor)
func (_Contract *ContractFilterer) ParseIFTBridgeRegistered(log types.Log) (*ContractIFTBridgeRegistered, error) {
	event := new(ContractIFTBridgeRegistered)
	if err := _Contract.contract.UnpackLog(event, "IFTBridgeRegistered", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTBridgeRemovedIterator is returned from FilterIFTBridgeRemoved and is used to iterate over the raw logs and unpacked data for IFTBridgeRemoved events raised by the Contract contract.
type ContractIFTBridgeRemovedIterator struct {
	Event *ContractIFTBridgeRemoved // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTBridgeRemovedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTBridgeRemoved)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTBridgeRemoved)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTBridgeRemovedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTBridgeRemovedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTBridgeRemoved represents a IFTBridgeRemoved event raised by the Contract contract.
type ContractIFTBridgeRemoved struct {
	ClientId string
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterIFTBridgeRemoved is a free log retrieval operation binding the contract event 0x142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a846557751.
//
// Solidity: event IFTBridgeRemoved(string clientId)
func (_Contract *ContractFilterer) FilterIFTBridgeRemoved(opts *bind.FilterOpts) (*ContractIFTBridgeRemovedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTBridgeRemoved")
	if err != nil {
		return nil, err
	}
	return &ContractIFTBridgeRemovedIterator{contract: _Contract.contract, event: "IFTBridgeRemoved", logs: logs, sub: sub}, nil
}

// WatchIFTBridgeRemoved is a free log subscription operation binding the contract event 0x142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a846557751.
//
// Solidity: event IFTBridgeRemoved(string clientId)
func (_Contract *ContractFilterer) WatchIFTBridgeRemoved(opts *bind.WatchOpts, sink chan<- *ContractIFTBridgeRemoved) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTBridgeRemoved")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTBridgeRemoved)
				if err := _Contract.contract.UnpackLog(event, "IFTBridgeRemoved", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTBridgeRemoved is a log parse operation binding the contract event 0x142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a846557751.
//
// Solidity: event IFTBridgeRemoved(string clientId)
func (_Contract *ContractFilterer) ParseIFTBridgeRemoved(log types.Log) (*ContractIFTBridgeRemoved, error) {
	event := new(ContractIFTBridgeRemoved)
	if err := _Contract.contract.UnpackLog(event, "IFTBridgeRemoved", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTMintReceivedIterator is returned from FilterIFTMintReceived and is used to iterate over the raw logs and unpacked data for IFTMintReceived events raised by the Contract contract.
type ContractIFTMintReceivedIterator struct {
	Event *ContractIFTMintReceived // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTMintReceivedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTMintReceived)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTMintReceived)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTMintReceivedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTMintReceivedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTMintReceived represents a IFTMintReceived event raised by the Contract contract.
type ContractIFTMintReceived struct {
	ClientId string
	Receiver common.Address
	Amount   *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterIFTMintReceived is a free log retrieval operation binding the contract event 0x3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae.
//
// Solidity: event IFTMintReceived(string clientId, address indexed receiver, uint256 amount)
func (_Contract *ContractFilterer) FilterIFTMintReceived(opts *bind.FilterOpts, receiver []common.Address) (*ContractIFTMintReceivedIterator, error) {

	var receiverRule []interface{}
	for _, receiverItem := range receiver {
		receiverRule = append(receiverRule, receiverItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTMintReceived", receiverRule)
	if err != nil {
		return nil, err
	}
	return &ContractIFTMintReceivedIterator{contract: _Contract.contract, event: "IFTMintReceived", logs: logs, sub: sub}, nil
}

// WatchIFTMintReceived is a free log subscription operation binding the contract event 0x3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae.
//
// Solidity: event IFTMintReceived(string clientId, address indexed receiver, uint256 amount)
func (_Contract *ContractFilterer) WatchIFTMintReceived(opts *bind.WatchOpts, sink chan<- *ContractIFTMintReceived, receiver []common.Address) (event.Subscription, error) {

	var receiverRule []interface{}
	for _, receiverItem := range receiver {
		receiverRule = append(receiverRule, receiverItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTMintReceived", receiverRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTMintReceived)
				if err := _Contract.contract.UnpackLog(event, "IFTMintReceived", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTMintReceived is a log parse operation binding the contract event 0x3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae.
//
// Solidity: event IFTMintReceived(string clientId, address indexed receiver, uint256 amount)
func (_Contract *ContractFilterer) ParseIFTMintReceived(log types.Log) (*ContractIFTMintReceived, error) {
	event := new(ContractIFTMintReceived)
	if err := _Contract.contract.UnpackLog(event, "IFTMintReceived", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTRateLimitSetIterator is returned from FilterIFTRateLimitSet and is used to iterate over the raw logs and unpacked data for IFTRateLimitSet events raised by the Contract contract.
type ContractIFTRateLimitSetIterator struct {
	Event *ContractIFTRateLimitSet // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTRateLimitSetIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTRateLimitSet)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTRateLimitSet)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTRateLimitSetIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTRateLimitSetIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTRateLimitSet represents a IFTRateLimitSet event raised by the Contract contract.
type ContractIFTRateLimitSet struct {
	Capacity *big.Int
	Window   *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterIFTRateLimitSet is a free log retrieval operation binding the contract event 0xab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a91.
//
// Solidity: event IFTRateLimitSet(uint208 capacity, uint48 window)
func (_Contract *ContractFilterer) FilterIFTRateLimitSet(opts *bind.FilterOpts) (*ContractIFTRateLimitSetIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTRateLimitSet")
	if err != nil {
		return nil, err
	}
	return &ContractIFTRateLimitSetIterator{contract: _Contract.contract, event: "IFTRateLimitSet", logs: logs, sub: sub}, nil
}

// WatchIFTRateLimitSet is a free log subscription operation binding the contract event 0xab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a91.
//
// Solidity: event IFTRateLimitSet(uint208 capacity, uint48 window)
func (_Contract *ContractFilterer) WatchIFTRateLimitSet(opts *bind.WatchOpts, sink chan<- *ContractIFTRateLimitSet) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTRateLimitSet")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTRateLimitSet)
				if err := _Contract.contract.UnpackLog(event, "IFTRateLimitSet", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTRateLimitSet is a log parse operation binding the contract event 0xab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a91.
//
// Solidity: event IFTRateLimitSet(uint208 capacity, uint48 window)
func (_Contract *ContractFilterer) ParseIFTRateLimitSet(log types.Log) (*ContractIFTRateLimitSet, error) {
	event := new(ContractIFTRateLimitSet)
	if err := _Contract.contract.UnpackLog(event, "IFTRateLimitSet", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTTransferCompletedIterator is returned from FilterIFTTransferCompleted and is used to iterate over the raw logs and unpacked data for IFTTransferCompleted events raised by the Contract contract.
type ContractIFTTransferCompletedIterator struct {
	Event *ContractIFTTransferCompleted // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTTransferCompletedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTTransferCompleted)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTTransferCompleted)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTTransferCompletedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTTransferCompletedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTTransferCompleted represents a IFTTransferCompleted event raised by the Contract contract.
type ContractIFTTransferCompleted struct {
	ClientId string
	Sequence uint64
	Sender   common.Address
	Amount   *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterIFTTransferCompleted is a free log retrieval operation binding the contract event 0x5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc928.
//
// Solidity: event IFTTransferCompleted(string clientId, uint64 sequence, address indexed sender, uint256 amount)
func (_Contract *ContractFilterer) FilterIFTTransferCompleted(opts *bind.FilterOpts, sender []common.Address) (*ContractIFTTransferCompletedIterator, error) {

	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTTransferCompleted", senderRule)
	if err != nil {
		return nil, err
	}
	return &ContractIFTTransferCompletedIterator{contract: _Contract.contract, event: "IFTTransferCompleted", logs: logs, sub: sub}, nil
}

// WatchIFTTransferCompleted is a free log subscription operation binding the contract event 0x5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc928.
//
// Solidity: event IFTTransferCompleted(string clientId, uint64 sequence, address indexed sender, uint256 amount)
func (_Contract *ContractFilterer) WatchIFTTransferCompleted(opts *bind.WatchOpts, sink chan<- *ContractIFTTransferCompleted, sender []common.Address) (event.Subscription, error) {

	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTTransferCompleted", senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTTransferCompleted)
				if err := _Contract.contract.UnpackLog(event, "IFTTransferCompleted", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTTransferCompleted is a log parse operation binding the contract event 0x5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc928.
//
// Solidity: event IFTTransferCompleted(string clientId, uint64 sequence, address indexed sender, uint256 amount)
func (_Contract *ContractFilterer) ParseIFTTransferCompleted(log types.Log) (*ContractIFTTransferCompleted, error) {
	event := new(ContractIFTTransferCompleted)
	if err := _Contract.contract.UnpackLog(event, "IFTTransferCompleted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTTransferInitiatedIterator is returned from FilterIFTTransferInitiated and is used to iterate over the raw logs and unpacked data for IFTTransferInitiated events raised by the Contract contract.
type ContractIFTTransferInitiatedIterator struct {
	Event *ContractIFTTransferInitiated // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTTransferInitiatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTTransferInitiated)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTTransferInitiated)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTTransferInitiatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTTransferInitiatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTTransferInitiated represents a IFTTransferInitiated event raised by the Contract contract.
type ContractIFTTransferInitiated struct {
	ClientId string
	Sequence uint64
	Sender   common.Address
	Receiver string
	Amount   *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterIFTTransferInitiated is a free log retrieval operation binding the contract event 0xd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a.
//
// Solidity: event IFTTransferInitiated(string clientId, uint64 sequence, address indexed sender, string receiver, uint256 amount)
func (_Contract *ContractFilterer) FilterIFTTransferInitiated(opts *bind.FilterOpts, sender []common.Address) (*ContractIFTTransferInitiatedIterator, error) {

	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTTransferInitiated", senderRule)
	if err != nil {
		return nil, err
	}
	return &ContractIFTTransferInitiatedIterator{contract: _Contract.contract, event: "IFTTransferInitiated", logs: logs, sub: sub}, nil
}

// WatchIFTTransferInitiated is a free log subscription operation binding the contract event 0xd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a.
//
// Solidity: event IFTTransferInitiated(string clientId, uint64 sequence, address indexed sender, string receiver, uint256 amount)
func (_Contract *ContractFilterer) WatchIFTTransferInitiated(opts *bind.WatchOpts, sink chan<- *ContractIFTTransferInitiated, sender []common.Address) (event.Subscription, error) {

	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTTransferInitiated", senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTTransferInitiated)
				if err := _Contract.contract.UnpackLog(event, "IFTTransferInitiated", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTTransferInitiated is a log parse operation binding the contract event 0xd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a.
//
// Solidity: event IFTTransferInitiated(string clientId, uint64 sequence, address indexed sender, string receiver, uint256 amount)
func (_Contract *ContractFilterer) ParseIFTTransferInitiated(log types.Log) (*ContractIFTTransferInitiated, error) {
	event := new(ContractIFTTransferInitiated)
	if err := _Contract.contract.UnpackLog(event, "IFTTransferInitiated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractIFTTransferRefundedIterator is returned from FilterIFTTransferRefunded and is used to iterate over the raw logs and unpacked data for IFTTransferRefunded events raised by the Contract contract.
type ContractIFTTransferRefundedIterator struct {
	Event *ContractIFTTransferRefunded // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractIFTTransferRefundedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIFTTransferRefunded)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractIFTTransferRefunded)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractIFTTransferRefundedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIFTTransferRefundedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIFTTransferRefunded represents a IFTTransferRefunded event raised by the Contract contract.
type ContractIFTTransferRefunded struct {
	ClientId string
	Sequence uint64
	Sender   common.Address
	Amount   *big.Int
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterIFTTransferRefunded is a free log retrieval operation binding the contract event 0xef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332.
//
// Solidity: event IFTTransferRefunded(string clientId, uint64 sequence, address indexed sender, uint256 amount)
func (_Contract *ContractFilterer) FilterIFTTransferRefunded(opts *bind.FilterOpts, sender []common.Address) (*ContractIFTTransferRefundedIterator, error) {

	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTTransferRefunded", senderRule)
	if err != nil {
		return nil, err
	}
	return &ContractIFTTransferRefundedIterator{contract: _Contract.contract, event: "IFTTransferRefunded", logs: logs, sub: sub}, nil
}

// WatchIFTTransferRefunded is a free log subscription operation binding the contract event 0xef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332.
//
// Solidity: event IFTTransferRefunded(string clientId, uint64 sequence, address indexed sender, uint256 amount)
func (_Contract *ContractFilterer) WatchIFTTransferRefunded(opts *bind.WatchOpts, sink chan<- *ContractIFTTransferRefunded, sender []common.Address) (event.Subscription, error) {

	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IFTTransferRefunded", senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIFTTransferRefunded)
				if err := _Contract.contract.UnpackLog(event, "IFTTransferRefunded", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseIFTTransferRefunded is a log parse operation binding the contract event 0xef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332.
//
// Solidity: event IFTTransferRefunded(string clientId, uint64 sequence, address indexed sender, uint256 amount)
func (_Contract *ContractFilterer) ParseIFTTransferRefunded(log types.Log) (*ContractIFTTransferRefunded, error) {
	event := new(ContractIFTTransferRefunded)
	if err := _Contract.contract.UnpackLog(event, "IFTTransferRefunded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractInitializedIterator is returned from FilterInitialized and is used to iterate over the raw logs and unpacked data for Initialized events raised by the Contract contract.
type ContractInitializedIterator struct {
	Event *ContractInitialized // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractInitializedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractInitialized)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractInitialized)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractInitializedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractInitializedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractInitialized represents a Initialized event raised by the Contract contract.
type ContractInitialized struct {
	Version uint64
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterInitialized is a free log retrieval operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_Contract *ContractFilterer) FilterInitialized(opts *bind.FilterOpts) (*ContractInitializedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Initialized")
	if err != nil {
		return nil, err
	}
	return &ContractInitializedIterator{contract: _Contract.contract, event: "Initialized", logs: logs, sub: sub}, nil
}

// WatchInitialized is a free log subscription operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_Contract *ContractFilterer) WatchInitialized(opts *bind.WatchOpts, sink chan<- *ContractInitialized) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Initialized")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractInitialized)
				if err := _Contract.contract.UnpackLog(event, "Initialized", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseInitialized is a log parse operation binding the contract event 0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2.
//
// Solidity: event Initialized(uint64 version)
func (_Contract *ContractFilterer) ParseInitialized(log types.Log) (*ContractInitialized, error) {
	event := new(ContractInitialized)
	if err := _Contract.contract.UnpackLog(event, "Initialized", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractPausedIterator is returned from FilterPaused and is used to iterate over the raw logs and unpacked data for Paused events raised by the Contract contract.
type ContractPausedIterator struct {
	Event *ContractPaused // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractPausedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractPaused)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractPaused)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractPausedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractPausedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractPaused represents a Paused event raised by the Contract contract.
type ContractPaused struct {
	Account common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterPaused is a free log retrieval operation binding the contract event 0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258.
//
// Solidity: event Paused(address account)
func (_Contract *ContractFilterer) FilterPaused(opts *bind.FilterOpts) (*ContractPausedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Paused")
	if err != nil {
		return nil, err
	}
	return &ContractPausedIterator{contract: _Contract.contract, event: "Paused", logs: logs, sub: sub}, nil
}

// WatchPaused is a free log subscription operation binding the contract event 0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258.
//
// Solidity: event Paused(address account)
func (_Contract *ContractFilterer) WatchPaused(opts *bind.WatchOpts, sink chan<- *ContractPaused) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Paused")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractPaused)
				if err := _Contract.contract.UnpackLog(event, "Paused", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParsePaused is a log parse operation binding the contract event 0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258.
//
// Solidity: event Paused(address account)
func (_Contract *ContractFilterer) ParsePaused(log types.Log) (*ContractPaused, error) {
	event := new(ContractPaused)
	if err := _Contract.contract.UnpackLog(event, "Paused", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractTransferIterator is returned from FilterTransfer and is used to iterate over the raw logs and unpacked data for Transfer events raised by the Contract contract.
type ContractTransferIterator struct {
	Event *ContractTransfer // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractTransferIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractTransfer)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractTransfer)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractTransferIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractTransferIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractTransfer represents a Transfer event raised by the Contract contract.
type ContractTransfer struct {
	From  common.Address
	To    common.Address
	Value *big.Int
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterTransfer is a free log retrieval operation binding the contract event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef.
//
// Solidity: event Transfer(address indexed from, address indexed to, uint256 value)
func (_Contract *ContractFilterer) FilterTransfer(opts *bind.FilterOpts, from []common.Address, to []common.Address) (*ContractTransferIterator, error) {

	var fromRule []interface{}
	for _, fromItem := range from {
		fromRule = append(fromRule, fromItem)
	}
	var toRule []interface{}
	for _, toItem := range to {
		toRule = append(toRule, toItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Transfer", fromRule, toRule)
	if err != nil {
		return nil, err
	}
	return &ContractTransferIterator{contract: _Contract.contract, event: "Transfer", logs: logs, sub: sub}, nil
}

// WatchTransfer is a free log subscription operation binding the contract event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef.
//
// Solidity: event Transfer(address indexed from, address indexed to, uint256 value)
func (_Contract *ContractFilterer) WatchTransfer(opts *bind.WatchOpts, sink chan<- *ContractTransfer, from []common.Address, to []common.Address) (event.Subscription, error) {

	var fromRule []interface{}
	for _, fromItem := range from {
		fromRule = append(fromRule, fromItem)
	}
	var toRule []interface{}
	for _, toItem := range to {
		toRule = append(toRule, toItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Transfer", fromRule, toRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractTransfer)
				if err := _Contract.contract.UnpackLog(event, "Transfer", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseTransfer is a log parse operation binding the contract event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef.
//
// Solidity: event Transfer(address indexed from, address indexed to, uint256 value)
func (_Contract *ContractFilterer) ParseTransfer(log types.Log) (*ContractTransfer, error) {
	event := new(ContractTransfer)
	if err := _Contract.contract.UnpackLog(event, "Transfer", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractUnpausedIterator is returned from FilterUnpaused and is used to iterate over the raw logs and unpacked data for Unpaused events raised by the Contract contract.
type ContractUnpausedIterator struct {
	Event *ContractUnpaused // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractUnpausedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractUnpaused)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractUnpaused)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractUnpausedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractUnpausedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractUnpaused represents a Unpaused event raised by the Contract contract.
type ContractUnpaused struct {
	Account common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterUnpaused is a free log retrieval operation binding the contract event 0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa.
//
// Solidity: event Unpaused(address account)
func (_Contract *ContractFilterer) FilterUnpaused(opts *bind.FilterOpts) (*ContractUnpausedIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Unpaused")
	if err != nil {
		return nil, err
	}
	return &ContractUnpausedIterator{contract: _Contract.contract, event: "Unpaused", logs: logs, sub: sub}, nil
}

// WatchUnpaused is a free log subscription operation binding the contract event 0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa.
//
// Solidity: event Unpaused(address account)
func (_Contract *ContractFilterer) WatchUnpaused(opts *bind.WatchOpts, sink chan<- *ContractUnpaused) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Unpaused")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractUnpaused)
				if err := _Contract.contract.UnpackLog(event, "Unpaused", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseUnpaused is a log parse operation binding the contract event 0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa.
//
// Solidity: event Unpaused(address account)
func (_Contract *ContractFilterer) ParseUnpaused(log types.Log) (*ContractUnpaused, error) {
	event := new(ContractUnpaused)
	if err := _Contract.contract.UnpackLog(event, "Unpaused", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractUpgradedIterator is returned from FilterUpgraded and is used to iterate over the raw logs and unpacked data for Upgraded events raised by the Contract contract.
type ContractUpgradedIterator struct {
	Event *ContractUpgraded // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *ContractUpgradedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractUpgraded)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(ContractUpgraded)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *ContractUpgradedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractUpgradedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractUpgraded represents a Upgraded event raised by the Contract contract.
type ContractUpgraded struct {
	Implementation common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterUpgraded is a free log retrieval operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_Contract *ContractFilterer) FilterUpgraded(opts *bind.FilterOpts, implementation []common.Address) (*ContractUpgradedIterator, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return &ContractUpgradedIterator{contract: _Contract.contract, event: "Upgraded", logs: logs, sub: sub}, nil
}

// WatchUpgraded is a free log subscription operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_Contract *ContractFilterer) WatchUpgraded(opts *bind.WatchOpts, sink chan<- *ContractUpgraded, implementation []common.Address) (event.Subscription, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractUpgraded)
				if err := _Contract.contract.UnpackLog(event, "Upgraded", log); err != nil {
					// If the signature doesn't match, skip this log.
					if errors.Is(err, bind.ErrEventSignatureMismatch) {
						continue
					}
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseUpgraded is a log parse operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_Contract *ContractFilterer) ParseUpgraded(log types.Log) (*ContractUpgraded, error) {
	event := new(ContractUpgraded)
	if err := _Contract.contract.UnpackLog(event, "Upgraded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
