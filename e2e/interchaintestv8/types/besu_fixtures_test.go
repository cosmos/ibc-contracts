// SPDX-License-Identifier: Apache-2.0

package types

import (
	"bytes"
	"crypto/ecdsa"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"testing"

	"github.com/stretchr/testify/require"

	ethcommon "github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/besumsgs"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/testvalues"
)

var updateBesuSynthetic = flag.Bool("update-besu-synthetic", false, "regenerate the synthetic Besu fixtures")

func TestBesuProofNodesEncoding(t *testing.T) {
	t.Chdir("../../..")
	fixtureJSON, err := os.ReadFile(filepath.Join(testvalues.BesuBFTFixturesDir, "qbft.json"))
	require.NoError(t, err)
	var fixture besuFixture
	require.NoError(t, json.Unmarshal(fixtureJSON, &fixture))
	want := ethcommon.FromHex(fixture.NonAdjacentUpdate.AccountProof)

	// Recover the input nodes from the independent Solidity fixture, then
	// exercise the production helper, including its removal of the selector.
	schema, err := besumsgs.BindingsMetaData.ParseABI()
	require.NoError(t, err)
	values, err := schema.Methods["proofNodes"].Inputs.Unpack(want)
	require.NoError(t, err)
	require.Len(t, values, 1)
	nodes, ok := values[0].([][]byte)
	require.True(t, ok, "proof nodes decoded as %T, want [][]byte", values[0])
	require.NotEmpty(t, nodes)
	// Generated packers include the function selector; wire payloads do not.
	require.Equal(t, want, besumsgs.NewBindings().PackProofNodes(nodes)[4:])
	hexNodes := make([]string, len(nodes))
	for i, node := range nodes {
		hexNodes[i] = encodeHex(node)
	}
	got := encodeProofNodes(hexNodes)
	require.Equal(t, want, got)
}

func TestBesuConsensusStateEncoding(t *testing.T) {
	t.Chdir("../../..")
	fixtureJSON, err := os.ReadFile(filepath.Join(testvalues.BesuBFTFixturesDir, "qbft.json"))
	require.NoError(t, err)
	var fixture besuFixture
	require.NoError(t, json.Unmarshal(fixtureJSON, &fixture))

	validators := make([]ethcommon.Address, len(fixture.InitialTrustedValidators))
	for i, validator := range fixture.InitialTrustedValidators {
		validators[i] = ethcommon.HexToAddress(validator)
	}
	state := besumsgs.IBesuLightClientMsgsConsensusState{
		Timestamp:   fixture.InitialTrustedTimestamp,
		StorageRoot: ethcommon.HexToHash(fixture.InitialTrustedStorageRoot),
		Validators:  validators,
	}
	// Captured from BesuQBFTLightClient.getConsensusStateHash after deploying
	// with qbft.json's initial trusted state in Foundry.
	want := ethcommon.HexToHash("0xe90678de9bc0821f64b0634c05ba8dc7726ff865d40fbf283d3bdb32ac38d4b9")
	got := crypto.Keccak256Hash(besumsgs.NewBindings().PackConsensusState(state)[4:])
	require.Equal(t, want, got)
}

func TestBesuIBFT2Fixture(t *testing.T) {
	t.Chdir("../../..")
	fixturePath := filepath.Join(testvalues.BesuBFTFixturesDir, "ibft2.json")
	fixtureJSON, err := os.ReadFile(fixturePath)
	require.NoError(t, err)
	var fixture map[string]json.RawMessage
	require.NoError(t, json.Unmarshal(fixtureJSON, &fixture))

	// Public test keys 1 through 7 make the intended signer overlaps reproducible.
	keys := make([]*ecdsa.PrivateKey, 8)
	for i := 1; i < len(keys); i++ {
		keys[i], err = crypto.HexToECDSA(fmt.Sprintf("%064x", i))
		require.NoError(t, err)
	}
	validatorsFor := func(ids []int) []ethcommon.Address {
		validators := make([]ethcommon.Address, len(ids))
		for i, id := range ids {
			validators[i] = crypto.PubkeyToAddress(keys[id].PublicKey)
		}
		slices.SortFunc(validators, func(a, b ethcommon.Address) int {
			return bytes.Compare(a[:], b[:])
		})
		return validators
	}

	var initialValidators []string
	require.NoError(t, json.Unmarshal(fixture["initialTrustedValidators"], &initialValidators))
	expectedInitialValidators := addressesToHex(validatorsFor([]int{1, 2, 3, 4}))
	changed := !slices.Equal(initialValidators, expectedInitialValidators)
	if !*updateBesuSynthetic {
		require.Equal(t, expectedInitialValidators, initialValidators)
	}
	fixture["initialTrustedValidators"], err = json.Marshal(expectedInitialValidators)
	require.NoError(t, err)

	for _, tc := range []struct {
		name       string
		validators []int
		signers    []int
	}{
		{name: "adjacentUpdate", validators: []int{1, 2, 3, 4}, signers: []int{1, 2, 3}},
		{name: "nonAdjacentUpdate", validators: []int{1, 2, 3, 5}, signers: []int{1, 2, 3}},
		{name: "lowQuorumUpdate", validators: []int{1, 2, 3, 5}, signers: []int{1, 2}},
		{name: "conflictingUpdate", validators: []int{1, 2, 3, 5}, signers: []int{1, 2, 3}},
		{name: "lowOverlapUpdate", validators: []int{1, 5, 6, 7}, signers: []int{1, 5, 6}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var update besuUpdateFixture
			require.NoError(t, json.Unmarshal(fixture[tc.name], &update))
			header, err := decodeMutableQBFTHeader(ethcommon.FromHex(update.HeaderRlp))
			require.NoError(t, err)
			validators := validatorsFor(tc.validators)
			header.setValidators(validators)
			// Both protocols share the header encoding, but IBFT2 omits the seal field
			// entirely when signing, whereas QBFT encodes an empty seal list.
			signingHeader := *header
			signingHeader.extraItems = header.extraItems[:4]
			signingRLP, err := signingHeader.encode()
			require.NoError(t, err)
			signerKeys := make([]*ecdsa.PrivateKey, len(tc.signers))
			for i, id := range tc.signers {
				signerKeys[i] = keys[id]
			}
			header.setCommitSeals(signCommitSeals(crypto.Keccak256Hash(signingRLP), signerKeys))
			headerRLP, err := header.encode()
			require.NoError(t, err)
			expectedHeader := encodeHex(headerRLP)
			expectedValidators := addressesToHex(validators)
			changed = changed || update.HeaderRlp != expectedHeader || !slices.Equal(update.ExpectedValidators, expectedValidators)
			if !*updateBesuSynthetic {
				require.Equal(t, expectedHeader, update.HeaderRlp)
				require.Equal(t, expectedValidators, update.ExpectedValidators)
			}
			update.HeaderRlp = expectedHeader
			update.ExpectedValidators = expectedValidators
			fixture[tc.name], err = json.Marshal(update)
			require.NoError(t, err)
		})
	}
	if *updateBesuSynthetic && changed {
		fixtureJSON, err = json.MarshalIndent(fixture, "", "  ")
		require.NoError(t, err)
		require.NoError(t, os.WriteFile(fixturePath, append(fixtureJSON, '\n'), 0o644)) //nolint:gosec // Shared test fixture.
	}
}

