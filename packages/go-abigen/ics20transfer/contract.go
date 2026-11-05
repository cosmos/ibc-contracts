// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package ics20transfer

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

// IIBCAppCallbacksOnRecvPacketCallback is an auto generated low-level Go binding around an user-defined struct.
type IIBCAppCallbacksOnRecvPacketCallback struct {
	SourceClient      string
	DestinationClient string
	Sequence          uint64
	Payload           IICS26RouterMsgsPayload
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

// IICS20TransferMsgsSendTransferMsg is an auto generated low-level Go binding around an user-defined struct.
type IICS20TransferMsgsSendTransferMsg struct {
	Denom            common.Address
	Amount           *big.Int
	Receiver         string
	SourceClient     string
	DestPort         string
	TimeoutTimestamp uint64
	Memo             string
}

// IICS26RouterMsgsPayload is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsPayload struct {
	SourcePort string
	DestPort   string
	Version    string
	Encoding   string
	Value      []byte
}

// ISignatureTransferPermitTransferFrom is an auto generated low-level Go binding around an user-defined struct.
type ISignatureTransferPermitTransferFrom struct {
	Permitted ISignatureTransferTokenPermissions
	Nonce     *big.Int
	Deadline  *big.Int
}

// ISignatureTransferTokenPermissions is an auto generated low-level Go binding around an user-defined struct.
type ISignatureTransferTokenPermissions struct {
	Token  common.Address
	Amount *big.Int
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"UPGRADE_INTERFACE_VERSION\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"authority\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getEscrow\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getEscrowBeacon\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getIBCERC20Beacon\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getPermit2\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ibcERC20Contract\",\"inputs\":[{\"name\":\"denom\",\"type\":\"string\",\"internalType\":\"string\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ibcERC20Denom\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"internalType\":\"string\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ics26\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"ics26Router\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"escrowLogic\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"ibcERC20Logic\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"permit2\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"isConsumingScheduledOp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"multicall\",\"inputs\":[{\"name\":\"data\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"outputs\":[{\"name\":\"results\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onAcknowledgementPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnAcknowledgementPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"acknowledgement\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onRecvPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnRecvPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"onTimeoutPacket\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIIBCAppCallbacks.OnTimeoutPacketCallback\",\"components\":[{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destinationClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Payload\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"pause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"paused\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"proxiableUUID\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"sendTransfer\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS20TransferMsgs.SendTransferMsg\",\"components\":[{\"name\":\"denom\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"memo\",\"type\":\"string\",\"internalType\":\"string\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"sendTransferWithPermit2\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS20TransferMsgs.SendTransferMsg\",\"components\":[{\"name\":\"denom\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"memo\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"name\":\"permit\",\"type\":\"tuple\",\"internalType\":\"structISignatureTransfer.PermitTransferFrom\",\"components\":[{\"name\":\"permitted\",\"type\":\"tuple\",\"internalType\":\"structISignatureTransfer.TokenPermissions\",\"components\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"name\":\"nonce\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"deadline\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"name\":\"signature\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"sendTransferWithSender\",\"inputs\":[{\"name\":\"msg_\",\"type\":\"tuple\",\"internalType\":\"structIICS20TransferMsgs.SendTransferMsg\",\"components\":[{\"name\":\"denom\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"memo\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"name\":\"sender\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setAuthority\",\"inputs\":[{\"name\":\"newAuthority\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setCustomERC20\",\"inputs\":[{\"name\":\"denom\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"unpause\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeEscrowTo\",\"inputs\":[{\"name\":\"newEscrowLogic\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeIBCERC20To\",\"inputs\":[{\"name\":\"newIBCERC20Logic\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeToAndCall\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"event\",\"name\":\"AuthorityUpdated\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"IBCERC20ContractCreated\",\"inputs\":[{\"name\":\"contractAddress\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"fullDenomPath\",\"type\":\"string\",\"indexed\":false,\"internalType\":\"string\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Paused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Unpaused\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"indexed\":false,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AccessManagedInvalidAuthority\",\"inputs\":[{\"name\":\"authority\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AccessManagedRequiredDelay\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"delay\",\"type\":\"uint32\",\"internalType\":\"uint32\"}]},{\"type\":\"error\",\"name\":\"AccessManagedUnauthorized\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967InvalidImplementation\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ERC1967NonPayable\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"EnforcedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ExpectedPause\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ICS20DenomAlreadyExists\",\"inputs\":[{\"name\":\"denom\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20DenomNotFound\",\"inputs\":[{\"name\":\"denom\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20EscrowNotFound\",\"inputs\":[{\"name\":\"clientID\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20InvalidAddress\",\"inputs\":[{\"name\":\"addr\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20InvalidAmount\",\"inputs\":[{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ICS20InvalidPort\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20Permit2TokenMismatch\",\"inputs\":[{\"name\":\"permitToken\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"sentToken\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS20TokenAlreadyExists\",\"inputs\":[{\"name\":\"denom\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20Unauthorized\",\"inputs\":[{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS20UnauthorizedPacketSender\",\"inputs\":[{\"name\":\"packetSender\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS20UnexpectedERC20Balance\",\"inputs\":[{\"name\":\"expected\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"actual\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"ICS20UnexpectedEncoding\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS20UnexpectedVersion\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"SafeERC20FailedOperation\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"StringsInsufficientHexLength\",\"inputs\":[{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UUPSUnauthorizedCallContext\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UUPSUnsupportedProxiableUUID\",\"inputs\":[{\"name\":\"slot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]}]",
	Bin: "0x60a080604052346100c257306080525f5160206153055f395f51905f525460ff8160401c166100b3576002600160401b03196001600160401b03821601610060575b60405161523e90816100c7823960805181818161132701526113f10152f35b6001600160401b0319166001600160401b039081175f5160206153055f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80610041565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe60806040526004361015610011575f80fd5b5f5f3560e01c806306ab20bc14612909578063078c4a79146121115780631459457a14611c835780631bbf2e2314611c3d5780631e5150e414611bf75780632ac3dc3814611ba45780633f4ba83a14611ac5578063428e4e171461168f5780634f1ef2861461139f57806352d1902d1461130c57806353816a7c14610e8a5780635c975abb14610e485780635e32b6b614610bfe5780637a9e5e4b14610b40578063826cae7a14610afa5780638456cb5914610a415780638fb36037146109ac578063969631d514610930578063a1d28f571461066c578063a50ee2b4146105d1578063aaa2c3431461050c578063ac9650d814610356578063ad3cb1cc146102f5578063b29c715d14610258578063bf7e214f14610212578063d413227d146101cc5763e163b1af14610143575f80fd5b346101c95760206003193601126101c9576101c56101aa6101b161019e6101686129c7565b6001600160a01b03165f527f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80260205260405f2090565b60405192838092612f68565b0382612aa8565b604051918291602083526020830190612a3a565b0390f35b80fd5b50346101c957806003193601126101c95760206001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8035416604051908152f35b50346101c957806003193601126101c95760206001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416604051908152f35b50346101c95760406003193601126101c95760043567ffffffffffffffff81116102f15760e060031982360301126102f157906102bc6020926102996129dd565b906102a2613409565b6102aa613395565b6102b4363361312a565b600401613709565b907f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d67ffffffffffffffff60405191168152f35b5080fd5b50346101c957806003193601126101c957506101c5604051610318604082612aa8565b600581527f352e302e300000000000000000000000000000000000000000000000000000006020820152604051918291602083526020830190612a3a565b50346101c95760206003193601126101c95760043567ffffffffffffffff81116102f157366023820112156102f15780600401359067ffffffffffffffff821161050857602481013660248460051b840101116105045760405160206103bc8183612aa8565b85825280820192601f1982013685376103d4866130d1565b946103e26040519687612aa8565b868652601f196103f1886130d1565b0183895b8281106104f457505050875b878110156104775760019061045b61045561042460248460051b87010187612bbb565b91908d898c856040519687958487013784018281018481528e519283915e010190815203601f198101835282612aa8565b30614244565b610465828a6130e9565b5261047081896130e9565b5001610401565b83898860405191838301848452825180915260408401948060408360051b870101940192955b8287106104aa5785850386f35b9091929382806104e4837fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc08a600196030186528851612a3a565b960192019601959291909261049d565b606082828b0101520184906103f5565b8380fd5b8280fd5b50346101c95760206003193601126101c957806105276129c7565b610531363361312a565b6001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f805541690813b156105cd576001600160a01b03602484928360405195869485937f3659cfe60000000000000000000000000000000000000000000000000000000085521660048401525af180156105c2576105b15750f35b816105bb91612aa8565b6101c95780f35b6040513d84823e3d90fd5b5050fd5b50346101c95760206003193601126101c95760043567ffffffffffffffff81116102f157610603903690600401612b1d565b6001600160a01b036106158284612edf565b541690811561062957602082604051908152f35b90506106686040519283927fe1275e2f000000000000000000000000000000000000000000000000000000008452602060048501526024840191612c47565b0390fd5b50346101c95760406003193601126101c95760043567ffffffffffffffff81116102f15761069e903690600401612b1d565b6106a99291926129dd565b6106b3363361312a565b6001600160a01b036106c58386612edf565b54166108f45761070e610708826001600160a01b03165f527f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80260205260405f2090565b54612f17565b15610749826001600160a01b03165f527f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80260205260405f2090565b90156108b657506107c79061075e8386612edf565b6001600160a01b03808316167fffffffffffffffffffffffff00000000000000000000000000000000000000008254161790556001600160a01b03165f527f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80260205260405f2090565b9067ffffffffffffffff8111610889576107eb816107e58454612f17565b84613008565b82601f8211600114610829578190849561081994959261081e575b50505f198260011b9260031b1c19161790565b905580f35b013590505f80610806565b601f198216948385526020852091855b878110610871575083600195969710610858575b505050811b01905580f35b5f1960f88560031b161c199101351690555f808061084d565b90926020600181928686013581550194019101610839565b6024837f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b610668906040519182917f778769c4000000000000000000000000000000000000000000000000000000008352602060048401526024830190612f68565b6040517f0c0ef5340000000000000000000000000000000000000000000000000000000081526020600482015280610668602482018588612c47565b50346101c95760206003193601126101c95760043567ffffffffffffffff81116102f1576020918261096e6001600160a01b03933690600401612b1d565b925082604051938492833781017f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8008152030190205416604051908152f35b50346101c957806003193601126101c9577ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005460a01c60ff1615610a39575060207f8fb36037000000000000000000000000000000000000000000000000000000005b7fffffffff0000000000000000000000000000000000000000000000000000000060405191168152f35b602090610a0f565b50346101c957806003193601126101c957610a5c363361312a565b610a64613409565b60017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff007fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005416177fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a2586020604051338152a180f35b50346101c957806003193601126101c95760206001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8045416604051908152f35b50346101c95760206003193601126101c957610b5a6129c7565b6001600160a01b037ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054163303610bd257803b15610b9e57610b9b906141a6565b80f35b7fc2f31e5e0000000000000000000000000000000000000000000000000000000082526001600160a01b0316600452602490fd5b6024827f068ca9d800000000000000000000000000000000000000000000000000000000815233600452fd5b50346101c95780610c0e36612a07565b610c44336001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80354163314612b4b565b610c4c613395565b610c54613409565b60608101610ca3610c7d610c75610c6b8486612b88565b6080810190612bbb565b810190612d64565b610c90610c8a8486612b88565b80612bbb565b90610c9b8680612bbb565b929091613abd565b9050610cae81614720565b80610e2a575b610ce1575b5050507f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d80f35b6001600160a01b031691823b15610e2557610df492610ded8580946080946001600160a01b03604051988997889687957f5e32b6b600000000000000000000000000000000000000000000000000000000875260206004880152610dbe610d9c610d5f610d4e8980613991565b60a060248d015260c48c0191612c47565b610d6c60208a018a613991565b907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc8c84030160448d0152612c47565b9167ffffffffffffffff610db260408a016139e1565b1660648a0152876139f6565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc888303016084890152613a28565b93016129f3565b1660a483015203925af180156105c257610e10575b8080610cb9565b81610e1a91612aa8565b6101c957805f610e09565b505050fd5b50610e34816147fe565b81610e40575b50610cb4565b90505f610e3a565b50346101c957806003193601126101c957602060ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f0330054166040519015158152f35b50346101c95760c06003193601126101c95760043567ffffffffffffffff81116102f157806004019060e060031982360301126105085760807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc3601126105085760a43567ffffffffffffffff811161050457610f0b903690600401612b1d565b610f16929192613409565b610f1e613395565b60248201359182156112e057610f32612eb5565b6001600160a01b0380610f4488612ecb565b16911614610f50612eb5565b610f5987612ecb565b91156112a6575050610f88610f83610f7c60646001600160a01b03940188612bbb565b3691612ae7565b6134d0565b16926001600160a01b03610f9b86612ecb565b1691604051917f70a08231000000000000000000000000000000000000000000000000000000008352856004840152602083602481875afa92831561129b578893611267575b506001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8065416604051906040820182811067ffffffffffffffff82111761123a57908a949392916040528882526020820191888352813b1561123657856001600160a01b03916110cc8296604051988997889687957f30f28b7a000000000000000000000000000000000000000000000000000000008752816110886129dd565b166004880152604435602488015260643560448801526084356064880152511660848601525160a48501523360c485015261010060e4850152610104840191612c47565b03925af180156105c257611221575b50506020602492604051938480927f70a082310000000000000000000000000000000000000000000000000000000082528860048301525afa9081156112165786916111dc575b61113e92506111318482613ce0565b908211806111d357613ced565b8361114884612ecb565b91833b156102f1576040517fb4f22eb70000000000000000000000000000000000000000000000000000000081526001600160a01b0393909316600484015233602484015260448301528160648183865af180156111c8576111b3575b6020846102bc338587613d24565b6111be848092612aa8565b610508575f6111a5565b6040513d86823e3d90fd5b50808214613ced565b90506020823d60201161120e575b816111f760209383612aa8565b8101031261120a5761113e915190611122565b5f80fd5b3d91506111ea565b6040513d88823e3d90fd5b8161122b91612aa8565b61123657855f6110db565b8580fd5b60248b7f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b9092506020813d602011611293575b8161128360209383612aa8565b8101031261120a5751915f610fe1565b3d9150611276565b6040513d8a823e3d90fd5b7fe36164780000000000000000000000000000000000000000000000000000000088526001600160a01b0390811660045216602452604486fd5b6024867f4f6df8d000000000000000000000000000000000000000000000000000000000815280600452fd5b50346101c957806003193601126101c9576001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001630036113775760206040517f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc8152f35b807fe07c8dba0000000000000000000000000000000000000000000000000000000060049252fd5b5060406003193601126101c9576113b46129c7565b9060243567ffffffffffffffff81116102f157366023820112156102f1576113e6903690602481600401359101612ae7565b916001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001680301490811561165a575b506116325761142b363361312a565b6001600160a01b03811690604051937f52d1902d000000000000000000000000000000000000000000000000000000008552602085600481865afa809585966115fa575b506114a057602484847f4c9c8ce3000000000000000000000000000000000000000000000000000000008252600452fd5b9091847f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc81036115cf5750823b156115a457807fffffffffffffffffffffffff00000000000000000000000000000000000000007f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc5416177f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b8480a28051156115725761156e91614244565b5080f35b50503461157c5780f35b807fb398979f0000000000000000000000000000000000000000000000000000000060049252fd5b7f4c9c8ce3000000000000000000000000000000000000000000000000000000008452600452602483fd5b7faa1d49a4000000000000000000000000000000000000000000000000000000008552600452602484fd5b9095506020813d60201161162a575b8161161660209383612aa8565b810103126116265751945f61146f565b8480fd5b3d9150611609565b6004827fe07c8dba000000000000000000000000000000000000000000000000000000008152fd5b90506001600160a01b037f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc541614155f61141c565b50346101c95760206003193601126101c9578060043567ffffffffffffffff8111611ac2578060040160c060031983360301126105cd576116fc336001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80354163314612b4b565b611704613395565b61170c613409565b60648201611720610c75610c6b8385612b88565b916084840190611733610f7c8383612bbb565b6020815191012093604094602080875161174d8982612aa8565b818152017f4774d4a575993f963b1c06573736617a457abef8589178db8d10c94b4ab511ab815220145f1461193e576117989061178d610c8a8685612b88565b90610c9b8580612bbb565b90506117a381614720565b80611920575b6117dc575b505050505050505b807f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d80f35b6001600160a01b031690813b1561191c578680948651978895869485937f8dfcd9ad0000000000000000000000000000000000000000000000000000000085528560048601528a60248601526118328280613991565b6044870160c0905261010487019061184992612c47565b6118566024860184613991565b60431988840301606489015261186c9291612c47565b90611879604486016139e1565b67ffffffffffffffff16608487015261189290836139f6565b908581036043190160a48701526118a891613a28565b916118b291613991565b6043198584030160c48601526118c89291612c47565b9060a4016118d5906129f3565b6001600160a01b031660e483015203925af190811561191357506118fe575b80808080806117ae565b8161190891612aa8565b6101c957805f6118f4565b513d84823e3d90fd5b8680fd5b5061192a816147fe565b81611936575b506117a9565b90505f611930565b602061194b91015161345c565b61195481614720565b80611aa4575b61196b575b505050505050506117b6565b6001600160a01b031690813b1561191c578680948651978895869485937f8dfcd9ad00000000000000000000000000000000000000000000000000000000855260048501600190528a60248601526119c38280613991565b6044870160c090526101048701906119da92612c47565b6119e76024860184613991565b6043198884030160648901526119fd9291612c47565b90611a0a604486016139e1565b67ffffffffffffffff166084870152611a2390836139f6565b908581036043190160a4870152611a3991613a28565b91611a4391613991565b6043198584030160c4860152611a599291612c47565b9060a401611a66906129f3565b6001600160a01b031660e483015203925af19081156119135750611a8f575b808080808061195f565b81611a9991612aa8565b6101c957805f611a85565b50611aae816147fe565b81611aba575b5061195a565b90505f611ab4565b50fd5b50346101c957806003193601126101c957611ae0363361312a565b7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f033005460ff811615611b7c577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00167fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300557f5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa6020604051338152a180f35b6004827f8dfc202b000000000000000000000000000000000000000000000000000000008152fd5b50346101c95760206003193601126101c95760043567ffffffffffffffff81116102f15760e060031982360301126102f157906102bc602092611be5613409565b611bed613395565b3390600401613709565b50346101c957806003193601126101c95760206001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8055416604051908152f35b50346101c957806003193601126101c95760206001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8065416604051908152f35b50346101c95760a06003193601126101c957611c9d6129c7565b611ca56129dd565b6044356001600160a01b038116810361050457606435926001600160a01b038416809403611626576084356001600160a01b0381168103611236577ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005467ffffffffffffffff8116806120e95760ff8260401c169081156120dd575b506120b5576001600160a01b039291680100000000000000027fffffffffffffffffffffffffffffffffffffffffffffff000000000000000000611da89316177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0055611d8b614406565b611d93614406565b611d9b614406565b611da3614406565b6141a6565b167fffffffffffffffffffffffff00000000000000000000000000000000000000007f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8035416177f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80355604051916105129283810181811067ffffffffffffffff82111761207d57611e52829161485e95878785396001600160a01b0316815230602082015260400190565b039086f080156120aa576001600160a01b03167fffffffffffffffffffffffff00000000000000000000000000000000000000007f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8045416177f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80455604051928084019284841067ffffffffffffffff85111761207d5791849391611f089385396001600160a01b0316815230602082015260400190565b039083f080156105c2576001600160a01b03167fffffffffffffffffffffffff00000000000000000000000000000000000000007f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8055416177f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f805557fffffffffffffffffffffffff00000000000000000000000000000000000000007f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8065416177f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f806557fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a0054167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160028152a180f35b6024877f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b6040513d87823e3d90fd5b6004877ff92ee8a9000000000000000000000000000000000000000000000000000000008152fd5b6002915010155f611d21565b6004887ff92ee8a9000000000000000000000000000000000000000000000000000000008152fd5b50346101c95761212036612a07565b90612157336001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80354163314612b4b565b61215f613395565b612167613409565b6060820191612186610f7c61217c8584612b88565b6040810190612bbb565b60208151910120612195612c0c565b60208151910120146121a5612c0c565b6121b261217c8685612b88565b9092156128d3575050506122046121cf610f7c610c8a8685612b88565b602081519101206121de612c8f565b60208151910120146121ee612c8f565b906121fc610c8a8786612b88565b929091612cca565b61221e610f7c6122148584612b88565b6060810190612bbb565b6020815191012061222d612d0e565b602081519101201461223d612d0e565b61224a6122148685612b88565b90921561289d5750505061229e612271610f7c6122678685612b88565b6020810190612bbb565b60208151910120612280612c8f565b6020815191012014612290612c8f565b906121fc6122678786612b88565b6122ae610c75610c6b8584612b88565b9260608401805115612871576122c7604086015161345c565b9160208401936122dd610f83610f7c8784612bbb565b96519461230c6122f0610c8a8585612b88565b6123076122fd8680612bbb565b9390923691612ae7565b61366a565b9261231784886142fe565b156124a457505090518451959687959092509060209087811890881102871880612349612344828b613066565b6130a0565b9803920101602087015e61235e855186614350565b959015868115612492575b50612469575b506001600160a01b03905b16905193813b15610504576040517f0779afe60000000000000000000000000000000000000000000000000000000081526001600160a01b039182166004820152921660248301526044820193909352918290606490829084905af180156105c257612454575b6101c582604051906123f4604083612aa8565b601182527f7b22726573756c74223a2241513d3d227d00000000000000000000000000000060208301527f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d604051918291602083526020830190612a3a565b61245f828092612aa8565b6101c9575f6123e1565b94506001600160a01b039061248c8261248188612e31565b541696871515612e6f565b9061236f565b6001600160a01b03915016155f612369565b61250e93506124d98360209894936123076122fd6124d16124c88d98978998612b88565b87810190612bbb565b939094612bbb565b926040519784899551918291018487015e8401908282018a8152815193849201905e010186815203601f198101855284612aa8565b6001600160a01b038516946001600160a01b0361252a85612e31565b54169384156125cb575b5084956001600160a01b038596959616908351823b1561191c576040517f40c10f190000000000000000000000000000000000000000000000000000000081526001600160a01b0392909216600483015260248201529085908290604490829084905af19081156120aa5785916125b6575b50506001600160a01b039061237a565b816125c091612aa8565b61050457835f6125a6565b93506001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80454166040517f4571e3a600000000000000000000000000000000000000000000000000000000602082015230602482015287604482015260606064820152612653816126456084820189612a3a565b03601f198101835282612aa8565b604051916104c28084019084821067ffffffffffffffff831117612844579184939161268393614d7086396134b0565b039086f080156120aa576001600160a01b0316956126a085612e31565b6001600160a01b0388167fffffffffffffffffffffffff000000000000000000000000000000000000000082541617905561270b876001600160a01b03165f527f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80260205260405f2090565b9685519767ffffffffffffffff8911612817576127328961272c8354612f17565b83613008565b602098601f81116001146127b55780612762918a9b8b9a9b916127aa575b505f198260011b9260031b1c19161790565b90555b7f6031fab685dd6d86e4dbac9a69eae347145f332c95b3a0d728d3730fc5233d6261279f8298604051918291602083526020830190612a3a565b0390a2959493612534565b90508a01515f612750565b818952898920601f1982168a5b8181106127ff5750908a9b83600194939c9b9c106127e7575b5050811b019055612765565b8b01515f1960f88460031b161c191690555f806127db565b8a8d0151835560209c8d019c600190930192016127c2565b6024887f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b60248a7f4e487b710000000000000000000000000000000000000000000000000000000081526041600452fd5b6024847f4f6df8d000000000000000000000000000000000000000000000000000000000815280600452fd5b610668906040519384937fd1ca953a00000000000000000000000000000000000000000000000000000000855260048501612c67565b610668906040519384937f094af3b800000000000000000000000000000000000000000000000000000000855260048501612c67565b503461120a57602060031936011261120a576129236129c7565b61292d363361312a565b6001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f804541690813b1561120a576001600160a01b0360245f928360405195869485937f3659cfe60000000000000000000000000000000000000000000000000000000085521660048401525af180156129bc576129ae575080f35b6129ba91505f90612aa8565b005b6040513d5f823e3d90fd5b600435906001600160a01b038216820361120a57565b602435906001600160a01b038216820361120a57565b35906001600160a01b038216820361120a57565b602060031982011261120a576004359067ffffffffffffffff821161120a576003198260a09203011261120a5760040190565b90601f19601f602080948051918291828752018686015e5f8582860101520116010190565b60a0810190811067ffffffffffffffff821117612a7b57604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b90601f601f19910116810190811067ffffffffffffffff821117612a7b57604052565b67ffffffffffffffff8111612a7b57601f01601f191660200190565b929192612af382612acb565b91612b016040519384612aa8565b82948184528183011161120a578281602093845f960137010152565b9181601f8401121561120a5782359167ffffffffffffffff831161120a576020838186019501011161120a57565b15612b535750565b6001600160a01b03907f2ecb3242000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff618136030182121561120a570190565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18136030182121561120a570180359067ffffffffffffffff821161120a5760200191813603831361120a57565b60405190612c1b604083612aa8565b600782527f69637332302d31000000000000000000000000000000000000000000000000006020830152565b601f8260209493601f1993818652868601375f8582860101520116010190565b91612c7e612c8c9492604085526040850190612a3a565b926020818503910152612c47565b90565b60405190612c9e604083612aa8565b600882527f7472616e736665720000000000000000000000000000000000000000000000006020830152565b9290919215612cd857505050565b610668906040519384937f5d3a3cdd00000000000000000000000000000000000000000000000000000000855260048501612c67565b60405190612d1d604083612aa8565b601a82527f6170706c69636174696f6e2f782d736f6c69646974792d6162690000000000006020830152565b9080601f8301121561120a57816020612c8c93359101612ae7565b60208183031261120a5780359067ffffffffffffffff821161120a570160a08183031261120a5760405191612d9883612a5f565b813567ffffffffffffffff811161120a5781612db5918401612d49565b8352602082013567ffffffffffffffff811161120a5781612dd7918401612d49565b6020840152604082013567ffffffffffffffff811161120a5781612dfc918401612d49565b604084015260608201356060840152608082013567ffffffffffffffff811161120a57612e299201612d49565b608082015290565b60208091604051928184925191829101835e81017f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80181520301902090565b15612e775750565b610668906040519182917fe1275e2f000000000000000000000000000000000000000000000000000000008352602060048401526024830190612a3a565b6024356001600160a01b038116810361120a5790565b356001600160a01b038116810361120a5790565b60209082604051938492833781017f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f80181520301902090565b90600182811c92168015612f5e575b6020831014612f3157565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b91607f1691612f26565b5f9291815491612f7783612f17565b8083529260018116908115612fcc5750600114612f9357505050565b5f9081526020812093945091925b838310612fb2575060209250010190565b600181602092949394548385870101520191019190612fa1565b905060209495507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0091509291921683830152151560051b010190565b919091601f8311613019575b505050565b81831161302557505050565b5f5260205f206020601f830160051c921061305e575b81601f9101920160051c03905f5b82811015613014575f82820155600101613049565b5f915061303b565b9190820391821161307357565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b906130aa82612acb565b6130b76040519182612aa8565b828152601f196130c78294612acb565b0190602036910137565b67ffffffffffffffff8111612a7b5760051b60200190565b80518210156130fd5760209160051b010190565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054916001600160a01b038316928160041161120a575f5f9060405f8151966001600160a01b0360208901917fb700961300000000000000000000000000000000000000000000000000000000835216978860248201523060448201527fffffffff000000000000000000000000000000000000000000000000000000008335166064820152606481526131df608482612aa8565b828052826020525190895afa613382575b156131fd575b5050505050565b63ffffffff1615613356577fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674010000000000000000000000000000000000000000177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055823b1561120a576020925f92836040518096819582947f94c7d7ee000000000000000000000000000000000000000000000000000000008452600484015260406024840152601f19601f6044850192808452808786860137868582860101520116010103925af180156129bc57613346575b507fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff7ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0054167ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00555f808080806131f6565b5f61335091612aa8565b5f6132d5565b827f068ca9d8000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b50505f516020518060201c1502906131f0565b7f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005c6133e15760017f9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f005d565b7f3ee5aeb5000000000000000000000000000000000000000000000000000000005f5260045ffd5b60ff7fcd5ed15c6e187e77e9aee88184c21f4f2182ab5827cb3b7e07fbedcd63f03300541661343457565b7fd93c0665000000000000000000000000000000000000000000000000000000005f5260045ffd5b613467815182614350565b919015613472575090565b610668906040519182917f3fed5d87000000000000000000000000000000000000000000000000000000008352602060048401526024830190612a3a565b6040906001600160a01b03612c8c94931681528160208201520190612a3a565b604051906001600160a01b03815192602081818501958087835e81017f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f800815203019020541691821561352157505090565b7f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f805547ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a00546040517f485cc9550000000000000000000000000000000000000000000000000000000060208201523060248201526001600160a01b039182166044808301919091528152939450919291166135bc606483612aa8565b604051916104c2908184019284841067ffffffffffffffff851117612a7b5784936135eb93614d7086396134b0565b03905ff080156129bc576001600160a01b036020911692604051928391518091835e81017f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8008152030190206001600160a01b0382167fffffffffffffffffffffffff000000000000000000000000000000000000000082541617905590565b6001809160208095612c8c958160405198858a9651918291018688015e8501917f2f0000000000000000000000000000000000000000000000000000000000000085840152602183013701017f2f000000000000000000000000000000000000000000000000000000000000008382015203017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1810184520182612aa8565b906020820135918215613965576001600160a01b03613731610f83610f7c6060850185612bbb565b16906001600160a01b0361374482612ecb565b166040517f70a08231000000000000000000000000000000000000000000000000000000008152836004820152602081602481855afa9081156129bc575f91613933575b50604051917f23b872dd000000000000000000000000000000000000000000000000000000005f5233600452846024528660445260205f60648180855af160015f5114811615613914575b836040525f606052156138e957826024816020937f70a082310000000000000000000000000000000000000000000000000000000082528860048301525afa9081156129bc575f916138b3575b61382f92506111318782613ce0565b61383881612ecb565b823b1561120a576040517fb4f22eb70000000000000000000000000000000000000000000000000000000081526001600160a01b0391909116600482015233602482015260448101949094525f8460648183865af19384156129bc57612c8c946138a3575b50613d24565b5f6138ad91612aa8565b5f61389d565b90506020823d6020116138e1575b816138ce60209383612aa8565b8101031261120a5761382f915190613820565b3d91506138c1565b7f5274afe7000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b600181151661392a57813b15153d1516166137d3565b833d5f823e3d90fd5b90506020813d60201161395d575b8161394e60209383612aa8565b8101031261120a57515f613788565b3d9150613941565b7f4f6df8d0000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b90357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18236030181121561120a57016020813591019167ffffffffffffffff821161120a57813603831361120a57565b359067ffffffffffffffff8216820361120a57565b90357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff618236030181121561120a570190565b612c8c91613aaf613aa4613a89613a6e613a53613a458780613991565b60a0885260a0880191612c47565b613a606020880188613991565b908783036020890152612c47565b613a7b6040870187613991565b908683036040880152612c47565b613a966060860186613991565b908583036060870152612c47565b926080810190613991565b916080818503910152612c47565b909493925f926001600160a01b03604051838382376020818581017f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f8008152030190205416928315613ca15791613b2991612307613b3094613b2160208a015161345c565b9a3691612ae7565b84516142fe565b15613c51576001600160a01b03613b478451612e31565b541692613b578151851515612e6f565b6060810151843b1561120a576040517f40c10f190000000000000000000000000000000000000000000000000000000081526001600160a01b038416600482015260248101919091525f8160448183895af180156129bc57613c3b575b506060905b0151813b15610508576040517f0779afe60000000000000000000000000000000000000000000000000000000081526001600160a01b0385811660048301528716602482015260448101919091529082908290606490829084905af180156105c257613c26575b50509190565b613c31828092612aa8565b6101c95780613c20565b613c489193505f90612aa8565b5f916060613bb4565b6001600160a01b03613c638451612e31565b5416928315613c75575b606090613bb9565b92506060613c83845161345c565b93613c9a81516001600160a01b0387161515612e6f565b9050613c6d565b50906106686040519283927f5778f378000000000000000000000000000000000000000000000000000000008452602060048501526024840191612c47565b9190820180921161307357565b15613cf6575050565b7f2fb30cfc000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b613d45613d3361016883612ecb565b92613d4c5f9460405193848092612f68565b0383612aa8565b818051155f146140e15750505060a0613d75613d6f613d6a84612ecb565b61445d565b9461445d565b93613d836040840184612bbb565b939095613dd4613db9613d9960c0850185612bbb565b99909760405196613da988612a5f565b8752602087019485523691612ae7565b9560408501968752606085019860208501358a523691612ae7565b94608084019586526001600160a01b037f823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f803541695613e156060850185612bbb565b96909401359467ffffffffffffffff86168096036140dd5790613f29613e82949392613f1b613e42612c8f565b9c613e4b612c8f565b94613ee4613e57612c0c565b97613eb3613e63612d0e565b9a6040519c8d986020808b01525160a060408b015260e08a0190612a3a565b90517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc08983030160608a0152612a3a565b90517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc0878303016080880152612a3a565b915160a0850152517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc08483030160c0850152612a3a565b03601f198101865285612aa8565b60405199613f368b612a5f565b8a5260208a0152604089015260608801526080870152604051926060840184811067ffffffffffffffff82111761207d579361407760209694613f8c613fe0958a9567ffffffffffffffff996040523691612ae7565b835287830190815260408301998a52604051998a97889687957f4d6e7ce30000000000000000000000000000000000000000000000000000000087528b600488015251606060248801526084870190612a3a565b925116604485015251907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdc8482030160648501526080614066614054614042614032865160a0875260a0870190612a3a565b8d8701518682038f880152612a3a565b60408601518582036040870152612a3a565b60608501518482036060860152612a3a565b920151906080818403910152612a3a565b03925af19182156140d057819261408d57505090565b9091506020813d6020116140c8575b816140a960209383612aa8565b810103126102f157519067ffffffffffffffff821682036101c9575090565b3d915061409c565b50604051903d90823e3d90fd5b8880fd5b61410c906141066140f3979497612c8f565b6141006060880188612bbb565b9161366a565b906142fe565b61411d575b50613d7560a09161445d565b6001600160a01b0361412e84612ecb565b16803b1561120a576040517f9dc29fac0000000000000000000000000000000000000000000000000000000081526001600160a01b03929092166004830152602084013560248301525f908290604490829084905af180156129bc57156141115761419c9193505f90612aa8565b5f91613d75614111565b60206001600160a01b037f2f658b440c35314f52658ea8a740e05b284cdc84dc9ae01e891f21b8933e7cad9216807fffffffffffffffffffffffff00000000000000000000000000000000000000007ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a005416177ff3177357ab46d8af007ab3fdb9af81da189e1068fefdc0073dca88a2cab40a0055604051908152a1565b905f8091602081519101845af480806142eb575b156142785750506040513d81523d5f602083013e60203d82010160405290565b156142b2576001600160a01b03907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b3d156142c3576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b503d1515806142585750813b1515614258565b805190825180921061434957805191828082109118028083189214158202821890602061432e6123448486613066565b92808285019503920101835e51902090602081519101201490565b5050505f90565b8051821180156143ff575b6143a85760018211806143b0575b158015908160011b918204600214171561307357602801806028116130735782036143a8576001600160a01b0392915f6143a29261454c565b90921690565b50505f905f90565b507f30780000000000000000000000000000000000000000000000000000000000007fffff00000000000000000000000000000000000000000000000000000000000060208301511614614369565b505f61435b565b60ff7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460401c161561443557565b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b6001600160a01b031680614471602a612acb565b9161447f6040519384612aa8565b602a835261448d602a612acb565b601f196020850191013682378351156130fd57603090538251600110156130fd576078602184015360295b600181116144f957506144c9575090565b7fe22e27eb000000000000000000000000000000000000000000000000000000005f52600452601460245260445ffd5b90600f811660108110156130fd5784518310156130fd577f3031323334353637383961626364656600000000000000000000000000000000901a8483016020015360041c908015613073575f19016144b8565b9290926001840180851161307357831180614603575b158015908160011b918204600214171561307357614585905f9492939495613ce0565b915b8183106145975750505060019190565b9092919360ff6145ce7fff000000000000000000000000000000000000000000000000000000000000006020888601015116614654565b16600f81116145f8578160041b918083046010149015171561307357600191019401919290614587565b505f94508493505050565b507f30780000000000000000000000000000000000000000000000000000000000007fffff000000000000000000000000000000000000000000000000000000000000602086840101511614614562565b60f81c602f811180614716575b1561468e577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd00160ff1690565b606081118061470c575b156146c5577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa90160ff1690565b6040811180614702575b156146fc577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc90160ff1690565b5060ff90565b50604781106146cf565b5060678110614698565b50603a8110614661565b7f01ffc9a7000000000000000000000000000000000000000000000000000000005f527f01ffc9a70000000000000000000000000000000000000000000000000000000060045260205f60248184617530fa5f511515601f3d1116816147f6575b50156147f1575f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fffffffff00000000000000000000000000000000000000000000000000000000600452617530fa5f511515601f3d1116816147eb575090565b90501590565b505f90565b90505f614781565b5f6024816020937f01ffc9a70000000000000000000000000000000000000000000000000000000082527fd3ce6f1b00000000000000000000000000000000000000000000000000000000600452617530fa905f511515601f3d11169056fe60803461013457601f61051238819003918201601f19168301916001600160401b03831184841017610138578084926040948552833981010312610134576100468161014c565b906001600160a01b039061005c9060200161014c565b16908115610121575f80546001600160a01b031981168417825560405193916001600160a01b03909116907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09080a3803b1561010157600180546001600160a01b0319166001600160a01b039290921691821790557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a26103b190816101618239f35b63211eb15960e21b5f9081526001600160a01b0391909116600452602490fd5b631e4fbdf760e01b5f525f60045260245ffd5b5f80fd5b634e487b7160e01b5f52604160045260245ffd5b51906001600160a01b03821682036101345756fe60806040526004361015610011575f80fd5b5f3560e01c80633659cfe61461027e5780635c60da1b1461022d578063715018a6146101935780638da5cb5b146101435763f2fde38b14610050575f80fd5b3461013f5760207ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc36011261013f5760043573ffffffffffffffffffffffffffffffffffffffff811680910361013f576100a8610358565b80156101135773ffffffffffffffffffffffffffffffffffffffff5f54827fffffffffffffffffffffffff00000000000000000000000000000000000000008216175f55167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e05f80a3005b7f1e4fbdf7000000000000000000000000000000000000000000000000000000005f525f60045260245ffd5b5f80fd5b3461013f575f7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc36011261013f57602073ffffffffffffffffffffffffffffffffffffffff5f5416604051908152f35b3461013f575f7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc36011261013f576101c9610358565b5f73ffffffffffffffffffffffffffffffffffffffff81547fffffffffffffffffffffffff000000000000000000000000000000000000000081168355167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e08280a3005b3461013f575f7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc36011261013f57602073ffffffffffffffffffffffffffffffffffffffff60015416604051908152f35b3461013f5760207ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc36011261013f5760043573ffffffffffffffffffffffffffffffffffffffff81169081810361013f576102d7610358565b3b1561032d57807fffffffffffffffffffffffff000000000000000000000000000000000000000060015416176001557fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b5f80a2005b7f847ac564000000000000000000000000000000000000000000000000000000005f5260045260245ffd5b73ffffffffffffffffffffffffffffffffffffffff5f5416330361037857565b7f118cdaa7000000000000000000000000000000000000000000000000000000005f523360045260245ffdfea164736f6c6343000825000a60a0806040526104c280380380916100178285610271565b833981016040828203126101b45761002e82610294565b602083015190926001600160401b0382116101b4570181601f820112156101b4578051906001600160401b03821161025d5760405192610078601f8401601f191660200185610271565b828452602083830101116101b457815f9260208093018386015e83010152813b1561023c577fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d5080546001600160a01b0319166001600160a01b038416908117909155604051635c60da1b60e01b8152909190602081600481865afa9081156101c0575f91610202575b50803b156101e25750817f1cf3b03a6cf19fa2baba4df148e9dcabedea7f8a5c07840e207e5c089be95d3e5f80a28051156101cb57602060049260405193848092635c60da1b60e01b82525afa80156101c0575f90610181575b61016592506102a8565b505b60805260405161018d908161033582396080518160460152f35b506020823d6020116101b8575b8161019b60209383610271565b810103126101b4576101af61016592610294565b61015b565b5f80fd5b3d915061018e565b6040513d5f823e3d90fd5b505034156101675763b398979f60e01b5f5260045ffd5b634c9c8ce360e01b5f9081526001600160a01b0391909116600452602490fd5b90506020813d602011610234575b8161021d60209383610271565b810103126101b45761022e90610294565b5f610101565b3d9150610210565b50631933b43b60e21b5f9081526001600160a01b0391909116600452602490fd5b634e487b7160e01b5f52604160045260245ffd5b601f909101601f19168101906001600160401b0382119082101761025d57604052565b51906001600160a01b03821682036101b457565b905f8091602081519101845af48080610321575b156102dc5750506040513d81523d5f602083013e60203d82010160405290565b1561030157639996b31560e01b5f9081526001600160a01b0391909116600452602490fd5b3d15610312576040513d5f823e3d90fd5b63d6bda27560e01b5f5260045ffd5b503d1515806102bc5750813b15156102bc56fe60806040527f5c60da1b000000000000000000000000000000000000000000000000000000006080526020608060048173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000165afa8015610107575f9015610163575060203d602011610100575b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f820116608001906080821067ffffffffffffffff8311176100d3576100ce91604052608001610112565b610163565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b503d610081565b6040513d5f823e3d90fd5b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80602091011261015f5760805173ffffffffffffffffffffffffffffffffffffffff8116810361015f5790565b5f80fd5b5f8091368280378136915af43d5f803e1561017c573d5ff35b3d5ffdfea164736f6c6343000825000aa164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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

// GetEscrow is a free data retrieval call binding the contract method 0x969631d5.
//
// Solidity: function getEscrow(string clientId) view returns(address)
func (_Contract *ContractCaller) GetEscrow(opts *bind.CallOpts, clientId string) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getEscrow", clientId)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetEscrow is a free data retrieval call binding the contract method 0x969631d5.
//
// Solidity: function getEscrow(string clientId) view returns(address)
func (_Contract *ContractSession) GetEscrow(clientId string) (common.Address, error) {
	return _Contract.Contract.GetEscrow(&_Contract.CallOpts, clientId)
}

// GetEscrow is a free data retrieval call binding the contract method 0x969631d5.
//
// Solidity: function getEscrow(string clientId) view returns(address)
func (_Contract *ContractCallerSession) GetEscrow(clientId string) (common.Address, error) {
	return _Contract.Contract.GetEscrow(&_Contract.CallOpts, clientId)
}

// GetEscrowBeacon is a free data retrieval call binding the contract method 0x1e5150e4.
//
// Solidity: function getEscrowBeacon() view returns(address)
func (_Contract *ContractCaller) GetEscrowBeacon(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getEscrowBeacon")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetEscrowBeacon is a free data retrieval call binding the contract method 0x1e5150e4.
//
// Solidity: function getEscrowBeacon() view returns(address)
func (_Contract *ContractSession) GetEscrowBeacon() (common.Address, error) {
	return _Contract.Contract.GetEscrowBeacon(&_Contract.CallOpts)
}

// GetEscrowBeacon is a free data retrieval call binding the contract method 0x1e5150e4.
//
// Solidity: function getEscrowBeacon() view returns(address)
func (_Contract *ContractCallerSession) GetEscrowBeacon() (common.Address, error) {
	return _Contract.Contract.GetEscrowBeacon(&_Contract.CallOpts)
}

// GetIBCERC20Beacon is a free data retrieval call binding the contract method 0x826cae7a.
//
// Solidity: function getIBCERC20Beacon() view returns(address)
func (_Contract *ContractCaller) GetIBCERC20Beacon(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getIBCERC20Beacon")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetIBCERC20Beacon is a free data retrieval call binding the contract method 0x826cae7a.
//
// Solidity: function getIBCERC20Beacon() view returns(address)
func (_Contract *ContractSession) GetIBCERC20Beacon() (common.Address, error) {
	return _Contract.Contract.GetIBCERC20Beacon(&_Contract.CallOpts)
}

// GetIBCERC20Beacon is a free data retrieval call binding the contract method 0x826cae7a.
//
// Solidity: function getIBCERC20Beacon() view returns(address)
func (_Contract *ContractCallerSession) GetIBCERC20Beacon() (common.Address, error) {
	return _Contract.Contract.GetIBCERC20Beacon(&_Contract.CallOpts)
}

// GetPermit2 is a free data retrieval call binding the contract method 0x1bbf2e23.
//
// Solidity: function getPermit2() view returns(address)
func (_Contract *ContractCaller) GetPermit2(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getPermit2")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetPermit2 is a free data retrieval call binding the contract method 0x1bbf2e23.
//
// Solidity: function getPermit2() view returns(address)
func (_Contract *ContractSession) GetPermit2() (common.Address, error) {
	return _Contract.Contract.GetPermit2(&_Contract.CallOpts)
}

// GetPermit2 is a free data retrieval call binding the contract method 0x1bbf2e23.
//
// Solidity: function getPermit2() view returns(address)
func (_Contract *ContractCallerSession) GetPermit2() (common.Address, error) {
	return _Contract.Contract.GetPermit2(&_Contract.CallOpts)
}

// IbcERC20Contract is a free data retrieval call binding the contract method 0xa50ee2b4.
//
// Solidity: function ibcERC20Contract(string denom) view returns(address)
func (_Contract *ContractCaller) IbcERC20Contract(opts *bind.CallOpts, denom string) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "ibcERC20Contract", denom)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// IbcERC20Contract is a free data retrieval call binding the contract method 0xa50ee2b4.
//
// Solidity: function ibcERC20Contract(string denom) view returns(address)
func (_Contract *ContractSession) IbcERC20Contract(denom string) (common.Address, error) {
	return _Contract.Contract.IbcERC20Contract(&_Contract.CallOpts, denom)
}

// IbcERC20Contract is a free data retrieval call binding the contract method 0xa50ee2b4.
//
// Solidity: function ibcERC20Contract(string denom) view returns(address)
func (_Contract *ContractCallerSession) IbcERC20Contract(denom string) (common.Address, error) {
	return _Contract.Contract.IbcERC20Contract(&_Contract.CallOpts, denom)
}

// IbcERC20Denom is a free data retrieval call binding the contract method 0xe163b1af.
//
// Solidity: function ibcERC20Denom(address token) view returns(string)
func (_Contract *ContractCaller) IbcERC20Denom(opts *bind.CallOpts, token common.Address) (string, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "ibcERC20Denom", token)

	if err != nil {
		return *new(string), err
	}

	out0 := *abi.ConvertType(out[0], new(string)).(*string)

	return out0, err

}

// IbcERC20Denom is a free data retrieval call binding the contract method 0xe163b1af.
//
// Solidity: function ibcERC20Denom(address token) view returns(string)
func (_Contract *ContractSession) IbcERC20Denom(token common.Address) (string, error) {
	return _Contract.Contract.IbcERC20Denom(&_Contract.CallOpts, token)
}

// IbcERC20Denom is a free data retrieval call binding the contract method 0xe163b1af.
//
// Solidity: function ibcERC20Denom(address token) view returns(string)
func (_Contract *ContractCallerSession) IbcERC20Denom(token common.Address) (string, error) {
	return _Contract.Contract.IbcERC20Denom(&_Contract.CallOpts, token)
}

// Ics26 is a free data retrieval call binding the contract method 0xd413227d.
//
// Solidity: function ics26() view returns(address)
func (_Contract *ContractCaller) Ics26(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "ics26")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Ics26 is a free data retrieval call binding the contract method 0xd413227d.
//
// Solidity: function ics26() view returns(address)
func (_Contract *ContractSession) Ics26() (common.Address, error) {
	return _Contract.Contract.Ics26(&_Contract.CallOpts)
}

// Ics26 is a free data retrieval call binding the contract method 0xd413227d.
//
// Solidity: function ics26() view returns(address)
func (_Contract *ContractCallerSession) Ics26() (common.Address, error) {
	return _Contract.Contract.Ics26(&_Contract.CallOpts)
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

// Initialize is a paid mutator transaction binding the contract method 0x1459457a.
//
// Solidity: function initialize(address ics26Router, address escrowLogic, address ibcERC20Logic, address permit2, address authority) returns()
func (_Contract *ContractTransactor) Initialize(opts *bind.TransactOpts, ics26Router common.Address, escrowLogic common.Address, ibcERC20Logic common.Address, permit2 common.Address, authority common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "initialize", ics26Router, escrowLogic, ibcERC20Logic, permit2, authority)
}

// Initialize is a paid mutator transaction binding the contract method 0x1459457a.
//
// Solidity: function initialize(address ics26Router, address escrowLogic, address ibcERC20Logic, address permit2, address authority) returns()
func (_Contract *ContractSession) Initialize(ics26Router common.Address, escrowLogic common.Address, ibcERC20Logic common.Address, permit2 common.Address, authority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, ics26Router, escrowLogic, ibcERC20Logic, permit2, authority)
}

// Initialize is a paid mutator transaction binding the contract method 0x1459457a.
//
// Solidity: function initialize(address ics26Router, address escrowLogic, address ibcERC20Logic, address permit2, address authority) returns()
func (_Contract *ContractTransactorSession) Initialize(ics26Router common.Address, escrowLogic common.Address, ibcERC20Logic common.Address, permit2 common.Address, authority common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, ics26Router, escrowLogic, ibcERC20Logic, permit2, authority)
}

