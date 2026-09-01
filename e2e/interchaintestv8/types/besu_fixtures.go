// SPDX-License-Identifier: Apache-2.0

package types

import (
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	ethcommon "github.com/ethereum/go-ethereum/common"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient/gethclient"
	"github.com/ethereum/go-ethereum/rlp"

	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibchostv2 "github.com/cosmos/ibc-go/v11/modules/core/24-host/v2"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type BesuFixtureGenerator struct {
	Enabled bool
}

type GenerateQBFTFixtureParams struct {
	SourceChain             *ethereum.Ethereum
	RouterAddress           ethcommon.Address
	Packet                  ics26router.IICS26RouterMsgsPacket
	InitialTrustedHeight    uint64
	AdjacentUpdateHeight    uint64
	NonAdjacentUpdateHeight uint64
	SyntheticSourceHeight   uint64
	TrustingPeriod          uint64
	MaxClockDrift           uint64
}

type besuFixture struct {
	RouterAddress             string                     `json:"routerAddress"`
	InitialTrustedHeight      uint64                     `json:"initialTrustedHeight"`
	InitialTrustedTimestamp   uint64                     `json:"initialTrustedTimestamp"`
	InitialTrustedStorageRoot string                     `json:"initialTrustedStorageRoot"`
	InitialTrustedValidators  []string                   `json:"initialTrustedValidators"`
	TrustingPeriod            uint64                     `json:"trustingPeriod"`
	MaxClockDrift             uint64                     `json:"maxClockDrift"`
	AdjacentUpdate            besuUpdateFixture          `json:"adjacentUpdate"`
	NonAdjacentUpdate         besuUpdateFixture          `json:"nonAdjacentUpdate"`
	LowQuorumUpdate           besuRejectionUpdateFixture `json:"lowQuorumUpdate"`
	ConflictingUpdate         besuRejectionUpdateFixture `json:"conflictingUpdate"`
	LowOverlapUpdate          besuRejectionUpdateFixture `json:"lowOverlapUpdate"`
	Membership                besuProofFixture           `json:"membership"`
	NonMembership             besuProofFixture           `json:"nonMembership"`
}

type besuUpdateFixture struct {
	Height              uint64   `json:"height"`
	HeaderRlp           string   `json:"headerRlp"`
	TrustedHeight       uint64   `json:"trustedHeight"`
	AccountProof        string   `json:"accountProof"`
	ExpectedTimestamp   uint64   `json:"expectedTimestamp"`
	ExpectedStorageRoot string   `json:"expectedStorageRoot"`
	ExpectedValidators  []string `json:"expectedValidators"`
}

type besuRejectionUpdateFixture struct {
	Height        uint64 `json:"height"`
	HeaderRlp     string `json:"headerRlp"`
	TrustedHeight uint64 `json:"trustedHeight"`
	AccountProof  string `json:"accountProof"`
}

type besuProofFixture struct {
	Proof             string `json:"proof"`
	ProofHeight       uint64 `json:"proofHeight"`
	Path              string `json:"path"`
	Value             string `json:"value,omitempty"`
	ExpectedTimestamp uint64 `json:"expectedTimestamp"`
}

type liveHeader struct {
	Header     *gethtypes.Header
	HeaderRLP  []byte
	Validators []ethcommon.Address
}

type mutableQBFTHeader struct {
	items      []rlp.RawValue
	extraItems []rlp.RawValue
}

var syntheticLowOverlapValidatorKeys = []string{
	"59c6995e998f97a5a0044966f094538f8e0f1c7f6d0bdf5f4b4a0d5c8fba8f5a",
	"5de4111a39c2d6f5f2f1df6b0ad72037d399a8ce28b5c82853453ea50ccaa43d",
	"7c852118294c1ec93b7d4c7d4b3f3b2d8f5f1e6d5c4b3a291817161514131211",
}

func NewBesuFixtureGenerator() *BesuFixtureGenerator {
	return &BesuFixtureGenerator{
		Enabled: os.Getenv(testvalues.EnvKeyGenerateBesuLightClientFixtures) == testvalues.EnvValueGenerateFixtures_True,
	}
}

func (g *BesuFixtureGenerator) GenerateAndSaveQBFTFixture(ctx context.Context, params GenerateQBFTFixtureParams) error {
	if !g.Enabled {
		return nil
	}

	fixture, err := generateQBFTFixture(ctx, params)
	if err != nil {
		return err
	}

	fixtureBz, err := json.MarshalIndent(fixture, "", "  ")
	if err != nil {
		return err
	}

	fixturePath := filepath.Join(testvalues.BesuBFTFixturesDir, "qbft.json")
	// The checked-in fixture is intentionally readable by all test processes.
	return os.WriteFile(fixturePath, fixtureBz, 0o644) //nolint:gosec
}

func generateQBFTFixture(ctx context.Context, params GenerateQBFTFixtureParams) (besuFixture, error) {
	if params.SourceChain == nil {
		return besuFixture{}, fmt.Errorf("missing source chain")
	}
	if params.InitialTrustedHeight == 0 {
		return besuFixture{}, fmt.Errorf("initial trusted height must be greater than zero")
	}
	if params.AdjacentUpdateHeight <= params.InitialTrustedHeight || params.AdjacentUpdateHeight-params.InitialTrustedHeight != 1 {
		return besuFixture{}, fmt.Errorf("adjacent update height must equal initial trusted height plus one")
	}
	if params.NonAdjacentUpdateHeight <= params.AdjacentUpdateHeight {
		return besuFixture{}, fmt.Errorf("non-adjacent update height must be greater than adjacent update height")
	}
	if params.SyntheticSourceHeight <= params.NonAdjacentUpdateHeight {
		return besuFixture{}, fmt.Errorf("synthetic source height must be greater than non-adjacent update height")
	}

	trustedHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.InitialTrustedHeight)
	if err != nil {
		return besuFixture{}, err
	}
	nonAdjacentHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.NonAdjacentUpdateHeight)
	if err != nil {
		return besuFixture{}, err
	}
	syntheticSourceHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.SyntheticSourceHeight)
	if err != nil {
		return besuFixture{}, err
	}

	validatorKeys, err := loadQBFTValidatorKeys()
	if err != nil {
		return besuFixture{}, err
	}

	trustedProof, _, err := fetchAccountProof(ctx, params.SourceChain, params.RouterAddress, params.InitialTrustedHeight)
	if err != nil {
		return besuFixture{}, err
	}
	adjacentUpdate, err := buildLiveUpdateFixture(ctx, params.SourceChain, params.RouterAddress, params.InitialTrustedHeight, params.AdjacentUpdateHeight)
	if err != nil {
		return besuFixture{}, err
	}
	nonAdjacentUpdate, err := buildLiveUpdateFixture(ctx, params.SourceChain, params.RouterAddress, params.InitialTrustedHeight, params.NonAdjacentUpdateHeight)
	if err != nil {
		return besuFixture{}, err
	}
	lowQuorumUpdate, err := buildLowQuorumFixture(nonAdjacentUpdate, nonAdjacentHeader)
	if err != nil {
		return besuFixture{}, err
	}
	conflictingUpdate, err := buildConflictingFixture(
		ctx,
		params.InitialTrustedHeight,
		nonAdjacentUpdate.Height,
		syntheticSourceHeader,
		validatorKeys,
		params.SourceChain,
		params.RouterAddress,
	)
	if err != nil {
		return besuFixture{}, err
	}
	lowOverlapUpdate, err := buildLowOverlapFixture(
		params.InitialTrustedHeight,
		nonAdjacentUpdate.Height+1,
		syntheticSourceHeader,
		validatorKeys,
	)
	if err != nil {
		return besuFixture{}, err
	}
	membership, err := buildMembershipFixture(ctx, params.SourceChain, params.RouterAddress, params.Packet, params.NonAdjacentUpdateHeight, nonAdjacentHeader.Header.Time)
	if err != nil {
		return besuFixture{}, err
	}
	nonMembership, err := buildNonMembershipFixture(ctx, params.SourceChain, params.RouterAddress, params.Packet, params.NonAdjacentUpdateHeight, nonAdjacentHeader.Header.Time)
	if err != nil {
		return besuFixture{}, err
	}

	return besuFixture{
		RouterAddress:             params.RouterAddress.Hex(),
		InitialTrustedHeight:      params.InitialTrustedHeight,
		InitialTrustedTimestamp:   trustedHeader.Header.Time,
		InitialTrustedStorageRoot: trustedProof.StorageHash.Hex(),
		InitialTrustedValidators:  addressesToHex(trustedHeader.Validators),
		TrustingPeriod:            params.TrustingPeriod,
		MaxClockDrift:             params.MaxClockDrift,
		AdjacentUpdate:            adjacentUpdate,
		NonAdjacentUpdate:         nonAdjacentUpdate,
		LowQuorumUpdate:           lowQuorumUpdate,
		ConflictingUpdate:         conflictingUpdate,
		LowOverlapUpdate:          lowOverlapUpdate,
		Membership:                membership,
		NonMembership:             nonMembership,
	}, nil
}

