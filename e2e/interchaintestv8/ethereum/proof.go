package ethereum

import (
	"context"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

type StorageProof struct {
	Key   string       `json:"key"`
	Value *hexutil.Big `json:"value"`
	Proof []string     `json:"proof"`
}

type AccountProof struct {
	Address      common.Address `json:"address"`
	AccountProof []string       `json:"accountProof"`
	Balance      *hexutil.Big   `json:"balance"`
	CodeHash     common.Hash    `json:"codeHash"`
	Nonce        hexutil.Uint64 `json:"nonce"`
	StorageHash  common.Hash    `json:"storageHash"`
	StorageProof []StorageProof `json:"storageProof"`
}

func (e *Ethereum) GetProof(ctx context.Context, address common.Address, storageKeys []string, block string) (AccountProof, error) {
	if storageKeys == nil {
		storageKeys = []string{}
	}

	var proof AccountProof
	err := e.RPCClient.Client().CallContext(ctx, &proof, "eth_getProof", address, storageKeys, block)
	return proof, err
}