// Multicall is a paid mutator transaction binding the contract method 0xac9650d8.
//
// Solidity: function multicall(bytes[] data) returns(bytes[] results)
func (_Contract *ContractTransactor) Multicall(opts *bind.TransactOpts, data [][]byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "multicall", data)
}

// Multicall is a paid mutator transaction binding the contract method 0xac9650d8.
//
// Solidity: function multicall(bytes[] data) returns(bytes[] results)
func (_Contract *ContractSession) Multicall(data [][]byte) (*types.Transaction, error) {
	return _Contract.Contract.Multicall(&_Contract.TransactOpts, data)
}

// Multicall is a paid mutator transaction binding the contract method 0xac9650d8.
//
// Solidity: function multicall(bytes[] data) returns(bytes[] results)
func (_Contract *ContractTransactorSession) Multicall(data [][]byte) (*types.Transaction, error) {
	return _Contract.Contract.Multicall(&_Contract.TransactOpts, data)
}

// OnAcknowledgementPacket is a paid mutator transaction binding the contract method 0x428e4e17.
//
// Solidity: function onAcknowledgementPacket((string,string,uint64,(string,string,string,string,bytes),bytes,address) msg_) returns()
func (_Contract *ContractTransactor) OnAcknowledgementPacket(opts *bind.TransactOpts, msg_ IIBCAppCallbacksOnAcknowledgementPacketCallback) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "onAcknowledgementPacket", msg_)
}

