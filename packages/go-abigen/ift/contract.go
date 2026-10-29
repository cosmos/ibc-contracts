// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package ift

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
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"allowance\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"approve\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"balanceOf\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"burn\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"burnFrom\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"decimals\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.IFTBridge\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"contractIIFTSendCallConstructor\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimit\",\"inputs\":[],\"outputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimitAvailable\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPendingTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.PendingTransfer\",\"components\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ics27\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIICS27GMP\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"iftMint\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"owner_\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"erc20Name\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"erc20Symbol\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"ics27Gmp\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"mint\",\"inputs\":[{\"name\":\"mintAddress\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"name\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"onAckPacket\",\"inputs\":[{\"name\":\"success\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnAcknowledgementPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onTimeoutPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnTimeoutPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"pause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"paused\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"registerIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"removeIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setIFTRateLimit\",\"inputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"supportsInterface\",\"inputs\":[{\"name\":\"interfaceId\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"symbol\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalSupply\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferFrom\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"unpause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"event\",\"name\":\"Approval\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRegistered\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRemoved\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTMintReceived\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTRateLimitSet\",\"inputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"indexed\":false,\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"indexed\":false,\"internalType\":\"uint48\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferCompleted\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferInitiated\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"receiver\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferRefunded\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Paused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Transfer\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Unpaused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ERC20InsufficientAllowance\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"allowance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InsufficientBalance\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidApprover\",\"inputs\":[{\"name\":\"approver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSpender\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"EnforcedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ExpectedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTBridgeNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTEmptyClientId\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyCounterpartyAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyReceiver\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTInvalidConstructorInterface\",\"inputs\":[{\"name\":\"callConstructor\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTInvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTOnlyICS27GMP\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTPendingTransferNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTTimeoutInPast\",\"inputs\":[{\"name\":\"timeout\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"currentTime\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTUnauthorizedMint\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTUnexpectedSalt\",\"inputs\":[{\"name\":\"salt\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"IFTZeroAddressConstructor\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTZeroAmount\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"RateLimitExceeded\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeCastOverflowedUintDowncast\",\"inputs\":[{\"name\":\"bits\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}]",
	Bin: "0x60a080604052346100c257306080525f516020613eb65f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b604051613def90816100c782396080518181816117a801526118700152f35b6001600160401b0319166001600160401b039081175f516020613eb65f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c90816301ffc9a7146123005750806306fdde031461221d578063095ea7b3146121225780630a7244e714611da857806318160ddd14611d6c57806323b872dd14611d34578063313ce56714611d195780633f4ba83a14611c3e57806340c10f1914611c11578063419e41c614611af557806342966c6814611ad85780634f1ef2861461182057806352d1902d1461178e578063599deb48146117495780635c975abb146117085780635e32b6b614611696578063613d25bb1461101e57806370a0823114610fc7578063711708b314610f49578063715018a614610e9a57806379cc679014610e6a5780638456cb5914610db55780638da5cb5b14610d705780638dfcd9ad14610d1d5780639226e08314610c5c57806395d89b4114610b22578063997d99b814610abf578063a50e31f114610a83578063a9059cbb14610a52578063ad3cb1cc146109ef578063b8ce241814610902578063d638f98b14610413578063d88a36fe14610337578063dd62ed3e146102be578063e529a26d146101d75763f2fde38b146101a8575f80fd5b346101d35760206003193601126101d3576101d16101c46123c3565b6101cc612e68565b6129e2565b005b5f80fd5b346101d35760206003193601126101d35760043567ffffffffffffffff81116101d35761020b61029f91369060040161249c565b905f6040805161021a816123ef565b606081526060602082015201526001600160a01b036102b261023c848461273f565b6102836040519561024c876123ef565b6102558361251b565b87528460026102666001860161251b565b9460208a019586520154169560408801968752875151151561292f565b604051958695602087525160606020880152608087019061239e565b9051601f1986830301604087015261239e565b91511660608301520390f35b346101d35760406003193601126101d3576102d76123c3565b6001600160a01b036103216102ea6123d9565b926001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f52602052602060405f2054604051908152f35b346101d35760606003193601126101d35760043567ffffffffffffffff81116101d35761036890369060040161249c565b9060243567ffffffffffffffff81116101d35761038990369060040161249c565b610394929192612ab5565b61038467ffffffffffffffff4216019267ffffffffffffffff84116103e6576103c1946044359333613170565b5f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b346101d35760606003193601126101d35760043567ffffffffffffffff81116101d35761044490369060040161249c565b9060243567ffffffffffffffff81116101d35761046590369060040161249c565b9092906044356001600160a01b03811691908281036101d357610486612e68565b83156108da5781156108b257821561088a576104a181613a4e565b9081610818575b50156107ec576040516104ba816123ef565b6104c5368587612466565b81526104d2368388612466565b6020820190815260408201908482526104eb868861273f565b925180519067ffffffffffffffff82116107175761050985546124ca565b601f81116107b2575b50602090601f831160011461074f5761054192915f9183610744575b50505f198260011b9260031b1c19161790565b83555b518051600184019167ffffffffffffffff82116107175761056583546124ca565b601f81116106d4575b50602090601f831160011461063c5761062696947fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89b96946105d1856001600160a01b039660029688965f926106315750505f198260011b9260031b1c19161790565b90555b5116920191167fffffffffffffffffffffffff0000000000000000000000000000000000000000825416179055610618604051968796606088526060880191612777565b918583036020870152612777565b9060408301520390a1005b015190505f8061052e565b90601f19831691845f52815f20925f5b8181106106bc5750946001856002957fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89f9a98956106269c9a956001600160a01b03998a98106106a4575b505050811b0190556105d4565b01515f1960f88460031b161c191690558f8080610697565b9293602060018192878601518155019501930161064c565b8281111561056e5761070990845f5260205f2090601f850160051c906020861061070f575b601f82910160051c039101612978565b8a61056e565b5f91506106f9565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b015190508b8061052e565b90601f19831691865f52815f20925f5b81811061079a5750908460019594939210610782575b505050811b018355610544565b01515f1960f88460031b161c191690558a8080610775565b9293602060018192878601518155019501930161075f565b82811115610512576107e690865f5260205f2090601f850160051c906020861061070f57601f82910160051c039101612978565b8a610512565b507f3730169b000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b6020915060245f80927f01ffc9a70000000000000000000000000000000000000000000000000000000082527f56d981a700000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681610882575b50866104a8565b90508661087b565b7ff3a03283000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f9624aeb8000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fc8ca373e000000000000000000000000000000000000000000000000000000005f5260045ffd5b346101d35760206003193601126101d35760043567ffffffffffffffff81116101d3576109547f142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a84655775191369060040161249c565b61095c612e68565b6109af818361096b828261273f565b6001600160a01b03600260405192610982846123ef565b61098b8161251b565b84526109996001820161251b565b602085015201541660408201525151151561292f565b5f60026109bc838561273f565b6109c581612993565b6109d160018201612993565b01556109ea604051928392602084526020840191612777565b0390a1005b346101d3575f6003193601126101d357610a4e604051610a10604082612427565b600581527f352e302e30000000000000000000000000000000000000000000000000000000602082015260405191829160208352602083019061239e565b0390f35b346101d35760406003193601126101d357610a78610a6e6123c3565b6024359033612d2c565b602060405160018152f35b346101d3575f6003193601126101d3576040610aa5610aa061368a565b61390b565b9050610ab2610aa06136e9565b8351928352602083015250f35b346101d3575f6003193601126101d3577f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00546040805179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316815260d09290921c602083015290f35b346101d3575f6003193601126101d3576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454610b61816124ca565b8084529060018116908115610c1a5750600114610b9d575b610a4e83610b8981850382612427565b60405191829160208352602083019061239e565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f9081527f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa939250905b808210610c0057509091508101602001610b89610b79565b919260018160209254838588010152019101909291610be8565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208086019190915291151560051b84019091019150610b899050610b79565b346101d35760406003193601126101d35760043567ffffffffffffffff81116101d357610c8d90369060040161249c565b6024359167ffffffffffffffff831683036101d357610d066040935f60208651610cb68161240b565b8281520152610cc58484612707565b67ffffffffffffffff82165f52602052845f20936001865195610ce78761240b565b6001600160a01b03815416875201549360208601948086521515612797565b6001600160a01b0383519251168252516020820152f35b346101d35760406003193601126101d35760043580151581036101d3576024359067ffffffffffffffff82116101d35760c060031983360301126101d3576103c191610d67612ab5565b600401906127f5565b346101d3575f6003193601126101d35760206001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c1993005416604051908152f35b346101d3575f6003193601126101d357610dcd612e68565b610dd5613762565b60017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff007fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005416177fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a2586020604051338152a1005b346101d35760406003193601126101d3576101d1610e866123c3565b60243590610e95823383612c10565b612ec7565b346101d3575f6003193601126101d357610eb2612e68565b5f6001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c199300547fffffffffffffffffffffffff000000000000000000000000000000000000000081167f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930055167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e08280a3005b346101d35760806003193601126101d35760043567ffffffffffffffff81116101d357610f7a90369060040161249c565b9060243567ffffffffffffffff81116101d357610f9b90369060040161249c565b606435929167ffffffffffffffff841684036101d3576103c194610fbd612ab5565b6044359333613170565b346101d35760206003193601126101d3576001600160a01b03610fe86123c3565b165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00602052602060405f2054604051908152f35b346101d35760806003193601126101d3576110376123c3565b60243567ffffffffffffffff81116101d35761105790369060040161249c565b60449291923567ffffffffffffffff81116101d35761107a90369060040161249c565b9390606435936001600160a01b0385168095036101d3577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549260ff8460401c16159467ffffffffffffffff85168015908161168e575b6001149081611684575b15908161167b575b506116535761115261116193868860017fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000006111699a16177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00556115fe575b5061114a6138b4565b6101cc6138b4565b61115a6138b4565b3691612466565b943691612466565b916111726138b4565b61117a6138b4565b6111826138b4565b835167ffffffffffffffff8111610717576111bd7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03546124ca565b601f8111611587575b50602094601f82116001146114e6576111f79293949582915f926114db5750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03555b825167ffffffffffffffff8111610717576112557f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04546124ca565b601f8111611464575b506020601f82116001146113c357819061128d9394955f926113b85750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04555b6112b86138b4565b7fffffffffffffffffffffffff00000000000000000000000000000000000000007f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416177f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005561132557005b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a1005b01519050858061052e565b601f198216907f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f52805f20915f5b81811061144c57509583600195969710611434575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04556112b0565b01515f1960f88460031b161c19169055848080611407565b9192602060018192868b0151815501940192016113f2565b8181111561125e577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f526114d5907f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa90601f840160051c906020851061070f57601f82910160051c039101612978565b8461125e565b01519050868061052e565b601f198216957f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52805f20915f5b88811061156f57508360019596979810611557575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035561121a565b01515f1960f88460031b161c1916905585808061152a565b91926020600181928685015181550194019201611515565b818111156111c6577f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f526115f8907f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab090601f840160051c906020851061070f57601f82910160051c039101612978565b856111c6565b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00558a611141565b7ff92ee8a9000000000000000000000000000000000000000000000000000000005f5260045ffd5b905015896110e3565b303b1591506110db565b8791506110d1565b346101d35760206003193601126101d35760043567ffffffffffffffff81116101d357806004019060a060031982360301126101d3576117036044916116fb6116f26103c1956116e4612ab5565b6116ec612fe2565b806126a1565b949092016126f2565b923691612466565b613041565b346101d3575f6003193601126101d357602060ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f0330054166040519015158152f35b346101d3575f6003193601126101d35760206001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051908152f35b346101d3575f6003193601126101d3576001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001630036117f85760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b7fe07c8dba000000000000000000000000000000000000000000000000000000005f5260045ffd5b60406003193601126101d3576118346123c3565b60243567ffffffffffffffff81116101d357366023820112156101d357611865903690602481600401359101612466565b906001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016803014908115611aa3575b506117f8576118a8612e68565b6001600160a01b038116916040517f52d1902d000000000000000000000000000000000000000000000000000000008152602081600481875afa5f9181611a6f575b5061191b57837f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b807f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc859203611a445750823b15611a1957807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a28051156119e8576101d191613c14565b5050346119f157005b7fb398979f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7faa1d49a4000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b9091506020813d602011611a9b575b81611a8b60209383612427565b810103126101d3575190856118ea565b3d9150611a7e565b90506001600160a01b037f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc541614158361189b565b346101d35760206003193601126101d3576101d160043533612ec7565b346101d35760406003193601126101d35760043579ffffffffffffffffffffffffffffffffffffffffffffffffffff8116908181036101d3576024359065ffffffffffff821682036101d3577fab4afaccf3aa45be43259d38a98ad9d645ddb34681869353a3606a54fe4f1a9192611b6b612e68565b611b7b611b7661368a565b6137c2565b611b86611b766136e9565b79ffffffffffffffffffffffffffffffffffffffffffffffffffff90811660d084901b7fffffffffffff000000000000000000000000000000000000000000000000000016177f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb00556040805192909116825265ffffffffffff909216602082015290819081016109ea565b346101d35760406003193601126101d3576101d1611c2d6123c3565b611c35612e68565b60243590612b29565b346101d3575f6003193601126101d357611c56612e68565b7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005460ff811615611cf1577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00167fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa6020604051338152a1005b7f8dfc202b000000000000000000000000000000000000000000000000000000005f5260045ffd5b346101d3575f6003193601126101d357602060405160128152f35b346101d35760606003193601126101d357610a78611d506123c3565b611d586123d9565b60443591611d67833383612c10565b612d2c565b346101d3575f6003193601126101d35760207f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254604051908152f35b346101d35760406003193601126101d357611dc16123c3565b60243590611dcd612ab5565b60245f6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051928380927fe57d7de00000000000000000000000000000000000000000000000000000000082523360048301525afa908115612117575f91612062575b506020808251604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a01815203019020611ee0604051611e87816123ef565b611e908361251b565b81526001600160a01b036002611ea86001860161251b565b94602084019586520154166040820152611ec8815151151585519061265b565b5160208151910120835190815160208301201461265b565b5180516020820120602083015190815160208301200361200e57505060408101518051611fcc5750611f10613762565b611f2183611f1c61368a565b613b2c565b15611fa457816001600160a01b03611f7492611f5e867f3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae96612b29565b519160405193849360408552604085019061239e565b95602084015216930390a25f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7fa74c1c5f000000000000000000000000000000000000000000000000000000005f5260045ffd5b61200a906040519182917f3ef1dc0200000000000000000000000000000000000000000000000000000000835260206004840152602483019061239e565b0390fd5b9061200a612050926040519384937ff39880b100000000000000000000000000000000000000000000000000000000855260406004860152604485019061239e565b9060031984830301602485015261239e565b90503d805f833e6120738183612427565b8101906020818303126101d35780519067ffffffffffffffff82116101d357016060818303126101d357604051916120aa836123ef565b815167ffffffffffffffff81116101d357816120c79184016125d9565b8352602082015167ffffffffffffffff81116101d357816120e99184016125d9565b6020840152604082015167ffffffffffffffff81116101d35761210c92016125d9565b604082015283611e3b565b6040513d5f823e3d90fd5b346101d35760406003193601126101d35761213b6123c3565b6024359033156121f1576001600160a01b03169081156121c557335f9081527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260409020825f526020528060405f20556040519081527f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560203392a3602060405160018152f35b7f94280d62000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7fe602df05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b346101d3575f6003193601126101d3576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035461225c816124ca565b8084529060018116908115610c1a575060011461228357610a4e83610b8981850382612427565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f9081527f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab0939250905b8082106122e657509091508101602001610b89610b79565b9192600181602092548385880101520191019092916122ce565b346101d35760206003193601126101d357600435907fffffffff0000000000000000000000000000000000000000000000000000000082168092036101d357817fd3ce6f1b0000000000000000000000000000000000000000000000000000000060209314908115612374575b5015158152f35b7f01ffc9a7000000000000000000000000000000000000000000000000000000009150148361236d565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b600435906001600160a01b03821682036101d357565b602435906001600160a01b03821682036101d357565b6060810190811067ffffffffffffffff82111761071757604052565b6040810190811067ffffffffffffffff82111761071757604052565b90601f601f19910116810190811067ffffffffffffffff82111761071757604052565b67ffffffffffffffff811161071757601f01601f191660200190565b9291926124728261244a565b916124806040519384612427565b8294818452818301116101d3578281602093845f960137010152565b9181601f840112156101d35782359167ffffffffffffffff83116101d357602083818601950101116101d357565b90600182811c92168015612511575b60208310146124e457565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f16916124d9565b9060405191825f82549261252e846124ca565b80845293600181169081156125995750600114612555575b5061255392500383612427565b565b90505f9291925260205f20905f915b81831061257d575050906020612553928201015f612546565b6020919350806001915483858901015201910190918492612564565b602093506125539592507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0091501682840152151560051b8201015f612546565b81601f820112156101d3576020815191016125f38261244a565b926126016040519485612427565b828452828201116101d357815f926020928386015e8301015290565b60208091604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b156126635750565b61200a906040519182917f24a61b9500000000000000000000000000000000000000000000000000000000835260206004840152602483019061239e565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1813603018212156101d3570180359067ffffffffffffffff82116101d3576020019181360383136101d357565b3567ffffffffffffffff811681036101d35790565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0181520301902090565b601f8260209493601f1993818652868601375f8582860101520116010190565b929092156127a457505050565b67ffffffffffffffff6127ea6040519485947fa4e158af000000000000000000000000000000000000000000000000000000008652604060048701526044860191612777565b911660248301520390fd5b6127fd612fe2565b15612919577f5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc92861283661283083806126a1565b90612707565b604083019067ffffffffffffffff61284d836126f2565b165f526020526128e360405f206001600160a01b036128db6001604051936128748561240b565b8381541685520154966128a46020850198808a5261289283806126a1565b9061289c8a6126f2565b921515612797565b6128b161283082806126a1565b67ffffffffffffffff6128c3886126f2565b165f526020525f6001604082208281550155806126a1565b9390946126f2565b915116945167ffffffffffffffff612908604051958695606087526060870191612777565b9216602084015260408301520390a2565b6040611703826116fb6116f282612553966126a1565b9190911561293b575050565b61200a6040519283927f24a61b95000000000000000000000000000000000000000000000000000000008452602060048501526024840191612777565b5f5b82811061298657505050565b5f8282015560010161297a565b61299d81546124ca565b90816129a7575050565b81601f5f93116001146129b8575055565b818352602083206129d591601f0160051c5f190190600101612978565b8082528160208120915555565b6001600160a01b03168015612a89576001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930054827fffffffffffffffffffffffff00000000000000000000000000000000000000008216177f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930055167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e05f80a3565b7f1e4fbdf7000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c612b015760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b6001600160a01b0316908115612be4577fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef602082612b895f947f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02546137b5565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02558484527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260408420818154019055604051908152a3565b7fec442f05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b9190612c4c836001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b6001600160a01b0382165f5260205260405f2054925f198410612c70575b50505050565b828410612cef576001600160a01b038116156121f1576001600160a01b038216156121c557612cd76001600160a01b03916001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f5260205260405f20910390555f808080612c6a565b506001600160a01b0383917ffb8f41b2000000000000000000000000000000000000000000000000000000005f521660045260245260445260645ffd5b6001600160a01b0316908115612e3c576001600160a01b0316918215612be457815f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054818110612e0a57817fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92602092855f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0084520360405f2055845f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260405f20818154019055604051908152a3565b827fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b7f96c6fd1e000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b6001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930054163303612e9b57565b7f118cdaa7000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b9091906001600160a01b03168015612e3c57805f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054838110612faf576020845f94957fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef938587527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace008452036040862055807f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254037f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0255604051908152a3565b91507fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416330361301557565b7f9a47cf30000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b9061304b8261261d565b67ffffffffffffffff82165f5260205260405f2060016040519161306e8361240b565b6001600160a01b03815416835201549260208201938085521561312857613093613762565b6130a08451611f1c61368a565b15611fa4577fef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332926001600160a01b03836130e282612908965116885190612b29565b6130eb8461261d565b67ffffffffffffffff84165f526020525f60016040822082815501555116945167ffffffffffffffff60405194859460608652606086019061239e565b6127ea908367ffffffffffffffff6040519384937fa4e158af00000000000000000000000000000000000000000000000000000000855260406004860152604485019061239e565b9594929091939484156108da57851561366257831561363a5767ffffffffffffffff169342851115613600576131a6818461273f565b9261320c604051926131b7846123ef565b6131c08661251b565b84526001600160a01b0360026131d86001890161251b565b976020870198895201541692604085019384528451602081519101206131ff368484612466565b602081519101201461292f565b613214613762565b61322085611f1c6136e9565b15611fa4576001600160a01b0390613238868a612ec7565b5116925f60405180957f56d981a700000000000000000000000000000000000000000000000000000000825260406004830152818061327b604482018d8a612777565b8a602483015203915afa938415612117575f946135c0575b506001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054169582519151966040519260c0840184811067ffffffffffffffff821117610717576040528352602083019788526020978893604051936132ff8686612427565b5f85526040820194855260608201988952608082019081526040516133248782612427565b5f815260a08301908152604051998a96879586957f7e5a65660000000000000000000000000000000000000000000000000000000087528a6004880152516024870160c0905260e487016133779161239e565b9051908681037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160448801526133ad9161239e565b9051908581037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160648701526133e39161239e565b9051908481037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160848601526134199161239e565b915167ffffffffffffffff1660a484015251908281037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160c484015261345f9161239e565b03915a905f91f1928315612117575f93613556575b50917fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a9561354b926135389695946001600160a01b036040519a6134b78c61240b565b16998a815260018882018881526134ce845161261d565b67ffffffffffffffff88165f528a526001600160a01b038060405f20945116167fffffffffffffffffffffffff000000000000000000000000000000000000000084541617835551910155519567ffffffffffffffff60405198899860808a5260808a019061239e565b9416908701528583036040870152612777565b9060608301520390a2565b9091948094935081813d83116135b9575b6135718183612427565b810103126101d357519067ffffffffffffffff821682036101d357929391929091907fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a613474565b503d613567565b9093503d805f833e6135d28183612427565b81016020828203126101d357815167ffffffffffffffff81116101d3576135f992016125d9565b925f613293565b847f0b8a29e4000000000000000000000000000000000000000000000000000000005f5260045267ffffffffffffffff421660245260445ffd5b7fcc2a76e5000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f5de17177000000000000000000000000000000000000000000000000000000005f5260045ffd5b602060405161369a604082612427565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e5f603b820152601c81526136e3603c82612427565b51902090565b60206040516136f9604082612427565b601b8152017f6962632e6966742e726174656c696d69742e646972656374696f6e00000000008152604051601b6020820192835e7f0100000000000000000000000000000000000000000000000000000000000000603b820152601c81526136e3603c82612427565b60ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300541661378d57565b7fd93c0665000000000000000000000000000000000000000000000000000000005f5260045ffd5b919082018092116103e657565b6125539065ffffffffffff6137d642613d9a565b613800837f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb006139ee565b509279ffffffffffffffffffffffffffffffffffffffffffffffffffff6040519461382a8661240b565b16845290821660208085019182525f9283527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb01905260409091209251905190911660d01b7fffffffffffff00000000000000000000000000000000000000000000000000001679ffffffffffffffffffffffffffffffffffffffffffffffffffff91909116179055565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c16156138e357565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b907f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb005479ffffffffffffffffffffffffffffffffffffffffffffffffffff8116925f527f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb0160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff6139a642613d9a565b9160d01c91160365ffffffffffff81116103e6578465ffffffffffff6139db9360d01c60018082119118026001189216613cce565b8103908111150291828103908111150290565b919091600181549179ffffffffffffffffffffffffffffffffffffffffffffffffffff8316945f520160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff6139a642613d9a565b7f01ffc9a7000000000000000000000000000000000000000000000000000000005f527f01ffc9a70000000000000000000000000000000000000000000000000000000060045260205f60248184617530fa5f511515601f3d111681613b24575b5015613b1f575f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fffffffff00000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d111681613b19575090565b90501590565b505f90565b90505f613aaf565b908015613c0d57613b5d827f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb006139ee565b9091908111613c0657613b7990613b7342613d9a565b926137b5565b9179ffffffffffffffffffffffffffffffffffffffffffffffffffff8311613bd557613bd0929165ffffffffffff9179ffffffffffffffffffffffffffffffffffffffffffffffffffff6040519461382a8661240b565b600190565b827f6dfcc650000000000000000000000000000000000000000000000000000000005f5260d060045260245260445ffd5b5050505f90565b5050600190565b905f8091602081519101845af48080613cbb575b15613c485750506040513d81523d5f602083013e60203d82010160405290565b15613c82576001600160a01b03907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d15613c93576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d151580613c285750813b1515613c28565b90915f198383099280830292838086109503948086039514613d5f5784831115613d475790829109815f0382168092046002816003021880820260020302808202600203028082026002030280820260020302808202600203028091026002030293600183805f03040190848311900302920304170290565b82634e487b715f52156003026011186020526024601cfd5b505080925015613d6d570490565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b65ffffffffffff8111613db25765ffffffffffff1690565b7f6dfcc650000000000000000000000000000000000000000000000000000000005f52603060045260245260445ffdfea164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_Contract *ContractCaller) Owner(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "owner")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_Contract *ContractSession) Owner() (common.Address, error) {
	return _Contract.Contract.Owner(&_Contract.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_Contract *ContractCallerSession) Owner() (common.Address, error) {
	return _Contract.Contract.Owner(&_Contract.CallOpts)
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
// Solidity: function initialize(address owner_, string erc20Name, string erc20Symbol, address ics27Gmp) returns()
func (_Contract *ContractTransactor) Initialize(opts *bind.TransactOpts, owner_ common.Address, erc20Name string, erc20Symbol string, ics27Gmp common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "initialize", owner_, erc20Name, erc20Symbol, ics27Gmp)
}

