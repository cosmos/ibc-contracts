// SPDX-License-Identifier: Apache-2.0

package ethereum

import (
	"context"
	"crypto/ecdsa"
	"math/big"
	"time"

	goethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"

	"github.com/cosmos/interchaintest/v11/testutil"
)

// BuildSignedTx constructs and signs an EIP-1559 transaction.
// - If explicitNonce == nil, the account's pending nonce is used.
// - If gas == 0, gas is estimated and padded with a safety margin.
// - If to == nil, the tx is treated as contract creation using data as init code.
func BuildSignedTx(
	ctx context.Context,
	c *ethclient.Client,
	priv *ecdsa.PrivateKey,
	to *common.Address,
	value *big.Int,
	data []byte,
	gas uint64,
	explicitNonce *uint64,
) (*types.Transaction, error) {
	from := crypto.PubkeyToAddress(priv.PublicKey)

	chainID, err := c.ChainID(ctx)
	if err != nil {
		return nil, err
	}

	// Resolve nonce
	var nonce uint64
	if explicitNonce != nil {
		nonce = *explicitNonce
	} else {
		n, err := c.PendingNonceAt(ctx, from)
		if err != nil {
			return nil, err
		}
		nonce = n
	}

	// Resolve gas limit
	if gas == 0 {
		msg := goethereum.CallMsg{From: from, To: to, Value: value, Data: data}
		g, err := c.EstimateGas(ctx, msg)
		if err != nil || g == 0 {
			gas = defaultGasLimit
		} else {
			gas = g + g*gasPaddingPercent/100 + gasPaddingFixed
		}
	}

	// Tip cap with fallback
	tipCap, err := c.SuggestGasTipCap(ctx)
	if err != nil || tipCap == nil || tipCap.Sign() <= 0 {
		tipCap = big.NewInt(1_000_000_000) // 1 gwei fallback
	}

	// Base fee (tolerate missing header/basefee by treating as zero)
	hdr, err := c.HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, err
	}
	baseFee := new(big.Int)
	if hdr != nil && hdr.BaseFee != nil {
		baseFee.Set(hdr.BaseFee)
	}

	val := big.NewInt(0)
	if value != nil {
		val = new(big.Int).Set(value)
	}

	// feeCap = 2*baseFee + tipCap (or tipCap if baseFee==0)
	feeCap := new(big.Int).Set(tipCap)
	if baseFee.Sign() > 0 {
		feeCap = new(big.Int).Add(new(big.Int).Mul(baseFee, big.NewInt(2)), tipCap)
	}

	tx := types.NewTx(&types.DynamicFeeTx{
		ChainID:   chainID,
		Nonce:     nonce,
		To:        to,
		Value:     val,
		Gas:       gas,
		GasFeeCap: new(big.Int).Set(feeCap),
		GasTipCap: new(big.Int).Set(tipCap),
		Data:      append([]byte(nil), data...),
	})

	signer := types.LatestSignerForChainID(chainID)
	return types.SignTx(tx, signer, priv)
}

// SendTx sends a transaction using EIP-1559.
// - If to == nil, this performs contract creation using data as init code.
// - If gas == 0, gas is estimated and buffered; otherwise, the provided gas limit is used.
func SendTx(
	ctx context.Context,
	c *ethclient.Client,
	priv *ecdsa.PrivateKey,
	to *common.Address,
	value *big.Int,
	data []byte,
	gas uint64,
) (common.Hash, error) {
	signed, err := BuildSignedTx(ctx, c, priv, to, value, data, gas, nil)
	if err != nil {
		return common.Hash{}, err
	}
	if err := c.SendTransaction(ctx, signed); err != nil {
		return common.Hash{}, err
	}
	return signed.Hash(), nil
}

func GetTransactOpts(ctx context.Context, c *ethclient.Client, key *ecdsa.PrivateKey) (*bind.TransactOpts, error) {
	chainID, err := c.ChainID(ctx)
	if err != nil {
		return nil, err
	}

	fromAddress := crypto.PubkeyToAddress(key.PublicKey)
	nonce, err := c.PendingNonceAt(context.Background(), fromAddress)
	if err != nil {
		nonce = 0
	}

	gasPrice, err := c.SuggestGasPrice(ctx)
	if err != nil {
		panic(err)
	}

	txOpts, err := bind.NewKeyedTransactorWithChainID(key, chainID)
	if err != nil {
		return nil, err
	}

	txOpts.Nonce = big.NewInt(int64(nonce))
	txOpts.GasPrice = gasPrice

	return txOpts, nil
}

func GetTxReciept(ctx context.Context, c *ethclient.Client, hash common.Hash) (*types.Receipt, error) {
	var receipt *types.Receipt
	err := testutil.WaitForCondition(time.Second*40, time.Second, func() (bool, error) {
		var err error
		receipt, err = c.TransactionReceipt(ctx, hash)
		if err != nil {
			return false, nil
		}

		return receipt != nil, nil
	})
	if err != nil {
		return nil, err
	}

	return receipt, nil
}