func buildLiveUpdateFixture(
	ctx context.Context,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
	trustedHeight uint64,
	targetHeight uint64,
) (besuUpdateFixture, error) {
	header, err := fetchLiveHeader(ctx, chain, targetHeight)
	if err != nil {
		return besuUpdateFixture{}, err
	}
	proof, accountProofRLP, err := fetchAccountProof(ctx, chain, routerAddress, targetHeight)
	if err != nil {
		return besuUpdateFixture{}, err
	}

	return besuUpdateFixture{
		Height:              targetHeight,
		HeaderRlp:           encodeHex(header.HeaderRLP),
		TrustedHeight:       trustedHeight,
		AccountProof:        encodeHex(accountProofRLP),
		ExpectedTimestamp:   header.Header.Time,
		ExpectedStorageRoot: proof.StorageHash.Hex(),
		ExpectedValidators:  addressesToHex(header.Validators),
	}, nil
}

func buildLowQuorumFixture(update besuUpdateFixture, header liveHeader) (besuRejectionUpdateFixture, error) {
	mutable, err := decodeMutableQBFTHeader(header.HeaderRLP)
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	commitSeals, err := mutable.commitSeals()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	if len(commitSeals) < 2 {
		return besuRejectionUpdateFixture{}, fmt.Errorf("expected at least two commit seals, got %d", len(commitSeals))
	}
	mutable.setCommitSeals(commitSeals[:2])
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}

	return besuRejectionUpdateFixture{
		Height:        update.Height,
		HeaderRlp:     encodeHex(mutatedHeader),
		TrustedHeight: update.TrustedHeight,
		AccountProof:  "0x",
	}, nil
}

