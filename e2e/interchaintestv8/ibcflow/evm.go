// SPDX-License-Identifier: Apache-2.0

// Package ibcflow contains reusable IBC flows for e2e suites whose chains, contracts and clients are already set up.
package ibcflow

import (
	"context"
	"crypto/ecdsa"
	"fmt"
	"math/big"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	ethcommon "github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"

	transfertypes "github.com/cosmos/ibc-go/v11/modules/apps/transfer/types"

	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics20transfer"
	"github.com/cosmos/solidity-ibc-eureka/packages/go-abigen/ics26router"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/e2esuite"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/ethereum"
	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/erc20"
	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

const (
	relayGasLimit = 15_000_000
	// timeoutOffset is how far past the latest block time of both chains a timed out packet expires.
	timeoutOffset = 15
)

// EVMEndpoint is one side of an IBC connection on an EVM chain.
type EVMEndpoint struct {
	Chain               *ethereum.Ethereum
	Contracts           ethereum.DeployedContracts
	ICS26               *ics26router.Contract
	ICS20               *ics20transfer.Contract
	ERC20               *erc20.Contract
	UserKey             *ecdsa.PrivateKey
	RelayerSubmitterKey *ecdsa.PrivateKey
	// ClientID is the client on this chain that tracks the counterparty.
	ClientID string
}

// NewEVMEndpoint binds the deployed ICS26, ICS20 and ERC20 contracts on chain.
func NewEVMEndpoint(
	t *testing.T,
	chain *ethereum.Ethereum,
	contracts ethereum.DeployedContracts,
	clientID string,
	userKey, relayerSubmitterKey *ecdsa.PrivateKey,
) *EVMEndpoint {
	t.Helper()
	ics26, err := ics26router.NewContract(ethcommon.HexToAddress(contracts.Ics26Router), chain.RPCClient)
	require.NoError(t, err)
	ics20, err := ics20transfer.NewContract(ethcommon.HexToAddress(contracts.Ics20Transfer), chain.RPCClient)
	require.NoError(t, err)
	erc20Contract, err := erc20.NewContract(ethcommon.HexToAddress(contracts.Erc20), chain.RPCClient)
	require.NoError(t, err)

	return &EVMEndpoint{
		Chain:               chain,
		Contracts:           contracts,
		ICS26:               ics26,
		ICS20:               ics20,
		ERC20:               erc20Contract,
		UserKey:             userKey,
		RelayerSubmitterKey: relayerSubmitterKey,
		ClientID:            clientID,
	}
}

// UserAddress returns the address of the endpoint's user.
func (e *EVMEndpoint) UserAddress() ethcommon.Address {
	return crypto.PubkeyToAddress(e.UserKey.PublicKey)
}

// SendTransfer approves ICS20 to spend amount of token and sends it to receiver over src.ClientID.
func SendTransfer(
	ctx context.Context,
	t *testing.T,
	src *EVMEndpoint,
	token ethcommon.Address,
	amount *big.Int,
	receiver string,
	timeout uint64,
) (*ethtypes.Receipt, ics26router.IICS26RouterMsgsPacket) {
	t.Helper()
	tokenContract, err := erc20.NewContract(token, src.Chain.RPCClient)
	require.NoError(t, err)

	approveTx, err := tokenContract.Approve(transactOpts(t, src, src.UserKey), ethcommon.HexToAddress(src.Contracts.Ics20Transfer), amount)
	require.NoError(t, err)
	requireSuccess(ctx, t, src, approveTx.Hash())

	sendTx, err := src.ICS20.SendTransfer(transactOpts(t, src, src.UserKey), ics20transfer.IICS20TransferMsgsSendTransferMsg{
		Denom:            token,
		Amount:           amount,
		Receiver:         strings.ToLower(receiver),
		TimeoutTimestamp: timeout,
		SourceClient:     src.ClientID,
		DestPort:         transfertypes.PortID,
		Memo:             "",
	})
	require.NoError(t, err)
	receipt := requireSuccess(ctx, t, src, sendTx.Hash())

	sendEvent, err := e2esuite.GetEvmEvent(receipt, src.ICS26.ParseSendPacket)
	require.NoError(t, err)
	return receipt, sendEvent.Packet
}

// RelayPackets relays the packets sent or acknowledged by sourceTxIDs on src to dst through the proof API.
func RelayPackets(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	src, dst *EVMEndpoint,
	sourceTxIDs [][]byte,
) *ethtypes.Receipt {
	t.Helper()
	return relay(ctx, t, api, src, dst, &proofapitypes.RelayByTxRequest{SourceTxIds: sourceTxIDs})
}

// RelayTimeouts relays timeouts for the packets sent by timeoutTxIDs on dst, proven against src, to dst.
func RelayTimeouts(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	src, dst *EVMEndpoint,
	timeoutTxIDs [][]byte,
) *ethtypes.Receipt {
	t.Helper()
	return relay(ctx, t, api, src, dst, &proofapitypes.RelayByTxRequest{TimeoutTxIds: timeoutTxIDs})
}

func relay(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	src, dst *EVMEndpoint,
	req *proofapitypes.RelayByTxRequest,
) *ethtypes.Receipt {
	t.Helper()
	req.SrcChain = src.Chain.ChainID.String()
	req.DstChain = dst.Chain.ChainID.String()
	req.SrcClientId = src.ClientID
	req.DstClientId = dst.ClientID

	resp, err := api.RelayByTx(ctx, req)
	require.NoError(t, err)
	require.NotEmpty(t, resp.Tx)
	require.Equal(t, strings.ToLower(dst.Contracts.Ics26Router), strings.ToLower(resp.Address))

	ics26Address := ethcommon.HexToAddress(dst.Contracts.Ics26Router)
	receipt, err := dst.Chain.BroadcastTx(ctx, dst.RelayerSubmitterKey, relayGasLimit, &ics26Address, resp.Tx)
	require.NoError(t, err)
	require.Equal(t, ethtypes.ReceiptStatusSuccessful, receipt.Status, fmt.Sprintf("Relay tx failed: %+v", receipt))
	return receipt
}

// Balances are the balances of one token held by an endpoint's user and by its ICS20 escrow for ClientID.
type Balances struct {
	User   *big.Int
	Escrow *big.Int
}

// GetBalances snapshots the token balances of ep. A zero token or an escrow that is not created yet counts as zero.
func GetBalances(t *testing.T, ep *EVMEndpoint, token ethcommon.Address) Balances {
	t.Helper()
	balances := Balances{User: big.NewInt(0), Escrow: big.NewInt(0)}
	if token == (ethcommon.Address{}) {
		return balances
	}

	tokenContract, err := erc20.NewContract(token, ep.Chain.RPCClient)
	require.NoError(t, err)
	balances.User, err = tokenContract.BalanceOf(nil, ep.UserAddress())
	require.NoError(t, err)

	escrow, err := ep.ICS20.GetEscrow(nil, ep.ClientID)
	require.NoError(t, err)
	if escrow != (ethcommon.Address{}) {
		balances.Escrow, err = tokenContract.BalanceOf(nil, escrow)
		require.NoError(t, err)
	}
	return balances
}

// RequireBalanceChange asserts that the token balances of ep changed by userDelta and escrowDelta since before.
func RequireBalanceChange(t *testing.T, ep *EVMEndpoint, token ethcommon.Address, before Balances, userDelta, escrowDelta *big.Int) {
	t.Helper()
	after := GetBalances(t, ep, token)
	require.Equal(t, new(big.Int).Add(before.User, userDelta).String(), after.User.String(), "user balance")
	require.Equal(t, new(big.Int).Add(before.Escrow, escrowDelta).String(), after.Escrow.String(), "escrow balance")
}

// TransferResult is the outcome of transfers sent from one endpoint, received on the other and acknowledged.
type TransferResult struct {
	SendReceipts []*ethtypes.Receipt
	Packets      []ics26router.IICS26RouterMsgsPacket
	RecvReceipt  *ethtypes.Receipt
	AckReceipt   *ethtypes.Receipt
	// Voucher is the IBC ERC20 minted on the destination. It is zero when the transfer unwinds a voucher.
	Voucher ethcommon.Address
}

// RoundtripResult is the outcome of transfers from a to b and back.
type RoundtripResult struct {
	Forward TransferResult
	Back    TransferResult
}

// TimeoutResult is the outcome of a transfer from a that timed out on b.
type TimeoutResult struct {
	SendReceipt    *ethtypes.Receipt
	Packet         ics26router.IICS26RouterMsgsPacket
	TimeoutReceipt *ethtypes.Receipt
}

// TransferWithAck sends numTransfers transfers of amount of a's ERC20 to b, relays them and relays the
// acknowledgements back, asserting balances relative to the start of the flow.
func TransferWithAck(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	a, b *EVMEndpoint,
	amount *big.Int,
	numTransfers int,
) TransferResult {
	t.Helper()
	token := ethcommon.HexToAddress(a.Contracts.Erc20)
	total := new(big.Int).Mul(amount, big.NewInt(int64(numTransfers)))

	beforeA := GetBalances(t, a, token)
	beforeB := GetBalances(t, b, voucherAddress(b, token))

	result := transfer(ctx, t, api, a, b, token, amount, numTransfers)

	require.True(t, t.Run("Verify balances", func(t *testing.T) {
		result.Voucher = voucherAddress(b, token)
		require.NotEqual(t, ethcommon.Address{}, result.Voucher)

		RequireBalanceChange(t, a, token, beforeA, new(big.Int).Neg(total), total)
		RequireBalanceChange(t, b, result.Voucher, beforeB, total, big.NewInt(0))
	}))
	return result
}

// Roundtrip runs TransferWithAck from a to b, then sends the vouchers back to a and acknowledges them on b.
func Roundtrip(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	a, b *EVMEndpoint,
	amount *big.Int,
	numTransfers int,
) RoundtripResult {
	t.Helper()
	var result RoundtripResult
	require.True(t, t.Run("A to B", func(t *testing.T) {
		result.Forward = TransferWithAck(ctx, t, api, a, b, amount, numTransfers)
	}))

	require.True(t, t.Run("B to A", func(t *testing.T) {
		token := ethcommon.HexToAddress(a.Contracts.Erc20)
		total := new(big.Int).Mul(amount, big.NewInt(int64(numTransfers)))

		beforeA := GetBalances(t, a, token)
		beforeB := GetBalances(t, b, result.Forward.Voucher)

		result.Back = transfer(ctx, t, api, b, a, result.Forward.Voucher, amount, numTransfers)

		require.True(t, t.Run("Verify balances", func(t *testing.T) {
			RequireBalanceChange(t, b, result.Forward.Voucher, beforeB, new(big.Int).Neg(total), big.NewInt(0))
			RequireBalanceChange(t, a, token, beforeA, total, new(big.Int).Neg(total))
		}))
	}))
	return result
}

// TimeoutFromA sends amount of a's ERC20 to b with a short timeout, waits for b to pass it and relays the timeout
// back to a, asserting that a's balances are restored.
func TimeoutFromA(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	a, b *EVMEndpoint,
	amount *big.Int,
) TimeoutResult {
	t.Helper()
	token := ethcommon.HexToAddress(a.Contracts.Erc20)
	before := GetBalances(t, a, token)

	var (
		result  TimeoutResult
		timeout uint64
	)
	require.True(t, t.Run("Send transfer with short timeout", func(t *testing.T) {
		timeA, err := a.Chain.GetBlockTime(ctx)
		require.NoError(t, err)
		timeB, err := b.Chain.GetBlockTime(ctx)
		require.NoError(t, err)
		timeout = uint64(max(timeA, timeB)) + timeoutOffset

		result.SendReceipt, result.Packet = SendTransfer(ctx, t, a, token, amount, b.UserAddress().Hex(), timeout)
		RequireBalanceChange(t, a, token, before, new(big.Int).Neg(amount), amount)
	}))

	require.True(t, t.Run("Wait for timeout on B", func(t *testing.T) {
		require.NoError(t, e2esuite.WaitForBlockTime(ctx, t, b.Chain, timeout))
	}))

	require.True(t, t.Run("Relay timeout to A", func(t *testing.T) {
		result.TimeoutReceipt = RelayTimeouts(ctx, t, api, b, a, [][]byte{result.SendReceipt.TxHash.Bytes()})

		timeoutEvent, err := e2esuite.GetEvmEvent(result.TimeoutReceipt, a.ICS26.ParseTimeoutPacket)
		require.NoError(t, err)
		require.Equal(t, result.Packet, timeoutEvent.Packet)
	}))

	require.True(t, t.Run("Verify refund on A", func(t *testing.T) {
		RequireBalanceChange(t, a, token, before, big.NewInt(0), big.NewInt(0))
	}))
	return result
}

// transfer sends numTransfers transfers of amount of token from src to dst, relays them in one batch and relays the
// acknowledgements back to src.
func transfer(
	ctx context.Context,
	t *testing.T,
	api proofapitypes.ProofApiServiceClient,
	src, dst *EVMEndpoint,
	token ethcommon.Address,
	amount *big.Int,
	numTransfers int,
) TransferResult {
	t.Helper()
	var result TransferResult
	require.True(t, t.Run("Send transfers", func(t *testing.T) {
		header, err := src.Chain.RPCClient.HeaderByNumber(ctx, nil)
		require.NoError(t, err)
		timeout := header.Time + 30*60

		for range numTransfers {
			receipt, packet := SendTransfer(ctx, t, src, token, amount, dst.UserAddress().Hex(), timeout)
			result.SendReceipts = append(result.SendReceipts, receipt)
			result.Packets = append(result.Packets, packet)
		}
	}))

	require.True(t, t.Run("Relay packets", func(t *testing.T) {
		sendTxIDs := make([][]byte, len(result.SendReceipts))
		for i, receipt := range result.SendReceipts {
			sendTxIDs[i] = receipt.TxHash.Bytes()
		}
		result.RecvReceipt = RelayPackets(ctx, t, api, src, dst, sendTxIDs)

		_, err := e2esuite.GetEvmEvent(result.RecvReceipt, dst.ICS26.ParseWriteAcknowledgement)
		require.NoError(t, err)
	}))

	require.True(t, t.Run("Relay acknowledgements", func(t *testing.T) {
		result.AckReceipt = RelayPackets(ctx, t, api, dst, src, [][]byte{result.RecvReceipt.TxHash.Bytes()})

		_, err := e2esuite.GetEvmEvent(result.AckReceipt, src.ICS26.ParseAckPacket)
		require.NoError(t, err)
	}))
	return result
}

// voucherAddress returns the IBC ERC20 on dst for srcToken received over dst.ClientID, or the zero address if it
// has not been created yet.
func voucherAddress(dst *EVMEndpoint, srcToken ethcommon.Address) ethcommon.Address {
	denom := fmt.Sprintf("%s/%s/%s", transfertypes.PortID, dst.ClientID, strings.ToLower(srcToken.Hex()))
	voucher, err := dst.ICS20.IbcERC20Contract(nil, denom)
	if err != nil {
		// ibcERC20Contract reverts with ICS20DenomNotFound before the first receive.
		return ethcommon.Address{}
	}
	return voucher
}

func requireSuccess(ctx context.Context, t *testing.T, ep *EVMEndpoint, hash ethcommon.Hash) *ethtypes.Receipt {
	t.Helper()
	receipt, err := ep.Chain.GetTxReciept(ctx, hash)
	require.NoError(t, err)
	require.Equal(t, ethtypes.ReceiptStatusSuccessful, receipt.Status)
	return receipt
}

func transactOpts(t *testing.T, ep *EVMEndpoint, key *ecdsa.PrivateKey) *bind.TransactOpts {
	t.Helper()
	opts, err := ep.Chain.GetTransactOpts(key)
	require.NoError(t, err)
	return opts
}
