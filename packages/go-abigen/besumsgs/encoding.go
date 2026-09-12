// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package besumsgs

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

// IBesuLightClientMsgsClientState is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsClientState struct {
	IbcRouter      common.Address
	LatestHeight   IICS02ClientMsgsHeight
	TrustingPeriod uint64
	MaxClockDrift  uint64
}

// IBesuLightClientMsgsConsensusState is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsConsensusState struct {
	Timestamp   uint64
	StorageRoot [32]byte
	Validators  []common.Address
}

// IBesuLightClientMsgsMembershipProof is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsMembershipProof struct {
	ConsensusStatePreimage IBesuLightClientMsgsConsensusState
	ProofNodes             [][]byte
}

// IBesuLightClientMsgsMsgUpdateClient is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsMsgUpdateClient struct {
	HeaderRlp              []byte
	TrustedHeight          IICS02ClientMsgsHeight
	ConsensusStatePreimage IBesuLightClientMsgsConsensusState
	AccountProof           []byte
}

// IICS02ClientMsgsHeight is an auto generated low-level Go binding around an user-defined struct.
type IICS02ClientMsgsHeight struct {
	RevisionNumber uint64
	RevisionHeight uint64
}

// EncodingMetaData contains all meta data concerning the Encoding contract.
var EncodingMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"function\",\"name\":\"clientState\",\"inputs\":[{\"name\":\"state\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ClientState\",\"components\":[{\"name\":\"ibcRouter\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"latestHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"trustingPeriod\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"maxClockDrift\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"consensusState\",\"inputs\":[{\"name\":\"state\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"storageRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"membershipProof\",\"inputs\":[{\"name\":\"proof\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MembershipProof\",\"components\":[{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"storageRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]},{\"name\":\"proofNodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"proofNodes\",\"inputs\":[{\"name\":\"nodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"message\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MsgUpdateClient\",\"components\":[{\"name\":\"headerRlp\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"trustedHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"storageRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]},{\"name\":\"accountProof\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"}]",
}

// EncodingABI is the input ABI used to generate the binding from.
// Deprecated: Use EncodingMetaData.ABI instead.
var EncodingABI = EncodingMetaData.ABI

// Encoding is an auto generated Go binding around an Ethereum contract.
type Encoding struct {
	EncodingCaller     // Read-only binding to the contract
	EncodingTransactor // Write-only binding to the contract
	EncodingFilterer   // Log filterer for contract events
}

// EncodingCaller is an auto generated read-only Go binding around an Ethereum contract.
type EncodingCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// EncodingTransactor is an auto generated write-only Go binding around an Ethereum contract.
type EncodingTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// EncodingFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type EncodingFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// EncodingSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type EncodingSession struct {
	Contract     *Encoding         // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// EncodingCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type EncodingCallerSession struct {
	Contract *EncodingCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts   // Call options to use throughout this session
}

// EncodingTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type EncodingTransactorSession struct {
	Contract     *EncodingTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// EncodingRaw is an auto generated low-level Go binding around an Ethereum contract.
type EncodingRaw struct {
	Contract *Encoding // Generic contract binding to access the raw methods on
}

// EncodingCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type EncodingCallerRaw struct {
	Contract *EncodingCaller // Generic read-only contract binding to access the raw methods on
}

// EncodingTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type EncodingTransactorRaw struct {
	Contract *EncodingTransactor // Generic write-only contract binding to access the raw methods on
}