func buildConflictingFixture(
	ctx context.Context,
	trustedHeight uint64,
	targetHeight uint64,
	baseHeader liveHeader,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
) (besuRejectionUpdateFixture, error) {
	mutable, err := decodeMutableQBFTHeader(baseHeader.HeaderRLP)
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	mutable.setHeight(targetHeight)
	validators, err := mutable.validators()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	signerKeys, err := signerKeysFor(validators[:3], validatorKeys)
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	mutable.setCommitSeals(signQBFTCommitSeals(mutable, signerKeys))
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}

	_, accountProofRLP, err := fetchAccountProof(ctx, chain, routerAddress, baseHeader.Header.Number.Uint64())
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}

	return besuRejectionUpdateFixture{
		Height:        targetHeight,
		HeaderRlp:     encodeHex(mutatedHeader),
		TrustedHeight: trustedHeight,
		AccountProof:  encodeHex(accountProofRLP),
	}, nil
}

func buildLowOverlapFixture(
	trustedHeight uint64,
	targetHeight uint64,
	baseHeader liveHeader,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
) (besuRejectionUpdateFixture, error) {
	mutable, err := decodeMutableQBFTHeader(baseHeader.HeaderRLP)
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	mutable.setHeight(targetHeight)

	baseValidators, err := mutable.validators()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	if len(baseValidators) == 0 {
		return besuRejectionUpdateFixture{}, fmt.Errorf("missing base validators")
	}

	syntheticKeys, err := loadSyntheticLowOverlapValidatorKeys()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	lowOverlapValidators := []ethcommon.Address{
		baseValidators[0],
		crypto.PubkeyToAddress(syntheticKeys[0].PublicKey),
		crypto.PubkeyToAddress(syntheticKeys[1].PublicKey),
		crypto.PubkeyToAddress(syntheticKeys[2].PublicKey),
	}
	mutable.setValidators(lowOverlapValidators)

	signerKeys, err := signerKeysFor([]ethcommon.Address{
		lowOverlapValidators[0],
		lowOverlapValidators[1],
		lowOverlapValidators[2],
	}, mergeValidatorKeyMaps(validatorKeys, toKeyMap(syntheticKeys)))
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	mutable.setCommitSeals(signQBFTCommitSeals(mutable, signerKeys))
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}

	return besuRejectionUpdateFixture{
		Height:        targetHeight,
		HeaderRlp:     encodeHex(mutatedHeader),
		TrustedHeight: trustedHeight,
		AccountProof:  "0x",
	}, nil
}