// OnAcknowledgementPacket is a paid mutator transaction binding the contract method 0x428e4e17.
//
// Solidity: function onAcknowledgementPacket((string,string,uint64,(string,string,string,string,bytes),bytes,address) msg_) returns()
func (_Contract *ContractSession) OnAcknowledgementPacket(msg_ IIBCAppCallbacksOnAcknowledgementPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnAcknowledgementPacket(&_Contract.TransactOpts, msg_)
}

// OnAcknowledgementPacket is a paid mutator transaction binding the contract method 0x428e4e17.
//
// Solidity: function onAcknowledgementPacket((string,string,uint64,(string,string,string,string,bytes),bytes,address) msg_) returns()
func (_Contract *ContractTransactorSession) OnAcknowledgementPacket(msg_ IIBCAppCallbacksOnAcknowledgementPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnAcknowledgementPacket(&_Contract.TransactOpts, msg_)
}

// OnRecvPacket is a paid mutator transaction binding the contract method 0x078c4a79.
//
// Solidity: function onRecvPacket((string,string,uint64,(string,string,string,string,bytes),address) msg_) returns(bytes)
func (_Contract *ContractTransactor) OnRecvPacket(opts *bind.TransactOpts, msg_ IIBCAppCallbacksOnRecvPacketCallback) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "onRecvPacket", msg_)
}

