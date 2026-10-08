// SPDX-License-Identifier: Apache-2.0

package types

import (
	"bytes"
	"context"
	"crypto/ecdsa"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"slices"
	"strings"

	ethcommon "github.com/ethereum/go-ethereum/common"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient/gethclient"
	"github.com/ethereum/go-ethereum/rlp"

	channeltypesv2 "github.com/cosmos/ibc-go/v11/modules/core/04-channel/v2/types"
	ibchostv2 "github.com/cosmos/ibc-go/v11/modules/core/24-host/v2"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besumsgs"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

type BesuFixtureGenerator struct {
	Enabled bool
}

type GenerateBesuFixtureParams struct {
	// Consensus is testvalues.BesuConsensusQBFT or testvalues.BesuConsensusIBFT2, and names the fixture file.
	Consensus               string
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
	RouterAddress            string                     `json:"routerAddress"`
	InitialTrustedHeight     uint64                     `json:"initialTrustedHeight"`
	InitialTrustedTimestamp  uint64                     `json:"initialTrustedTimestamp"`
	InitialTrustedStateRoot  string                     `json:"initialTrustedStateRoot"`
	InitialTrustedValidators []string                   `json:"initialTrustedValidators"`
	TrustingPeriod           uint64                     `json:"trustingPeriod"`
	MaxClockDrift            uint64                     `json:"maxClockDrift"`
	AdjacentUpdate           besuUpdateFixture          `json:"adjacentUpdate"`
	NonAdjacentUpdate        besuUpdateFixture          `json:"nonAdjacentUpdate"`
	LowQuorumUpdate          besuRejectionUpdateFixture `json:"lowQuorumUpdate"`
	ConflictingUpdate        besuRejectionUpdateFixture `json:"conflictingUpdate"`
	LowOverlapUpdate         besuRejectionUpdateFixture `json:"lowOverlapUpdate"`
	Membership               besuProofFixture           `json:"membership"`
	NonMembership            besuProofFixture           `json:"nonMembership"`
}

type besuUpdateFixture struct {
	Height             uint64   `json:"height"`
	HeaderRlp          string   `json:"headerRlp"`
	TrustedHeight      uint64   `json:"trustedHeight"`
	ExpectedTimestamp  uint64   `json:"expectedTimestamp"`
	ExpectedStateRoot  string   `json:"expectedStateRoot"`
	ExpectedValidators []string `json:"expectedValidators"`
}

type besuRejectionUpdateFixture struct {
	Height        uint64 `json:"height"`
	HeaderRlp     string `json:"headerRlp"`
	TrustedHeight uint64 `json:"trustedHeight"`
}

type besuProofFixture struct {
	Proof             string `json:"proof"`
	AccountProof      string `json:"accountProof"`
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

type mutableBesuHeader struct {
	consensus  string
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

func (g *BesuFixtureGenerator) GenerateAndSaveFixture(ctx context.Context, params GenerateBesuFixtureParams) error {
	if !g.Enabled {
		return nil
	}

	fixture, err := generateBesuFixture(ctx, params)
	if err != nil {
		return err
	}

	fixtureBz, err := json.MarshalIndent(fixture, "", "  ")
	if err != nil {
		return err
	}

	fixturePath := filepath.Join(testvalues.BesuBFTFixturesDir, params.Consensus+".json")
	// The checked-in fixture is intentionally readable by all test processes.
	return os.WriteFile(fixturePath, fixtureBz, 0o644) //nolint:gosec
}

func generateBesuFixture(ctx context.Context, params GenerateBesuFixtureParams) (besuFixture, error) {
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

	trustedHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.Consensus, params.InitialTrustedHeight)
	if err != nil {
		return besuFixture{}, err
	}
	nonAdjacentHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.Consensus, params.NonAdjacentUpdateHeight)
	if err != nil {
		return besuFixture{}, err
	}
	syntheticSourceHeader, err := fetchLiveHeader(ctx, params.SourceChain, params.Consensus, params.SyntheticSourceHeight)
	if err != nil {
		return besuFixture{}, err
	}

	validatorKeys, err := loadBesuValidatorKeys()
	if err != nil {
		return besuFixture{}, err
	}

	adjacentUpdate, err := buildLiveUpdateFixture(ctx, params.SourceChain, params.Consensus, params.InitialTrustedHeight, params.AdjacentUpdateHeight)
	if err != nil {
		return besuFixture{}, err
	}
	nonAdjacentUpdate, err := buildLiveUpdateFixture(ctx, params.SourceChain, params.Consensus, params.InitialTrustedHeight, params.NonAdjacentUpdateHeight)
	if err != nil {
		return besuFixture{}, err
	}
	lowQuorumUpdate, err := buildLowQuorumFixture(params.Consensus, nonAdjacentUpdate, nonAdjacentHeader)
	if err != nil {
		return besuFixture{}, err
	}
	conflictingUpdate, err := buildConflictingFixture(
		params.Consensus,
		params.InitialTrustedHeight,
		nonAdjacentUpdate.Height,
		syntheticSourceHeader,
		validatorKeys,
	)
	if err != nil {
		return besuFixture{}, err
	}
	lowOverlapUpdate, err := buildLowOverlapFixture(
		params.Consensus,
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
		RouterAddress:            params.RouterAddress.Hex(),
		InitialTrustedHeight:     params.InitialTrustedHeight,
		InitialTrustedTimestamp:  trustedHeader.Header.Time,
		InitialTrustedStateRoot:  trustedHeader.Header.Root.Hex(),
		InitialTrustedValidators: addressesToHex(trustedHeader.Validators),
		TrustingPeriod:           params.TrustingPeriod,
		MaxClockDrift:            params.MaxClockDrift,
		AdjacentUpdate:           adjacentUpdate,
		NonAdjacentUpdate:        nonAdjacentUpdate,
		LowQuorumUpdate:          lowQuorumUpdate,
		ConflictingUpdate:        conflictingUpdate,
		LowOverlapUpdate:         lowOverlapUpdate,
		Membership:               membership,
		NonMembership:            nonMembership,
	}, nil
}

func buildLiveUpdateFixture(
	ctx context.Context,
	chain *ethereum.Ethereum,
	consensus string,
	trustedHeight uint64,
	targetHeight uint64,
) (besuUpdateFixture, error) {
	header, err := fetchLiveHeader(ctx, chain, consensus, targetHeight)
	if err != nil {
		return besuUpdateFixture{}, err
	}

	return besuUpdateFixture{
		Height:             targetHeight,
		HeaderRlp:          encodeHex(header.HeaderRLP),
		TrustedHeight:      trustedHeight,
		ExpectedTimestamp:  header.Header.Time,
		ExpectedStateRoot:  header.Header.Root.Hex(),
		ExpectedValidators: addressesToHex(header.Validators),
	}, nil
}

func buildLowQuorumFixture(consensus string, update besuUpdateFixture, header liveHeader) (besuRejectionUpdateFixture, error) {
	mutable, err := decodeMutableBesuHeader(header.HeaderRLP, consensus)
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
	}, nil
}

