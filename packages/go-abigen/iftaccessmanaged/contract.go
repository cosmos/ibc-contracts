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
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"allowance\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"approve\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"authority\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"balanceOf\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"burn\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"burnFrom\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"decimals\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.IFTBridge\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"contractIIFTSendCallConstructor\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimit\",\"inputs\":[],\"outputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimitAvailable\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPendingTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.PendingTransfer\",\"components\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ics27\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIICS27GMP\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"iftMint\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"authority_\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"erc20Name\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"erc20Symbol\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"ics27Gmp\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"isConsumingScheduledOp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"mint\",\"inputs\":[{\"name\":\"mintAddress\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"name\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"onAckPacket\",\"inputs\":[{\"name\":\"success\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnAcknowledgementPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onTimeoutPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnTimeoutPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"registerIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"removeIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setAuthority\",\"inputs\":[{\"name\":\"newAuthority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setIFTRateLimit\",\"inputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"supportsInterface\",\"inputs\":[{\"name\":\"interfaceId\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"symbol\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalSupply\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferFrom\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"event\",\"name\":\"Approval\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"AuthorityUpdated\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRegistered\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRemoved\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTMintReceived\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTRateLimitSet\",\"inputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"indexed\":false,\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"indexed\":false,\"internalType\":\"uint48\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferCompleted\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferInitiated\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"receiver\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferRefunded\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Transfer\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AccessManagedInvalidAuthority\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AccessManagedRequiredDelay\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"delay\",\"type\":\"uint32\",\"internalType\":\"uint32\"}]},{\"type\":\"error\",\"name\":\"AccessManagedUnauthorized\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ERC20InsufficientAllowance\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"allowance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InsufficientBalance\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidApprover\",\"inputs\":[{\"name\":\"approver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSpender\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTBridgeNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTEmptyClientId\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyCounterpartyAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyReceiver\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTInvalidConstructorInterface\",\"inputs\":[{\"name\":\"callConstructor\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTInvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTOnlyICS27GMP\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTPendingTransferNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTTimeoutInPast\",\"inputs\":[{\"name\":\"timeout\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"currentTime\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTUnauthorizedMint\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTUnexpectedSalt\",\"inputs\":[{\"name\":\"salt\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"IFTZeroAddressConstructor\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTZeroAmount\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"RateLimitExceeded\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeCastOverflowedUintDowncast\",\"inputs\":[{\"name\":\"bits\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}]",
	Bin: "0x60a080604052346100c257306080525f516020613ea55f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b604051613dde90816100c7823960805181818161170801526117d00152f35b6001600160401b0319166001600160401b039081175f516020613ea55f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c90816301ffc9a7146121835750806306fdde03146120a0578063095ea7b314611fa55780630a7244e714611c3357806318160ddd14611bf757806323b872dd14611bbf578063313ce56714611ba457806340c10f1914611b75578063419e41c614611a5757806342966c6814611a3a5780634f1ef2861461178057806352d1902d146116ee578063599deb48146116a95780635e32b6b614611637578063613d25bb14610fc257806370a0823114610f6b578063711708b314610eed57806379cc679014610ebd5780637a9e5e4b14610e005780638dfcd9ad14610dad5780638fb3603714610d1a5780639226e08314610c5957806395d89b4114610b1f578063997d99b814610abc578063a50e31f114610a80578063a9059cbb14610a4f578063ad3cb1cc146109ec578063b8ce2418146108fd578063bf7e214f146108b8578063d638f98b146103c7578063d88a36fe146102eb578063dd62ed3e146102725763e529a26d14610187575f80fd5b3461026e57602060031936011261026e5760043567ffffffffffffffff811161026e576101bb61024f91369060040161231f565b905f604080516101ca81612272565b606081526060602082015201526001600160a01b036102626101ec84846125c2565b610233604051956101fc87612272565b6102058361239e565b87528460026102166001860161239e565b9460208a01958652015416956040880196875287515115156127b2565b6040519586956020875251606060208801526080870190612221565b9051601f19868303016040870152612221565b91511660608301520390f35b5f80fd5b3461026e57604060031936011261026e5761028b612246565b6001600160a01b036102d561029e61225c565b926001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f52602052602060405f2054604051908152f35b3461026e57606060031936011261026e5760043567ffffffffffffffff811161026e5761031c90369060040161231f565b9060243567ffffffffffffffff811161026e5761033d90369060040161231f565b610348929192612865565b61038467ffffffffffffffff4216019267ffffffffffffffff841161039a57610375946044359333613124565b5f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b3461026e57606060031936011261026e5760043567ffffffffffffffff811161026e576103f890369060040161231f565b9060243567ffffffffffffffff811161026e5761041990369060040161231f565b9092906044356001600160a01b038116919082810361026e5761043c3633612c18565b83156108905781156108685782156108405761045781613a3d565b90816107ce575b50156107a25760405161047081612272565b61047b3685876122e9565b81526104883683886122e9565b6020820190815260408201908482526104a186886125c2565b925180519067ffffffffffffffff82116106cd576104bf855461234d565b601f8111610768575b50602090601f8311600114610705576104f792915f91836106fa575b50505f198260011b9260031b1c19161790565b83555b518051600184019167ffffffffffffffff82116106cd5761051b835461234d565b601f811161068a575b50602090601f83116001146105f2576105dc96947fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89b9694610587856001600160a01b039660029688965f926105e75750505f198260011b9260031b1c19161790565b90555b5116920191167fffffffffffffffffffffffff00000000000000000000000000000000000000008254161790556105ce6040519687966060885260608801916125fa565b9185830360208701526125fa565b9060408301520390a1005b015190505f806104e4565b90601f19831691845f52815f20925f5b8181106106725750946001856002957fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89f9a98956105dc9c9a956001600160a01b03998a981061065a575b505050811b01905561058a565b01515f1960f88460031b161c191690558f808061064d565b92936020600181928786015181550195019301610602565b82811115610524576106bf90845f5260205f2090601f850160051c90602086106106c5575b601f82910160051c0391016127fb565b8a610524565b5f91506106af565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b015190508b806104e4565b90601f19831691865f52815f20925f5b8181106107505750908460019594939210610738575b505050811b0183556104fa565b01515f1960f88460031b161c191690558a808061072b565b92936020600181928786015181550195019301610715565b828111156104c85761079c90865f5260205f2090601f850160051c90602086106106c557601f82910160051c0391016127fb565b8a6104c8565b507f3730169b000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b6020915060245f80927f01ffc9a70000000000000000000000000000000000000000000000000000000082527f56d981a700000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681610838575b508661045e565b905086610831565b7ff3a03283000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f9624aeb8000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fc8ca373e000000000000000000000000000000000000000000000000000000005f5260045ffd5b3461026e575f60031936011261026e5760206001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416604051908152f35b3461026e57602060031936011261026e5760043567ffffffffffffffff811161026e5761094f7f142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a84655775191369060040161231f565b6109593633612c18565b6109ac818361096882826125c2565b6001600160a01b0360026040519261097f84612272565b6109888161239e565b84526109966001820161239e565b60208501520154166040820152515115156127b2565b5f60026109b983856125c2565b6109c281612816565b6109ce60018201612816565b01556109e76040519283926020845260208401916125fa565b0390a1005b3461026e575f60031936011261026e57610a4b604051610a0d6040826122aa565b600581527f352e302e300000000000000000000000000000000000000000000000000000006020820152604051918291602083526020830190612221565b0390f35b3461026e57604060031936011261026e57610a75610a6b612246565b6024359033612adc565b602060405160018152f35b3461026e575f60031936011261026e576040610aa2610a9d6136cc565b6138fa565b9050610aaf610a9d61372b565b8351928352602083015250f35b3461026e575f60031936011261026e577f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00546040805179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316815260d09290921c602083015290f35b3461026e575f60031936011261026e576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454610b5e8161234d565b8084529060018116908115610c175750600114610b9a575b610a4b83610b86818503826122aa565b604051918291602083526020830190612221565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f9081527f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa939250905b808210610bfd57509091508101602001610b86610b76565b919260018160209254838588010152019101909291610be5565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208086019190915291151560051b84019091019150610b869050610b76565b3461026e57604060031936011261026e5760043567ffffffffffffffff811161026e57610c8a90369060040161231f565b6024359167ffffffffffffffff8316830361026e57610d036040935f60208651610cb38161228e565b8281520152610cc2848461258a565b67ffffffffffffffff82165f52602052845f20936001865195610ce48761228e565b6001600160a01b0381541687520154936020860194808652151561261a565b6001600160a01b0383519251168252516020820152f35b3461026e575f60031936011261026e577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005460a01c60ff1615610da55760207f8fb36037000000000000000000000000000000000000000000000000000000005b7fffffffff0000000000000000000000000000000000000000000000000000000060405191168152f35b60205f610d7b565b3461026e57604060031936011261026e57600435801515810361026e576024359067ffffffffffffffff821161026e5760c0600319833603011261026e5761037591610df7612865565b60040190612678565b3461026e57602060031936011261026e57610e19612246565b6001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054163303610e9157803b15610e5c57610e5a9061362e565b005b6001600160a01b03907fc2f31e5e000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b7f068ca9d8000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b3461026e57604060031936011261026e57610e5a610ed9612246565b60243590610ee88233836129c0565b612e83565b3461026e57608060031936011261026e5760043567ffffffffffffffff811161026e57610f1e90369060040161231f565b9060243567ffffffffffffffff811161026e57610f3f90369060040161231f565b606435929167ffffffffffffffff8416840361026e5761037594610f61612865565b6044359333613124565b3461026e57602060031936011261026e576001600160a01b03610f8c612246565b165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00602052602060405f2054604051908152f35b3461026e57608060031936011261026e57610fdb612246565b60243567ffffffffffffffff811161026e57610ffb90369060040161231f565b60449291923567ffffffffffffffff811161026e5761101e90369060040161231f565b9390606435936001600160a01b03851680950361026e577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549260ff8460401c16159467ffffffffffffffff85168015908161162f575b6001149081611625575b15908161161c575b506115f4576110fb61110293868860017fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000061110a9a16177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005561159f575b506110ee6138a3565b6110f66138a3565b61362e565b36916122e9565b9436916122e9565b916111136138a3565b61111b6138a3565b6111236138a3565b835167ffffffffffffffff81116106cd5761115e7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035461234d565b601f8111611528575b50602094601f8211600114611487576111989293949582915f9261147c5750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03555b825167ffffffffffffffff81116106cd576111f67f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045461234d565b601f8111611405575b506020601f821160011461136457819061122e9394955f926113595750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04555b6112596138a3565b7fffffffffffffffffffffffff00000000000000000000000000000000000000007f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416177f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a00556112c657005b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a1005b0151905085806104e4565b601f198216907f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f52805f20915f5b8181106113ed575095836001959697106113d5575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0455611251565b01515f1960f88460031b161c191690558480806113a8565b9192602060018192868b015181550194019201611393565b818111156111ff577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f52611476907f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa90601f840160051c90602085106106c557601f82910160051c0391016127fb565b846111ff565b0151905086806104e4565b601f198216957f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52805f20915f5b888110611510575083600195969798106114f8575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03556111bb565b01515f1960f88460031b161c191690558580806114cb565b919260206001819286850151815501940192016114b6565b81811115611167577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52611599907f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab090601f840160051c90602085106106c557601f82910160051c0391016127fb565b85611167565b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00558a6110e5565b7ff92ee8a9000000000000000000000000000000000000000000000000000000005f5260045ffd5b90501589611087565b303b15915061107f565b879150611075565b3461026e57602060031936011261026e5760043567ffffffffffffffff811161026e57806004019060a0600319823603011261026e576116a460449161169c61169361037595611685612865565b61168d612f9e565b80612524565b94909201612575565b9236916122e9565b612ffd565b3461026e575f60031936011261026e5760206001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051908152f35b3461026e575f60031936011261026e576001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001630036117585760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b7fe07c8dba000000000000000000000000000000000000000000000000000000005f5260045ffd5b604060031936011261026e57611794612246565b60243567ffffffffffffffff811161026e573660238201121561026e576117c59036906024816004013591016122e9565b906001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016803014908115611a05575b506117585761180a3633612c18565b6001600160a01b038116916040517f52d1902d000000000000000000000000000000000000000000000000000000008152602081600481875afa5f91816119d1575b5061187d57837f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b807f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8592036119a65750823b1561197b57807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a280511561194a57610e5a91613c03565b50503461195357005b7fb398979f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7faa1d49a4000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b9091506020813d6020116119fd575b816119ed602093836122aa565b8101031261026e5751908561184c565b3d91506119e0565b90506001600160a01b037f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc54161415836117fb565b3461026e57602060031936011261026e57610e5a60043533612e83565b3461026e57604060031936011261026e5760043579ffffffffffffffffffffffffffffffffffffffffffffffffffff81169081810361026e576024359065ffffffffffff8216820361026e577fab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a9192611acf3633612c18565b611adf611ada6136cc565b6137b1565b611aea611ada61372b565b79ffffffffffffffffffffffffffffffffffffffffffffffffffff90811660d084901b7fffffffffffff000000000000000000000000000000000000000000000000000016177f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00556040805192909116825265ffffffffffff909216602082015290819081016109e7565b3461026e57604060031936011261026e57610e5a611b91612246565b611b9b3633612c18565b602435906128d9565b3461026e575f60031936011261026e57602060405160128152f35b3461026e57606060031936011261026e57610a75611bdb612246565b611be361225c565b60443591611bf28333836129c0565b612adc565b3461026e575f60031936011261026e5760207f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254604051908152f35b3461026e57604060031936011261026e57611c4c612246565b60243590611c58612865565b60245f6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051928380927fe57d7de00000000000000000000000000000000000000000000000000000000082523360048301525afa908115611f9a575f91611ee5575b506020808251604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a01815203019020611d6b604051611d1281612272565b611d1b8361239e565b81526001600160a01b036002611d336001860161239e565b94602084019586520154166040820152611d5381515115158551906124de565b516020815191012083519081516020830120146124de565b51805160208201206020830151908151602083012003611e9157505060408101518051611e4f5750611da483611d9f6136cc565b613b1b565b15611e2757816001600160a01b03611df792611de1867f3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae966128d9565b5191604051938493604085526040850190612221565b95602084015216930390a25f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7fa74c1c5f000000000000000000000000000000000000000000000000000000005f5260045ffd5b611e8d906040519182917f3ef1dc02000000000000000000000000000000000000000000000000000000008352602060048401526024830190612221565b0390fd5b90611e8d611ed3926040519384937ff39880b1000000000000000000000000000000000000000000000000000000008552604060048601526044850190612221565b90600319848303016024850152612221565b90503d805f833e611ef681836122aa565b81019060208183031261026e5780519067ffffffffffffffff821161026e570160608183031261026e5760405191611f2d83612272565b815167ffffffffffffffff811161026e5781611f4a91840161245c565b8352602082015167ffffffffffffffff811161026e5781611f6c91840161245c565b6020840152604082015167ffffffffffffffff811161026e57611f8f920161245c565b604082015283611cc6565b6040513d5f823e3d90fd5b3461026e57604060031936011261026e57611fbe612246565b602435903315612074576001600160a01b031690811561204857335f9081527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260409020825f526020528060405f20556040519081527f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560203392a3602060405160018152f35b7f94280d62000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7fe602df05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b3461026e575f60031936011261026e576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03546120df8161234d565b8084529060018116908115610c17575060011461210657610a4b83610b86818503826122aa565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f9081527f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab0939250905b80821061216957509091508101602001610b86610b76565b919260018160209254838588010152019101909291612151565b3461026e57602060031936011261026e57600435907fffffffff00000000000000000000000000000000000000000000000000000000821680920361026e57817fd3ce6f1b00000000000000000000000000000000000000000000000000000000602093149081156121f7575b5015158152f35b7f01ffc9a700000000000000000000000000000000000000000000000000000000915014836121f0565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b600435906001600160a01b038216820361026e57565b602435906001600160a01b038216820361026e57565b6060810190811067ffffffffffffffff8211176106cd57604052565b6040810190811067ffffffffffffffff8211176106cd57604052565b90601f601f19910116810190811067ffffffffffffffff8211176106cd57604052565b67ffffffffffffffff81116106cd57601f01601f191660200190565b9291926122f5826122cd565b9161230360405193846122aa565b82948184528183011161026e578281602093845f960137010152565b9181601f8401121561026e5782359167ffffffffffffffff831161026e576020838186019501011161026e57565b90600182811c92168015612394575b602083101461236757565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f169161235c565b9060405191825f8254926123b18461234d565b808452936001811690811561241c57506001146123d8575b506123d6925003836122aa565b565b90505f9291925260205f20905f915b8183106124005750509060206123d6928201015f6123c9565b60209193508060019154838589010152019101909184926123e7565b602093506123d69592507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0091501682840152151560051b8201015f6123c9565b81601f8201121561026e57602081519101612476826122cd565b9261248460405194856122aa565b8284528282011161026e57815f926020928386015e8301015290565b60208091604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b156124e65750565b611e8d906040519182917f24a61b95000000000000000000000000000000000000000000000000000000008352602060048401526024830190612221565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18136030182121561026e570180359067ffffffffffffffff821161026e5760200191813603831361026e57565b3567ffffffffffffffff8116810361026e5790565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0181520301902090565b601f8260209493601f1993818652868601375f8582860101520116010190565b9290921561262757505050565b67ffffffffffffffff61266d6040519485947fa4e158af0000000000000000000000000000000000000000000000000000000086526040600487015260448601916125fa565b911660248301520390fd5b612680612f9e565b1561279c577f5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc9286126b96126b38380612524565b9061258a565b604083019067ffffffffffffffff6126d083612575565b165f5260205261276660405f206001600160a01b0361275e6001604051936126f78561228e565b8381541685520154966127276020850198808a526127158380612524565b9061271f8a612575565b92151561261a565b6127346126b38280612524565b67ffffffffffffffff61274688612575565b165f526020525f600160408220828155015580612524565b939094612575565b915116945167ffffffffffffffff61278b6040519586956060875260608701916125fa565b9216602084015260408301520390a2565b60406116a48261169c611693826123d696612524565b919091156127be575050565b611e8d6040519283927f24a61b950000000000000000000000000000000000000000000000000000000084526020600485015260248401916125fa565b5f5b82811061280957505050565b5f828201556001016127fd565b612820815461234d565b908161282a575050565b81601f5f931160011461283b575055565b8183526020832061285891601f0160051c5f1901906001016127fb565b8082528160208120915555565b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c6128b15760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b6001600160a01b0316908115612994577fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef6020826129395f947f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02546137a4565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02558484527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260408420818154019055604051908152a3565b7fec442f05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b91906129fc836001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b6001600160a01b0382165f5260205260405f2054925f198410612a20575b50505050565b828410612a9f576001600160a01b03811615612074576001600160a01b0382161561204857612a876001600160a01b03916001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f5260205260405f20910390555f808080612a1a565b506001600160a01b0383917ffb8f41b2000000000000000000000000000000000000000000000000000000005f521660045260245260445260645ffd5b6001600160a01b0316908115612bec576001600160a01b031691821561299457815f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054818110612bba57817fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92602092855f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0084520360405f2055845f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260405f20818154019055604051908152a3565b827fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b7f96c6fd1e000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054916001600160a01b038316928160041161026e575f5f9060405f8151966001600160a01b0360208901917fb700961300000000000000000000000000000000000000000000000000000000835216978860248201523060448201527fffffffff00000000000000000000000000000000000000000000000000000000833516606482015260648152612ccd6084826122aa565b828052826020525190895afa612e70575b15612ceb575b5050505050565b63ffffffff1615612e44577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055823b1561026e576020925f92836040518096819582947f94c7d7ee000000000000000000000000000000000000000000000000000000008452600484015260406024840152601f19601f6044850192808452808786860137868582860101520116010103925af18015611f9a57612e34575b507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00555f80808080612ce4565b5f612e3e916122aa565b5f612dc3565b827f068ca9d8000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b50505f516020518060201c150290612cde565b9091906001600160a01b03168015612bec57805f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054838110612f6b576020845f94957fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef938587527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace008452036040862055807f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254037f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0255604051908152a3565b91507fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054163303612fd157565b7f9a47cf30000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b90613007826124a0565b67ffffffffffffffff82165f5260205260405f2060016040519161302a8361228e565b6001600160a01b0381541683520154926020820193808552156130dc576130548451611d9f6136cc565b15611e27577fef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332926001600160a01b03836130968261278b9651168851906128d9565b61309f846124a0565b67ffffffffffffffff84165f526020525f60016040822082815501555116945167ffffffffffffffff604051948594606086526060860190612221565b61266d908367ffffffffffffffff6040519384937fa4e158af000000000000000000000000000000000000000000000000000000008552604060048601526044850190612221565b9594929091939484156108905785156136065783156135de5767ffffffffffffffff1693428511156135a45761315c84611d9f61372b565b15611e275761316b8488612e83565b6001600160a01b0361317d82856125c2565b936131dc6040519361318e85612272565b6131978761239e565b85528360026131a860018a0161239e565b9860208801998a5201541692604086019384528551602081519101206131cf3684846122e9565b60208151910120146127b2565b5116925f60405180957f56d981a700000000000000000000000000000000000000000000000000000000825260406004830152818061321f604482018d8a6125fa565b8a602483015203915afa938415611f9a575f94613564575b506001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054169582519151966040519260c0840184811067ffffffffffffffff8211176106cd576040528352602083019788526020978893604051936132a386866122aa565b5f85526040820194855260608201988952608082019081526040516132c887826122aa565b5f815260a08301908152604051998a96879586957f7e5a65660000000000000000000000000000000000000000000000000000000087528a6004880152516024870160c0905260e4870161331b91612221565b9051908681037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc01604488015261335191612221565b9051908581037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc01606487015261338791612221565b9051908481037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160848601526133bd91612221565b915167ffffffffffffffff1660a484015251908281037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160c484015261340391612221565b03915a905f91f1928315611f9a575f936134fa575b50917fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a956134ef926134dc9695946001600160a01b036040519a61345b8c61228e565b16998a8152600188820188815261347284516124a0565b67ffffffffffffffff88165f528a526001600160a01b038060405f20945116167fffffffffffffffffffffffff000000000000000000000000000000000000000084541617835551910155519567ffffffffffffffff60405198899860808a5260808a0190612221565b94169087015285830360408701526125fa565b9060608301520390a2565b9091948094935081813d831161355d575b61351581836122aa565b8101031261026e57519067ffffffffffffffff8216820361026e57929391929091907fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a613418565b503d61350b565b9093503d805f833e61357681836122aa565b810160208282031261026e57815167ffffffffffffffff811161026e5761359d920161245c565b925f613237565b847f0b8a29e4000000000000000000000000000000000000000000000000000000005f5260045267ffffffffffffffff421660245260445ffd5b7fcc2a76e5000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f5de17177000000000000000000000000000000000000000000000000000000005f5260045ffd5b60206001600160a01b037f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad9216807fffffffffffffffffffffffff00000000000000000000000000000000000000007ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055604051908152a1565b60206040516136dc6040826122aa565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e5f603b820152601c8152613725603c826122aa565b51902090565b602060405161373b6040826122aa565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e7f0100000000000000000000000000000000000000000000000000000000000000603b820152601c8152613725603c826122aa565b9190820180921161039a57565b6123d69065ffffffffffff6137c542613d89565b6137ef837f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb006139dd565b509279ffffffffffffffffffffffffffffffffffffffffffffffffffff604051946138198661228e565b16845290821660208085019182525f9283527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb01905260409091209251905190911660d01b7fffffffffffff00000000000000000000000000000000000000000000000000001679ffffffffffffffffffffffffffffffffffffffffffffffffffff91909116179055565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c16156138d257565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b907f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb005479ffffffffffffffffffffffffffffffffffffffffffffffffffff8116925f527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb0160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff61399542613d89565b9160d01c91160365ffffffffffff811161039a578465ffffffffffff6139ca9360d01c60018082119118026001189216613cbd565b8103908111150291828103908111150290565b919091600181549179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316945f520160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff61399542613d89565b7f01ffc9a7000000000000000000000000000000000000000000000000000000005f527f01ffc9a70000000000000000000000000000000000000000000000000000000060045260205f60248184617530fa5f511515601f3d111681613b13575b5015613b0e575f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fffffffff00000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681613b08575090565b90501590565b505f90565b90505f613a9e565b908015613bfc57613b4c827f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb006139dd565b9091908111613bf557613b6890613b6242613d89565b926137a4565b9179ffffffffffffffffffffffffffffffffffffffffffffffffffff8311613bc457613bbf929165ffffffffffff9179ffffffffffffffffffffffffffffffffffffffffffffffffffff604051946138198661228e565b600190565b827f6dfcc650000000000000000000000000000000000000000000000000000000005f5260d060045260245260445ffd5b5050505f90565b5050600190565b905f8091602081519101845af48080613caa575b15613c375750506040513d81523d5f602083013e60203d82010160405290565b15613c71576001600160a01b03907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d15613c82576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d151580613c175750813b1515613c17565b90915f198383099280830292838086109503948086039514613d4e5784831115613d365790829109815f0382168092046002816003021880820260020302808202600203028082026002030280820260020302808202600203028091026002030293600183805f03040190848311900302920304170290565b82634e487b715f52156003026011186020526024601cfd5b505080925015613d5c570490565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b65ffffffffffff8111613da15765ffffffffffff1690565b7f6dfcc650000000000000000000000000000000000000000000000000000000005f52603060045260245260445ffdfea164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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
