package types

import (
	"context"
	"crypto/ecdsa"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"strings"

	ethcommon "github.com/ethereum/go-ethereum/common"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rlp"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type BesuFixtureGenerator struct {
	Enabled bool
}

type GenerateQBFTFixtureParams struct {
	SourceChain           *ethereum.Ethereum
	RouterAddress         ethcommon.Address
	Packet                ics26router.IICS26RouterMsgsPacket
	InitialTrustedHeight  uint64
	UpdateHeight11        uint64
	UpdateHeight12        uint64
	SyntheticSourceHeight uint64
	TrustingPeriod        uint64
	MaxClockDrift         uint64
}

type besuFixture struct {
	RouterAddress             string            `json:"routerAddress"`
	InitialTrustedHeight      uint64            `json:"initialTrustedHeight"`
	InitialTrustedTimestamp   uint64            `json:"initialTrustedTimestamp"`
	InitialTrustedStorageRoot string            `json:"initialTrustedStorageRoot"`
	InitialTrustedValidators  []string          `json:"initialTrustedValidators"`
	TrustingPeriod            uint64            `json:"trustingPeriod"`
	MaxClockDrift             uint64            `json:"maxClockDrift"`
	UpdateHeight11            besuUpdateFixture `json:"updateHeight11"`
	UpdateHeight12            besuUpdateFixture `json:"updateHeight12"`
	LowQuorumHeight12         besuUpdateFixture `json:"lowQuorumHeight12"`
	ConflictingHeight12       besuUpdateFixture `json:"conflictingHeight12"`
	LowOverlapHeight13        besuUpdateFixture `json:"lowOverlapHeight13"`
	Membership                besuProofFixture  `json:"membership"`
	NonMembership             besuProofFixture  `json:"nonMembership"`
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
	return os.WriteFile(fixturePath, fixtureBz, 0o644)
}

func generateQBFTFixture(ctx context.Context, params GenerateQBFTFixtureParams) (besuFixture, error) {
	if params.SourceChain == nil {
		return besuFixture{}, fmt.Errorf("missing source chain")
	}
	if params.InitialTrustedHeight == 0 {
		return besuFixture{}, fmt.Errorf("initial trusted height must be greater than zero")
	}
	if params.UpdateHeight11 <= params.InitialTrustedHeight {
		return besuFixture{}, fmt.Errorf("updateHeight11 must be greater than initial trusted height")
	}
	if params.UpdateHeight12 <= params.UpdateHeight11 {
		return besuFixture{}, fmt.Errorf("updateHeight12 must be greater than updateHeight11")
	}
	if params.SyntheticSourceHeight <= params.UpdateHeight12 {
		return besuFixture{}, fmt.Errorf("synthetic source height must be greater than updateHeight12")
	}

	trustedHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.InitialTrustedHeight)
	if err != nil {
		return besuFixture{}, err
	}
	update12Header, err := fetchLiveHeader(ctx, params.SourceChain, params.UpdateHeight12)
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

	trustedProof, trustedProofRLP, err := fetchAccountProof(ctx, params.SourceChain, params.RouterAddress, params.InitialTrustedHeight)
	if err != nil {
		return besuFixture{}, err
	}
	_ = trustedProofRLP

	update11, err := buildLiveUpdateFixture(ctx, params.SourceChain, params.RouterAddress, params.InitialTrustedHeight, params.UpdateHeight11)
	if err != nil {
		return besuFixture{}, err
	}
	update12, err := buildLiveUpdateFixture(ctx, params.SourceChain, params.RouterAddress, params.InitialTrustedHeight, params.UpdateHeight12)
	if err != nil {
		return besuFixture{}, err
	}
	lowQuorumHeight12, err := buildLowQuorumFixture(update12, update12Header)
	if err != nil {
		return besuFixture{}, err
	}
	conflictingHeight12, err := buildConflictingFixture(
		params.InitialTrustedHeight,
		update12.Height,
		syntheticSourceHeader,
		validatorKeys,
		params.SourceChain,
		params.RouterAddress,
	)
	if err != nil {
		return besuFixture{}, err
	}
	lowOverlapHeight13, err := buildLowOverlapFixture(
		params.InitialTrustedHeight,
		update12.Height+1,
		syntheticSourceHeader,
		validatorKeys,
		params.SourceChain,
		params.RouterAddress,
	)
	if err != nil {
		return besuFixture{}, err
	}
	membership, err := buildMembershipFixture(ctx, params.SourceChain, params.RouterAddress, params.Packet, params.UpdateHeight12, update12Header.Header.Time)
	if err != nil {
		return besuFixture{}, err
	}
	nonMembership, err := buildNonMembershipFixture(ctx, params.SourceChain, params.RouterAddress, params.Packet, params.UpdateHeight12, update12Header.Header.Time)
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
		UpdateHeight11:            update11,
		UpdateHeight12:            update12,
		LowQuorumHeight12:         lowQuorumHeight12,
		ConflictingHeight12:       conflictingHeight12,
		LowOverlapHeight13:        lowOverlapHeight13,
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

