// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package relayerhelper

import (
	"errors"
	"math/big"
	"strings"

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
)

// IICS26RouterMsgsPacket is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsPacket struct {
	Sequence         uint64
	SourceClient     string
	DestClient       string
	TimeoutTimestamp uint64
	Payloads         []IICS26RouterMsgsPayload
}

// IICS26RouterMsgsPayload is an auto generated low-level Go binding around an user-defined struct.
type IICS26RouterMsgsPayload struct {
	SourcePort string
	DestPort   string
	Version    string
	Encoding   string
	Value      []byte
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"_ics26Router\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"ICS26_ROUTER\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isPacketReceiveSuccessful\",\"inputs\":[{\"name\":\"packet\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isPacketReceived\",\"inputs\":[{\"name\":\"packet\",\"type\":\"tuple\",\"internalType\":\"structIICS26RouterMsgs.Packet\",\"components\":[{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"sourceClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destClient\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payloads\",\"type\":\"tuple[]\",\"internalType\":\"structIICS26RouterMsgs.Payload[]\",\"components\":[{\"name\":\"sourcePort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"destPort\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"encoding\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"value\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"queryAckCommitment\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"queryPacketCommitment\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"queryPacketReceipt\",\"inputs\":[{\"name\":\"clientId\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"sequence\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"error\",\"name\":\"NoAcknowledgements\",\"inputs\":[]}]",
	Bin: "0x60a034607457601f610d1d38819003918201601f19168301916001600160401b03831184841017607857808492602094604052833981010312607457516001600160a01b0381168103607457608052604051610c90908161008d82396080518181816101620152818161024501526104df0152f35b5f80fd5b634e487b7160e01b5f52604160045260245ffdfe60806040526004361015610011575f80fd5b5f3560e01c80630bb1774c146102815780637ecfb0c214610269578063c20f00e1146101fb578063db8b4b63146101da578063dfdcf16d146100815763ede05f161461005b575f80fd5b3461007d57602061007361006e36610333565b610a65565b6040519015158152f35b5f80fd5b3461007d5761009c610092366102af565b9291903691610541565b90610116600960405180937fffffffffffffffff000000000000000000000000000000000000000000000000602080808501988051918291018a5e840101917f0100000000000000000000000000000000000000000000000000000000000000835260c01b1660018201520301601f1981018352826103eb565b519020604051907f7795820c000000000000000000000000000000000000000000000000000000008252600482015260208160248173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000165afa80156101cf575f9061019c575b602090604051908152f35b506020813d6020116101c7575b816101b6602093836103eb565b8101031261007d5760209051610191565b3d91506101a9565b6040513d5f823e3d90fd5b3461007d5760206101f36101ed366102af565b916109a6565b604051908152f35b3461007d575f7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc36011261007d57602060405173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000168152f35b3461007d57602061007361027c36610333565b61060b565b3461007d5760206101f3610294366102af565b9161040e565b359067ffffffffffffffff8216820361007d57565b9060407ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc83011261007d5760043567ffffffffffffffff811161007d578260238201121561007d5780600401359267ffffffffffffffff841161007d576024848301011161007d57602401919060243567ffffffffffffffff8116810361007d5790565b60207ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc82011261007d576004359067ffffffffffffffff821161007d577ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc8260a09203011261007d5760040190565b60a0810190811067ffffffffffffffff8211176103be57604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b90601f601f19910116810190811067ffffffffffffffff8211176103be57604052565b610419913691610541565b90610493600960405180937fffffffffffffffff000000000000000000000000000000000000000000000000602080808501988051918291018a5e840101917f0200000000000000000000000000000000000000000000000000000000000000835260c01b1660018201520301601f1981018352826103eb565b519020604051907f7795820c000000000000000000000000000000000000000000000000000000008252600482015260208160248173ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000165afa9081156101cf575f91610512575090565b90506020813d602011610539575b8161052d602093836103eb565b8101031261007d575190565b3d9150610520565b92919267ffffffffffffffff82116103be576040519161056b601f8201601f1916602001846103eb565b82948184528183011161007d578281602093845f960137010152565b9080601f8301121561007d578160206105a293359101610541565b90565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe18136030182121561007d570180359067ffffffffffffffff821161007d5760200191813603831361007d57565b3567ffffffffffffffff8116810361007d5790565b60a08136031261007d5760405190610622826103a2565b61062b8161029a565b8252602081013567ffffffffffffffff811161007d5761064e9036908301610587565b91602081019283526040820192833567ffffffffffffffff811161007d576106799036908501610587565b6040830190815261068c6060850161029a565b9060608401918252608085013567ffffffffffffffff811161007d5785019036601f8301121561007d5781359167ffffffffffffffff83116103be578260051b9060208201936106df60405195866103eb565b84526020808501928201019036821161007d5760208101925b82841061088d575050505061077267ffffffffffffffff91608087019384526107416040519684602089019960208b52511660408901525160a0606089015260e0880190610c5e565b90517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc0878303016080880152610c5e565b92511660a084015251907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc08382030160c0840152815180825260208201916020808360051b8301019401925f915b8383106108015750505050506107fd9392826107eb610294946107f59403601f1981018352826103eb565b51902094836105a5565b9190926105f6565b1490565b909192939460208061087e83601f198660019603018752895190608061086d61085b610849610839865160a0875260a0870190610c5e565b888701518682038a880152610c5e565b60408601518582036040870152610c5e565b60608501518482036060860152610c5e565b920151906080818403910152610c5e565b970193019301919392906107c0565b833567ffffffffffffffff811161007d5782019060a0601f19833603011261007d57604051916108bc836103a2565b602081013567ffffffffffffffff811161007d576108e09060203691840101610587565b8352604081013567ffffffffffffffff811161007d576109069060203691840101610587565b6020840152606081013567ffffffffffffffff811161007d5761092f9060203691840101610587565b6040840152608081013567ffffffffffffffff811161007d576109589060203691840101610587565b606084015260a081013567ffffffffffffffff811161007d5760209101019036601f8301121561007d57602092610996849336908581359101610541565b60808201528152019301926106f8565b6109b1913691610541565b90610493600960405180937fffffffffffffffff000000000000000000000000000000000000000000000000602080808501988051918291018a5e840101917f0300000000000000000000000000000000000000000000000000000000000000835260c01b1660018201520301601f1981018352826103eb565b805115610a385760200190565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b610a6e8161060b565b15610c5957604091825190610a8384836103eb565b60018252601f1984015f5b818110610c485750508351610aa385826103eb565b602081527f4774d4a575993f963b1c06573736617a457abef8589178db8d10c94b4ab511ab6020820152610ad683610a2b565b52610ae082610a2b565b50815115610c20576020918451610af784826103eb565b5f8152915f925b8251841015610b78578251841015610a3857845f81808760051b870101518a51918183925191829101835e8101838152039060025afa15610b6e5784610b6681806001945f518c519582879351918291018585015e82019083820152030180845201826103eb565b930192610afe565b86513d5f823e3d90fd5b9095949392505f91508451610bd160218286808201957f020000000000000000000000000000000000000000000000000000000000000087528051918291018484015e810186838201520301601f1981018352826103eb565b8551918291518091835e8101838152039060025afa15610c1657610c00906101ed6107f55f51948301836105a5565b908115159182610c0f57505090565b1415919050565b50513d5f823e3d90fd5b7f760d6a9b000000000000000000000000000000000000000000000000000000005f5260045ffd5b806060602080938701015201610a8e565b505f90565b90601f19601f602080948051918291828752018686015e5f858286010152011601019056fea164736f6c634300081c000a",
}

