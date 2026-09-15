// Code generated via abigen V2 - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package besumsgs

import (
	"bytes"
	"errors"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind/v2"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
)

// Reference imports to suppress errors if they are not otherwise used.
var (
	_ = bytes.Equal
	_ = errors.New
	_ = big.NewInt
	_ = common.Big1
	_ = types.BloomLookup
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

// BindingsMetaData contains all meta data concerning the Bindings contract.
var BindingsMetaData = bind.MetaData{
	ABI: "[{\"type\":\"function\",\"name\":\"clientState\",\"inputs\":[{\"name\":\"state\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ClientState\",\"components\":[{\"name\":\"ibcRouter\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"latestHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"trustingPeriod\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"maxClockDrift\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"consensusState\",\"inputs\":[{\"name\":\"state\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"storageRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"membershipProof\",\"inputs\":[{\"name\":\"proof\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MembershipProof\",\"components\":[{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"storageRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]},{\"name\":\"proofNodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"proofNodes\",\"inputs\":[{\"name\":\"nodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"outputs\":[],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"message\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MsgUpdateClient\",\"components\":[{\"name\":\"headerRlp\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"trustedHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"storageRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]},{\"name\":\"accountProof\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}],\"outputs\":[],\"stateMutability\":\"pure\"}]",
	ID:  "Bindings",
}

// Bindings is an auto generated Go binding around an Ethereum contract.
type Bindings struct {
	abi abi.ABI
}

// GetABI returns the ABI associated with this contract binding.
func (c *Bindings) GetABI() abi.ABI {
	return c.abi
}

// NewBindings creates a new instance of Bindings.
func NewBindings() *Bindings {
	parsed, err := BindingsMetaData.ParseABI()
	if err != nil {
		panic(errors.New("invalid ABI: " + err.Error()))
	}
	return &Bindings{abi: *parsed}
}

// Instance creates a wrapper for a deployed contract instance at the given address.
// Use this to create the instance object passed to abigen v2 library functions Call, Transact, etc.
func (c *Bindings) Instance(backend bind.ContractBackend, addr common.Address) *bind.BoundContract {
	return bind.NewBoundContract(addr, c.abi, backend, backend, backend)
}

// PackClientState is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xd7f6d9e1.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64) state) pure returns()
func (bindings *Bindings) PackClientState(state IBesuLightClientMsgsClientState) []byte {
	enc, err := bindings.abi.Pack("clientState", state)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackClientState is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xd7f6d9e1.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64) state) pure returns()
func (bindings *Bindings) TryPackClientState(state IBesuLightClientMsgsClientState) ([]byte, error) {
	return bindings.abi.Pack("clientState", state)
}

// PackConsensusState is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xc6fe84b1.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns()
func (bindings *Bindings) PackConsensusState(state IBesuLightClientMsgsConsensusState) []byte {
	enc, err := bindings.abi.Pack("consensusState", state)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackConsensusState is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xc6fe84b1.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns()
func (bindings *Bindings) TryPackConsensusState(state IBesuLightClientMsgsConsensusState) ([]byte, error) {
	return bindings.abi.Pack("consensusState", state)
}

// PackMembershipProof is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x7feb0512.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[]) proof) pure returns()
func (bindings *Bindings) PackMembershipProof(proof IBesuLightClientMsgsMembershipProof) []byte {
	enc, err := bindings.abi.Pack("membershipProof", proof)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackMembershipProof is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x7feb0512.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[]) proof) pure returns()
func (bindings *Bindings) TryPackMembershipProof(proof IBesuLightClientMsgsMembershipProof) ([]byte, error) {
	return bindings.abi.Pack("membershipProof", proof)
}

// PackProofNodes is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x0cab8d85.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns()
func (bindings *Bindings) PackProofNodes(nodes [][]byte) []byte {
	enc, err := bindings.abi.Pack("proofNodes", nodes)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackProofNodes is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x0cab8d85.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns()
func (bindings *Bindings) TryPackProofNodes(nodes [][]byte) ([]byte, error) {
	return bindings.abi.Pack("proofNodes", nodes)
}

// PackUpdateClient is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x13823715.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[]),bytes) message) pure returns()
func (bindings *Bindings) PackUpdateClient(message IBesuLightClientMsgsMsgUpdateClient) []byte {
	enc, err := bindings.abi.Pack("updateClient", message)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackUpdateClient is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x13823715.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[]),bytes) message) pure returns()
func (bindings *Bindings) TryPackUpdateClient(message IBesuLightClientMsgsMsgUpdateClient) ([]byte, error) {
	return bindings.abi.Pack("updateClient", message)
}