// NewEncoding creates a new instance of Encoding, bound to a specific deployed contract.
func NewEncoding(address common.Address, backend bind.ContractBackend) (*Encoding, error) {
	contract, err := bindEncoding(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &Encoding{EncodingCaller: EncodingCaller{contract: contract}, EncodingTransactor: EncodingTransactor{contract: contract}, EncodingFilterer: EncodingFilterer{contract: contract}}, nil
}

// NewEncodingCaller creates a new read-only instance of Encoding, bound to a specific deployed contract.
func NewEncodingCaller(address common.Address, caller bind.ContractCaller) (*EncodingCaller, error) {
	contract, err := bindEncoding(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &EncodingCaller{contract: contract}, nil
}

// NewEncodingTransactor creates a new write-only instance of Encoding, bound to a specific deployed contract.
func NewEncodingTransactor(address common.Address, transactor bind.ContractTransactor) (*EncodingTransactor, error) {
	contract, err := bindEncoding(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &EncodingTransactor{contract: contract}, nil
}

// NewEncodingFilterer creates a new log filterer instance of Encoding, bound to a specific deployed contract.
func NewEncodingFilterer(address common.Address, filterer bind.ContractFilterer) (*EncodingFilterer, error) {
	contract, err := bindEncoding(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &EncodingFilterer{contract: contract}, nil
}

// bindEncoding binds a generic wrapper to an already deployed contract.
func bindEncoding(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := EncodingMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Encoding *EncodingRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Encoding.Contract.EncodingCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Encoding *EncodingRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Encoding.Contract.EncodingTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Encoding *EncodingRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Encoding.Contract.EncodingTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Encoding *EncodingCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Encoding.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Encoding *EncodingTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Encoding.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Encoding *EncodingTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Encoding.Contract.contract.Transact(opts, method, params...)
}

// ClientState is a free data retrieval call binding the contract method 0xd7f6d9e1.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64) state) pure returns()
func (_Encoding *EncodingCaller) ClientState(opts *bind.CallOpts, state IBesuLightClientMsgsClientState) error {
	var out []interface{}
	err := _Encoding.contract.Call(opts, &out, "clientState", state)

	if err != nil {
		return err
	}

	return err

}

// ClientState is a free data retrieval call binding the contract method 0xd7f6d9e1.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64) state) pure returns()
func (_Encoding *EncodingSession) ClientState(state IBesuLightClientMsgsClientState) error {
	return _Encoding.Contract.ClientState(&_Encoding.CallOpts, state)
}

// ClientState is a free data retrieval call binding the contract method 0xd7f6d9e1.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64) state) pure returns()
func (_Encoding *EncodingCallerSession) ClientState(state IBesuLightClientMsgsClientState) error {
	return _Encoding.Contract.ClientState(&_Encoding.CallOpts, state)
}

// ConsensusState is a free data retrieval call binding the contract method 0xc6fe84b1.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns()
func (_Encoding *EncodingCaller) ConsensusState(opts *bind.CallOpts, state IBesuLightClientMsgsConsensusState) error {
	var out []interface{}
	err := _Encoding.contract.Call(opts, &out, "consensusState", state)

	if err != nil {
		return err
	}

	return err

}

// ConsensusState is a free data retrieval call binding the contract method 0xc6fe84b1.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns()
func (_Encoding *EncodingSession) ConsensusState(state IBesuLightClientMsgsConsensusState) error {
	return _Encoding.Contract.ConsensusState(&_Encoding.CallOpts, state)
}

// ConsensusState is a free data retrieval call binding the contract method 0xc6fe84b1.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns()
func (_Encoding *EncodingCallerSession) ConsensusState(state IBesuLightClientMsgsConsensusState) error {
	return _Encoding.Contract.ConsensusState(&_Encoding.CallOpts, state)
}

// MembershipProof is a free data retrieval call binding the contract method 0x7feb0512.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[]) proof) pure returns()
func (_Encoding *EncodingCaller) MembershipProof(opts *bind.CallOpts, proof IBesuLightClientMsgsMembershipProof) error {
	var out []interface{}
	err := _Encoding.contract.Call(opts, &out, "membershipProof", proof)

	if err != nil {
		return err
	}

	return err

}

// MembershipProof is a free data retrieval call binding the contract method 0x7feb0512.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[]) proof) pure returns()
func (_Encoding *EncodingSession) MembershipProof(proof IBesuLightClientMsgsMembershipProof) error {
	return _Encoding.Contract.MembershipProof(&_Encoding.CallOpts, proof)
}

// MembershipProof is a free data retrieval call binding the contract method 0x7feb0512.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[]) proof) pure returns()
func (_Encoding *EncodingCallerSession) MembershipProof(proof IBesuLightClientMsgsMembershipProof) error {
	return _Encoding.Contract.MembershipProof(&_Encoding.CallOpts, proof)
}

// ProofNodes is a free data retrieval call binding the contract method 0x0cab8d85.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns()
func (_Encoding *EncodingCaller) ProofNodes(opts *bind.CallOpts, nodes [][]byte) error {
	var out []interface{}
	err := _Encoding.contract.Call(opts, &out, "proofNodes", nodes)

	if err != nil {
		return err
	}

	return err

}

// ProofNodes is a free data retrieval call binding the contract method 0x0cab8d85.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns()
func (_Encoding *EncodingSession) ProofNodes(nodes [][]byte) error {
	return _Encoding.Contract.ProofNodes(&_Encoding.CallOpts, nodes)
}

// ProofNodes is a free data retrieval call binding the contract method 0x0cab8d85.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns()
func (_Encoding *EncodingCallerSession) ProofNodes(nodes [][]byte) error {
	return _Encoding.Contract.ProofNodes(&_Encoding.CallOpts, nodes)
}

// UpdateClient is a free data retrieval call binding the contract method 0x13823715.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[]),bytes) message) pure returns()
func (_Encoding *EncodingCaller) UpdateClient(opts *bind.CallOpts, message IBesuLightClientMsgsMsgUpdateClient) error {
	var out []interface{}
	err := _Encoding.contract.Call(opts, &out, "updateClient", message)

	if err != nil {
		return err
	}

	return err

}

// UpdateClient is a free data retrieval call binding the contract method 0x13823715.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[]),bytes) message) pure returns()
func (_Encoding *EncodingSession) UpdateClient(message IBesuLightClientMsgsMsgUpdateClient) error {
	return _Encoding.Contract.UpdateClient(&_Encoding.CallOpts, message)
}

// UpdateClient is a free data retrieval call binding the contract method 0x13823715.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[]),bytes) message) pure returns()
func (_Encoding *EncodingCallerSession) UpdateClient(message IBesuLightClientMsgsMsgUpdateClient) error {
	return _Encoding.Contract.UpdateClient(&_Encoding.CallOpts, message)
}