// BuildBesuUpdate returns an abi-encoded MsgUpdateClient carrying the live header at height, trusting the live
// consensus state at trustedHeight. consensus is testvalues.BesuConsensusQBFT or testvalues.BesuConsensusIBFT2.
func BuildBesuUpdate(ctx context.Context, chain *ethereum.Ethereum, consensus string, trustedHeight, height uint64) ([]byte, error) {
	return buildBesuUpdate(ctx, chain, consensus, trustedHeight, height, nil)
}

// BuildBesuDoubleSignUpdate is BuildBesuUpdate with the header's stateRoot replaced and re-sealed, so a client that
// already stores height detects a double sign.
func BuildBesuDoubleSignUpdate(ctx context.Context, chain *ethereum.Ethereum, consensus string, trustedHeight, height uint64) ([]byte, error) {
	return buildBesuUpdate(ctx, chain, consensus, trustedHeight, height, func(h *mutableBesuHeader) {
		h.setStateRoot(crypto.Keccak256Hash([]byte("double sign")))
	})
}

// BuildBesuTimestampUpdate is BuildBesuUpdate with the header's timestamp replaced and re-sealed.
func BuildBesuTimestampUpdate(ctx context.Context, chain *ethereum.Ethereum, consensus string, trustedHeight, height, timestamp uint64) ([]byte, error) {
	return buildBesuUpdate(ctx, chain, consensus, trustedHeight, height, func(h *mutableBesuHeader) {
		h.setTimestamp(timestamp)
	})
}