func TestBesuQBFTLowOverlapFixture(t *testing.T) {
	t.Chdir("../../..")
	fixturePath := filepath.Join(testvalues.BesuBFTFixturesDir, "qbft.json")
	fixtureJSON, err := os.ReadFile(fixturePath)
	require.NoError(t, err)
	var fixture besuFixture
	require.NoError(t, json.Unmarshal(fixtureJSON, &fixture))
	validatorKeys, err := loadQBFTValidatorKeys()
	require.NoError(t, err)

	// The conflicting case retains the live source header's fields other than height.
	// Reusing it lets us regenerate only the synthetic low-overlap case offline.
	update, err := buildLowOverlapFixture(
		fixture.InitialTrustedHeight,
		fixture.LowOverlapUpdate.Height,
		liveHeader{HeaderRLP: ethcommon.FromHex(fixture.ConflictingUpdate.HeaderRlp)},
		validatorKeys,
	)
	require.NoError(t, err)
	if *updateBesuSynthetic {
		fixture.LowOverlapUpdate = update
		fixtureJSON, err = json.MarshalIndent(fixture, "", "  ")
		require.NoError(t, err)
		require.NoError(t, os.WriteFile(fixturePath, fixtureJSON, 0o644)) //nolint:gosec // Shared test fixture.
	}
	require.Equal(t, fixture.LowOverlapUpdate, update)

	syntheticKeys, err := loadSyntheticLowOverlapValidatorKeys()
	require.NoError(t, err)
	// Key 1 sorts after all three synthetic validators, catching positional signer selection.
	lastKey, err := crypto.HexToECDSA("0000000000000000000000000000000000000000000000000000000000000001")
	require.NoError(t, err)
	firstKey := validatorKeys[ethcommon.HexToAddress(fixture.InitialTrustedValidators[0])]
	require.NotNil(t, firstKey)

	for _, tc := range []struct {
		name string
		key  *ecdsa.PrivateKey
	}{
		{name: "trusted validator sorts first", key: firstKey},
		{name: "trusted validator sorts last", key: lastKey},
	} {
		t.Run(tc.name, func(t *testing.T) {
			trusted := crypto.PubkeyToAddress(tc.key.PublicKey)
			base, err := decodeMutableQBFTHeader(ethcommon.FromHex(fixture.ConflictingUpdate.HeaderRlp))
			require.NoError(t, err)
			base.setValidators([]ethcommon.Address{trusted})
			baseRLP, err := base.encode()
			require.NoError(t, err)
			update, err := buildLowOverlapFixture(1, 3, liveHeader{HeaderRLP: baseRLP}, toKeyMap([]*ecdsa.PrivateKey{tc.key}))
			require.NoError(t, err)
			header, err := decodeMutableQBFTHeader(ethcommon.FromHex(update.HeaderRlp))
			require.NoError(t, err)
			validators, err := header.validators()
			require.NoError(t, err)
			require.Len(t, validators, 4)
			for i := 1; i < len(validators); i++ {
				require.Negative(t, bytes.Compare(validators[i-1][:], validators[i][:]))
			}
			require.ElementsMatch(t, []ethcommon.Address{
				trusted,
				crypto.PubkeyToAddress(syntheticKeys[0].PublicKey),
				crypto.PubkeyToAddress(syntheticKeys[1].PublicKey),
				crypto.PubkeyToAddress(syntheticKeys[2].PublicKey),
			}, validators)
			seals, err := header.commitSeals()
			require.NoError(t, err)
			require.Len(t, seals, 3)
			digest := header.commitSealDigest()
			signers := make([]ethcommon.Address, len(seals))
			for i, seal := range seals {
				pubkey, err := crypto.SigToPub(digest.Bytes(), seal)
				require.NoError(t, err)
				signers[i] = crypto.PubkeyToAddress(*pubkey)
			}
			require.ElementsMatch(t, []ethcommon.Address{
				trusted,
				crypto.PubkeyToAddress(syntheticKeys[0].PublicKey),
				crypto.PubkeyToAddress(syntheticKeys[1].PublicKey),
			}, signers)
		})
	}
}