func buildMembershipFixture(
	ctx context.Context,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
	packet ics26router.IICS26RouterMsgsPacket,
	proofHeight uint64,
	expectedTimestamp uint64,
) (besuProofFixture, error) {
	path := ibchostv2.PacketCommitmentKey(packet.SourceClient, packet.Sequence)
	proofRLP, err := fetchStorageProof(ctx, chain, routerAddress, path, proofHeight)
	if err != nil {
		return besuProofFixture{}, err
	}

	return besuProofFixture{
		Proof:             encodeHex(proofRLP),
		ProofHeight:       proofHeight,
		Path:              encodeHex(path),
		Value:             encodeHex(packetCommitment(packet)),
		ExpectedTimestamp: expectedTimestamp,
	}, nil
}

func buildNonMembershipFixture(
	ctx context.Context,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
	packet ics26router.IICS26RouterMsgsPacket,
	proofHeight uint64,
	expectedTimestamp uint64,
) (besuProofFixture, error) {
	path := ibchostv2.PacketReceiptKey(packet.DestClient, packet.Sequence)
	proofRLP, err := fetchStorageProof(ctx, chain, routerAddress, path, proofHeight)
	if err != nil {
		return besuProofFixture{}, err
	}

	return besuProofFixture{
		Proof:             encodeHex(proofRLP),
		ProofHeight:       proofHeight,
		Path:              encodeHex(path),
		ExpectedTimestamp: expectedTimestamp,
	}, nil
}

func fetchLiveHeader(ctx context.Context, chain *ethereum.Ethereum, height uint64) (liveHeader, error) {
	header, err := chain.RPCClient.HeaderByNumber(ctx, newUint64(height))
	if err != nil {
		return liveHeader{}, fmt.Errorf("fetch header at height %d: %w", height, err)
	}
	headerRLP, err := rlp.EncodeToBytes(header)
	if err != nil {
		return liveHeader{}, fmt.Errorf("encode header rlp at height %d: %w", height, err)
	}
	mutable, err := decodeMutableQBFTHeader(headerRLP)
	if err != nil {
		return liveHeader{}, fmt.Errorf("decode header at height %d: %w", height, err)
	}
	validators, err := mutable.validators()
	if err != nil {
		return liveHeader{}, fmt.Errorf("extract validators at height %d: %w", height, err)
	}

	return liveHeader{
		Header:     header,
		HeaderRLP:  headerRLP,
		Validators: validators,
	}, nil
}

func fetchAccountProof(
	ctx context.Context,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
	height uint64,
) (*gethclient.AccountResult, []byte, error) {
	proof, err := gethclient.New(chain.RPCClient.Client()).GetProof(ctx, routerAddress, nil, newUint64(height))
	if err != nil {
		return nil, nil, fmt.Errorf("fetch account proof at height %d: %w", height, err)
	}
	proofRLP, err := encodeProofNodes(proof.AccountProof)
	if err != nil {
		return nil, nil, fmt.Errorf("encode account proof at height %d: %w", height, err)
	}
	return proof, proofRLP, nil
}

func fetchStorageProof(
	ctx context.Context,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
	path []byte,
	height uint64,
) ([]byte, error) {
	storageKey := ethereum.GetCommitmentsStorageKey(path)
	proof, err := gethclient.New(chain.RPCClient.Client()).GetProof(ctx, routerAddress, []string{storageKey.Hex()}, newUint64(height))
	if err != nil {
		return nil, fmt.Errorf("fetch storage proof at height %d: %w", height, err)
	}
	if len(proof.StorageProof) != 1 {
		return nil, fmt.Errorf("expected one storage proof at height %d, got %d", height, len(proof.StorageProof))
	}
	proofRLP, err := encodeProofNodes(proof.StorageProof[0].Proof)
	if err != nil {
		return nil, fmt.Errorf("encode storage proof at height %d: %w", height, err)
	}
	return proofRLP, nil
}

