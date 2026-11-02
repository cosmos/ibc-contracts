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
	Bin: "0x60a080604052346100c257306080525f5160206141165f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b60405161404f90816100c7823960805181818161182901526118f10152f35b6001600160401b0319166001600160401b039081175f5160206141165f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c90816301ffc9a7146123895750806306fdde03146122a6578063095ea7b3146121ab5780630a7244e714611e3157806318160ddd14611df557806323b872dd14611dbd578063313ce56714611da25780633f4ba83a14611cc557806340c10f1914611c96578063419e41c614611b7857806342966c6814611b5b5780634f1ef286146118a157806352d1902d1461180f578063599deb48146117ca5780635c975abb146117895780635e32b6b614611717578063613d25bb1461109a57806370a0823114611043578063711708b314610fc557806379cc679014610f955780637a9e5e4b14610ed85780638456cb5914610e215780638dfcd9ad14610dce5780638fb3603714610d3b5780639226e08314610c7a57806395d89b4114610b40578063997d99b814610add578063a50e31f114610aa1578063a9059cbb14610a70578063ad3cb1cc14610a0d578063b8ce24181461091e578063bf7e214f146108d9578063d638f98b146103e8578063d88a36fe1461030c578063dd62ed3e146102935763e529a26d146101a8575f80fd5b3461028f57602060031936011261028f5760043567ffffffffffffffff811161028f576101dc610270913690600401612525565b905f604080516101eb81612478565b606081526060602082015201526001600160a01b0361028361020d84846127c8565b6102546040519561021d87612478565b610226836125a4565b8752846002610237600186016125a4565b9460208a01958652015416956040880196875287515115156129b8565b6040519586956020875251606060208801526080870190612427565b9051601f19868303016040870152612427565b91511660608301520390f35b5f80fd5b3461028f57604060031936011261028f576102ac61244c565b6001600160a01b036102f66102bf612462565b926001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f52602052602060405f2054604051908152f35b3461028f57606060031936011261028f5760043567ffffffffffffffff811161028f5761033d903690600401612525565b9060243567ffffffffffffffff811161028f5761035e903690600401612525565b610369929192612a6b565b61038467ffffffffffffffff4216019267ffffffffffffffff84116103bb57610396946044359333613332565b5f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b3461028f57606060031936011261028f5760043567ffffffffffffffff811161028f57610419903690600401612525565b9060243567ffffffffffffffff811161028f5761043a903690600401612525565b9092906044356001600160a01b038116919082810361028f5761045d3633612e1e565b83156108b15781156108895782156108615761047881613cae565b90816107ef575b50156107c35760405161049181612478565b61049c3685876124ef565b81526104a93683886124ef565b6020820190815260408201908482526104c286886127c8565b925180519067ffffffffffffffff82116106ee576104e08554612553565b601f8111610789575b50602090601f83116001146107265761051892915f918361071b575b50505f198260011b9260031b1c19161790565b83555b518051600184019167ffffffffffffffff82116106ee5761053c8354612553565b601f81116106ab575b50602090601f8311600114610613576105fd96947fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89b96946105a8856001600160a01b039660029688965f926106085750505f198260011b9260031b1c19161790565b90555b5116920191167fffffffffffffffffffffffff00000000000000000000000000000000000000008254161790556105ef604051968796606088526060880191612800565b918583036020870152612800565b9060408301520390a1005b015190505f80610505565b90601f19831691845f52815f20925f5b8181106106935750946001856002957fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89f9a98956105fd9c9a956001600160a01b03998a981061067b575b505050811b0190556105ab565b01515f1960f88460031b161c191690558f808061066e565b92936020600181928786015181550195019301610623565b82811115610545576106e090845f5260205f2090601f850160051c90602086106106e6575b601f82910160051c039101612a01565b8a610545565b5f91506106d0565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b015190508b80610505565b90601f19831691865f52815f20925f5b8181106107715750908460019594939210610759575b505050811b01835561051b565b01515f1960f88460031b161c191690558a808061074c565b92936020600181928786015181550195019301610736565b828111156104e9576107bd90865f5260205f2090601f850160051c90602086106106e657601f82910160051c039101612a01565b8a6104e9565b507f3730169b000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b6020915060245f80927f01ffc9a70000000000000000000000000000000000000000000000000000000082527f56d981a700000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681610859575b508661047f565b905086610852565b7ff3a03283000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f9624aeb8000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fc8ca373e000000000000000000000000000000000000000000000000000000005f5260045ffd5b3461028f575f60031936011261028f5760206001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416604051908152f35b3461028f57602060031936011261028f5760043567ffffffffffffffff811161028f576109707f142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a846557751913690600401612525565b61097a3633612e1e565b6109cd818361098982826127c8565b6001600160a01b036002604051926109a084612478565b6109a9816125a4565b84526109b7600182016125a4565b60208501520154166040820152515115156129b8565b5f60026109da83856127c8565b6109e381612a1c565b6109ef60018201612a1c565b0155610a08604051928392602084526020840191612800565b0390a1005b3461028f575f60031936011261028f57610a6c604051610a2e6040826124b0565b600581527f352e302e300000000000000000000000000000000000000000000000000000006020820152604051918291602083526020830190612427565b0390f35b3461028f57604060031936011261028f57610a96610a8c61244c565b6024359033612ce2565b602060405160018152f35b3461028f575f60031936011261028f576040610ac3610abe6138ea565b613b6b565b9050610ad0610abe613949565b8351928352602083015250f35b3461028f575f60031936011261028f577f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00546040805179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316815260d09290921c602083015290f35b3461028f575f60031936011261028f576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454610b7f81612553565b8084529060018116908115610c385750600114610bbb575b610a6c83610ba7818503826124b0565b604051918291602083526020830190612427565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f9081527f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa939250905b808210610c1e57509091508101602001610ba7610b97565b919260018160209254838588010152019101909291610c06565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208086019190915291151560051b84019091019150610ba79050610b97565b3461028f57604060031936011261028f5760043567ffffffffffffffff811161028f57610cab903690600401612525565b6024359167ffffffffffffffff8316830361028f57610d246040935f60208651610cd481612494565b8281520152610ce38484612790565b67ffffffffffffffff82165f52602052845f20936001865195610d0587612494565b6001600160a01b03815416875201549360208601948086521515612820565b6001600160a01b0383519251168252516020820152f35b3461028f575f60031936011261028f577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005460a01c60ff1615610dc65760207f8fb36037000000000000000000000000000000000000000000000000000000005b7fffffffff0000000000000000000000000000000000000000000000000000000060405191168152f35b60205f610d9c565b3461028f57604060031936011261028f57600435801515810361028f576024359067ffffffffffffffff821161028f5760c0600319833603011261028f5761039691610e18612a6b565b6004019061287e565b3461028f575f60031936011261028f57610e3b3633612e1e565b610e436139c2565b60017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff007fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005416177fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a2586020604051338152a1005b3461028f57602060031936011261028f57610ef161244c565b6001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054163303610f6957803b15610f3457610f329061384c565b005b6001600160a01b03907fc2f31e5e000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b7f068ca9d8000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b3461028f57604060031936011261028f57610f32610fb161244c565b60243590610fc0823383612bc6565b613089565b3461028f57608060031936011261028f5760043567ffffffffffffffff811161028f57610ff6903690600401612525565b9060243567ffffffffffffffff811161028f57611017903690600401612525565b606435929167ffffffffffffffff8416840361028f5761039694611039612a6b565b6044359333613332565b3461028f57602060031936011261028f576001600160a01b0361106461244c565b165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00602052602060405f2054604051908152f35b3461028f57608060031936011261028f576110b361244c565b60243567ffffffffffffffff811161028f576110d3903690600401612525565b60449291923567ffffffffffffffff811161028f576110f6903690600401612525565b9390606435936001600160a01b03851680950361028f577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549260ff8460401c16159467ffffffffffffffff85168015908161170f575b6001149081611705575b1590816116fc575b506116d4576111d36111e293868860017fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000006111ea9a16177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005561167f575b506111c6613b14565b6111ce613b14565b61384c565b6111db613b14565b36916124ef565b9436916124ef565b916111f3613b14565b6111fb613b14565b611203613b14565b835167ffffffffffffffff81116106ee5761123e7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0354612553565b601f8111611608575b50602094601f8211600114611567576112789293949582915f9261155c5750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03555b825167ffffffffffffffff81116106ee576112d67f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454612553565b601f81116114e5575b506020601f821160011461144457819061130e9394955f926114395750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04555b611339613b14565b7fffffffffffffffffffffffff00000000000000000000000000000000000000007f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416177f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a00556113a657005b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a1005b015190508580610505565b601f198216907f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f52805f20915f5b8181106114cd575095836001959697106114b5575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0455611331565b01515f1960f88460031b161c19169055848080611488565b9192602060018192868b015181550194019201611473565b818111156112df577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f52611556907f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa90601f840160051c90602085106106e657601f82910160051c039101612a01565b846112df565b015190508680610505565b601f198216957f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52805f20915f5b8881106115f0575083600195969798106115d8575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035561129b565b01515f1960f88460031b161c191690558580806115ab565b91926020600181928685015181550194019201611596565b81811115611247577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52611679907f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab090601f840160051c90602085106106e657601f82910160051c039101612a01565b85611247565b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00558a6111bd565b7ff92ee8a9000000000000000000000000000000000000000000000000000000005f5260045ffd5b9050158961115f565b303b159150611157565b87915061114d565b3461028f57602060031936011261028f5760043567ffffffffffffffff811161028f57806004019060a0600319823603011261028f5761178460449161177c61177361039695611765612a6b565b61176d6131a4565b8061272a565b9490920161277b565b9236916124ef565b613203565b3461028f575f60031936011261028f57602060ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f0330054166040519015158152f35b3461028f575f60031936011261028f5760206001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051908152f35b3461028f575f60031936011261028f576001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001630036118795760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b7fe07c8dba000000000000000000000000000000000000000000000000000000005f5260045ffd5b604060031936011261028f576118b561244c565b60243567ffffffffffffffff811161028f573660238201121561028f576118e69036906024816004013591016124ef565b906001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016803014908115611b26575b506118795761192b3633612e1e565b6001600160a01b038116916040517f52d1902d000000000000000000000000000000000000000000000000000000008152602081600481875afa5f9181611af2575b5061199e57837f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b807f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc859203611ac75750823b15611a9c57807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a2805115611a6b57610f3291613e74565b505034611a7457005b7fb398979f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7faa1d49a4000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b9091506020813d602011611b1e575b81611b0e602093836124b0565b8101031261028f5751908561196d565b3d9150611b01565b90506001600160a01b037f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc541614158361191c565b3461028f57602060031936011261028f57610f3260043533613089565b3461028f57604060031936011261028f5760043579ffffffffffffffffffffffffffffffffffffffffffffffffffff81169081810361028f576024359065ffffffffffff8216820361028f577fab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a9192611bf03633612e1e565b611c00611bfb6138ea565b613a22565b611c0b611bfb613949565b79ffffffffffffffffffffffffffffffffffffffffffffffffffff90811660d084901b7fffffffffffff000000000000000000000000000000000000000000000000000016177f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00556040805192909116825265ffffffffffff90921660208201529081908101610a08565b3461028f57604060031936011261028f57610f32611cb261244c565b611cbc3633612e1e565b60243590612adf565b3461028f575f60031936011261028f57611cdf3633612e1e565b7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005460ff811615611d7a577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00167fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa6020604051338152a1005b7f8dfc202b000000000000000000000000000000000000000000000000000000005f5260045ffd5b3461028f575f60031936011261028f57602060405160128152f35b3461028f57606060031936011261028f57610a96611dd961244c565b611de1612462565b60443591611df0833383612bc6565b612ce2565b3461028f575f60031936011261028f5760207f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254604051908152f35b3461028f57604060031936011261028f57611e4a61244c565b60243590611e56612a6b565b60245f6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051928380927fe57d7de00000000000000000000000000000000000000000000000000000000082523360048301525afa9081156121a0575f916120eb575b506020808251604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a01815203019020611f69604051611f1081612478565b611f19836125a4565b81526001600160a01b036002611f31600186016125a4565b94602084019586520154166040820152611f5181515115158551906126e4565b516020815191012083519081516020830120146126e4565b51805160208201206020830151908151602083012003612097575050604081015180516120555750611f996139c2565b611faa83611fa56138ea565b613d8c565b1561202d57816001600160a01b03611ffd92611fe7867f3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae96612adf565b5191604051938493604085526040850190612427565b95602084015216930390a25f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7fa74c1c5f000000000000000000000000000000000000000000000000000000005f5260045ffd5b612093906040519182917f3ef1dc02000000000000000000000000000000000000000000000000000000008352602060048401526024830190612427565b0390fd5b906120936120d9926040519384937ff39880b1000000000000000000000000000000000000000000000000000000008552604060048601526044850190612427565b90600319848303016024850152612427565b90503d805f833e6120fc81836124b0565b81019060208183031261028f5780519067ffffffffffffffff821161028f570160608183031261028f576040519161213383612478565b815167ffffffffffffffff811161028f5781612150918401612662565b8352602082015167ffffffffffffffff811161028f5781612172918401612662565b6020840152604082015167ffffffffffffffff811161028f576121959201612662565b604082015283611ec4565b6040513d5f823e3d90fd5b3461028f57604060031936011261028f576121c461244c565b60243590331561227a576001600160a01b031690811561224e57335f9081527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260409020825f526020528060405f20556040519081527f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560203392a3602060405160018152f35b7f94280d62000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7fe602df05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b3461028f575f60031936011261028f576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03546122e581612553565b8084529060018116908115610c38575060011461230c57610a6c83610ba7818503826124b0565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f9081527f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab0939250905b80821061236f57509091508101602001610ba7610b97565b919260018160209254838588010152019101909291612357565b3461028f57602060031936011261028f57600435907fffffffff00000000000000000000000000000000000000000000000000000000821680920361028f57817fd3ce6f1b00000000000000000000000000000000000000000000000000000000602093149081156123fd575b5015158152f35b7f01ffc9a700000000000000000000000000000000000000000000000000000000915014836123f6565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b600435906001600160a01b038216820361028f57565b602435906001600160a01b038216820361028f57565b6060810190811067ffffffffffffffff8211176106ee57604052565b6040810190811067ffffffffffffffff8211176106ee57604052565b90601f601f19910116810190811067ffffffffffffffff8211176106ee57604052565b67ffffffffffffffff81116106ee57601f01601f191660200190565b9291926124fb826124d3565b9161250960405193846124b0565b82948184528183011161028f578281602093845f960137010152565b9181601f8401121561028f5782359167ffffffffffffffff831161028f576020838186019501011161028f57565b90600182811c9216801561259a575b602083101461256d57565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f1691612562565b9060405191825f8254926125b784612553565b808452936001811690811561262257506001146125de575b506125dc925003836124b0565b565b90505f9291925260205f20905f915b8183106126065750509060206125dc928201015f6125cf565b60209193508060019154838589010152019101909184926125ed565b602093506125dc9592507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0091501682840152151560051b8201015f6125cf565b81601f8201121561028f5760208151910161267c826124d3565b9261268a60405194856124b0565b8284528282011161028f57815f926020928386015e8301015290565b60208091604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b156126ec5750565b612093906040519182917f24a61b95000000000000000000000000000000000000000000000000000000008352602060048401526024830190612427565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18136030182121561028f570180359067ffffffffffffffff821161028f5760200191813603831361028f57565b3567ffffffffffffffff8116810361028f5790565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0181520301902090565b601f8260209493601f1993818652868601375f8582860101520116010190565b9290921561282d57505050565b67ffffffffffffffff6128736040519485947fa4e158af000000000000000000000000000000000000000000000000000000008652604060048701526044860191612800565b911660248301520390fd5b6128866131a4565b156129a2577f5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc9286128bf6128b9838061272a565b90612790565b604083019067ffffffffffffffff6128d68361277b565b165f5260205261296c60405f206001600160a01b036129646001604051936128fd85612494565b83815416855201549661292d6020850198808a5261291b838061272a565b906129258a61277b565b921515612820565b61293a6128b9828061272a565b67ffffffffffffffff61294c8861277b565b165f526020525f60016040822082815501558061272a565b93909461277b565b915116945167ffffffffffffffff612991604051958695606087526060870191612800565b9216602084015260408301520390a2565b60406117848261177c611773826125dc9661272a565b919091156129c4575050565b6120936040519283927f24a61b95000000000000000000000000000000000000000000000000000000008452602060048501526024840191612800565b5f5b828110612a0f57505050565b5f82820155600101612a03565b612a268154612553565b9081612a30575050565b81601f5f9311600114612a41575055565b81835260208320612a5e91601f0160051c5f190190600101612a01565b8082528160208120915555565b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c612ab75760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b6001600160a01b0316908115612b9a577fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef602082612b3f5f947f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254613a15565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02558484527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260408420818154019055604051908152a3565b7fec442f05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b9190612c02836001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b6001600160a01b0382165f5260205260405f2054925f198410612c26575b50505050565b828410612ca5576001600160a01b0381161561227a576001600160a01b0382161561224e57612c8d6001600160a01b03916001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f5260205260405f20910390555f808080612c20565b506001600160a01b0383917ffb8f41b2000000000000000000000000000000000000000000000000000000005f521660045260245260445260645ffd5b6001600160a01b0316908115612df2576001600160a01b0316918215612b9a57815f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054818110612dc057817fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92602092855f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0084520360405f2055845f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260405f20818154019055604051908152a3565b827fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b7f96c6fd1e000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054916001600160a01b038316928160041161028f575f5f9060405f8151966001600160a01b0360208901917fb700961300000000000000000000000000000000000000000000000000000000835216978860248201523060448201527fffffffff00000000000000000000000000000000000000000000000000000000833516606482015260648152612ed36084826124b0565b828052826020525190895afa613076575b15612ef1575b5050505050565b63ffffffff161561304a577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055823b1561028f576020925f92836040518096819582947f94c7d7ee000000000000000000000000000000000000000000000000000000008452600484015260406024840152601f19601f6044850192808452808786860137868582860101520116010103925af180156121a05761303a575b507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00555f80808080612eea565b5f613044916124b0565b5f612fc9565b827f068ca9d8000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b50505f516020518060201c150290612ee4565b9091906001600160a01b03168015612df257805f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054838110613171576020845f94957fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef938587527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace008452036040862055807f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254037f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0255604051908152a3565b91507fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a00541633036131d757565b7f9a47cf30000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b9061320d826126a6565b67ffffffffffffffff82165f5260205260405f2060016040519161323083612494565b6001600160a01b0381541683520154926020820193808552156132ea576132556139c2565b6132628451611fa56138ea565b1561202d577fef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332926001600160a01b03836132a482612991965116885190612adf565b6132ad846126a6565b67ffffffffffffffff84165f526020525f60016040822082815501555116945167ffffffffffffffff604051948594606086526060860190612427565b612873908367ffffffffffffffff6040519384937fa4e158af000000000000000000000000000000000000000000000000000000008552604060048601526044850190612427565b9594929091939484156108b15785156138245783156137fc5767ffffffffffffffff1693428511156137c25761336881846127c8565b926133ce6040519261337984612478565b613382866125a4565b84526001600160a01b03600261339a600189016125a4565b976020870198895201541692604085019384528451602081519101206133c13684846124ef565b60208151910120146129b8565b6133d66139c2565b6133e285611fa5613949565b1561202d576001600160a01b03906133fa868a613089565b5116925f60405180957f56d981a700000000000000000000000000000000000000000000000000000000825260406004830152818061343d604482018d8a612800565b8a602483015203915afa9384156121a0575f94613782575b506001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054169582519151966040519260c0840184811067ffffffffffffffff8211176106ee576040528352602083019788526020978893604051936134c186866124b0565b5f85526040820194855260608201988952608082019081526040516134e687826124b0565b5f815260a08301908152604051998a96879586957f7e5a65660000000000000000000000000000000000000000000000000000000087528a6004880152516024870160c0905260e4870161353991612427565b9051908681037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc01604488015261356f91612427565b9051908581037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160648701526135a591612427565b9051908481037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160848601526135db91612427565b915167ffffffffffffffff1660a484015251908281037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160c484015261362191612427565b03915a905f91f19283156121a0575f93613718575b50917fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a9561370d926136fa9695946001600160a01b036040519a6136798c612494565b16998a8152600188820188815261369084516126a6565b67ffffffffffffffff88165f528a526001600160a01b038060405f20945116167fffffffffffffffffffffffff000000000000000000000000000000000000000084541617835551910155519567ffffffffffffffff60405198899860808a5260808a0190612427565b9416908701528583036040870152612800565b9060608301520390a2565b9091948094935081813d831161377b575b61373381836124b0565b8101031261028f57519067ffffffffffffffff8216820361028f57929391929091907fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a613636565b503d613729565b9093503d805f833e61379481836124b0565b810160208282031261028f57815167ffffffffffffffff811161028f576137bb9201612662565b925f613455565b847f0b8a29e4000000000000000000000000000000000000000000000000000000005f5260045267ffffffffffffffff421660245260445ffd5b7fcc2a76e5000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f5de17177000000000000000000000000000000000000000000000000000000005f5260045ffd5b60206001600160a01b037f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad9216807fffffffffffffffffffffffff00000000000000000000000000000000000000007ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055604051908152a1565b60206040516138fa6040826124b0565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e5f603b820152601c8152613943603c826124b0565b51902090565b60206040516139596040826124b0565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e7f0100000000000000000000000000000000000000000000000000000000000000603b820152601c8152613943603c826124b0565b60ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f0330054166139ed57565b7fd93c0665000000000000000000000000000000000000000000000000000000005f5260045ffd5b919082018092116103bb57565b6125dc9065ffffffffffff613a3642613ffa565b613a60837f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00613c4e565b509279ffffffffffffffffffffffffffffffffffffffffffffffffffff60405194613a8a86612494565b16845290821660208085019182525f9283527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb01905260409091209251905190911660d01b7fffffffffffff00000000000000000000000000000000000000000000000000001679ffffffffffffffffffffffffffffffffffffffffffffffffffff91909116179055565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c1615613b4357565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b907f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb005479ffffffffffffffffffffffffffffffffffffffffffffffffffff8116925f527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb0160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff613c0642613ffa565b9160d01c91160365ffffffffffff81116103bb578465ffffffffffff613c3b9360d01c60018082119118026001189216613f2e565b8103908111150291828103908111150290565b919091600181549179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316945f520160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff613c0642613ffa565b7f01ffc9a7000000000000000000000000000000000000000000000000000000005f527f01ffc9a70000000000000000000000000000000000000000000000000000000060045260205f60248184617530fa5f511515601f3d111681613d84575b5015613d7f575f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fffffffff00000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681613d79575090565b90501590565b505f90565b90505f613d0f565b908015613e6d57613dbd827f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00613c4e565b9091908111613e6657613dd990613dd342613ffa565b92613a15565b9179ffffffffffffffffffffffffffffffffffffffffffffffffffff8311613e3557613e30929165ffffffffffff9179ffffffffffffffffffffffffffffffffffffffffffffffffffff60405194613a8a86612494565b600190565b827f6dfcc650000000000000000000000000000000000000000000000000000000005f5260d060045260245260445ffd5b5050505f90565b5050600190565b905f8091602081519101845af48080613f1b575b15613ea85750506040513d81523d5f602083013e60203d82010160405290565b15613ee2576001600160a01b03907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d15613ef3576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d151580613e885750813b1515613e88565b90915f198383099280830292838086109503948086039514613fbf5784831115613fa75790829109815f0382168092046002816003021880820260020302808202600203028082026002030280820260020302808202600203028091026002030293600183805f03040190848311900302920304170290565b82634e487b715f52156003026011186020526024601cfd5b505080925015613fcd570490565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b65ffffffffffff81116140125765ffffffffffff1690565b7f6dfcc650000000000000000000000000000000000000000000000000000000005f52603060045260245260445ffdfea164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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
