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
	IsFrozen       bool
}

// IBesuLightClientMsgsConsensusState is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsConsensusState struct {
	Timestamp  uint64
	StateRoot  [32]byte
	Validators []common.Address
}

// IBesuLightClientMsgsMembershipProof is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsMembershipProof struct {
	ConsensusStatePreimage IBesuLightClientMsgsConsensusState
	AccountProofNodes      [][]byte
	ProofNodes             [][]byte
}

// IBesuLightClientMsgsMsgUpdateClient is an auto generated low-level Go binding around an user-defined struct.
type IBesuLightClientMsgsMsgUpdateClient struct {
	HeaderRlp              []byte
	TrustedHeight          IICS02ClientMsgsHeight
	ConsensusStatePreimage IBesuLightClientMsgsConsensusState
}

// IICS02ClientMsgsHeight is an auto generated low-level Go binding around an user-defined struct.
type IICS02ClientMsgsHeight struct {
	RevisionNumber uint64
	RevisionHeight uint64
}

// BindingsMetaData contains all meta data concerning the Bindings contract.
var BindingsMetaData = bind.MetaData{
	ABI: "[{\"type\":\"function\",\"name\":\"clientState\",\"inputs\":[{\"name\":\"state\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ClientState\",\"components\":[{\"name\":\"ibcRouter\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"latestHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"trustingPeriod\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"maxClockDrift\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"isFrozen\",\"type\":\"bool\",\"internalType\":\"bool\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ClientState\",\"components\":[{\"name\":\"ibcRouter\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"latestHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"trustingPeriod\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"maxClockDrift\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"isFrozen\",\"type\":\"bool\",\"internalType\":\"bool\"}]}],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"consensusState\",\"inputs\":[{\"name\":\"state\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]}],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"membershipProof\",\"inputs\":[{\"name\":\"proof\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MembershipProof\",\"components\":[{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]},{\"name\":\"accountProofNodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"proofNodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MembershipProof\",\"components\":[{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]},{\"name\":\"accountProofNodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"proofNodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}]}],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"proofNodes\",\"inputs\":[{\"name\":\"nodes\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"}],\"stateMutability\":\"pure\"},{\"type\":\"function\",\"name\":\"updateClient\",\"inputs\":[{\"name\":\"message\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MsgUpdateClient\",\"components\":[{\"name\":\"headerRlp\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"trustedHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]}]}],\"outputs\":[{\"name\":\"\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.MsgUpdateClient\",\"components\":[{\"name\":\"headerRlp\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"trustedHeight\",\"type\":\"tuple\",\"internalType\":\"structIICS02ClientMsgs.Height\",\"components\":[{\"name\":\"revisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"name\":\"consensusStatePreimage\",\"type\":\"tuple\",\"internalType\":\"structIBesuLightClientMsgs.ConsensusState\",\"components\":[{\"name\":\"timestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"validators\",\"type\":\"address[]\",\"internalType\":\"address[]\"}]}]}],\"stateMutability\":\"pure\"}]",
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
// the contract method with ID 0xb0b3c567.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64,bool) state) pure returns((address,(uint64,uint64),uint64,uint64,bool))
func (bindings *Bindings) PackClientState(state IBesuLightClientMsgsClientState) []byte {
	enc, err := bindings.abi.Pack("clientState", state)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackClientState is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xb0b3c567.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64,bool) state) pure returns((address,(uint64,uint64),uint64,uint64,bool))
func (bindings *Bindings) TryPackClientState(state IBesuLightClientMsgsClientState) ([]byte, error) {
	return bindings.abi.Pack("clientState", state)
}

// UnpackClientState is the Go binding that unpacks the parameters returned
// from invoking the contract method with ID 0xb0b3c567.
//
// Solidity: function clientState((address,(uint64,uint64),uint64,uint64,bool) state) pure returns((address,(uint64,uint64),uint64,uint64,bool))
func (bindings *Bindings) UnpackClientState(data []byte) (IBesuLightClientMsgsClientState, error) {
	out, err := bindings.abi.Unpack("clientState", data)
	if err != nil {
		return *new(IBesuLightClientMsgsClientState), err
	}
	out0 := *abi.ConvertType(out[0], new(IBesuLightClientMsgsClientState)).(*IBesuLightClientMsgsClientState)
	return out0, nil
}

// PackConsensusState is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xc6fe84b1.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns((uint64,bytes32,address[]))
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
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns((uint64,bytes32,address[]))
func (bindings *Bindings) TryPackConsensusState(state IBesuLightClientMsgsConsensusState) ([]byte, error) {
	return bindings.abi.Pack("consensusState", state)
}