// FetchBesuConsensusState returns the consensus state preimage of the live header at height.
func FetchBesuConsensusState(ctx context.Context, chain *ethereum.Ethereum, consensus string, height uint64) (besumsgs.IBesuLightClientMsgsConsensusState, error) {
	header, err := fetchLiveHeader(ctx, chain, consensus, height)
	if err != nil {
		return besumsgs.IBesuLightClientMsgsConsensusState{}, err
	}
	return header.consensusState(), nil
}

// FetchBesuCommitSealSigners returns the signers of the commit seals in the live header at height.
func FetchBesuCommitSealSigners(ctx context.Context, chain *ethereum.Ethereum, consensus string, height uint64) ([]ethcommon.Address, error) {
	header, err := fetchLiveHeader(ctx, chain, consensus, height)
	if err != nil {
		return nil, err
	}
	mutable, err := decodeMutableBesuHeader(header.HeaderRLP, consensus)
	if err != nil {
		return nil, err
	}
	seals, err := mutable.commitSeals()
	if err != nil {
		return nil, err
	}
	digest := mutable.commitSealDigest()
	signers := make([]ethcommon.Address, len(seals))
	for i, seal := range seals {
		pubkey, err := crypto.SigToPub(digest.Bytes(), seal)
		if err != nil {
			return nil, err
		}
		signers[i] = crypto.PubkeyToAddress(*pubkey)
	}
	return signers, nil
}

// buildBesuUpdate applies mutate, if non-nil, to the live header at height and re-seals it with a quorum of the local
// validator keys, read relative to the repository root.
func buildBesuUpdate(
	ctx context.Context,
	chain *ethereum.Ethereum,
	consensus string,
	trustedHeight uint64,
	height uint64,
	mutate func(*mutableBesuHeader),
) ([]byte, error) {
	trusted, err := fetchLiveHeader(ctx, chain, consensus, trustedHeight)
	if err != nil {
		return nil, err
	}
	header, err := fetchLiveHeader(ctx, chain, consensus, height)
	if err != nil {
		return nil, err
	}

	headerRLP := header.HeaderRLP
	if mutate != nil {
		validatorKeys, err := loadBesuValidatorKeys()
		if err != nil {
			return nil, err
		}
		if headerRLP, err = resealWithQuorum(header.HeaderRLP, consensus, validatorKeys, mutate); err != nil {
			return nil, err
		}
	}

	// The light client expects abi.encode(MsgUpdateClient), without the function selector.
	return besumsgs.NewBindings().PackUpdateClient(besumsgs.IBesuLightClientMsgsMsgUpdateClient{
		HeaderRlp:              headerRLP,
		TrustedHeight:          besumsgs.IICS02ClientMsgsHeight{RevisionHeight: trustedHeight},
		ConsensusStatePreimage: trusted.consensusState(),
	})[4:], nil
}

func buildConflictingFixture(
	consensus string,
	trustedHeight uint64,
	targetHeight uint64,
	baseHeader liveHeader,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
) (besuRejectionUpdateFixture, error) {
	mutatedHeader, err := resealWithQuorum(baseHeader.HeaderRLP, consensus, validatorKeys, func(h *mutableBesuHeader) {
		h.setHeight(targetHeight)
	})
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}

	return besuRejectionUpdateFixture{
		Height:        targetHeight,
		HeaderRlp:     encodeHex(mutatedHeader),
		TrustedHeight: trustedHeight,
	}, nil
}

// resealWithQuorum applies mutate to headerRLP and re-seals it with the keys of the first BFT quorum
// (ceil(2n/3)) of the header's validators.
func resealWithQuorum(
	headerRLP []byte,
	consensus string,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
	mutate func(*mutableBesuHeader),
) ([]byte, error) {
	mutable, err := decodeMutableBesuHeader(headerRLP, consensus)
	if err != nil {
		return nil, err
	}
	mutate(mutable)
	validators, err := mutable.validators()
	if err != nil {
		return nil, err
	}
	quorum := (2*len(validators) + 2) / 3
	signerKeys, err := signerKeysFor(validators[:quorum], validatorKeys)
	if err != nil {
		return nil, err
	}
	mutable.setCommitSeals(signBesuCommitSeals(mutable, signerKeys))
	return mutable.encode()
}