func buildLowQuorumFixture(update besuUpdateFixture, header liveHeader) (besuUpdateFixture, error) {
	mutable, err := decodeMutableQBFTHeader(header.HeaderRLP)
	if err != nil {
		return besuUpdateFixture{}, err
	}
	commitSeals, err := mutable.commitSeals()
	if err != nil {
		return besuUpdateFixture{}, err
	}
	if len(commitSeals) < 2 {
		return besuUpdateFixture{}, fmt.Errorf("expected at least two commit seals, got %d", len(commitSeals))
	}
	mutable.setCommitSeals(commitSeals[:2])
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuUpdateFixture{}, err
	}

	update.HeaderRlp = encodeHex(mutatedHeader)
	return update, nil
}

func buildConflictingFixture(
	trustedHeight uint64,
	targetHeight uint64,
	baseHeader liveHeader,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
) (besuUpdateFixture, error) {
	mutable, err := decodeMutableQBFTHeader(baseHeader.HeaderRLP)
	if err != nil {
		return besuUpdateFixture{}, err
	}
	mutable.setHeight(targetHeight)
	validators, err := mutable.validators()
	if err != nil {
		return besuUpdateFixture{}, err
	}
	signerKeys, err := signerKeysFor(validators[:3], validatorKeys)
	if err != nil {
		return besuUpdateFixture{}, err
	}
	mutable.setCommitSeals(signQBFTCommitSeals(mutable, signerKeys))
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuUpdateFixture{}, err
	}

	proof, accountProofRLP, err := fetchAccountProof(context.Background(), chain, routerAddress, baseHeader.Header.Number.Uint64())
	if err != nil {
		return besuUpdateFixture{}, err
	}

	return besuUpdateFixture{
		Height:              targetHeight,
		HeaderRlp:           encodeHex(mutatedHeader),
		TrustedHeight:       trustedHeight,
		AccountProof:        encodeHex(accountProofRLP),
		ExpectedTimestamp:   baseHeader.Header.Time,
		ExpectedStorageRoot: proof.StorageHash.Hex(),
		ExpectedValidators:  addressesToHex(validators),
	}, nil
}

