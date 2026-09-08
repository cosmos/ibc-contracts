// SPDX-License-Identifier: Apache-2.0

package main

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"os"
	"strconv"

	"github.com/cosmos/cosmos-sdk/types/bech32"
	"github.com/ethereum/go-ethereum/crypto"
)

const (
	// gmpAccountsKey is the module key used by ibc-go's GMP module to derive interchain account addresses.
	// Formula: SHA256(SHA256("module") + gmpAccountsKey + 0x00 + derivationKey)
	gmpAccountsKey = "gmp-accounts"
)

func main() {
	if len(os.Args) < 5 {
		fmt.Fprintf(os.Stderr, "Usage: %s <private-key-hex> <nonce> <client-id> <bech32-prefix> [salt]\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "Example: %s ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 18 08-wasm-0 wf\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "\nComputes the IFT contract address and its corresponding ICA address.\n")
		os.Exit(1)
	}

	privateKeyHex := os.Args[1]
	nonce, err := strconv.ParseUint(os.Args[2], 10, 64)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing nonce: %v\n", err)
		os.Exit(1)
	}
	clientID := os.Args[3]
	bech32Prefix := os.Args[4]
	salt := ""
	if len(os.Args) > 5 {
		salt = os.Args[5]
	}

	// Compute IFT address from private key + nonce
	privateKey, err := crypto.HexToECDSA(privateKeyHex)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing private key: %v\n", err)
		os.Exit(1)
	}
	deployer := crypto.PubkeyToAddress(privateKey.PublicKey)
	iftAddress := crypto.CreateAddress(deployer, nonce)

	// Compute ICA address from client ID + IFT address + salt
	icaAddress, err := computeICAAddress(clientID, iftAddress.Hex(), salt, bech32Prefix)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error computing ICA address: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("IFT Address: %s\n", iftAddress.Hex())
	fmt.Printf("ICA Address: %s\n", icaAddress)
}

func computeICAAddress(clientID, sender, salt, bech32Prefix string) (string, error) {
	key := buildKey(clientID, sender, salt)
	combined := append([]byte(gmpAccountsKey), 0x00)
	combined = append(combined, key...)
	moduleHash := sha256.Sum256([]byte("module"))
	finalInput := append(moduleHash[:], combined...)
	addrHash := sha256.Sum256(finalInput)
	addr := addrHash[:]
	return bech32.ConvertAndEncode(bech32Prefix, addr)
}

func buildKey(clientID, sender, salt string) []byte {
	clientIDBytes := []byte(clientID)
	senderBytes := []byte(sender)
	saltBytes := []byte(salt)
	size := 8 + len(clientIDBytes) + 8 + len(senderBytes) + 8 + len(saltBytes)
	key := make([]byte, 0, size)
	key = appendLengthPrefixed(key, clientIDBytes)
	key = appendLengthPrefixed(key, senderBytes)
	key = appendLengthPrefixed(key, saltBytes)
	return key
}

func appendLengthPrefixed(dst, data []byte) []byte {
	lenBuf := make([]byte, 8)
	binary.BigEndian.PutUint64(lenBuf, uint64(len(data)))
	dst = append(dst, lenBuf...)
	dst = append(dst, data...)
	return dst
}