// OnRecvPacket is a paid mutator transaction binding the contract method 0x078c4a79.
//
// Solidity: function onRecvPacket((string,string,uint64,(string,string,string,string,bytes),address) msg_) returns(bytes)
func (_Contract *ContractSession) OnRecvPacket(msg_ IIBCAppCallbacksOnRecvPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnRecvPacket(&_Contract.TransactOpts, msg_)
}

// OnRecvPacket is a paid mutator transaction binding the contract method 0x078c4a79.
//
// Solidity: function onRecvPacket((string,string,uint64,(string,string,string,string,bytes),address) msg_) returns(bytes)
func (_Contract *ContractTransactorSession) OnRecvPacket(msg_ IIBCAppCallbacksOnRecvPacketCallback) (*types.Transaction, error) {
	return _Contract.Contract.OnRecvPacket(&_Contract.TransactOpts, msg_)
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

// SendTransfer is a paid mutator transaction binding the contract method 0x2ac3dc38.
//
// Solidity: function sendTransfer((address,uint256,string,string,string,uint64,string) msg_) returns(uint64)
func (_Contract *ContractTransactor) SendTransfer(opts *bind.TransactOpts, msg_ IICS20TransferMsgsSendTransferMsg) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "sendTransfer", msg_)
}

// SendTransfer is a paid mutator transaction binding the contract method 0x2ac3dc38.
//
// Solidity: function sendTransfer((address,uint256,string,string,string,uint64,string) msg_) returns(uint64)
func (_Contract *ContractSession) SendTransfer(msg_ IICS20TransferMsgsSendTransferMsg) (*types.Transaction, error) {
	return _Contract.Contract.SendTransfer(&_Contract.TransactOpts, msg_)
}

