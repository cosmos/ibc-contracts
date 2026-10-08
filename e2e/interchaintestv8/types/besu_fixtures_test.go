// SPDX-License-Identifier: Apache-2.0

package types

import (
	"bytes"
	"crypto/ecdsa"
	"encoding/json"
	"flag"
	"os"
	"path/filepath"
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
	want := ethcommon.FromHex(fixture.Membership.AccountProof)

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
		Timestamp:  fixture.InitialTrustedTimestamp,
		StateRoot:  ethcommon.HexToHash(fixture.InitialTrustedStateRoot),
		Validators: validators,
	}
	// keccak256(abi.encode(ConsensusState)) for qbft.json's initial trusted state, computed
	// independently of these bindings with `cast abi-encode "f((uint64,bytes32,address[]))" ... | cast keccak`.
	want := ethcommon.HexToHash("0xecfbf308ea6a516295fb15952c2b0518e06cf2dcb769175a23d786c04d8ad656")
	got := crypto.Keccak256Hash(besumsgs.NewBindings().PackConsensusState(state)[4:])
	require.Equal(t, want, got)
}

func TestBesuLowOverlapFixture(t *testing.T) {
	t.Chdir("../../..")
	for _, consensus := range []string{testvalues.BesuConsensusQBFT, testvalues.BesuConsensusIBFT2} {
		t.Run(consensus, func(t *testing.T) {
			testBesuLowOverlapFixture(t, consensus)
		})
	}
}

func testBesuLowOverlapFixture(t *testing.T, consensus string) {
	t.Helper()
	fixturePath := filepath.Join(testvalues.BesuBFTFixturesDir, consensus+".json")
	fixtureJSON, err := os.ReadFile(fixturePath)
	require.NoError(t, err)
	var fixture besuFixture
	require.NoError(t, json.Unmarshal(fixtureJSON, &fixture))
	validatorKeys, err := loadBesuValidatorKeys()
	require.NoError(t, err)

	// The conflicting case retains the live source header's fields other than height.
	// Reusing it lets us regenerate only the synthetic low-overlap case offline.
	update, err := buildLowOverlapFixture(
		consensus,
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
			base, err := decodeMutableBesuHeader(ethcommon.FromHex(fixture.ConflictingUpdate.HeaderRlp), consensus)
			require.NoError(t, err)
			base.setValidators([]ethcommon.Address{trusted})
			baseRLP, err := base.encode()
			require.NoError(t, err)
			update, err := buildLowOverlapFixture(consensus, 1, 3, liveHeader{HeaderRLP: baseRLP}, toKeyMap([]*ecdsa.PrivateKey{tc.key}))
			require.NoError(t, err)
			header, err := decodeMutableBesuHeader(ethcommon.FromHex(update.HeaderRlp), consensus)
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

func TestBesuSortedCommitSeals(t *testing.T) {
	t.Chdir("../../..")
	for _, consensus := range []string{testvalues.BesuConsensusQBFT, testvalues.BesuConsensusIBFT2} {
		t.Run(consensus, func(t *testing.T) {
			testBesuSortedCommitSeals(t, consensus)
		})
	}
}

func testBesuSortedCommitSeals(t *testing.T, consensus string) {
	t.Helper()
	fixturePath := filepath.Join(testvalues.BesuBFTFixturesDir, consensus+".json")
	fixtureJSON, err := os.ReadFile(fixturePath)
	require.NoError(t, err)
	var fixture besuFixture
	require.NoError(t, json.Unmarshal(fixtureJSON, &fixture))

	// The live headers keep Besu's seal order unless sorted at generation time.
	for _, headerRlp := range []*string{
		&fixture.AdjacentUpdate.HeaderRlp,
		&fixture.NonAdjacentUpdate.HeaderRlp,
		&fixture.LowQuorumUpdate.HeaderRlp,
		&fixture.ConflictingUpdate.HeaderRlp,
		&fixture.LowOverlapUpdate.HeaderRlp,
	} {
		header, err := decodeMutableBesuHeader(ethcommon.FromHex(*headerRlp), consensus)
		require.NoError(t, err)
		require.NoError(t, header.sortCommitSeals())
		sortedRLP, err := header.encode()
		require.NoError(t, err)
		if *updateBesuSynthetic {
			*headerRlp = encodeHex(sortedRLP)
		}
		require.Equal(t, encodeHex(sortedRLP), *headerRlp)
	}
	if *updateBesuSynthetic {
		fixtureJSON, err = json.MarshalIndent(fixture, "", "  ")
		require.NoError(t, err)
		require.NoError(t, os.WriteFile(fixturePath, fixtureJSON, 0o644)) //nolint:gosec // Shared test fixture.
	}
}
