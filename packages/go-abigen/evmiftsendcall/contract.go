// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package evmiftsendcall

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

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"function\",\"name\":\"constructMintCall\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"supportsInterface\",\"inputs\":[{\"name\":\"interfaceId\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"error\",\"name\":\"EVMIFTInvalidReceiver\",\"inputs\":[{\"name\":\"receiver\",\"type\":\"string\",\"internalType\":\"string\"}]}]",
	Bin: "0x608080604052346015576105f0908161001a8239f35b5f80fdfe6080806040526004361015610012575f80fd5b5f3560e01c90816301ffc9a7146101f657506356d981a714610032575f80fd5b346101f25760407ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc3601126101f25760043567ffffffffffffffff81116101f257366023820112156101f257806004013567ffffffffffffffff81116101f257602482019160248236920101116101f2577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f820116916100f76040516100dd60208601826102b2565b838152838360208301375f60208583010152805190610320565b9390156101a657836040805173ffffffffffffffffffffffffffffffffffffffff60208201937f0a7244e700000000000000000000000000000000000000000000000000000000855216602482015260243560448201526044815261015d6064826102b2565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f8351948593602085525180918160208701528686015e5f85828601015201168101030190f35b906044915f83856040519687957fddd8c4b60000000000000000000000000000000000000000000000000000000087526020600488015281602488015283870137840101528101030190fd5b5f80fd5b346101f25760207ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc3601126101f257600435907fffffffff0000000000000000000000000000000000000000000000000000000082168092036101f257817f56d981a70000000000000000000000000000000000000000000000000000000060209314908115610288575b5015158152f35b7f01ffc9a70000000000000000000000000000000000000000000000000000000091501483610281565b90601f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0910116810190811067ffffffffffffffff8211176102f357604052565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b805182118015610409575b6103855760018211806103ba575b158015908160011b918204600214171561038d576028018060281161038d5782036103855773ffffffffffffffffffffffffffffffffffffffff92915f61037f92610410565b90921690565b50505f905f90565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b507f30780000000000000000000000000000000000000000000000000000000000007fffff00000000000000000000000000000000000000000000000000000000000060208301511614610339565b505f61032b565b9290926001840180851161038d578311806104c6575b15938415948560011b958604600214171561038d575f94810180911161038d579192905b81831061045a5750505060019190565b9092919360ff6104917fff000000000000000000000000000000000000000000000000000000000000006020888601015116610517565b16600f81116104bb578160041b918083046010149015171561038d5760019101940191929061044a565b505f94508493505050565b507f30780000000000000000000000000000000000000000000000000000000000007fffff000000000000000000000000000000000000000000000000000000000000602086840101511614610426565b60f81c602f8111806105d9575b15610551577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd00160ff1690565b60608111806105cf575b15610588577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa90160ff1690565b60408111806105c5575b156105bf577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc90160ff1690565b5060ff90565b5060478110610592565b506067811061055b565b50603a811061052456fea164736f6c634300081c000a",
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

// ConstructMintCall is a free data retrieval call binding the contract method 0x56d981a7.
//
// Solidity: function constructMintCall(string receiver, uint256 amount) pure returns(bytes)
func (_Contract *ContractCaller) ConstructMintCall(opts *bind.CallOpts, receiver string, amount *big.Int) ([]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "constructMintCall", receiver, amount)

	if err != nil {
		return *new([]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([]byte)).(*[]byte)

	return out0, err

}

// ConstructMintCall is a free data retrieval call binding the contract method 0x56d981a7.
//
// Solidity: function constructMintCall(string receiver, uint256 amount) pure returns(bytes)
func (_Contract *ContractSession) ConstructMintCall(receiver string, amount *big.Int) ([]byte, error) {
	return _Contract.Contract.ConstructMintCall(&_Contract.CallOpts, receiver, amount)
}

// ConstructMintCall is a free data retrieval call binding the contract method 0x56d981a7.
//
// Solidity: function constructMintCall(string receiver, uint256 amount) pure returns(bytes)
func (_Contract *ContractCallerSession) ConstructMintCall(receiver string, amount *big.Int) ([]byte, error) {
	return _Contract.Contract.ConstructMintCall(&_Contract.CallOpts, receiver, amount)
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
