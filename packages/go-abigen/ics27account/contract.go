// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package ics27account

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

// IICS27AccountMsgsCall is an auto generated low-level Go binding around an user-defined struct.
type IICS27AccountMsgsCall struct {
	Target common.Address
	Data   []byte
	Value  *big.Int
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"delegateExecute\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"execute\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"executeBatch\",\"inputs\":[{\"name\":\"calls\",\"type\":\"tuple[]\",\"internalType\":\"structIICS27AccountMsgs.Call[]\",\"components\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"functionCall\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"ics27\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"ics27_\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"sendValue\",\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\",\"internalType\":\"addresspayable\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ICS27AccountNotFound\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidAddress\",\"inputs\":[{\"name\":\"addr\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidPort\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27PayloadEmpty\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ICS27Unauthorized\",\"inputs\":[{\"name\":\"expected\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS27UnexpectedEncoding\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27UnexpectedVersion\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"InsufficientBalance\",\"inputs\":[{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]}]",
	Bin: "0x6080806040523460aa575f516020610bbd5f395f51905f525460ff8160401c16609b576002600160401b03196001600160401b038216016049575b604051610b0e90816100af8239f35b6001600160401b0319166001600160401b039081175f516020610bbd5f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80603a565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe60806040526004361015610011575f80fd5b5f3560e01c806324a084df146106a1578063599deb481461064f5780638a89b44b14610420578063a04a0908146103bc578063a0b5ffb014610357578063b10cc7281461030b5763c4d66de814610066575f80fd5b346103075760206003193601126103075761007f610731565b7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460ff8160401c16159067ffffffffffffffff8116801590816102ff575b60011490816102f5575b1590816102ec575b506102c4578160017fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000008316177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005561026f575b507ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549160ff8360401c16156102475773ffffffffffffffffffffffffffffffffffffffff167fffffffffffffffffffffffff00000000000000000000000000000000000000007f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c005416177f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c00556101d657005b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a1005b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00555f610123565b7ff92ee8a9000000000000000000000000000000000000000000000000000000005f5260045ffd5b9050155f6100d0565b303b1591506100c8565b8391506100be565b5f80fd5b346103075761035361033f61033961032236610782565b61033293929333303033146107d3565b36916108ad565b90610aca565b6040519182916020835260208301906106ee565b0390f35b346103075761035361033f6103b661036e36610782565b61033273ffffffffffffffffffffffffffffffffffffffff7f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c00959495541633908033146107d3565b906109ee565b34610307576060600319360112610307576103d5610731565b6024359067ffffffffffffffff82116103075761041a61033f91610400610353943690600401610754565b929061040f33303033146107d3565b6044359336916108ad565b90610a6f565b346103075760206003193601126103075760043567ffffffffffffffff811161030757366023820112156103075780600401359067ffffffffffffffff8211610307573660248360051b830101116103075761047f33303033146107d3565b61049061048b83610895565b610824565b918083527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe06104be82610895565b015f5b81811061063e575050905f907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7d81360301915b838110156105bb5760248160051b830101358381121561030757820190602482013573ffffffffffffffffffffffffffffffffffffffff811681036103075760448301357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffbd368590030181121561030757830160240180359067ffffffffffffffff8211610307576020019181360383136103075761041a61059f93606460019701359336916108ad565b6105a9828861090c565b526105b4818761090c565b50016104f4565b846040518091602082016020835281518091526040830190602060408260051b8601019301915f905b8282106105f357505050500390f35b9193602061062e827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc0600195979984950301865288516106ee565b96019201920185949391926105e4565b8060606020809388010152016104c1565b34610307575f60031936011261030757602073ffffffffffffffffffffffffffffffffffffffff7f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c005416604051908152f35b346103075760406003193601126103075760043573ffffffffffffffffffffffffffffffffffffffff81168103610307576106ec906106e333303033146107d3565b6024359061094d565b005b907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f602080948051918291828752018686015e5f8582860101520116010190565b6004359073ffffffffffffffffffffffffffffffffffffffff8216820361030757565b9181601f840112156103075782359167ffffffffffffffff8311610307576020838186019501011161030757565b9060406003198301126103075760043573ffffffffffffffffffffffffffffffffffffffff8116810361030757916024359067ffffffffffffffff8211610307576107cf91600401610754565b9091565b156107dc575050565b9073ffffffffffffffffffffffffffffffffffffffff80927f2a99fb3c000000000000000000000000000000000000000000000000000000005f52166004521660245260445ffd5b907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f604051930116820182811067ffffffffffffffff82111761086857604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b67ffffffffffffffff81116108685760051b60200190565b92919267ffffffffffffffff8211610868576108f060207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f85011601610824565b938285528282011161030757815f926020928387013784010152565b80518210156109205760209160051b010190565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b8147106109be575f80809373ffffffffffffffffffffffffffffffffffffffff82948361097a6020610824565b52165af11561098557565b3d15610996576040513d5f823e3d90fd5b7fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b50477fcf479181000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b905f809160208151910182855af18080610a5c575b15610a15575050610a12610ae8565b90565b156109855773ffffffffffffffffffffffffffffffffffffffff907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b503d151580610a035750813b1515610a03565b91804710610a9b57815f92916020849351920190855af18080610a5c5715610a15575050610a12610ae8565b477fcf479181000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b905f8091602081519101845af48080610a5c5715610a15575050610a125b604051903d82523d5f602084013e60203d83010160405256fea164736f6c634300081c000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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

// DelegateExecute is a paid mutator transaction binding the contract method 0xb10cc728.
//
// Solidity: function delegateExecute(address target, bytes data) returns(bytes)
func (_Contract *ContractTransactor) DelegateExecute(opts *bind.TransactOpts, target common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "delegateExecute", target, data)
}