func buildLowOverlapFixture(
	consensus string,
	trustedHeight uint64,
	targetHeight uint64,
	baseHeader liveHeader,
	validatorKeys map[ethcommon.Address]*ecdsa.PrivateKey,
) (besuRejectionUpdateFixture, error) {
	mutable, err := decodeMutableBesuHeader(baseHeader.HeaderRLP, consensus)
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
	signerKeys, err := signerKeysFor([]ethcommon.Address{
		lowOverlapValidators[0],
		lowOverlapValidators[1],
		lowOverlapValidators[2],
	}, mergeValidatorKeyMaps(validatorKeys, toKeyMap(syntheticKeys)))
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}
	// Preserve the selected signers (one trusted, two synthetic) independently of validator order.
	slices.SortFunc(lowOverlapValidators, func(a, b ethcommon.Address) int {
		return bytes.Compare(a[:], b[:])
	})
	mutable.setValidators(lowOverlapValidators)
	mutable.setCommitSeals(signBesuCommitSeals(mutable, signerKeys))
	mutatedHeader, err := mutable.encode()
	if err != nil {
		return besuRejectionUpdateFixture{}, err
	}

	return besuRejectionUpdateFixture{
		Height:        targetHeight,
		HeaderRlp:     encodeHex(mutatedHeader),
		TrustedHeight: trustedHeight,
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
	storageProofRLP, accountProofRLP, err := fetchStorageProof(ctx, chain, routerAddress, path, proofHeight)
	if err != nil {
		return besuProofFixture{}, err
	}

	return besuProofFixture{
		Proof:             encodeHex(storageProofRLP),
		AccountProof:      encodeHex(accountProofRLP),
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
	storageProofRLP, accountProofRLP, err := fetchStorageProof(ctx, chain, routerAddress, path, proofHeight)
	if err != nil {
		return besuProofFixture{}, err
	}

	return besuProofFixture{
		Proof:             encodeHex(storageProofRLP),
		AccountProof:      encodeHex(accountProofRLP),
		ProofHeight:       proofHeight,
		Path:              encodeHex(path),
		ExpectedTimestamp: expectedTimestamp,
	}, nil
}

func fetchLiveHeader(ctx context.Context, chain *ethereum.Ethereum, consensus string, height uint64) (liveHeader, error) {
	header, err := chain.RPCClient.HeaderByNumber(ctx, newUint64(height))
	if err != nil {
		return liveHeader{}, fmt.Errorf("fetch header at height %d: %w", height, err)
	}
	headerRLP, err := rlp.EncodeToBytes(header)
	if err != nil {
		return liveHeader{}, fmt.Errorf("encode header rlp at height %d: %w", height, err)
	}
	mutable, err := decodeMutableBesuHeader(headerRLP, consensus)
	if err != nil {
		return liveHeader{}, fmt.Errorf("decode header at height %d: %w", height, err)
	}
	// Besu does not order commit seals by signer, but the light client requires it.
	if err := mutable.sortCommitSeals(); err != nil {
		return liveHeader{}, fmt.Errorf("sort commit seals at height %d: %w", height, err)
	}
	headerRLP, err = mutable.encode()
	if err != nil {
		return liveHeader{}, fmt.Errorf("encode sorted header at height %d: %w", height, err)
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

func (h liveHeader) consensusState() besumsgs.IBesuLightClientMsgsConsensusState {
	return besumsgs.IBesuLightClientMsgsConsensusState{
		Timestamp:  h.Header.Time,
		StateRoot:  h.Header.Root,
		Validators: h.Validators,
	}
}

// fetchStorageProof returns the ABI-encoded storage proof nodes for the commitment at path and the
// ABI-encoded account proof nodes for the router, both taken from one eth_getProof call at height.
func fetchStorageProof(
	ctx context.Context,
	chain *ethereum.Ethereum,
	routerAddress ethcommon.Address,
	path []byte,
	height uint64,
) ([]byte, []byte, error) {
	storageKey := ethereum.GetCommitmentsStorageKey(path)
	proof, err := gethclient.New(chain.RPCClient.Client()).GetProof(ctx, routerAddress, []string{storageKey.Hex()}, newUint64(height))
	if err != nil {
		return nil, nil, fmt.Errorf("fetch storage proof at height %d: %w", height, err)
	}
	if len(proof.StorageProof) != 1 {
		return nil, nil, fmt.Errorf("expected one storage proof at height %d, got %d", height, len(proof.StorageProof))
	}
	return encodeProofNodes(proof.StorageProof[0].Proof), encodeProofNodes(proof.AccountProof), nil
}

func encodeProofNodes(nodes []string) []byte {
	proofNodes := make([][]byte, len(nodes))
	for i, node := range nodes {
		proofNodes[i] = ethcommon.FromHex(node)
	}
	// The light client expects abi.encode(bytes[]), without the function selector.
	return besumsgs.NewBindings().PackProofNodes(proofNodes)[4:]
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

func decodeMutableBesuHeader(headerRLP []byte, consensus string) (*mutableBesuHeader, error) {
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
	return &mutableBesuHeader{consensus: consensus, items: items, extraItems: extraItems}, nil
}

func (h *mutableBesuHeader) encode() ([]byte, error) {
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

func (h *mutableBesuHeader) validators() ([]ethcommon.Address, error) {
	var validators []ethcommon.Address
	if err := rlp.DecodeBytes(h.extraItems[1], &validators); err != nil {
		return nil, err
	}
	return validators, nil
}

func (h *mutableBesuHeader) commitSeals() ([][]byte, error) {
	var seals [][]byte
	if err := rlp.DecodeBytes(h.extraItems[4], &seals); err != nil {
		return nil, err
	}
	return seals, nil
}

func (h *mutableBesuHeader) setHeight(height uint64) {
	h.items[8] = mustRLP(height)
}

func (h *mutableBesuHeader) setStateRoot(stateRoot ethcommon.Hash) {
	h.items[3] = mustRLP(stateRoot)
}

func (h *mutableBesuHeader) setTimestamp(timestamp uint64) {
	h.items[11] = mustRLP(timestamp)
}

func (h *mutableBesuHeader) setValidators(validators []ethcommon.Address) {
	h.extraItems[1] = mustRLP(validators)
}

func (h *mutableBesuHeader) setCommitSeals(seals [][]byte) {
	h.extraItems[4] = mustRLP(seals)
}

// sortCommitSeals orders the commit seals by recovered signer address, as the light client requires.
func (h *mutableBesuHeader) sortCommitSeals() error {
	seals, err := h.commitSeals()
	if err != nil {
		return err
	}
	digest := h.commitSealDigest()
	signers := make(map[string]ethcommon.Address, len(seals))
	for _, seal := range seals {
		pubkey, err := crypto.SigToPub(digest.Bytes(), seal)
		if err != nil {
			return err
		}
		signers[string(seal)] = crypto.PubkeyToAddress(*pubkey)
	}
	slices.SortFunc(seals, func(a, b []byte) int {
		signerA, signerB := signers[string(a)], signers[string(b)]
		return bytes.Compare(signerA[:], signerB[:])
	})
	h.setCommitSeals(seals)
	return nil
}

func signBesuCommitSeals(header *mutableBesuHeader, keys []*ecdsa.PrivateKey) [][]byte {
	return signCommitSeals(header.commitSealDigest(), keys)
}

// signCommitSeals signs digest with every key, ordering the seals by signer address as the light client requires.
func signCommitSeals(digest ethcommon.Hash, keys []*ecdsa.PrivateKey) [][]byte {
	keys = slices.Clone(keys)
	slices.SortFunc(keys, func(a, b *ecdsa.PrivateKey) int {
		signerA, signerB := crypto.PubkeyToAddress(a.PublicKey), crypto.PubkeyToAddress(b.PublicKey)
		return bytes.Compare(signerA[:], signerB[:])
	})
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

// commitSealDigest is the hash validators sign. QBFT signs an empty seal list, whereas IBFT2 omits the seal field.
func (h *mutableBesuHeader) commitSealDigest() ethcommon.Hash {
	signingExtraItems := cloneRawValues(h.extraItems)
	if h.consensus == testvalues.BesuConsensusIBFT2 {
		signingExtraItems = signingExtraItems[:4]
	} else {
		signingExtraItems[4] = rlp.RawValue{0xc0}
	}
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

func loadBesuValidatorKeys() (map[ethcommon.Address]*ecdsa.PrivateKey, error) {
	validatorKeyPaths := []string{
		"e2e/interchaintestv8/chainconfig/testdata/besu/keys/validator1/key",
		"e2e/interchaintestv8/chainconfig/testdata/besu/keys/validator2/key",
		"e2e/interchaintestv8/chainconfig/testdata/besu/keys/validator3/key",
		"e2e/interchaintestv8/chainconfig/testdata/besu/keys/validator4/key",
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
