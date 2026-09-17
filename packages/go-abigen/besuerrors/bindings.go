// Code generated via abigen V2 - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package besuerrors

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

// BindingsMetaData contains all meta data concerning the Bindings contract.
var BindingsMetaData = bind.MetaData{
	ABI: "[{\"type\":\"error\",\"name\":\"ConflictingConsensusState\",\"inputs\":[{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"ConsensusStateExpired\",\"inputs\":[{\"name\":\"trustedTimestamp\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"currentTimestamp\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"trustingPeriod\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"ConsensusStateNotFound\",\"inputs\":[{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"ConsensusStatePreimageMismatch\",\"inputs\":[{\"name\":\"expectedHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"actualHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"DuplicateCommitSealSigner\",\"inputs\":[{\"name\":\"signer\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"EmptyValidatorSet\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"HeaderFromFuture\",\"inputs\":[{\"name\":\"currentTimestamp\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"headerTimestamp\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"maxClockDrift\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InsufficientTrustedValidatorOverlap\",\"inputs\":[{\"name\":\"actual\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"required\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InsufficientValidatorQuorum\",\"inputs\":[{\"name\":\"actual\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"required\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidCommitSeal\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidCommitmentValue\",\"inputs\":[{\"name\":\"expectedValue\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"actualValue\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"InvalidDifficulty\",\"inputs\":[{\"name\":\"actualDifficulty\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidECDSASignatureLength\",\"inputs\":[{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidExclusionProof\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidExtraDataFormat\",\"inputs\":[{\"name\":\"itemsLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidHeaderFormat\",\"inputs\":[{\"name\":\"itemsLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidHeaderHeight\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidHeaderTimestamp\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidMixHash\",\"inputs\":[{\"name\":\"actualMixHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"InvalidNonce\",\"inputs\":[{\"name\":\"actualNonce\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]},{\"type\":\"error\",\"name\":\"InvalidOmmersHash\",\"inputs\":[{\"name\":\"actualOmmersHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"type\":\"error\",\"name\":\"InvalidPathLength\",\"inputs\":[{\"name\":\"expectedLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"actualLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"InvalidRevisionNumber\",\"inputs\":[{\"name\":\"providedRevisionNumber\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"InvalidTrustingPeriod\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidValidatorAddress\",\"inputs\":[{\"name\":\"validator\",\"type\":\"address\",\"internalType\":\"address\"}]},{\"type\":\"error\",\"name\":\"InvalidValueLength\",\"inputs\":[{\"name\":\"expectedLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"actualLength\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"StorageRootNotInCache\",\"inputs\":[{\"name\":\"revisionHeight\",\"type\":\"uint64\",\"internalType\":\"uint64\"}]},{\"type\":\"error\",\"name\":\"UnsortedValidatorSet\",\"inputs\":[{\"name\":\"index\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"type\":\"error\",\"name\":\"UnsupportedMisbehaviour\",\"inputs\":[]}]",
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

// UnpackError attempts to decode the provided error data using user-defined
// error definitions.
func (bindings *Bindings) UnpackError(raw []byte) (any, error) {
	if bytes.Equal(raw[:4], bindings.abi.Errors["ConflictingConsensusState"].ID.Bytes()[:4]) {
		return bindings.UnpackConflictingConsensusStateError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["ConsensusStateExpired"].ID.Bytes()[:4]) {
		return bindings.UnpackConsensusStateExpiredError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["ConsensusStateNotFound"].ID.Bytes()[:4]) {
		return bindings.UnpackConsensusStateNotFoundError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["ConsensusStatePreimageMismatch"].ID.Bytes()[:4]) {
		return bindings.UnpackConsensusStatePreimageMismatchError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["DuplicateCommitSealSigner"].ID.Bytes()[:4]) {
		return bindings.UnpackDuplicateCommitSealSignerError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["EmptyValidatorSet"].ID.Bytes()[:4]) {
		return bindings.UnpackEmptyValidatorSetError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["HeaderFromFuture"].ID.Bytes()[:4]) {
		return bindings.UnpackHeaderFromFutureError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InsufficientTrustedValidatorOverlap"].ID.Bytes()[:4]) {
		return bindings.UnpackInsufficientTrustedValidatorOverlapError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InsufficientValidatorQuorum"].ID.Bytes()[:4]) {
		return bindings.UnpackInsufficientValidatorQuorumError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidCommitSeal"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidCommitSealError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidCommitmentValue"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidCommitmentValueError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidDifficulty"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidDifficultyError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidECDSASignatureLength"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidECDSASignatureLengthError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidExclusionProof"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidExclusionProofError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidExtraDataFormat"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidExtraDataFormatError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidHeaderFormat"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidHeaderFormatError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidHeaderHeight"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidHeaderHeightError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidHeaderTimestamp"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidHeaderTimestampError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidMixHash"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidMixHashError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidNonce"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidNonceError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidOmmersHash"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidOmmersHashError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidPathLength"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidPathLengthError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidRevisionNumber"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidRevisionNumberError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidTrustingPeriod"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidTrustingPeriodError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidValidatorAddress"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidValidatorAddressError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["InvalidValueLength"].ID.Bytes()[:4]) {
		return bindings.UnpackInvalidValueLengthError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["StorageRootNotInCache"].ID.Bytes()[:4]) {
		return bindings.UnpackStorageRootNotInCacheError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["UnsortedValidatorSet"].ID.Bytes()[:4]) {
		return bindings.UnpackUnsortedValidatorSetError(raw[4:])
	}
	if bytes.Equal(raw[:4], bindings.abi.Errors["UnsupportedMisbehaviour"].ID.Bytes()[:4]) {
		return bindings.UnpackUnsupportedMisbehaviourError(raw[4:])
	}
	return nil, errors.New("Unknown error")
}

// BindingsConflictingConsensusState represents a ConflictingConsensusState error raised by the Bindings contract.
type BindingsConflictingConsensusState struct {
	RevisionHeight uint64
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error ConflictingConsensusState(uint64 revisionHeight)
func BindingsConflictingConsensusStateErrorID() common.Hash {
	return common.HexToHash("0xc1aea5f771727a12f1fa132a61d693d481402a66b30685d67bb11fd749aabf06")
}

// UnpackConflictingConsensusStateError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error ConflictingConsensusState(uint64 revisionHeight)
func (bindings *Bindings) UnpackConflictingConsensusStateError(raw []byte) (*BindingsConflictingConsensusState, error) {
	out := new(BindingsConflictingConsensusState)
	if err := bindings.abi.UnpackIntoInterface(out, "ConflictingConsensusState", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsConsensusStateExpired represents a ConsensusStateExpired error raised by the Bindings contract.
type BindingsConsensusStateExpired struct {
	TrustedTimestamp uint64
	CurrentTimestamp *big.Int
	TrustingPeriod   uint64
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error ConsensusStateExpired(uint64 trustedTimestamp, uint256 currentTimestamp, uint64 trustingPeriod)
func BindingsConsensusStateExpiredErrorID() common.Hash {
	return common.HexToHash("0xeae357255337224e7fb9f083110bb4a6bdd37d5d382938805ba6285bc9e53bdd")
}

// UnpackConsensusStateExpiredError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error ConsensusStateExpired(uint64 trustedTimestamp, uint256 currentTimestamp, uint64 trustingPeriod)
func (bindings *Bindings) UnpackConsensusStateExpiredError(raw []byte) (*BindingsConsensusStateExpired, error) {
	out := new(BindingsConsensusStateExpired)
	if err := bindings.abi.UnpackIntoInterface(out, "ConsensusStateExpired", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsConsensusStateNotFound represents a ConsensusStateNotFound error raised by the Bindings contract.
type BindingsConsensusStateNotFound struct {
	RevisionHeight uint64
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error ConsensusStateNotFound(uint64 revisionHeight)
func BindingsConsensusStateNotFoundErrorID() common.Hash {
	return common.HexToHash("0xf761631bcaa925e1882ef81e9e14dd39ed16ca43ff5dd107abf05309d7081157")
}

// UnpackConsensusStateNotFoundError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error ConsensusStateNotFound(uint64 revisionHeight)
func (bindings *Bindings) UnpackConsensusStateNotFoundError(raw []byte) (*BindingsConsensusStateNotFound, error) {
	out := new(BindingsConsensusStateNotFound)
	if err := bindings.abi.UnpackIntoInterface(out, "ConsensusStateNotFound", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsConsensusStatePreimageMismatch represents a ConsensusStatePreimageMismatch error raised by the Bindings contract.
type BindingsConsensusStatePreimageMismatch struct {
	ExpectedHash [32]byte
	ActualHash   [32]byte
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error ConsensusStatePreimageMismatch(bytes32 expectedHash, bytes32 actualHash)
func BindingsConsensusStatePreimageMismatchErrorID() common.Hash {
	return common.HexToHash("0xa2f1e16c4b2b29515c884028ad40daeb12bfaec2157d817961908d123f9dc339")
}

// UnpackConsensusStatePreimageMismatchError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error ConsensusStatePreimageMismatch(bytes32 expectedHash, bytes32 actualHash)
func (bindings *Bindings) UnpackConsensusStatePreimageMismatchError(raw []byte) (*BindingsConsensusStatePreimageMismatch, error) {
	out := new(BindingsConsensusStatePreimageMismatch)
	if err := bindings.abi.UnpackIntoInterface(out, "ConsensusStatePreimageMismatch", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsDuplicateCommitSealSigner represents a DuplicateCommitSealSigner error raised by the Bindings contract.
type BindingsDuplicateCommitSealSigner struct {
	Signer common.Address
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error DuplicateCommitSealSigner(address signer)
func BindingsDuplicateCommitSealSignerErrorID() common.Hash {
	return common.HexToHash("0x33af0d5f5b6e9d5e6bcd3744d09854f4cd4fd41459d0b4b6dac048b5636a3b37")
}

// UnpackDuplicateCommitSealSignerError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error DuplicateCommitSealSigner(address signer)
func (bindings *Bindings) UnpackDuplicateCommitSealSignerError(raw []byte) (*BindingsDuplicateCommitSealSigner, error) {
	out := new(BindingsDuplicateCommitSealSigner)
	if err := bindings.abi.UnpackIntoInterface(out, "DuplicateCommitSealSigner", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsEmptyValidatorSet represents a EmptyValidatorSet error raised by the Bindings contract.
type BindingsEmptyValidatorSet struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error EmptyValidatorSet()
func BindingsEmptyValidatorSetErrorID() common.Hash {
	return common.HexToHash("0x339e1ffb1d19111b2d23380664c0d52cafed71303eaf7faa0a11ec5acdb34368")
}

// UnpackEmptyValidatorSetError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error EmptyValidatorSet()
func (bindings *Bindings) UnpackEmptyValidatorSetError(raw []byte) (*BindingsEmptyValidatorSet, error) {
	out := new(BindingsEmptyValidatorSet)
	if err := bindings.abi.UnpackIntoInterface(out, "EmptyValidatorSet", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsHeaderFromFuture represents a HeaderFromFuture error raised by the Bindings contract.
type BindingsHeaderFromFuture struct {
	CurrentTimestamp *big.Int
	HeaderTimestamp  *big.Int
	MaxClockDrift    *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error HeaderFromFuture(uint256 currentTimestamp, uint256 headerTimestamp, uint256 maxClockDrift)
func BindingsHeaderFromFutureErrorID() common.Hash {
	return common.HexToHash("0x9b77227709f1e7746a2948d7f7c2e4bf39689d2942415ad37496f127ce0fb1ac")
}

// UnpackHeaderFromFutureError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error HeaderFromFuture(uint256 currentTimestamp, uint256 headerTimestamp, uint256 maxClockDrift)
func (bindings *Bindings) UnpackHeaderFromFutureError(raw []byte) (*BindingsHeaderFromFuture, error) {
	out := new(BindingsHeaderFromFuture)
	if err := bindings.abi.UnpackIntoInterface(out, "HeaderFromFuture", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInsufficientTrustedValidatorOverlap represents a InsufficientTrustedValidatorOverlap error raised by the Bindings contract.
type BindingsInsufficientTrustedValidatorOverlap struct {
	Actual   *big.Int
	Required *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InsufficientTrustedValidatorOverlap(uint256 actual, uint256 required)
func BindingsInsufficientTrustedValidatorOverlapErrorID() common.Hash {
	return common.HexToHash("0xe2dcb1b943198a6d762396b792a363ca76af0e49913c9495ca7f98b75559b475")
}

// UnpackInsufficientTrustedValidatorOverlapError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InsufficientTrustedValidatorOverlap(uint256 actual, uint256 required)
func (bindings *Bindings) UnpackInsufficientTrustedValidatorOverlapError(raw []byte) (*BindingsInsufficientTrustedValidatorOverlap, error) {
	out := new(BindingsInsufficientTrustedValidatorOverlap)
	if err := bindings.abi.UnpackIntoInterface(out, "InsufficientTrustedValidatorOverlap", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInsufficientValidatorQuorum represents a InsufficientValidatorQuorum error raised by the Bindings contract.
type BindingsInsufficientValidatorQuorum struct {
	Actual   *big.Int
	Required *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InsufficientValidatorQuorum(uint256 actual, uint256 required)
func BindingsInsufficientValidatorQuorumErrorID() common.Hash {
	return common.HexToHash("0x7818f4e49a924e13427444647e781cecfabd65ad72b8163749e192aa3a4b232a")
}

// UnpackInsufficientValidatorQuorumError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InsufficientValidatorQuorum(uint256 actual, uint256 required)
func (bindings *Bindings) UnpackInsufficientValidatorQuorumError(raw []byte) (*BindingsInsufficientValidatorQuorum, error) {
	out := new(BindingsInsufficientValidatorQuorum)
	if err := bindings.abi.UnpackIntoInterface(out, "InsufficientValidatorQuorum", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidCommitSeal represents a InvalidCommitSeal error raised by the Bindings contract.
type BindingsInvalidCommitSeal struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidCommitSeal()
func BindingsInvalidCommitSealErrorID() common.Hash {
	return common.HexToHash("0x09cb44e6ed2e775babda254c07e58897b654648306a381d89344fd5494bbd0da")
}

// UnpackInvalidCommitSealError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidCommitSeal()
func (bindings *Bindings) UnpackInvalidCommitSealError(raw []byte) (*BindingsInvalidCommitSeal, error) {
	out := new(BindingsInvalidCommitSeal)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidCommitSeal", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidCommitmentValue represents a InvalidCommitmentValue error raised by the Bindings contract.
type BindingsInvalidCommitmentValue struct {
	ExpectedValue [32]byte
	ActualValue   [32]byte
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidCommitmentValue(bytes32 expectedValue, bytes32 actualValue)
func BindingsInvalidCommitmentValueErrorID() common.Hash {
	return common.HexToHash("0x56ed1b63a5f431a268c02bef833bf02952c04f6064eaf6c3c7b34cb6d0206434")
}

// UnpackInvalidCommitmentValueError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidCommitmentValue(bytes32 expectedValue, bytes32 actualValue)
func (bindings *Bindings) UnpackInvalidCommitmentValueError(raw []byte) (*BindingsInvalidCommitmentValue, error) {
	out := new(BindingsInvalidCommitmentValue)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidCommitmentValue", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidDifficulty represents a InvalidDifficulty error raised by the Bindings contract.
type BindingsInvalidDifficulty struct {
	ActualDifficulty *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidDifficulty(uint256 actualDifficulty)
func BindingsInvalidDifficultyErrorID() common.Hash {
	return common.HexToHash("0xf71b722fdec985e9915d1440eb300382d619fba59ee6160625d69585e173886b")
}

// UnpackInvalidDifficultyError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidDifficulty(uint256 actualDifficulty)
func (bindings *Bindings) UnpackInvalidDifficultyError(raw []byte) (*BindingsInvalidDifficulty, error) {
	out := new(BindingsInvalidDifficulty)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidDifficulty", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidECDSASignatureLength represents a InvalidECDSASignatureLength error raised by the Bindings contract.
type BindingsInvalidECDSASignatureLength struct {
	Length *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidECDSASignatureLength(uint256 length)
func BindingsInvalidECDSASignatureLengthErrorID() common.Hash {
	return common.HexToHash("0xf2db9ce15d5046279bde848428f3dded093f8376e6d8775f8b536bae7d72f106")
}

// UnpackInvalidECDSASignatureLengthError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidECDSASignatureLength(uint256 length)
func (bindings *Bindings) UnpackInvalidECDSASignatureLengthError(raw []byte) (*BindingsInvalidECDSASignatureLength, error) {
	out := new(BindingsInvalidECDSASignatureLength)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidECDSASignatureLength", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidExclusionProof represents a InvalidExclusionProof error raised by the Bindings contract.
type BindingsInvalidExclusionProof struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidExclusionProof()
func BindingsInvalidExclusionProofErrorID() common.Hash {
	return common.HexToHash("0x47418ca8b59e9d3c2ff85b968fb7e98191205c83472e533cce75865df859f11c")
}

// UnpackInvalidExclusionProofError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidExclusionProof()
func (bindings *Bindings) UnpackInvalidExclusionProofError(raw []byte) (*BindingsInvalidExclusionProof, error) {
	out := new(BindingsInvalidExclusionProof)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidExclusionProof", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidExtraDataFormat represents a InvalidExtraDataFormat error raised by the Bindings contract.
type BindingsInvalidExtraDataFormat struct {
	ItemsLength *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidExtraDataFormat(uint256 itemsLength)
func BindingsInvalidExtraDataFormatErrorID() common.Hash {
	return common.HexToHash("0x30c6792a4a036028880fa7af6f6652ee2bdd6b2d0d49f2e71c1f1aa971667480")
}

// UnpackInvalidExtraDataFormatError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidExtraDataFormat(uint256 itemsLength)
func (bindings *Bindings) UnpackInvalidExtraDataFormatError(raw []byte) (*BindingsInvalidExtraDataFormat, error) {
	out := new(BindingsInvalidExtraDataFormat)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidExtraDataFormat", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidHeaderFormat represents a InvalidHeaderFormat error raised by the Bindings contract.
type BindingsInvalidHeaderFormat struct {
	ItemsLength *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidHeaderFormat(uint256 itemsLength)
func BindingsInvalidHeaderFormatErrorID() common.Hash {
	return common.HexToHash("0xbc2777328c368ec20f7301ae50d66ad5e52fa03b25c8363f4abea65d8db84236")
}

// UnpackInvalidHeaderFormatError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidHeaderFormat(uint256 itemsLength)
func (bindings *Bindings) UnpackInvalidHeaderFormatError(raw []byte) (*BindingsInvalidHeaderFormat, error) {
	out := new(BindingsInvalidHeaderFormat)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidHeaderFormat", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidHeaderHeight represents a InvalidHeaderHeight error raised by the Bindings contract.
type BindingsInvalidHeaderHeight struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidHeaderHeight()
func BindingsInvalidHeaderHeightErrorID() common.Hash {
	return common.HexToHash("0x59d119e3f4f6c0fdfb65bb869e8dfe7274ed9b466704fd6c6a993a71a8d280e1")
}

// UnpackInvalidHeaderHeightError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidHeaderHeight()
func (bindings *Bindings) UnpackInvalidHeaderHeightError(raw []byte) (*BindingsInvalidHeaderHeight, error) {
	out := new(BindingsInvalidHeaderHeight)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidHeaderHeight", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidHeaderTimestamp represents a InvalidHeaderTimestamp error raised by the Bindings contract.
type BindingsInvalidHeaderTimestamp struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidHeaderTimestamp()
func BindingsInvalidHeaderTimestampErrorID() common.Hash {
	return common.HexToHash("0x574f4e610b28d7c09ace791af6f5d2def2007843dfb6a4ea829d24d0f114c51e")
}

// UnpackInvalidHeaderTimestampError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidHeaderTimestamp()
func (bindings *Bindings) UnpackInvalidHeaderTimestampError(raw []byte) (*BindingsInvalidHeaderTimestamp, error) {
	out := new(BindingsInvalidHeaderTimestamp)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidHeaderTimestamp", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidMixHash represents a InvalidMixHash error raised by the Bindings contract.
type BindingsInvalidMixHash struct {
	ActualMixHash [32]byte
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidMixHash(bytes32 actualMixHash)
func BindingsInvalidMixHashErrorID() common.Hash {
	return common.HexToHash("0xa19e609ce41eb9fa5cafdbcc98da09cf7f5b83a1c468a158538d97573c52adbb")
}

// UnpackInvalidMixHashError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidMixHash(bytes32 actualMixHash)
func (bindings *Bindings) UnpackInvalidMixHashError(raw []byte) (*BindingsInvalidMixHash, error) {
	out := new(BindingsInvalidMixHash)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidMixHash", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidNonce represents a InvalidNonce error raised by the Bindings contract.
type BindingsInvalidNonce struct {
	ActualNonce []byte
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidNonce(bytes actualNonce)
func BindingsInvalidNonceErrorID() common.Hash {
	return common.HexToHash("0xd5edd2a51a5640d6f31405a1abb684996572dd38fbe29276c2fe3ba56ddd6335")
}

// UnpackInvalidNonceError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidNonce(bytes actualNonce)
func (bindings *Bindings) UnpackInvalidNonceError(raw []byte) (*BindingsInvalidNonce, error) {
	out := new(BindingsInvalidNonce)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidNonce", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidOmmersHash represents a InvalidOmmersHash error raised by the Bindings contract.
type BindingsInvalidOmmersHash struct {
	ActualOmmersHash [32]byte
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidOmmersHash(bytes32 actualOmmersHash)
func BindingsInvalidOmmersHashErrorID() common.Hash {
	return common.HexToHash("0xb8120dcafb14ba94497dce3a5870b1e8a5923fff1da1b068a53bd4ceec3e07c6")
}

// UnpackInvalidOmmersHashError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidOmmersHash(bytes32 actualOmmersHash)
func (bindings *Bindings) UnpackInvalidOmmersHashError(raw []byte) (*BindingsInvalidOmmersHash, error) {
	out := new(BindingsInvalidOmmersHash)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidOmmersHash", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidPathLength represents a InvalidPathLength error raised by the Bindings contract.
type BindingsInvalidPathLength struct {
	ExpectedLength *big.Int
	ActualLength   *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidPathLength(uint256 expectedLength, uint256 actualLength)
func BindingsInvalidPathLengthErrorID() common.Hash {
	return common.HexToHash("0x88b3170e84c54197122bc51eeb919468ec158ea79d8e6a7dad0147dc9818dc07")
}

// UnpackInvalidPathLengthError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidPathLength(uint256 expectedLength, uint256 actualLength)
func (bindings *Bindings) UnpackInvalidPathLengthError(raw []byte) (*BindingsInvalidPathLength, error) {
	out := new(BindingsInvalidPathLength)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidPathLength", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidRevisionNumber represents a InvalidRevisionNumber error raised by the Bindings contract.
type BindingsInvalidRevisionNumber struct {
	ProvidedRevisionNumber uint64
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidRevisionNumber(uint64 providedRevisionNumber)
func BindingsInvalidRevisionNumberErrorID() common.Hash {
	return common.HexToHash("0x3ce598a93a7420334ba6e91c82fa516215f36710a09c51b457b6e84f460c78e3")
}

// UnpackInvalidRevisionNumberError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidRevisionNumber(uint64 providedRevisionNumber)
func (bindings *Bindings) UnpackInvalidRevisionNumberError(raw []byte) (*BindingsInvalidRevisionNumber, error) {
	out := new(BindingsInvalidRevisionNumber)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidRevisionNumber", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidTrustingPeriod represents a InvalidTrustingPeriod error raised by the Bindings contract.
type BindingsInvalidTrustingPeriod struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidTrustingPeriod()
func BindingsInvalidTrustingPeriodErrorID() common.Hash {
	return common.HexToHash("0x5dd52a2d3edae5ed60f24921016df3c19b95f5fd9b3e67722d0c6c13f66b0119")
}

// UnpackInvalidTrustingPeriodError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidTrustingPeriod()
func (bindings *Bindings) UnpackInvalidTrustingPeriodError(raw []byte) (*BindingsInvalidTrustingPeriod, error) {
	out := new(BindingsInvalidTrustingPeriod)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidTrustingPeriod", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidValidatorAddress represents a InvalidValidatorAddress error raised by the Bindings contract.
type BindingsInvalidValidatorAddress struct {
	Validator common.Address
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidValidatorAddress(address validator)
func BindingsInvalidValidatorAddressErrorID() common.Hash {
	return common.HexToHash("0x59bff38791e0527c457b9f92d7aeb8e4673e6d7955e297bd9ca00b9b2d047b38")
}

// UnpackInvalidValidatorAddressError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidValidatorAddress(address validator)
func (bindings *Bindings) UnpackInvalidValidatorAddressError(raw []byte) (*BindingsInvalidValidatorAddress, error) {
	out := new(BindingsInvalidValidatorAddress)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidValidatorAddress", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsInvalidValueLength represents a InvalidValueLength error raised by the Bindings contract.
type BindingsInvalidValueLength struct {
	ExpectedLength *big.Int
	ActualLength   *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error InvalidValueLength(uint256 expectedLength, uint256 actualLength)
func BindingsInvalidValueLengthErrorID() common.Hash {
	return common.HexToHash("0x24fadac8b67aec3983002d02e787383bb44e00bf0d3b87b93bcff79616faa65f")
}

// UnpackInvalidValueLengthError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error InvalidValueLength(uint256 expectedLength, uint256 actualLength)
func (bindings *Bindings) UnpackInvalidValueLengthError(raw []byte) (*BindingsInvalidValueLength, error) {
	out := new(BindingsInvalidValueLength)
	if err := bindings.abi.UnpackIntoInterface(out, "InvalidValueLength", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsStorageRootNotInCache represents a StorageRootNotInCache error raised by the Bindings contract.
type BindingsStorageRootNotInCache struct {
	RevisionHeight uint64
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error StorageRootNotInCache(uint64 revisionHeight)
func BindingsStorageRootNotInCacheErrorID() common.Hash {
	return common.HexToHash("0x0adf8e9533b8eeaea6e4b02e62ef7586ba69b1c2872287253a8226e6e1b376de")
}

// UnpackStorageRootNotInCacheError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error StorageRootNotInCache(uint64 revisionHeight)
func (bindings *Bindings) UnpackStorageRootNotInCacheError(raw []byte) (*BindingsStorageRootNotInCache, error) {
	out := new(BindingsStorageRootNotInCache)
	if err := bindings.abi.UnpackIntoInterface(out, "StorageRootNotInCache", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsUnsortedValidatorSet represents a UnsortedValidatorSet error raised by the Bindings contract.
type BindingsUnsortedValidatorSet struct {
	Index *big.Int
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error UnsortedValidatorSet(uint256 index)
func BindingsUnsortedValidatorSetErrorID() common.Hash {
	return common.HexToHash("0x85d54b78174a2b3845b2b6f706172c9f382b667ef1f0a0a124425ec94f9f5016")
}

// UnpackUnsortedValidatorSetError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error UnsortedValidatorSet(uint256 index)
func (bindings *Bindings) UnpackUnsortedValidatorSetError(raw []byte) (*BindingsUnsortedValidatorSet, error) {
	out := new(BindingsUnsortedValidatorSet)
	if err := bindings.abi.UnpackIntoInterface(out, "UnsortedValidatorSet", raw); err != nil {
		return nil, err
	}
	return out, nil
}

// BindingsUnsupportedMisbehaviour represents a UnsupportedMisbehaviour error raised by the Bindings contract.
type BindingsUnsupportedMisbehaviour struct {
}

// ErrorID returns the hash of canonical representation of the error's signature.
//
// Solidity: error UnsupportedMisbehaviour()
func BindingsUnsupportedMisbehaviourErrorID() common.Hash {
	return common.HexToHash("0x2c1ed462fc3e85857a02759ec28f8d9fbdd690e456c1e0bdc057f89d0c421496")
}

// UnpackUnsupportedMisbehaviourError is the Go binding used to decode the provided
// error data into the corresponding Go error struct.
//
// Solidity: error UnsupportedMisbehaviour()
func (bindings *Bindings) UnpackUnsupportedMisbehaviourError(raw []byte) (*BindingsUnsupportedMisbehaviour, error) {
	out := new(BindingsUnsupportedMisbehaviour)
	if err := bindings.abi.UnpackIntoInterface(out, "UnsupportedMisbehaviour", raw); err != nil {
		return nil, err
	}
	return out, nil
}