// DelegateExecute is a paid mutator transaction binding the contract method 0xb10cc728.
//
// Solidity: function delegateExecute(address target, bytes data) returns(bytes)
func (_Contract *ContractSession) DelegateExecute(target common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.DelegateExecute(&_Contract.TransactOpts, target, data)
}

// DelegateExecute is a paid mutator transaction binding the contract method 0xb10cc728.
//
// Solidity: function delegateExecute(address target, bytes data) returns(bytes)
func (_Contract *ContractTransactorSession) DelegateExecute(target common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.DelegateExecute(&_Contract.TransactOpts, target, data)
}

// Execute is a paid mutator transaction binding the contract method 0xa04a0908.
//
// Solidity: function execute(address target, bytes data, uint256 value) returns(bytes)
func (_Contract *ContractTransactor) Execute(opts *bind.TransactOpts, target common.Address, data []byte, value *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "execute", target, data, value)
}

// Execute is a paid mutator transaction binding the contract method 0xa04a0908.
//
// Solidity: function execute(address target, bytes data, uint256 value) returns(bytes)
func (_Contract *ContractSession) Execute(target common.Address, data []byte, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Execute(&_Contract.TransactOpts, target, data, value)
}

// Execute is a paid mutator transaction binding the contract method 0xa04a0908.
//
// Solidity: function execute(address target, bytes data, uint256 value) returns(bytes)
func (_Contract *ContractTransactorSession) Execute(target common.Address, data []byte, value *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.Execute(&_Contract.TransactOpts, target, data, value)
}

// ExecuteBatch is a paid mutator transaction binding the contract method 0x8a89b44b.
//
// Solidity: function executeBatch((address,bytes,uint256)[] calls) returns(bytes[])
func (_Contract *ContractTransactor) ExecuteBatch(opts *bind.TransactOpts, calls []IICS27AccountMsgsCall) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "executeBatch", calls)
}

// ExecuteBatch is a paid mutator transaction binding the contract method 0x8a89b44b.
//
// Solidity: function executeBatch((address,bytes,uint256)[] calls) returns(bytes[])
func (_Contract *ContractSession) ExecuteBatch(calls []IICS27AccountMsgsCall) (*types.Transaction, error) {
	return _Contract.Contract.ExecuteBatch(&_Contract.TransactOpts, calls)
}

// ExecuteBatch is a paid mutator transaction binding the contract method 0x8a89b44b.
//
// Solidity: function executeBatch((address,bytes,uint256)[] calls) returns(bytes[])
func (_Contract *ContractTransactorSession) ExecuteBatch(calls []IICS27AccountMsgsCall) (*types.Transaction, error) {
	return _Contract.Contract.ExecuteBatch(&_Contract.TransactOpts, calls)
}

// FunctionCall is a paid mutator transaction binding the contract method 0xa0b5ffb0.
//
// Solidity: function functionCall(address target, bytes data) returns(bytes)
func (_Contract *ContractTransactor) FunctionCall(opts *bind.TransactOpts, target common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "functionCall", target, data)
}

// FunctionCall is a paid mutator transaction binding the contract method 0xa0b5ffb0.
//
// Solidity: function functionCall(address target, bytes data) returns(bytes)
func (_Contract *ContractSession) FunctionCall(target common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.FunctionCall(&_Contract.TransactOpts, target, data)
}

// FunctionCall is a paid mutator transaction binding the contract method 0xa0b5ffb0.
//
// Solidity: function functionCall(address target, bytes data) returns(bytes)
func (_Contract *ContractTransactorSession) FunctionCall(target common.Address, data []byte) (*types.Transaction, error) {
	return _Contract.Contract.FunctionCall(&_Contract.TransactOpts, target, data)
}

// Initialize is a paid mutator transaction binding the contract method 0xc4d66de8.
//
// Solidity: function initialize(address ics27_) returns()
func (_Contract *ContractTransactor) Initialize(opts *bind.TransactOpts, ics27_ common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "initialize", ics27_)
}

// Initialize is a paid mutator transaction binding the contract method 0xc4d66de8.
//
// Solidity: function initialize(address ics27_) returns()
func (_Contract *ContractSession) Initialize(ics27_ common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, ics27_)
}

// Initialize is a paid mutator transaction binding the contract method 0xc4d66de8.
//
// Solidity: function initialize(address ics27_) returns()
func (_Contract *ContractTransactorSession) Initialize(ics27_ common.Address) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, ics27_)
}

// SendValue is a paid mutator transaction binding the contract method 0x24a084df.
//
// Solidity: function sendValue(address recipient, uint256 amount) returns()
func (_Contract *ContractTransactor) SendValue(opts *bind.TransactOpts, recipient common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "sendValue", recipient, amount)
}

// SendValue is a paid mutator transaction binding the contract method 0x24a084df.
//
// Solidity: function sendValue(address recipient, uint256 amount) returns()
func (_Contract *ContractSession) SendValue(recipient common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.SendValue(&_Contract.TransactOpts, recipient, amount)
}

// SendValue is a paid mutator transaction binding the contract method 0x24a084df.
//
// Solidity: function sendValue(address recipient, uint256 amount) returns()
func (_Contract *ContractTransactorSession) SendValue(recipient common.Address, amount *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.SendValue(&_Contract.TransactOpts, recipient, amount)
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