func encodeProofNodes(nodes []string) ([]byte, error) {
	proofNodes := make([][]byte, len(nodes))
	for i, node := range nodes {
		proofNodes[i] = ethcommon.FromHex(node)
	}
	bytesArrayType, err := abi.NewType("bytes[]", "", nil)
	if err != nil {
		return nil, fmt.Errorf("create bytes[] ABI type: %w", err)
	}
	return (abi.Arguments{{Type: bytesArrayType}}).Pack(proofNodes)
}

func packetCommitment(packet ics26router.IICS26RouterMsgsPacket) []byte {
	payloads := make([]channeltypesv2.Payload, len(packet.Payloads))
	for i, payload := range packet.Payloads {
		payloads[i] = channeltypesv2.Payload{
			SourcePort:      payload.SourcePort,
			DestinationPort: payload.DestPort,
			Version:         payload.Version,
			Encoding:        payload.Encoding,
			Value:           payload.Value,
		}
	}
	return channeltypesv2.CommitPacket(channeltypesv2.Packet{
		Sequence:          packet.Sequence,
		SourceClient:      packet.SourceClient,
		DestinationClient: packet.DestClient,
		TimeoutTimestamp:  packet.TimeoutTimestamp,
		Payloads:          payloads,
	})
}

func decodeMutableQBFTHeader(headerRLP []byte) (*mutableQBFTHeader, error) {
	var items []rlp.RawValue
	if err := rlp.DecodeBytes(headerRLP, &items); err != nil {
		return nil, err
	}
	if len(items) < 15 {
		return nil, fmt.Errorf("expected at least 15 header items, got %d", len(items))
	}
	var extraData []byte
	if err := rlp.DecodeBytes(items[12], &extraData); err != nil {
		return nil, err
	}
	var extraItems []rlp.RawValue
	if err := rlp.DecodeBytes(extraData, &extraItems); err != nil {
		return nil, err
	}
	if len(extraItems) != 5 {
		return nil, fmt.Errorf("expected 5 extraData items, got %d", len(extraItems))
	}
	return &mutableQBFTHeader{items: items, extraItems: extraItems}, nil
}

func (h *mutableQBFTHeader) encode() ([]byte, error) {
	extraData, err := rlp.EncodeToBytes(h.extraItems)
	if err != nil {
		return nil, err
	}
	items := cloneRawValues(h.items)
	items[12], err = rlp.EncodeToBytes(extraData)
	if err != nil {
		return nil, err
	}
	return rlp.EncodeToBytes(items)
}

func (h *mutableQBFTHeader) validators() ([]ethcommon.Address, error) {
	var validators []ethcommon.Address
	if err := rlp.DecodeBytes(h.extraItems[1], &validators); err != nil {
		return nil, err
	}
	return validators, nil
}

func (h *mutableQBFTHeader) commitSeals() ([][]byte, error) {
	var seals [][]byte
	if err := rlp.DecodeBytes(h.extraItems[4], &seals); err != nil {
		return nil, err
	}
	return seals, nil
}

func (h *mutableQBFTHeader) setHeight(height uint64) {
	h.items[8] = mustRLP(height)
}

func (h *mutableQBFTHeader) setValidators(validators []ethcommon.Address) {
	h.extraItems[1] = mustRLP(validators)
}

func (h *mutableQBFTHeader) setCommitSeals(seals [][]byte) {
	h.extraItems[4] = mustRLP(seals)
}

func signQBFTCommitSeals(header *mutableQBFTHeader, keys []*ecdsa.PrivateKey) [][]byte {
	digest := header.commitSealDigest()
	seals := make([][]byte, len(keys))
	for i, key := range keys {
		seal, err := crypto.Sign(digest.Bytes(), key)
		if err != nil {
			panic(err)
		}
		seals[i] = seal
	}
	return seals
}

