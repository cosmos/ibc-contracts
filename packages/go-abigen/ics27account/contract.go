// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package ics27account

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

// IICS27AccountMsgsCall is an auto generated low-level Go binding around an user-defined struct.
type IICS27AccountMsgsCall struct {
	Target common.Address
	Data   []byte
	Value  *big.Int
}

// IICS27AccountMsgsCallWithGas is an auto generated low-level Go binding around an user-defined struct.
type IICS27AccountMsgsCallWithGas struct {
	Target common.Address
	Data   []byte
	Value  *big.Int
	Gas    *big.Int
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"execute\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"executeBatch\",\"inputs\":[{\"name\":\"calls\",\"type\":\"tuple[]\",\"internalType\":\"structIICS27AccountMsgs.Call[]\",\"components\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"executeBatchWithGas\",\"inputs\":[{\"name\":\"calls\",\"type\":\"tuple[]\",\"internalType\":\"structIICS27AccountMsgs.CallWithGas[]\",\"components\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"gas\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"executeWithGas\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"value\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"gas\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"functionCall\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"ics27\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"ics27_\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"sendValue\",\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\",\"internalType\":\"addresspayable\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"Initialized\",\"inputs\":[{\"name\":\"version\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"inputs\":[{\"name\":\"target\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"FailedCall\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ICS27AccountNotFound\",\"inputs\":[{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidAddress\",\"inputs\":[{\"name\":\"addr\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidPort\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27InvalidSender\",\"inputs\":[{\"name\":\"sender\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27PayloadEmpty\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"ICS27Unauthorized\",\"inputs\":[{\"name\":\"expected\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"caller\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"ICS27UnexpectedEncoding\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"actual\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"ICS27UnexpectedVersion\",\"inputs\":[{\"name\":\"expected\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"version\",\"type\":\"string\",\"internalType\":\"string\"}]},{\"type\":\"error\",\"name\":\"InsufficientBalance\",\"inputs\":[{\"name\":\"balance\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"needed\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidInitialization\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotInitializing\",\"inputs\":[]}]",
	Bin: "0x6080806040523460aa575f516020610d785f395f51905f525460ff8160401c16609b576002600160401b03196001600160401b038216016049575b604051610cc990816100af8239f35b6001600160401b0319166001600160401b039081175f516020610d785f395f51905f525581527fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d290602090a15f80603a565b63f92ee8a960e01b5f5260045ffd5b5f80fdfe60806040526004361015610011575f80fd5b5f3560e01c806324a084df1461075c578063599deb481461070a5780635b7f628c146106a95780638a89b44b146105d4578063a04a09081461055b578063a0b5ffb014610407578063c4d66de8146101665763effc227214610071575f80fd5b346101505760206003193601126101505760043567ffffffffffffffff8111610150576100a2903690600401610856565b6100af3330303314610904565b6100b881610a43565b915f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8182360301905b83811015610154578060051b830135908282121561015057610134600192850161012e61010d82610a9f565b61011a6020840184610ac0565b929060406060860135950135933691610a00565b90610c09565b61013e8288610b11565b526101498187610b11565b50016100e1565b5f80fd5b604051806101628782610887565b0390f35b346101505760206003193601126101505761017f6107c2565b7ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005460ff8160401c16159067ffffffffffffffff8116801590816103ff575b60011490816103f5575b1590816103ec575b506103c4578160017fffffffffffffffffffffffffffffffffffffffffffffffff00000000000000008316177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005561036f575b507ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00549160ff8360401c16156103475773ffffffffffffffffffffffffffffffffffffffff167fffffffffffffffffffffffff00000000000000000000000000000000000000007f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c005416177f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c00556102d657005b7fffffffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff167ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00557fc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2602060405160018152a1005b7fd7e6bcf8000000000000000000000000000000000000000000000000000000005f5260045ffd5b7fffffffffffffffffffffffffffffffffffffffffffffff0000000000000000001668010000000000000001177ff0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a005582610223565b7ff92ee8a9000000000000000000000000000000000000000000000000000000005f5260045ffd5b905015846101d0565b303b1591506101c8565b8391506101be565b34610150576040600319360112610150576104206107c2565b60243567ffffffffffffffff8111610150576104436104939136906004016107e5565b919061048a73ffffffffffffffffffffffffffffffffffffffff7f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c0054163390803314610904565b5a923691610a00565b5f8160208293519101828686f13d15610556573d6104b86104b3826109c6565b610955565b9081525f60203d92013e5b1561055157503d15801590610547575b15610505576101626040513d81523d5f602083013e60203d820101604052604051918291602083526020830190610813565b73ffffffffffffffffffffffffffffffffffffffff907f9996b315000000000000000000000000000000000000000000000000000000005f521660045260245ffd5b50803b15156104d3565b610b8d565b6104c3565b34610150576060600319360112610150576105746107c2565b60243567ffffffffffffffff8111610150576101629161059b6105c09236906004016107e5565b90916105aa3330303314610904565b6105ba5a93604435933691610a00565b90610c47565b604051918291602083526020830190610813565b346101505760206003193601126101505760043567ffffffffffffffff811161015057610605903690600401610856565b6106123330303314610904565b61061b81610a43565b915f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa182360301905b83811015610154578060051b83013590828212156101505761068d60019285016105ba61067082610a9f565b61067d6020840184610ac0565b929060405a950135933691610a00565b6106978288610b11565b526106a28187610b11565b5001610644565b34610150576080600319360112610150576106c26107c2565b60243567ffffffffffffffff811161015057610162916106e96105c09236906004016107e5565b90916106f83330303314610904565b61012e60643593604435933691610a00565b34610150575f60031936011261015057602073ffffffffffffffffffffffffffffffffffffffff7f319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c005416604051908152f35b346101505760406003193601126101505760043573ffffffffffffffffffffffffffffffffffffffff8116809103610150575f8080806024356107a23330303314610904565b6107ab81610b52565b5a95826107b86020610955565b525af11561055157005b6004359073ffffffffffffffffffffffffffffffffffffffff8216820361015057565b9181601f840112156101505782359167ffffffffffffffff8311610150576020838186019501011161015057565b907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f602080948051918291828752018686015e5f8582860101520116010190565b9181601f840112156101505782359167ffffffffffffffff8311610150576020808501948460051b01011161015057565b602081016020825282518091526040820191602060408360051b8301019401925f915b8383106108b957505050505090565b90919293946020806108f5837fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc086600196030187528951610813565b970193019301919392906108aa565b1561090d575050565b9073ffffffffffffffffffffffffffffffffffffffff80927f2a99fb3c000000000000000000000000000000000000000000000000000000005f52166004521660245260445ffd5b907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f604051930116820182811067ffffffffffffffff82111761099957604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b67ffffffffffffffff811161099957601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01660200190565b929192610a0f6104b3836109c6565b938285528282011161015057815f926020928387013784010152565b67ffffffffffffffff81116109995760051b60200190565b90610a506104b383610a2b565b8281527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0610a7e8294610a2b565b01905f5b828110610a8e57505050565b806060602080938501015201610a82565b3573ffffffffffffffffffffffffffffffffffffffff811681036101505790565b9035907fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe181360301821215610150570180359067ffffffffffffffff82116101505760200191813603831361015057565b8051821015610b255760209160051b010190565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b8047104790610b5f575050565b7fcf479181000000000000000000000000000000000000000000000000000000005f5260045260245260445ffd5b3d610bc4575a9060031c11610150577fd6bda275000000000000000000000000000000000000000000000000000000005f5260045ffd5b6040513d5f823e3d90fd5b91908201809211610bdc57565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b9291905a610c27610c1d603f860486610bcf565b8560031c90610bcf565b90619c408201809211610bdc5710610c4557610c4293610c47565b90565bfe5b92915f918291610c5681610b52565b602082519201908686f13d15610cb7573d610c736104b3826109c6565b9081525f60203d92013e5b1561055157503d15801590610cad575b1561050557506040513d81523d5f602083013e60203d82010160405290565b50803b1515610c8e565b610c7e56fea164736f6c6343000825000af0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00",
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

// ExecuteBatchWithGas is a paid mutator transaction binding the contract method 0xeffc2272.
//
// Solidity: function executeBatchWithGas((address,bytes,uint256,uint256)[] calls) returns(bytes[])
func (_Contract *ContractTransactor) ExecuteBatchWithGas(opts *bind.TransactOpts, calls []IICS27AccountMsgsCallWithGas) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "executeBatchWithGas", calls)
}

