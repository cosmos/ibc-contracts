// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"crypto/ecdsa"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"

	solanago "github.com/gagliardetto/solana-go"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"
	"github.com/cosmos/interchaintest/v11/ibc"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

// RelayToEVM fetches the relay tx for req from the proof API and broadcasts it to the ICS26 router on chain.
func RelayToEVM(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	chain *ethereum.Ethereum,
	submitter *ecdsa.PrivateKey,
	req *proofapitypes.RelayByTxRequest,
) *ethtypes.Receipt {
	t.Helper()
	resp, err := api.RelayByTx(ctx, req)
	require.NoError(t, err)
	require.NotEmpty(t, resp.Tx)
	require.NotEmpty(t, resp.Address)

	ics26Address := ethcommon.HexToAddress(resp.Address)
	receipt, err := chain.BroadcastTx(ctx, submitter, 15_000_000, &ics26Address, resp.Tx)
	require.NoError(t, err)
	require.Equal(t, ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Relay tx failed: %+v", receipt))
	return receipt
}

// RelayToCosmos fetches the relay tx for req from the proof API and broadcasts it on chain.
func (s *TestSuite) RelayToCosmos(
	ctx context.Context,
	api proofapitypes.ProofApiServiceClient,
	chain *cosmos.CosmosChain,
	submitter ibc.Wallet,
	gas uint64,
	req *proofapitypes.RelayByTxRequest,
) *sdk.TxResponse {
	resp, err := api.RelayByTx(ctx, req)
	s.Require().NoError(err)
	s.Require().NotEmpty(resp.Tx)
	s.Require().Empty(resp.Address)

	return s.MustBroadcastSdkTxBody(ctx, chain, submitter, gas, resp.Tx)
}

// RelayToSolana fetches the relay tx for req from the proof API and submits its chunks on chain.
func RelayToSolana(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	chain *solana.Solana,
	submitter *solanago.Wallet,
	req *proofapitypes.RelayByTxRequest,
) solanago.Signature {
	t.Helper()
	resp, err := api.RelayByTx(ctx, req)
	require.NoError(t, err)
	require.NotEmpty(t, resp.Tx)

	sig, err := chain.SubmitChunkedRelayPackets(ctx, t, resp, submitter)
	require.NoError(t, err)
	return sig
}
