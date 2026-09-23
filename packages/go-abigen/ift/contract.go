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
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"allowance\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"approve\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"balanceOf\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"burn\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"burnFrom\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"decimals\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"uint8\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.IFTBridge\",\"components\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"contractIIFTSendCallConstructor\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimit\",\"inputs\":[{\"name\":\"direction\",\"type\":\"uint8\",\"internalType\":\"enumIIFTMsgs.IFTRateLimitDirection\"}],\"outputs\":[{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIFTRateLimitAvailable\",\"inputs\":[{\"name\":\"direction\",\"type\":\"uint8\",\"internalType\":\"enumIIFTMsgs.IFTRateLimitDirection\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPendingTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIIFTMsgs.PendingTransfer\",\"components\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ics27\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIICS27GMP\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"iftMint\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"iftTransfer\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"owner_\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"erc20Name\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"erc20Symbol\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"ics27Gmp\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"mint\",\"inputs\":[{\"name\":\"mintAddress\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"name\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"onAckPacket\",\"inputs\":[{\"name\":\"success\",\"type\":\"bool\",\"internalType\":\"bool\"},{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnAcknowledgementPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onTimeoutPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnTimeoutPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"registerIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"removeIFTBridge\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"renounceOwnership\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setIFTRateLimit\",\"inputs\":[{\"name\":\"direction\",\"type\":\"uint8\",\"internalType\":\"enumIIFTMsgs.IFTRateLimitDirection\"},{\"name\":\"capacity\",\"type\":\"uint208\",\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"internalType\":\"uint48\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"supportsInterface\",\"inputs\":[{\"name\":\"interfaceId\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"symbol\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"totalSupply\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transfer\",\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferFrom\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"event\",\"name\":\"Approval\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"spender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRegistered\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"counterpartyIFTAddress\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"iftSendCallConstructor\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTBridgeRemoved\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTMintReceived\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"receiver\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTRateLimitSet\",\"inputs\":[{\"name\":\"direction\",\"type\":\"uint8\",\"indexed\":false,\"internalType\":\"enumIIFTMsgs.IFTRateLimitDirection\"},{\"name\":\"capacity\",\"type\":\"uint208\",\"indexed\":false,\"internalType\":\"uint208\"},{\"name\":\"window\",\"type\":\"uint48\",\"indexed\":false,\"internalType\":\"uint48\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferCompleted\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferInitiated\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"receiver\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IFTTransferRefunded\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"sender\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"newOwner\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Transfer\",\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"to\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ERC20InsufficientAllowance\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"allowance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InsufficientBalance\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidApprover\",\"inputs\":[{\"name\":\"approver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC20InvalidSpender\",\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTBridgeNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTEmptyClientId\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyCounterpartyAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTEmptyReceiver\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTInvalidConstructorInterface\",\"inputs\":[{\"name\":\"callConstructor\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTInvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTOnlyICS27GMP\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"IFTPendingTransferNotFound\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTTimeoutInPast\",\"inputs\":[{\"name\":\"timeout\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"currentTime\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"IFTUnauthorizedMint\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"IFTUnexpectedSalt\",\"inputs\":[{\"name\":\"salt\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"IFTUnknownRateLimitDirection\",\"inputs\":[{\"name\":\"direction\",\"type\":\"uint8\",\"internalType\":\"enumIIFTMsgs.IFTRateLimitDirection\"}]},{\"type\":\"error\",\"name\":\"IFTZeroAddressConstructor\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"IFTZeroAmount\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"OwnableInvalidOwner\",\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"OwnableUnauthorizedAccount\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"RateLimitExceeded\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeCastOverflowedUintDowncast\",\"inputs\":[{\"name\":\"bits\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}]",
	Bin: "0x60a080604052346100c257306080525f516020613c325f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b604051613b6b90816100c782396080518181816117f701526118bf0152f35b6001600160401b0319166001600160401b039081175f516020613c325f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c90816301ffc9a71461216e5750806306fdde031461208b578063095ea7b314611f905780630a7244e714611c0057806318160ddd14611bc457806323b872dd14611b8c578063313ce56714611b7157806340c10f1914611b4457806342966c6814611b275780634f1ef2861461186f57806352d1902d146117dd578063599deb48146117985780635e32b6b614611726578063613d25bb146110ca57806370a0823114611073578063711708b314610ff5578063715018a614610f4657806372b3449c14610ef157806379cc679014610ec157806386c13d6414610e875780638da5cb5b14610e425780638dfcd9ad14610def5780639226e08314610d2e57806395d89b4114610bf4578063a9059cbb14610bc3578063ad3cb1cc14610b60578063b8ce241814610a73578063d638f98b14610596578063d88a36fe146104ba578063dd62ed3e14610441578063e529a26d1461035a578063e915ec0e146101b65763f2fde38b14610187575f80fd5b346101b25760206003193601126101b2576101b06101a3612231565b6101ab612cd0565b61284a565b005b5f80fd5b346101b25760606003193601126101b25760043560028110156101b2576024359079ffffffffffffffffffffffffffffffffffffffffffffffffffff82168092036101b2576044359065ffffffffffff82168083036101b2577fd56b392721db10260a8376f16903b557916e8e2a95fe49f84d79f1d0fd89946a9361034760609461023f612cd0565b61024885613517565b6102fa61025442613b16565b65ffffffffffff6102648461363e565b509179ffffffffffffffffffffffffffffffffffffffffffffffffffff6040519361028e85612279565b168352811660208084019182525f80805260018701909152604090209251905190911660d01b7fffffffffffff00000000000000000000000000000000000000000000000000001679ffffffffffffffffffffffffffffffffffffffffffffffffffff91909116179055565b79ffffffffffffffffffffffffffffffffffffffffffffffffffff841660d09290921b7fffffffffffff000000000000000000000000000000000000000000000000000016919091179055565b60405192835260208301526040820152a1005b346101b25760206003193601126101b25760043567ffffffffffffffff81116101b25761038e61042291369060040161230a565b905f6040805161039d8161225d565b606081526060602082015201526001600160a01b036104356103bf84846125ad565b610406604051956103cf8761225d565b6103d883612389565b87528460026103e960018601612389565b9460208a019586520154169560408801968752875151151561279d565b604051958695602087525160606020880152608087019061220c565b9051601f1986830301604087015261220c565b91511660608301520390f35b346101b25760406003193601126101b25761045a612231565b6001600160a01b036104a461046d612247565b926001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f52602052602060405f2054604051908152f35b346101b25760606003193601126101b25760043567ffffffffffffffff81116101b2576104eb90369060040161230a565b9060243567ffffffffffffffff81116101b25761050c90369060040161230a565b61051792919261291d565b61038467ffffffffffffffff4216019267ffffffffffffffff841161056957610544946044359333612fef565b5f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b346101b25760606003193601126101b25760043567ffffffffffffffff81116101b2576105c790369060040161230a565b9060243567ffffffffffffffff81116101b2576105e890369060040161230a565b9092906044356001600160a01b03811691908281036101b257610609612cd0565b8315610a4b578115610a235782156109fb57610624816136e4565b9081610989575b501561095d5760405161063d8161225d565b6106483685876122d4565b81526106553683886122d4565b60208201908152604082019084825261066e86886125ad565b925180519067ffffffffffffffff82116108925761068c8554612338565b601f811161092d575b50602090601f83116001146108ca576106c492915f91836108bf575b50505f198260011b9260031b1c19161790565b83555b518051600184019167ffffffffffffffff8211610892576106e88354612338565b601f8111610857575b50602090601f83116001146107bf576107a996947fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89b9694610754856001600160a01b039660029688965f926107b45750505f198260011b9260031b1c19161790565b90555b5116920191167fffffffffffffffffffffffff000000000000000000000000000000000000000082541617905561079b6040519687966060885260608801916125e5565b9185830360208701526125e5565b9060408301520390a1005b015190505f806106b1565b90601f19831691845f52815f20925f5b81811061083f5750946001856002957fa9dbac0cd4605f5b06a0dc7c0723ae2d582d6462ed81dcdc621910346f20ccb89f9a98956107a99c9a956001600160a01b03998a9810610827575b505050811b019055610757565b01515f1960f88460031b161c191690558f808061081a565b929360206001819287860151815501950193016107cf565b61088290845f5260205f20601f850160051c81019160208610610888575b601f0160051c01906127e6565b8a6106f1565b9091508190610875565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b015190508b806106b1565b90601f19831691865f52815f20925f5b81811061091557509084600195949392106108fd575b505050811b0183556106c7565b01515f1960f88460031b161c191690558a80806108f0565b929360206001819287860151815501950193016108da565b61095790865f5260205f20601f850160051c8101916020861061088857601f0160051c01906127e6565b8a610695565b507f3730169b000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b6020915060245f80927f01ffc9a70000000000000000000000000000000000000000000000000000000082527f56d981a700000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d1116816109f3575b508661062b565b9050866109ec565b7ff3a03283000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f9624aeb8000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fc8ca373e000000000000000000000000000000000000000000000000000000005f5260045ffd5b346101b25760206003193601126101b25760043567ffffffffffffffff81116101b257610ac57f142e2df85014c111942cda309ac57951014e6a19a066079496d1b6a84655775191369060040161230a565b610acd612cd0565b610b208183610adc82826125ad565b6001600160a01b03600260405192610af38461225d565b610afc81612389565b8452610b0a60018201612389565b602085015201541660408201525151151561279d565b5f6002610b2d83856125ad565b610b36816127fc565b610b42600182016127fc565b0155610b5b6040519283926020845260208401916125e5565b0390a1005b346101b2575f6003193601126101b257610bbf604051610b81604082612295565b600581527f352e302e30000000000000000000000000000000000000000000000000000000602082015260405191829160208352602083019061220c565b0390f35b346101b25760406003193601126101b257610be9610bdf612231565b6024359033612b94565b602060405160018152f35b346101b2575f6003193601126101b2576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454610c3381612338565b8084529060018116908115610cec5750600114610c6f575b610bbf83610c5b81850382612295565b60405191829160208352602083019061220c565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f9081527f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa939250905b808210610cd257509091508101602001610c5b610c4b565b919260018160209254838588010152019101909291610cba565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff001660208086019190915291151560051b84019091019150610c5b9050610c4b565b346101b25760406003193601126101b25760043567ffffffffffffffff81116101b257610d5f90369060040161230a565b6024359167ffffffffffffffff831683036101b257610dd86040935f60208651610d8881612279565b8281520152610d978484612575565b67ffffffffffffffff82165f52602052845f20936001865195610db987612279565b6001600160a01b03815416875201549360208601948086521515612605565b6001600160a01b0383519251168252516020820152f35b346101b25760406003193601126101b25760043580151581036101b2576024359067ffffffffffffffff82116101b25760c060031983360301126101b25761054491610e3961291d565b60040190612663565b346101b2575f6003193601126101b25760206001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c1993005416604051908152f35b346101b25760206003193601126101b25760043560028110156101b257610eb7610eb2602092613517565b61363e565b9050604051908152f35b346101b25760406003193601126101b2576101b0610edd612231565b60243590610eec823383612a78565b612d2f565b346101b25760206003193601126101b25760043560028110156101b257610f19604091613517565b5481519079ffffffffffffffffffffffffffffffffffffffffffffffffffff8116825260d01c6020820152f35b346101b2575f6003193601126101b257610f5e612cd0565b5f6001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c199300547fffffffffffffffffffffffff000000000000000000000000000000000000000081167f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930055167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e08280a3005b346101b25760806003193601126101b25760043567ffffffffffffffff81116101b25761102690369060040161230a565b9060243567ffffffffffffffff81116101b25761104790369060040161230a565b606435929167ffffffffffffffff841684036101b2576105449461106961291d565b6044359333612fef565b346101b25760206003193601126101b2576001600160a01b03611094612231565b165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00602052602060405f2054604051908152f35b346101b25760806003193601126101b2576110e3612231565b60243567ffffffffffffffff81116101b25761110390369060040161230a565b60449291923567ffffffffffffffff81116101b25761112690369060040161230a565b9390606435936001600160a01b0385168095036101b2577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549260ff8460401c16159467ffffffffffffffff85168015908161171e575b6001149081611714575b15908161170b575b506116e3576111fe61120593868860017fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000061120d9a16177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005561168e575b506111f66135e7565b6101ab6135e7565b36916122d4565b9436916122d4565b916112166135e7565b61121e6135e7565b6112266135e7565b835167ffffffffffffffff8111610892576112617f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0354612338565b601f8111611621575b50602094601f82116001146115805761129b9293949582915f926115755750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03555b825167ffffffffffffffff8111610892576112f97f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0454612338565b601f8111611508575b506020601f82116001146114675781906113319394955f9261145c5750505f198260011b9260031b1c19161790565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace04555b61135c6135e7565b7fffffffffffffffffffffffff00000000000000000000000000000000000000007f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416177f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a00556113c957005b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a1005b0151905085806106b1565b601f198216907f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f52805f20915f5b8181106114f0575095836001959697106114d8575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0455611354565b01515f1960f88460031b161c191690558480806114ab565b9192602060018192868b015181550194019201611496565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace045f5261156f907f46a2803e59a4de4e7a4c574b1243f25977ac4c77d5a1a4a609b5394cebb4a2aa601f840160051c8101916020851061088857601f0160051c01906127e6565b84611302565b0151905086806106b1565b601f198216957f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52805f20915f5b888110611609575083600195969798106115f1575b505050811b017f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03556112be565b01515f1960f88460031b161c191690558580806115c4565b919260206001819286850151815501940192016115af565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f52611688907f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab0601f840160051c8101916020851061088857601f0160051c01906127e6565b8561126a565b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00558a6111ed565b7ff92ee8a9000000000000000000000000000000000000000000000000000000005f5260045ffd5b9050158961118f565b303b159150611187565b87915061117d565b346101b25760206003193601126101b25760043567ffffffffffffffff81116101b257806004019060a060031982360301126101b25761179360449161178b6117826105449561177461291d565b61177c612e4a565b8061250f565b94909201612560565b9236916122d4565b612ea9565b346101b2575f6003193601126101b25760206001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051908152f35b346101b2575f6003193601126101b2576001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001630036118475760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b7fe07c8dba000000000000000000000000000000000000000000000000000000005f5260045ffd5b60406003193601126101b257611883612231565b60243567ffffffffffffffff81116101b257366023820112156101b2576118b49036906024816004013591016122d4565b906001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016803014908115611af2575b50611847576118f7612cd0565b6001600160a01b038116916040517f52d1902d000000000000000000000000000000000000000000000000000000008152602081600481875afa5f9181611abe575b5061196a57837f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b807f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc859203611a935750823b15611a6857807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a2805115611a37576101b091613990565b505034611a4057005b7fb398979f000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b7faa1d49a4000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b9091506020813d602011611aea575b81611ada60209383612295565b810103126101b257519085611939565b3d9150611acd565b90506001600160a01b037f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc54161415836118ea565b346101b25760206003193601126101b2576101b060043533612d2f565b346101b25760406003193601126101b2576101b0611b60612231565b611b68612cd0565b60243590612991565b346101b2575f6003193601126101b257602060405160128152f35b346101b25760606003193601126101b257610be9611ba8612231565b611bb0612247565b60443591611bbf833383612a78565b612b94565b346101b2575f6003193601126101b25760207f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254604051908152f35b346101b25760406003193601126101b257611c19612231565b60243590611c2561291d565b60245f6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a005416604051928380927fe57d7de00000000000000000000000000000000000000000000000000000000082523360048301525afa908115611f85575f91611ed0575b506020808251604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a01815203019020611d38604051611cdf8161225d565b611ce883612389565b81526001600160a01b036002611d0060018601612389565b94602084019586520154166040820152611d2081515115158551906124c9565b516020815191012083519081516020830120146124c9565b51805160208201206020830151908151602083012003611e7c57505060408101518051611e3a5750611d8f837f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb006137c2565b6137c2565b15611e1257816001600160a01b03611de292611dcc867f3af3114fdfc07ec4a9b7737970ccbbb6de9bc72e9b14c4b3d3b7958ef6eb6cae96612991565b519160405193849360408552604085019061220c565b95602084015216930390a25f7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d005b7fa74c1c5f000000000000000000000000000000000000000000000000000000005f5260045ffd5b611e78906040519182917f3ef1dc0200000000000000000000000000000000000000000000000000000000835260206004840152602483019061220c565b0390fd5b90611e78611ebe926040519384937ff39880b100000000000000000000000000000000000000000000000000000000855260406004860152604485019061220c565b9060031984830301602485015261220c565b90503d805f833e611ee18183612295565b8101906020818303126101b25780519067ffffffffffffffff82116101b257016060818303126101b25760405191611f188361225d565b815167ffffffffffffffff81116101b25781611f35918401612447565b8352602082015167ffffffffffffffff81116101b25781611f57918401612447565b6020840152604082015167ffffffffffffffff81116101b257611f7a9201612447565b604082015283611c93565b6040513d5f823e3d90fd5b346101b25760406003193601126101b257611fa9612231565b60243590331561205f576001600160a01b031690811561203357335f9081527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260409020825f526020528060405f20556040519081527f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560203392a3602060405160018152f35b7f94280d62000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7fe602df05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b346101b2575f6003193601126101b2576040515f7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace03546120ca81612338565b8084529060018116908115610cec57506001146120f157610bbf83610c5b81850382612295565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace035f9081527f2ae08a8e29253f69ac5d979a101956ab8f8d9d7ded63fa7a83b16fc47648eab0939250905b80821061215457509091508101602001610c5b610c4b565b91926001816020925483858801015201910190929161213c565b346101b25760206003193601126101b257600435907fffffffff0000000000000000000000000000000000000000000000000000000082168092036101b257817fd3ce6f1b00000000000000000000000000000000000000000000000000000000602093149081156121e2575b5015158152f35b7f01ffc9a700000000000000000000000000000000000000000000000000000000915014836121db565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b600435906001600160a01b03821682036101b257565b602435906001600160a01b03821682036101b257565b6060810190811067ffffffffffffffff82111761089257604052565b6040810190811067ffffffffffffffff82111761089257604052565b90601f601f19910116810190811067ffffffffffffffff82111761089257604052565b67ffffffffffffffff811161089257601f01601f191660200190565b9291926122e0826122b8565b916122ee6040519384612295565b8294818452818301116101b2578281602093845f960137010152565b9181601f840112156101b25782359167ffffffffffffffff83116101b257602083818601950101116101b257565b90600182811c9216801561237f575b602083101461235257565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f1691612347565b9060405191825f82549261239c84612338565b808452936001811690811561240757506001146123c3575b506123c192500383612295565b565b90505f9291925260205f20905f915b8183106123eb5750509060206123c1928201015f6123b4565b60209193508060019154838589010152019101909184926123d2565b602093506123c19592507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0091501682840152151560051b8201015f6123b4565b81601f820112156101b257602081519101612461826122b8565b9261246f6040519485612295565b828452828201116101b257815f926020928386015e8301015290565b60208091604051928184925191829101835e81017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b156124d15750565b611e78906040519182917f24a61b9500000000000000000000000000000000000000000000000000000000835260206004840152602483019061220c565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1813603018212156101b2570180359067ffffffffffffffff82116101b2576020019181360383136101b257565b3567ffffffffffffffff811681036101b25790565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0281520301902090565b60209082604051938492833781017f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0181520301902090565b601f8260209493601f1993818652868601375f8582860101520116010190565b9290921561261257505050565b67ffffffffffffffff6126586040519485947fa4e158af0000000000000000000000000000000000000000000000000000000086526040600487015260448601916125e5565b911660248301520390fd5b61266b612e4a565b15612787577f5a753d7102c7e00b9562e9ce9bc60bc17ac26a8509ee5f8061a0c5a5d46fc9286126a461269e838061250f565b90612575565b604083019067ffffffffffffffff6126bb83612560565b165f5260205261275160405f206001600160a01b036127496001604051936126e285612279565b8381541685520154966127126020850198808a52612700838061250f565b9061270a8a612560565b921515612605565b61271f61269e828061250f565b67ffffffffffffffff61273188612560565b165f526020525f60016040822082815501558061250f565b939094612560565b915116945167ffffffffffffffff6127766040519586956060875260608701916125e5565b9216602084015260408301520390a2565b60406117938261178b611782826123c19661250f565b919091156127a9575050565b611e786040519283927f24a61b950000000000000000000000000000000000000000000000000000000084526020600485015260248401916125e5565b8181106127f1575050565b5f81556001016127e6565b6128068154612338565b9081612810575050565b81601f5f9311600114612821575055565b8183526020832061283d91601f0160051c8101906001016127e6565b8082528160208120915555565b6001600160a01b031680156128f1576001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930054827fffffffffffffffffffffffff00000000000000000000000000000000000000008216177f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930055167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e05f80a3565b7f1e4fbdf7000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c6129695760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b6001600160a01b0316908115612a4c577fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef6020826129f15f947f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02546135da565b7f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace02558484527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260408420818154019055604051908152a3565b7fec442f05000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b9190612ab4836001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b6001600160a01b0382165f5260205260405f2054925f198410612ad8575b50505050565b828410612b57576001600160a01b0381161561205f576001600160a01b0382161561203357612b3f6001600160a01b03916001600160a01b03165f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0160205260405f2090565b91165f5260205260405f20910390555f808080612ad2565b506001600160a01b0383917ffb8f41b2000000000000000000000000000000000000000000000000000000005f521660045260245260445260645ffd5b6001600160a01b0316908115612ca4576001600160a01b0316918215612a4c57815f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054818110612c7257817fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92602092855f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0084520360405f2055845f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00825260405f20818154019055604051908152a3565b827fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b7f96c6fd1e000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b6001600160a01b037f9016d09d72d40fdae2fd8ceac6b6234c7706214fd39c1cd1e609a0528c19930054163303612d0357565b7f118cdaa7000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b9091906001600160a01b03168015612ca457805f527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0060205260405f2054838110612e17576020845f94957fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef938587527f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace008452036040862055807f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0254037f52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace0255604051908152a3565b91507fe450d38c000000000000000000000000000000000000000000000000000000005f5260045260245260445260645ffd5b6001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054163303612e7d57565b7f9a47cf30000000000000000000000000000000000000000000000000000000005f523360045260245ffd5b90612eb38261248b565b67ffffffffffffffff82165f5260205260405f20600160405191612ed683612279565b6001600160a01b038154168352015492602082019380855215612fa757612f1f8451611d8a7f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb0090565b15611e12577fef07437086457e782a927ae99e10d3c29643a0ef377870168f464d72ebd1a332926001600160a01b0383612f6182612776965116885190612991565b612f6a8461248b565b67ffffffffffffffff84165f526020525f60016040822082815501555116945167ffffffffffffffff60405194859460608652606086019061220c565b612658908367ffffffffffffffff6040519384937fa4e158af00000000000000000000000000000000000000000000000000000000855260406004860152604485019061220c565b959492909193948415610a4b5785156134ef5783156134c75767ffffffffffffffff16934285111561348d57613045847f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb026137c2565b15611e12576130548488612d2f565b6001600160a01b0361306682856125ad565b936130c5604051936130778561225d565b61308087612389565b855283600261309160018a01612389565b9860208801998a5201541692604086019384528551602081519101206130b83684846122d4565b602081519101201461279d565b5116925f60405180957f56d981a7000000000000000000000000000000000000000000000000000000008252604060048301528180613108604482018d8a6125e5565b8a602483015203915afa938415611f85575f9461344d575b506001600160a01b037f35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a0054169582519151966040519260c0840184811067ffffffffffffffff8211176108925760405283526020830197885260209788936040519361318c8686612295565b5f85526040820194855260608201988952608082019081526040516131b18782612295565b5f815260a08301908152604051998a96879586957f7e5a65660000000000000000000000000000000000000000000000000000000087528a6004880152516024870160c0905260e487016132049161220c565b9051908681037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc01604488015261323a9161220c565b9051908581037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160648701526132709161220c565b9051908481037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160848601526132a69161220c565b915167ffffffffffffffff1660a484015251908281037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc0160c48401526132ec9161220c565b03915a905f91f1928315611f85575f936133e3575b50917fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a956133d8926133c59695946001600160a01b036040519a6133448c612279565b16998a8152600188820188815261335b845161248b565b67ffffffffffffffff88165f528a526001600160a01b038060405f20945116167fffffffffffffffffffffffff000000000000000000000000000000000000000084541617835551910155519567ffffffffffffffff60405198899860808a5260808a019061220c565b94169087015285830360408701526125e5565b9060608301520390a2565b9091948094935081813d8311613446575b6133fe8183612295565b810103126101b257519067ffffffffffffffff821682036101b257929391929091907fd569da060650ae48d011d7f3fa8e52094f4a293ef7fdebe89f4b1aeea9fb685a613301565b503d6133f4565b9093503d805f833e61345f8183612295565b81016020828203126101b257815167ffffffffffffffff81116101b2576134869201612447565b925f613120565b847f0b8a29e4000000000000000000000000000000000000000000000000000000005f5260045267ffffffffffffffff421660245260445ffd5b7fcc2a76e5000000000000000000000000000000000000000000000000000000005f5260045ffd5b7f5de17177000000000000000000000000000000000000000000000000000000005f5260045ffd5b6002811015613588578061354a57507f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb0090565b90600182146135b557507fbd6a4b5d000000000000000000000000000000000000000000000000000000005f5260028110156135885760045260245ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602160045260245ffd5b7f74921a43196ad1dc887934f7da2d890232ee1cc23065a139ea8687a00466cb029150565b9190820180921161056957565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c161561361657565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b908154600179ffffffffffffffffffffffffffffffffffffffffffffffffffff8216935f80520160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff61369c42613b16565b9160d01c91160365ffffffffffff8111610569578465ffffffffffff6136d19360d01c60018082119118026001189216613a4a565b8103908111150291828103908111150290565b7f01ffc9a7000000000000000000000000000000000000000000000000000000005f527f01ffc9a70000000000000000000000000000000000000000000000000000000060045260205f60248184617530fa5f511515601f3d1116816137ba575b50156137b5575f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fffffffff00000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d1116816137af575090565b90501590565b505f90565b90505f613745565b9080156139895781549079ffffffffffffffffffffffffffffffffffffffffffffffffffff8216915f80526001840160205260405f20549079ffffffffffffffffffffffffffffffffffffffffffffffffffff82169165ffffffffffff61382842613b16565b9160d01c91160365ffffffffffff8111610569578465ffffffffffff61385d9360d01c60018082119118026001189216613a4a565b810390811115029182810390811115028111613982576138869061388042613b16565b926135da565b9179ffffffffffffffffffffffffffffffffffffffffffffffffffff83116139515761394c9291600165ffffffffffff9279ffffffffffffffffffffffffffffffffffffffffffffffffffff604051956138df87612279565b16855291831660208086019182525f80805292909301909252604090209251905190911660d01b7fffffffffffff00000000000000000000000000000000000000000000000000001679ffffffffffffffffffffffffffffffffffffffffffffffffffff91909116179055565b600190565b827f6dfcc650000000000000000000000000000000000000000000000000000000005f5260d060045260245260445ffd5b5050505f90565b5050600190565b905f8091602081519101845af48080613a37575b156139c45750506040513d81523d5f602083013e60203d82010160405290565b156139fe576001600160a01b03907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d15613a0f576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d1515806139a45750813b15156139a4565b90915f198383099280830292838086109503948086039514613adb5784831115613ac35790829109815f0382168092046002816003021880820260020302808202600203028082026002030280820260020302808202600203028091026002030293600183805f03040190848311900302920304170290565b82634e487b715f52156003026011186020526024601cfd5b505080925015613ae9570490565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b65ffffffffffff8111613b2e5765ffffffffffff1690565b7f6dfcc650000000000000000000000000000000000000000000000000000000005f52603060045260245260445ffdfea164736f6c634300081c000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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