// SendTransfer is a paid mutator transaction binding the contract method 0x2ac3dc38.
//
// Solidity: function sendTransfer((address,uint256,string,string,string,uint64,string) msg_) returns(uint64)
func (_Contract *ContractTransactorSession) SendTransfer(msg_ IICS20TransferMsgsSendTransferMsg) (*types.Transaction, error) {
	return _Contract.Contract.SendTransfer(&_Contract.TransactOpts, msg_)
}

// SendTransferWithPermit2 is a paid mutator transaction binding the contract method 0x53816a7c.
//
// Solidity: function sendTransferWithPermit2((address,uint256,string,string,string,uint64,string) msg_, ((address,uint256),uint256,uint256) permit, bytes signature) returns(uint64)
func (_Contract *ContractTransactor) SendTransferWithPermit2(opts *bind.TransactOpts, msg_ IICS20TransferMsgsSendTransferMsg, permit ISignatureTransferPermitTransferFrom, signature []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "sendTransferWithPermit2", msg_, permit, signature)
}

// SendTransferWithPermit2 is a paid mutator transaction binding the contract method 0x53816a7c.
//
// Solidity: function sendTransferWithPermit2((address,uint256,string,string,string,uint64,string) msg_, ((address,uint256),uint256,uint256) permit, bytes signature) returns(uint64)
func (_Contract *ContractSession) SendTransferWithPermit2(msg_ IICS20TransferMsgsSendTransferMsg, permit ISignatureTransferPermitTransferFrom, signature []byte) (*types.Transaction, error) {
	return _Contract.Contract.SendTransferWithPermit2(&_Contract.TransactOpts, msg_, permit, signature)
}