func (h *mutableQBFTHeader) commitSealDigest() ethcommon.Hash {
	signingExtraItems := cloneRawValues(h.extraItems)
	signingExtraItems[4] = rlp.RawValue{0xc0}
	signingExtraData, err := rlp.EncodeToBytes(signingExtraItems)
	if err != nil {
		panic(err)
	}

	items := cloneRawValues(h.items)
	items[12], err = rlp.EncodeToBytes(signingExtraData)
	if err != nil {
		panic(err)
	}
	payload, err := rlp.EncodeToBytes(items)
	if err != nil {
		panic(err)
	}
	return crypto.Keccak256Hash(payload)
}

func loadQBFTValidatorKeys() (map[ethcommon.Address]*ecdsa.PrivateKey, error) {
	validatorKeyPaths := []string{
		"e2e/interchaintestv8/chainconfig/testdata/besu/qbft/keys/validator1/key",
		"e2e/interchaintestv8/chainconfig/testdata/besu/qbft/keys/validator2/key",
		"e2e/interchaintestv8/chainconfig/testdata/besu/qbft/keys/validator3/key",
		"e2e/interchaintestv8/chainconfig/testdata/besu/qbft/keys/validator4/key",
	}

	keys := make(map[ethcommon.Address]*ecdsa.PrivateKey, len(validatorKeyPaths))
	for _, keyPath := range validatorKeyPaths {
		keyHex, err := os.ReadFile(keyPath)
		if err != nil {
			return nil, err
		}
		key, err := crypto.HexToECDSA(strings.TrimPrefix(strings.TrimSpace(string(keyHex)), "0x"))
		if err != nil {
			return nil, err
		}
		keys[crypto.PubkeyToAddress(key.PublicKey)] = key
	}
	return keys, nil
}

func loadSyntheticLowOverlapValidatorKeys() ([]*ecdsa.PrivateKey, error) {
	keys := make([]*ecdsa.PrivateKey, len(syntheticLowOverlapValidatorKeys))
	for i, keyHex := range syntheticLowOverlapValidatorKeys {
		key, err := crypto.HexToECDSA(strings.TrimPrefix(keyHex, "0x"))
		if err != nil {
			return nil, err
		}
		keys[i] = key
	}
	return keys, nil
}

func signerKeysFor(validators []ethcommon.Address, keyMap map[ethcommon.Address]*ecdsa.PrivateKey) ([]*ecdsa.PrivateKey, error) {
	keys := make([]*ecdsa.PrivateKey, len(validators))
	for i, validator := range validators {
		key, ok := keyMap[validator]
		if !ok {
			return nil, fmt.Errorf("missing validator key for %s", validator.Hex())
		}
		keys[i] = key
	}
	return keys, nil
}

func toKeyMap(keys []*ecdsa.PrivateKey) map[ethcommon.Address]*ecdsa.PrivateKey {
	out := make(map[ethcommon.Address]*ecdsa.PrivateKey, len(keys))
	for _, key := range keys {
		out[crypto.PubkeyToAddress(key.PublicKey)] = key
	}
	return out
}

func mergeValidatorKeyMaps(
	primary map[ethcommon.Address]*ecdsa.PrivateKey,
	secondary map[ethcommon.Address]*ecdsa.PrivateKey,
) map[ethcommon.Address]*ecdsa.PrivateKey {
	out := make(map[ethcommon.Address]*ecdsa.PrivateKey, len(primary)+len(secondary))
	for address, key := range primary {
		out[address] = key
	}
	for address, key := range secondary {
		out[address] = key
	}
	return out
}

func cloneRawValues(values []rlp.RawValue) []rlp.RawValue {
	cloned := make([]rlp.RawValue, len(values))
	copy(cloned, values)
	return cloned
}

func mustRLP(value interface{}) rlp.RawValue {
	encoded, err := rlp.EncodeToBytes(value)
	if err != nil {
		panic(err)
	}
	return encoded
}

func addressesToHex(addresses []ethcommon.Address) []string {
	encoded := make([]string, len(addresses))
	for i, address := range addresses {
		encoded[i] = address.Hex()
	}
	return encoded
}

func encodeHex(value []byte) string {
	return "0x" + hex.EncodeToString(value)
}

func newUint64(value uint64) *big.Int {
	return new(big.Int).SetUint64(value)
}