// ExecuteBatchWithGas is a paid mutator transaction binding the contract method 0xeffc2272.
//
// Solidity: function executeBatchWithGas((address,bytes,uint256,uint256)[] calls) returns(bytes[])
func (_Contract *ContractSession) ExecuteBatchWithGas(calls []IICS27AccountMsgsCallWithGas) (*types.Transaction, error) {
	return _Contract.Contract.ExecuteBatchWithGas(&_Contract.TransactOpts, calls)
}

// ExecuteBatchWithGas is a paid mutator transaction binding the contract method 0xeffc2272.
//
// Solidity: function executeBatchWithGas((address,bytes,uint256,uint256)[] calls) returns(bytes[])
func (_Contract *ContractTransactorSession) ExecuteBatchWithGas(calls []IICS27AccountMsgsCallWithGas) (*types.Transaction, error) {
	return _Contract.Contract.ExecuteBatchWithGas(&_Contract.TransactOpts, calls)
}

// ExecuteWithGas is a paid mutator transaction binding the contract method 0x5b7f628c.
//
// Solidity: function executeWithGas(address target, bytes data, uint256 value, uint256 gas) returns(bytes)
func (_Contract *ContractTransactor) ExecuteWithGas(opts *bind.TransactOpts, target common.Address, data []byte, value *big.Int, gas *big.Int) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "executeWithGas", target, data, value, gas)
}

// ExecuteWithGas is a paid mutator transaction binding the contract method 0x5b7f628c.
//
// Solidity: function executeWithGas(address target, bytes data, uint256 value, uint256 gas) returns(bytes)
func (_Contract *ContractSession) ExecuteWithGas(target common.Address, data []byte, value *big.Int, gas *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.ExecuteWithGas(&_Contract.TransactOpts, target, data, value, gas)
}

// ExecuteWithGas is a paid mutator transaction binding the contract method 0x5b7f628c.
//
// Solidity: function executeWithGas(address target, bytes data, uint256 value, uint256 gas) returns(bytes)
func (_Contract *ContractTransactorSession) ExecuteWithGas(target common.Address, data []byte, value *big.Int, gas *big.Int) (*types.Transaction, error) {
	return _Contract.Contract.ExecuteWithGas(&_Contract.TransactOpts, target, data, value, gas)
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