// SendTransferWithPermit2 is a paid mutator transaction binding the contract method 0x53816a7c.
//
// Solidity: function sendTransferWithPermit2((address,uint256,string,string,string,uint64,string) msg_, ((address,uint256),uint256,uint256) permit, bytes signature) returns(uint64)
func (_Contract *ContractTransactorSession) SendTransferWithPermit2(msg_ IICS20TransferMsgsSendTransferMsg, permit ISignatureTransferPermitTransferFrom, signature []byte) (*types.Transaction, error) {
	return _Contract.Contract.SendTransferWithPermit2(&_Contract.TransactOpts, msg_, permit, signature)
}

// SendTransferWithSender is a paid mutator transaction binding the contract method 0xb29c715d.
//
// Solidity: function sendTransferWithSender((address,uint256,string,string,string,uint64,string) msg_, address sender) returns(uint64)
func (_Contract *ContractTransactor) SendTransferWithSender(opts *bind.TransactOpts, msg_ IICS20TransferMsgsSendTransferMsg, sender common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "sendTransferWithSender", msg_, sender)
}

// SendTransferWithSender is a paid mutator transaction binding the contract method 0xb29c715d.
//
// Solidity: function sendTransferWithSender((address,uint256,string,string,string,uint64,string) msg_, address sender) returns(uint64)
func (_Contract *ContractSession) SendTransferWithSender(msg_ IICS20TransferMsgsSendTransferMsg, sender common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SendTransferWithSender(&_Contract.TransactOpts, msg_, sender)
}