// GetIFTRateLimit is a free data retrieval call binding the contract method 0x72b3449c.
//
// Solidity: function getIFTRateLimit(uint8 direction) view returns(uint208 capacity, uint48 window)
func (_Contract *ContractCaller) GetIFTRateLimit(opts *bind.CallOpts, direction uint8) (struct {
	Capacity *big.Int
	Window   *big.Int
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIFTRateLimit", direction)

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

// GetIFTRateLimit is a free data retrieval call binding the contract method 0x72b3449c.
//
// Solidity: function getIFTRateLimit(uint8 direction) view returns(uint208 capacity, uint48 window)
func (_Contract *ContractSession) GetIFTRateLimit(direction uint8) (struct {
	Capacity *big.Int
	Window   *big.Int
}, error) {
	return _Contract.Contract.GetIFTRateLimit(&_Contract.CallOpts, direction)
}

// GetIFTRateLimit is a free data retrieval call binding the contract method 0x72b3449c.
//
// Solidity: function getIFTRateLimit(uint8 direction) view returns(uint208 capacity, uint48 window)
func (_Contract *ContractCallerSession) GetIFTRateLimit(direction uint8) (struct {
	Capacity *big.Int
	Window   *big.Int
}, error) {
	return _Contract.Contract.GetIFTRateLimit(&_Contract.CallOpts, direction)
}

// GetIFTRateLimitAvailable is a free data retrieval call binding the contract method 0x86c13d64.
//
// Solidity: function getIFTRateLimitAvailable(uint8 direction) view returns(uint256)
func (_Contract *ContractCaller) GetIFTRateLimitAvailable(opts *bind.CallOpts, direction uint8) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIFTRateLimitAvailable", direction)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetIFTRateLimitAvailable is a free data retrieval call binding the contract method 0x86c13d64.
//
// Solidity: function getIFTRateLimitAvailable(uint8 direction) view returns(uint256)
func (_Contract *ContractSession) GetIFTRateLimitAvailable(direction uint8) (*big.Int, error) {
	return _Contract.Contract.GetIFTRateLimitAvailable(&_Contract.CallOpts, direction)
}

// GetIFTRateLimitAvailable is a free data retrieval call binding the contract method 0x86c13d64.
//
// Solidity: function getIFTRateLimitAvailable(uint8 direction) view returns(uint256)
func (_Contract *ContractCallerSession) GetIFTRateLimitAvailable(direction uint8) (*big.Int, error) {
	return _Contract.Contract.GetIFTRateLimitAvailable(&_Contract.CallOpts, direction)
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

// SetIFTRateLimit is a paid mutator transaction binding the contract method 0xe915ec0e.
//
// Solidity: function setIFTRateLimit(uint8 direction, uint208 capacity, uint48 window) returns()
func (_Contract *ContractTransactor) SetIFTRateLimit(opts *bind.TransactOpts, direction uint8, capacity *big.Int, window *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "setIFTRateLimit", direction, capacity, window)
}

// SetIFTRateLimit is a paid mutator transaction binding the contract method 0xe915ec0e.
//
// Solidity: function setIFTRateLimit(uint8 direction, uint208 capacity, uint48 window) returns()
func (_Contract *ContractSession) SetIFTRateLimit(direction uint8, capacity *big.Int, window *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.SetIFTRateLimit(&_Contract.TransactOpts, direction, capacity, window)
}

// SetIFTRateLimit is a paid mutator transaction binding the contract method 0xe915ec0e.
//
// Solidity: function setIFTRateLimit(uint8 direction, uint208 capacity, uint48 window) returns()
func (_Contract *ContractTransactorSession) SetIFTRateLimit(direction uint8, capacity *big.Int, window *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.SetIFTRateLimit(&_Contract.TransactOpts, direction, capacity, window)
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
	Direction uint8
	Capacity  *big.Int
	Window    *big.Int
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterIFTRateLimitSet is a free log retrieval operation binding the contract event 0xd56b392721db10260a8376f16903b557916e8e2a95fe49f84d79f1d0fd89946a.
//
// Solidity: event IFTRateLimitSet(uint8 direction, uint208 capacity, uint48 window)
func (_Contract *ContractFilterer) FilterIFTRateLimitSet(opts *bind.FilterOpts) (*ContractIFTRateLimitSetIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IFTRateLimitSet")
	if err != nil {
		return nil, err
	}
	return &ContractIFTRateLimitSetIterator{contract: _Contract.contract, event: "IFTRateLimitSet", logs: logs, sub: sub}, nil
}

// WatchIFTRateLimitSet is a free log subscription operation binding the contract event 0xd56b392721db10260a8376f16903b557916e8e2a95fe49f84d79f1d0fd89946a.
//
// Solidity: event IFTRateLimitSet(uint8 direction, uint208 capacity, uint48 window)
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

// ParseIFTRateLimitSet is a log parse operation binding the contract event 0xd56b392721db10260a8376f16903b557916e8e2a95fe49f84d79f1d0fd89946a.
//
// Solidity: event IFTRateLimitSet(uint8 direction, uint208 capacity, uint48 window)
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