// Initialize is a paid mutator transaction binding the contract method 0x613d25bb.
//
// Solidity: function initialize(address owner_, string erc20Name, string erc20Symbol, address ics27Gmp) returns()
func (_Contract *ContractSession) Initialize(owner_ common.Address, erc20Name string, erc20Symbol string, ics27Gmp common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, owner_, erc20Name, erc20Symbol, ics27Gmp)
}

// Initialize is a paid mutator transaction binding the contract method 0x613d25bb.
//
// Solidity: function initialize(address owner_, string erc20Name, string erc20Symbol, address ics27Gmp) returns()
func (_Contract *ContractTransactorSession) Initialize(owner_ common.Address, erc20Name string, erc20Symbol string, ics27Gmp common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, owner_, erc20Name, erc20Symbol, ics27Gmp)
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

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_Contract *ContractTransactor) RenounceOwnership(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "renounceOwnership")
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_Contract *ContractSession) RenounceOwnership() (*types.Transaction, error) {
	return _Contract.Contract.RenounceOwnership(&_Contract.TransactOpts)
}

// RenounceOwnership is a paid mutator transaction binding the contract method 0x715018a6.
//
// Solidity: function renounceOwnership() returns()
func (_Contract *ContractTransactorSession) RenounceOwnership() (*types.Transaction, error) {
	return _Contract.Contract.RenounceOwnership(&_Contract.TransactOpts)
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

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_Contract *ContractTransactor) TransferOwnership(opts *bind.TransactOpts, newOwner common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "transferOwnership", newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_Contract *ContractSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _Contract.Contract.TransferOwnership(&_Contract.TransactOpts, newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_Contract *ContractTransactorSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _Contract.Contract.TransferOwnership(&_Contract.TransactOpts, newOwner)
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

// ContractOwnershipTransferredIterator is returned from FilterOwnershipTransferred and is used to iterate over the raw logs and unpacked data for OwnershipTransferred events raised by the Contract contract.
type ContractOwnershipTransferredIterator struct {
	Event *ContractOwnershipTransferred // Event containing the contract specifics and raw log

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
func (it *ContractOwnershipTransferredIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractOwnershipTransferred)
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
		it.Event = new(ContractOwnershipTransferred)
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
func (it *ContractOwnershipTransferredIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractOwnershipTransferredIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractOwnershipTransferred represents a OwnershipTransferred event raised by the Contract contract.
type ContractOwnershipTransferred struct {
	PreviousOwner common.Address
	NewOwner      common.Address
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterOwnershipTransferred is a free log retrieval operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_Contract *ContractFilterer) FilterOwnershipTransferred(opts *bind.FilterOpts, previousOwner []common.Address, newOwner []common.Address) (*ContractOwnershipTransferredIterator, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return &ContractOwnershipTransferredIterator{contract: _Contract.contract, event: "OwnershipTransferred", logs: logs, sub: sub}, nil
}

// WatchOwnershipTransferred is a free log subscription operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_Contract *ContractFilterer) WatchOwnershipTransferred(opts *bind.WatchOpts, sink chan<- *ContractOwnershipTransferred, previousOwner []common.Address, newOwner []common.Address) (event.Subscription, error) {

	var previousOwnerRule []interface{}
	for _, previousOwnerItem := range previousOwner {
		previousOwnerRule = append(previousOwnerRule, previousOwnerItem)
	}
	var newOwnerRule []interface{}
	for _, newOwnerItem := range newOwner {
		newOwnerRule = append(newOwnerRule, newOwnerItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "OwnershipTransferred", previousOwnerRule, newOwnerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractOwnershipTransferred)
				if err := _Contract.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
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

// ParseOwnershipTransferred is a log parse operation binding the contract event 0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0.
//
// Solidity: event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
func (_Contract *ContractFilterer) ParseOwnershipTransferred(log types.Log) (*ContractOwnershipTransferred, error) {
	event := new(ContractOwnershipTransferred)
	if err := _Contract.contract.UnpackLog(event, "OwnershipTransferred", log); err != nil {
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