// SendTransferWithSender is a paid mutator transaction binding the contract method 0xb29c715d.
//
// Solidity: function sendTransferWithSender((address,uint256,string,string,string,uint64,string) msg_, address sender) returns(uint64)
func (_Contract *ContractTransactorSession) SendTransferWithSender(msg_ IICS20TransferMsgsSendTransferMsg, sender common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SendTransferWithSender(&_Contract.TransactOpts, msg_, sender)
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

// SetCustomERC20 is a paid mutator transaction binding the contract method 0xa1d28f57.
//
// Solidity: function setCustomERC20(string denom, address token) returns()
func (_Contract *ContractTransactor) SetCustomERC20(opts *bind.TransactOpts, denom string, token common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "setCustomERC20", denom, token)
}

// SetCustomERC20 is a paid mutator transaction binding the contract method 0xa1d28f57.
//
// Solidity: function setCustomERC20(string denom, address token) returns()
func (_Contract *ContractSession) SetCustomERC20(denom string, token common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SetCustomERC20(&_Contract.TransactOpts, denom, token)
}

// SetCustomERC20 is a paid mutator transaction binding the contract method 0xa1d28f57.
//
// Solidity: function setCustomERC20(string denom, address token) returns()
func (_Contract *ContractTransactorSession) SetCustomERC20(denom string, token common.Address) (*types.Transaction, error) {
	return _Contract.Contract.SetCustomERC20(&_Contract.TransactOpts, denom, token)
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

// UpgradeEscrowTo is a paid mutator transaction binding the contract method 0xaaa2c343.
//
// Solidity: function upgradeEscrowTo(address newEscrowLogic) returns()
func (_Contract *ContractTransactor) UpgradeEscrowTo(opts *bind.TransactOpts, newEscrowLogic common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "upgradeEscrowTo", newEscrowLogic)
}