func buildLowOverlapFixture(
	trustedHeight uint64,
	targetHeight uint64,
	baseHeader liveHeader,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
) (besuUpdateFixture, error) {
	mutable, err := decodeMutableQBFTHeader(baseHeader.HeaderRLP)
	if err != nil {
		return besuUpdateFixture{}, err
	}
	mutable.setHeight(targetHeight)

	baseValidators, err := mutable.validators()
	if err != nil {
		return besuUpdateFixture{}, err
	}
	if len(baseValidators) == 0 {
		return besuUpdateFixture{}, fmt.Errorf("missing base validators")
	}

	syntheticKeys, err := loadSyntheticLowOverlapValidatorKeys()
	if err != nil {
		return besuUpdateFixture{}, err
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
		return besuUpdateFixture{}, err
	}
	mutable.setCommitSeals(signQBFTCommitSeals(mutable, signerKeys))
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuUpdateFixture{}, err
	}

	proof, accountProofRLP, err := fetchAccountProof(context.Background(), chain, routerAddress, baseHeader.Header.Number.Uint64())
	if err != nil {
		return besuUpdateFixture{}, err
	}

	return besuUpdateFixture{
		Height:              targetHeight,
		HeaderRlp:           encodeHex(mutatedHeader),
		TrustedHeight:       trustedHeight,
		AccountProof:        encodeHex(accountProofRLP),
		ExpectedTimestamp:   baseHeader.Header.Time,
		ExpectedStorageRoot: proof.StorageHash.Hex(),
		ExpectedValidators:  addressesToHex(lowOverlapValidators),
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
	path := packetCommitmentPath(packet)
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
	path := packetReceiptCommitmentPath(packet)
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
) (ethereum.AccountProof, []byte, error) {
	proof, err := chain.GetProof(ctx, routerAddress, nil, fmt.Sprintf("0x%x", height))
	if err != nil {
		return ethereum.AccountProof{}, nil, fmt.Errorf("fetch account proof at height %d: %w", height, err)
	}
	proofRLP, err := encodeProofNodes(proof.AccountProof)
	if err != nil {
		return ethereum.AccountProof{}, nil, fmt.Errorf("encode account proof at height %d: %w", height, err)
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
	storageKey, err := evmICS26CommitmentStorageKey(path)
	if err != nil {
		return nil, err
	}
	proof, err := chain.GetProof(ctx, routerAddress, []string{storageKey.Hex()}, fmt.Sprintf("0x%x", height))
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
	rawNodes := make([]rlp.RawValue, len(nodes))
	for i, node := range nodes {
		rawNodes[i] = ethcommon.FromHex(node)
	}
	return rlp.EncodeToBytes(rawNodes)
}

func evmICS26CommitmentStorageKey(path []byte) (ethcommon.Hash, error) {
	slotHex := strings.TrimPrefix(testvalues.IbcCommitmentSlotHex, "0x")
	slot, err := hex.DecodeString(slotHex)
	if err != nil {
		return ethcommon.Hash{}, fmt.Errorf("decode IBC commitment slot: %w", err)
	}
	if len(slot) != 32 {
		return ethcommon.Hash{}, fmt.Errorf("invalid IBC commitment slot length: %d", len(slot))
	}
	pathHash := crypto.Keccak256(path)
	return crypto.Keccak256Hash(pathHash, slot), nil
}

func packetCommitmentPath(packet ics26router.IICS26RouterMsgsPacket) []byte {
	path := make([]byte, 0, len(packet.SourceClient)+1+8)
	path = append(path, []byte(packet.SourceClient)...)
	path = append(path, 1)
	return appendUint64(path, packet.Sequence)
}

func packetReceiptCommitmentPath(packet ics26router.IICS26RouterMsgsPacket) []byte {
	path := make([]byte, 0, len(packet.DestClient)+1+8)
	path = append(path, []byte(packet.DestClient)...)
	path = append(path, 2)
	return appendUint64(path, packet.Sequence)
}

func packetCommitment(packet ics26router.IICS26RouterMsgsPacket) []byte {
	destHash := sha256.Sum256([]byte(packet.DestClient))
	timeoutBytes := make([]byte, 8)
	binary.BigEndian.PutUint64(timeoutBytes, packet.TimeoutTimestamp)
	timeoutHash := sha256.Sum256(timeoutBytes)

	appBytes := make([]byte, 0, len(packet.Payloads)*32)
	for _, payload := range packet.Payloads {
		appBytes = append(appBytes, payloadCommitmentHash(payload)...)
	}
	appHash := sha256.Sum256(appBytes)

	final := make([]byte, 0, 1+32+32+32)
	final = append(final, 2)
	final = append(final, destHash[:]...)
	final = append(final, timeoutHash[:]...)
	final = append(final, appHash[:]...)

	commitment := sha256.Sum256(final)
	return commitment[:]
}

func payloadCommitmentHash(payload ics26router.IICS26RouterMsgsPayload) []byte {
	parts := [][]byte{
		sha256Bytes([]byte(payload.SourcePort)),
		sha256Bytes([]byte(payload.DestPort)),
		sha256Bytes([]byte(payload.Version)),
		sha256Bytes([]byte(payload.Encoding)),
		sha256Bytes(payload.Value),
	}

	concatenated := make([]byte, 0, len(parts)*32)
	for _, part := range parts {
		concatenated = append(concatenated, part...)
	}
	return sha256Bytes(concatenated)
}

func sha256Bytes(input []byte) []byte {
	sum := sha256.Sum256(input)
	return sum[:]
}

func appendUint64(dst []byte, value uint64) []byte {
	var buf [8]byte
	binary.BigEndian.PutUint64(buf[:], value)
	return append(dst, buf[:]...)
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