// ContractABI is the input ABI used to generate the binding from.
// Deprecated: Use ContractMetaData.ABI instead.
var ContractABI = ContractMetaData.ABI

// ContractBin is the compiled bytecode used for deploying new contracts.
// Deprecated: Use ContractMetaData.Bin instead.
var ContractBin = ContractMetaData.Bin

// DeployContract deploys a new Ethereum contract, binding an instance of Contract to it.
func DeployContract(auth *bind.TransactOpts, backend bind.ContractBackend, _ics26Router common.Address) (common.Address, *types.Transaction, *Contract, error) {
	parsed, err := ContractMetaData.GetAbi()
	if err != nil {
		return common.Address{}, nil, nil, err
	}
	if parsed == nil {
		return common.Address{}, nil, nil, errors.New("GetABI returned nil")
	}

	address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex(ContractBin), backend, _ics26Router)
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

// ICS26ROUTER is a free data retrieval call binding the contract method 0xc20f00e1.
//
// Solidity: function ICS26_ROUTER() view returns(address)
func (_Contract *ContractCaller) ICS26ROUTER(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "ICS26_ROUTER")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ICS26ROUTER is a free data retrieval call binding the contract method 0xc20f00e1.
//
// Solidity: function ICS26_ROUTER() view returns(address)
func (_Contract *ContractSession) ICS26ROUTER() (common.Address, error) {
	return _Contract.Contract.ICS26ROUTER(&_Contract.CallOpts)
}

// ICS26ROUTER is a free data retrieval call binding the contract method 0xc20f00e1.
//
// Solidity: function ICS26_ROUTER() view returns(address)
func (_Contract *ContractCallerSession) ICS26ROUTER() (common.Address, error) {
	return _Contract.Contract.ICS26ROUTER(&_Contract.CallOpts)
}

// IsPacketReceiveSuccessful is a free data retrieval call binding the contract method 0xede05f16.
//
// Solidity: function isPacketReceiveSuccessful((uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet) view returns(bool)
func (_Contract *ContractCaller) IsPacketReceiveSuccessful(opts *bind.CallOpts, packet IICS26RouterMsgsPacket) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "isPacketReceiveSuccessful", packet)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsPacketReceiveSuccessful is a free data retrieval call binding the contract method 0xede05f16.
//
// Solidity: function isPacketReceiveSuccessful((uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet) view returns(bool)
func (_Contract *ContractSession) IsPacketReceiveSuccessful(packet IICS26RouterMsgsPacket) (bool, error) {
	return _Contract.Contract.IsPacketReceiveSuccessful(&_Contract.CallOpts, packet)
}