// UpgradeEscrowTo is a paid mutator transaction binding the contract method 0xaaa2c343.
//
// Solidity: function upgradeEscrowTo(address newEscrowLogic) returns()
func (_Contract *ContractSession) UpgradeEscrowTo(newEscrowLogic common.Address) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeEscrowTo(&_Contract.TransactOpts, newEscrowLogic)
}

// UpgradeEscrowTo is a paid mutator transaction binding the contract method 0xaaa2c343.
//
// Solidity: function upgradeEscrowTo(address newEscrowLogic) returns()
func (_Contract *ContractTransactorSession) UpgradeEscrowTo(newEscrowLogic common.Address) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeEscrowTo(&_Contract.TransactOpts, newEscrowLogic)
}

// UpgradeIBCERC20To is a paid mutator transaction binding the contract method 0x06ab20bc.
//
// Solidity: function upgradeIBCERC20To(address newIBCERC20Logic) returns()
func (_Contract *ContractTransactor) UpgradeIBCERC20To(opts *bind.TransactOpts, newIBCERC20Logic common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "upgradeIBCERC20To", newIBCERC20Logic)
}

// UpgradeIBCERC20To is a paid mutator transaction binding the contract method 0x06ab20bc.
//
// Solidity: function upgradeIBCERC20To(address newIBCERC20Logic) returns()
func (_Contract *ContractSession) UpgradeIBCERC20To(newIBCERC20Logic common.Address) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeIBCERC20To(&_Contract.TransactOpts, newIBCERC20Logic)
}

// UpgradeIBCERC20To is a paid mutator transaction binding the contract method 0x06ab20bc.
//
// Solidity: function upgradeIBCERC20To(address newIBCERC20Logic) returns()
func (_Contract *ContractTransactorSession) UpgradeIBCERC20To(newIBCERC20Logic common.Address) (*types.Transaction, error) {
	return _Contract.Contract.UpgradeIBCERC20To(&_Contract.TransactOpts, newIBCERC20Logic)
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

// ContractIBCERC20ContractCreatedIterator is returned from FilterIBCERC20ContractCreated and is used to iterate over the raw logs and unpacked data for IBCERC20ContractCreated events raised by the Contract contract.
type ContractIBCERC20ContractCreatedIterator struct {
	Event *ContractIBCERC20ContractCreated // Event containing the contract specifics and raw log

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
func (it *ContractIBCERC20ContractCreatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractIBCERC20ContractCreated)
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
		it.Event = new(ContractIBCERC20ContractCreated)
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
func (it *ContractIBCERC20ContractCreatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractIBCERC20ContractCreatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractIBCERC20ContractCreated represents a IBCERC20ContractCreated event raised by the Contract contract.
type ContractIBCERC20ContractCreated struct {
	ContractAddress common.Address
	FullDenomPath   string
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterIBCERC20ContractCreated is a free log retrieval operation binding the contract event 0x6031fab685dd6d86e4dbac9a69eae347145f332c95b3a0d728d3730fc5233d62.
//
// Solidity: event IBCERC20ContractCreated(address indexed contractAddress, string fullDenomPath)
func (_Contract *ContractFilterer) FilterIBCERC20ContractCreated(opts *bind.FilterOpts, contractAddress []common.Address) (*ContractIBCERC20ContractCreatedIterator, error) {

	var contractAddressRule []interface{}
	for _, contractAddressItem := range contractAddress {
		contractAddressRule = append(contractAddressRule, contractAddressItem)
	}

	logs, sub, err := _Contract.contract.FilterLogs(opts, "IBCERC20ContractCreated", contractAddressRule)
	if err != nil {
		return nil, err
	}
	return &ContractIBCERC20ContractCreatedIterator{contract: _Contract.contract, event: "IBCERC20ContractCreated", logs: logs, sub: sub}, nil
}

// WatchIBCERC20ContractCreated is a free log subscription operation binding the contract event 0x6031fab685dd6d86e4dbac9a69eae347145f332c95b3a0d728d3730fc5233d62.
//
// Solidity: event IBCERC20ContractCreated(address indexed contractAddress, string fullDenomPath)
func (_Contract *ContractFilterer) WatchIBCERC20ContractCreated(opts *bind.WatchOpts, sink chan<- *ContractIBCERC20ContractCreated, contractAddress []common.Address) (event.Subscription, error) {

	var contractAddressRule []interface{}
	for _, contractAddressItem := range contractAddress {
		contractAddressRule = append(contractAddressRule, contractAddressItem)
	}

	logs, sub, err := _Contract.contract.WatchLogs(opts, "IBCERC20ContractCreated", contractAddressRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractIBCERC20ContractCreated)
				if err := _Contract.contract.UnpackLog(event, "IBCERC20ContractCreated", log); err != nil {
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

// ParseIBCERC20ContractCreated is a log parse operation binding the contract event 0x6031fab685dd6d86e4dbac9a69eae347145f332c95b3a0d728d3730fc5233d62.
//
// Solidity: event IBCERC20ContractCreated(address indexed contractAddress, string fullDenomPath)
func (_Contract *ContractFilterer) ParseIBCERC20ContractCreated(log types.Log) (*ContractIBCERC20ContractCreated, error) {
	event := new(ContractIBCERC20ContractCreated)
	if err := _Contract.contract.UnpackLog(event, "IBCERC20ContractCreated", log); err != nil {
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