// UnpackConsensusState is the Go binding that unpacks the parameters returned
// from invoking the contract method with ID 0xc6fe84b1.
//
// Solidity: function consensusState((uint64,bytes32,address[]) state) pure returns((uint64,bytes32,address[]))
func (bindings *Bindings) UnpackConsensusState(data []byte) (IBesuLightClientMsgsConsensusState, error) {
	out, err := bindings.abi.Unpack("consensusState", data)
	if err != nil {
		return *new(IBesuLightClientMsgsConsensusState), err
	}
	out0 := *abi.ConvertType(out[0], new(IBesuLightClientMsgsConsensusState)).(*IBesuLightClientMsgsConsensusState)
	return out0, nil
}

// PackMembershipProof is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x723e4e38.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[],bytes[]) proof) pure returns(((uint64,bytes32,address[]),bytes[],bytes[]))
func (bindings *Bindings) PackMembershipProof(proof IBesuLightClientMsgsMembershipProof) []byte {
	enc, err := bindings.abi.Pack("membershipProof", proof)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackMembershipProof is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x723e4e38.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[],bytes[]) proof) pure returns(((uint64,bytes32,address[]),bytes[],bytes[]))
func (bindings *Bindings) TryPackMembershipProof(proof IBesuLightClientMsgsMembershipProof) ([]byte, error) {
	return bindings.abi.Pack("membershipProof", proof)
}

// UnpackMembershipProof is the Go binding that unpacks the parameters returned
// from invoking the contract method with ID 0x723e4e38.
//
// Solidity: function membershipProof(((uint64,bytes32,address[]),bytes[],bytes[]) proof) pure returns(((uint64,bytes32,address[]),bytes[],bytes[]))
func (bindings *Bindings) UnpackMembershipProof(data []byte) (IBesuLightClientMsgsMembershipProof, error) {
	out, err := bindings.abi.Unpack("membershipProof", data)
	if err != nil {
		return *new(IBesuLightClientMsgsMembershipProof), err
	}
	out0 := *abi.ConvertType(out[0], new(IBesuLightClientMsgsMembershipProof)).(*IBesuLightClientMsgsMembershipProof)
	return out0, nil
}

// PackProofNodes is the Go binding used to pack the parameters required for calling
// the contract method with ID 0x0cab8d85.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns(bytes[])
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
// Solidity: function proofNodes(bytes[] nodes) pure returns(bytes[])
func (bindings *Bindings) TryPackProofNodes(nodes [][]byte) ([]byte, error) {
	return bindings.abi.Pack("proofNodes", nodes)
}

// UnpackProofNodes is the Go binding that unpacks the parameters returned
// from invoking the contract method with ID 0x0cab8d85.
//
// Solidity: function proofNodes(bytes[] nodes) pure returns(bytes[])
func (bindings *Bindings) UnpackProofNodes(data []byte) ([][]byte, error) {
	out, err := bindings.abi.Unpack("proofNodes", data)
	if err != nil {
		return *new([][]byte), err
	}
	out0 := *abi.ConvertType(out[0], new([][]byte)).(*[][]byte)
	return out0, nil
}

// PackUpdateClient is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xa9ca8472.  This method will panic if any
// invalid/nil inputs are passed.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[])) message) pure returns((bytes,(uint64,uint64),(uint64,bytes32,address[])))
func (bindings *Bindings) PackUpdateClient(message IBesuLightClientMsgsMsgUpdateClient) []byte {
	enc, err := bindings.abi.Pack("updateClient", message)
	if err != nil {
		panic(err)
	}
	return enc
}

// TryPackUpdateClient is the Go binding used to pack the parameters required for calling
// the contract method with ID 0xa9ca8472.  This method will return an error
// if any inputs are invalid/nil.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[])) message) pure returns((bytes,(uint64,uint64),(uint64,bytes32,address[])))
func (bindings *Bindings) TryPackUpdateClient(message IBesuLightClientMsgsMsgUpdateClient) ([]byte, error) {
	return bindings.abi.Pack("updateClient", message)
}

// UnpackUpdateClient is the Go binding that unpacks the parameters returned
// from invoking the contract method with ID 0xa9ca8472.
//
// Solidity: function updateClient((bytes,(uint64,uint64),(uint64,bytes32,address[])) message) pure returns((bytes,(uint64,uint64),(uint64,bytes32,address[])))
func (bindings *Bindings) UnpackUpdateClient(data []byte) (IBesuLightClientMsgsMsgUpdateClient, error) {
	out, err := bindings.abi.Unpack("updateClient", data)
	if err != nil {
		return *new(IBesuLightClientMsgsMsgUpdateClient), err
	}
	out0 := *abi.ConvertType(out[0], new(IBesuLightClientMsgsMsgUpdateClient)).(*IBesuLightClientMsgsMsgUpdateClient)
	return out0, nil
}