// IsPacketReceiveSuccessful is a free data retrieval call binding the contract method 0xede05f16.
//
// Solidity: function isPacketReceiveSuccessful((uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet) view returns(bool)
func (_Contract *ContractCallerSession) IsPacketReceiveSuccessful(packet IICS26RouterMsgsPacket) (bool, error) {
	return _Contract.Contract.IsPacketReceiveSuccessful(&_Contract.CallOpts, packet)
}

// IsPacketReceived is a free data retrieval call binding the contract method 0x7ecfb0c2.
//
// Solidity: function isPacketReceived((uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet) view returns(bool)
func (_Contract *ContractCaller) IsPacketReceived(opts *bind.CallOpts, packet IICS26RouterMsgsPacket) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "isPacketReceived", packet)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsPacketReceived is a free data retrieval call binding the contract method 0x7ecfb0c2.
//
// Solidity: function isPacketReceived((uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet) view returns(bool)
func (_Contract *ContractSession) IsPacketReceived(packet IICS26RouterMsgsPacket) (bool, error) {
	return _Contract.Contract.IsPacketReceived(&_Contract.CallOpts, packet)
}

// IsPacketReceived is a free data retrieval call binding the contract method 0x7ecfb0c2.
//
// Solidity: function isPacketReceived((uint64,string,string,uint64,(string,string,string,string,bytes)[]) packet) view returns(bool)
func (_Contract *ContractCallerSession) IsPacketReceived(packet IICS26RouterMsgsPacket) (bool, error) {
	return _Contract.Contract.IsPacketReceived(&_Contract.CallOpts, packet)
}

// QueryAckCommitment is a free data retrieval call binding the contract method 0xdb8b4b63.
//
// Solidity: function queryAckCommitment(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractCaller) QueryAckCommitment(opts *bind.CallOpts, clientId string, sequence uint64) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "queryAckCommitment", clientId, sequence)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// QueryAckCommitment is a free data retrieval call binding the contract method 0xdb8b4b63.
//
// Solidity: function queryAckCommitment(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractSession) QueryAckCommitment(clientId string, sequence uint64) ([32]byte, error) {
	return _Contract.Contract.QueryAckCommitment(&_Contract.CallOpts, clientId, sequence)
}

// QueryAckCommitment is a free data retrieval call binding the contract method 0xdb8b4b63.
//
// Solidity: function queryAckCommitment(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractCallerSession) QueryAckCommitment(clientId string, sequence uint64) ([32]byte, error) {
	return _Contract.Contract.QueryAckCommitment(&_Contract.CallOpts, clientId, sequence)
}

// QueryPacketCommitment is a free data retrieval call binding the contract method 0xdfdcf16d.
//
// Solidity: function queryPacketCommitment(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractCaller) QueryPacketCommitment(opts *bind.CallOpts, clientId string, sequence uint64) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "queryPacketCommitment", clientId, sequence)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// QueryPacketCommitment is a free data retrieval call binding the contract method 0xdfdcf16d.
//
// Solidity: function queryPacketCommitment(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractSession) QueryPacketCommitment(clientId string, sequence uint64) ([32]byte, error) {
	return _Contract.Contract.QueryPacketCommitment(&_Contract.CallOpts, clientId, sequence)
}

// QueryPacketCommitment is a free data retrieval call binding the contract method 0xdfdcf16d.
//
// Solidity: function queryPacketCommitment(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractCallerSession) QueryPacketCommitment(clientId string, sequence uint64) ([32]byte, error) {
	return _Contract.Contract.QueryPacketCommitment(&_Contract.CallOpts, clientId, sequence)
}

// QueryPacketReceipt is a free data retrieval call binding the contract method 0x0bb1774c.
//
// Solidity: function queryPacketReceipt(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractCaller) QueryPacketReceipt(opts *bind.CallOpts, clientId string, sequence uint64) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "queryPacketReceipt", clientId, sequence)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// QueryPacketReceipt is a free data retrieval call binding the contract method 0x0bb1774c.
//
// Solidity: function queryPacketReceipt(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractSession) QueryPacketReceipt(clientId string, sequence uint64) ([32]byte, error) {
	return _Contract.Contract.QueryPacketReceipt(&_Contract.CallOpts, clientId, sequence)
}

// QueryPacketReceipt is a free data retrieval call binding the contract method 0x0bb1774c.
//
// Solidity: function queryPacketReceipt(string clientId, uint64 sequence) view returns(bytes32)
func (_Contract *ContractCallerSession) QueryPacketReceipt(clientId string, sequence uint64) ([32]byte, error) {
	return _Contract.Contract.QueryPacketReceipt(&_Contract.CallOpts, clientId, sequence)
}
