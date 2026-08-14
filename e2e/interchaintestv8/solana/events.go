// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/base64"
	"fmt"
	"strings"

	bin "github.com/gagliardetto/binary"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	ics26router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"
)

// EventDiscriminator calculates the 8-byte discriminator for an Anchor event.
// This matches the Rust implementation: sha256("event:{eventName}")[..8]
func EventDiscriminator(eventName string) []byte {
	hash := sha256.Sum256(fmt.Appendf(nil, "event:%s", eventName))
	return hash[:8]
}

// SendPacketEventDiscriminator returns the discriminator for SendPacketEvent
func SendPacketEventDiscriminator() []byte {
	return EventDiscriminator("SendPacketEvent")
}

// SendPacketEvent represents the event emitted when a packet is sent on Solana.
// This matches the Rust struct in ibc-solana/packages/solana-ibc-types/src/events.rs
type SendPacketEvent struct {
	ClientID         string
	Sequence         uint64
	Packet           SolanaPacket
	TimeoutTimestamp int64
}

// SolanaPacket represents a packet in the Solana IBC implementation
type SolanaPacket struct {
	Sequence         uint64
	SourceClient     string
	DestClient       string
	TimeoutTimestamp int64
	Payloads         []SolanaPayload
}

// SolanaPayload represents a payload within a packet
type SolanaPayload struct {
	SourcePort string
	DestPort   string
	Version    string
	Encoding   string
	Value      []byte
}

// ParseSendPacketEvent parses a SendPacketEvent from Solana transaction logs.
func ParseSendPacketEvent(logs []string) (*SendPacketEvent, error) {
	discriminator := SendPacketEventDiscriminator()

	for _, log := range logs {
		if !strings.HasPrefix(log, "Program data: ") {
			continue
		}

		dataStr := strings.TrimPrefix(log, "Program data: ")
		data, err := base64.StdEncoding.DecodeString(dataStr)
		if err != nil || len(data) < 8 {
			continue
		}

		if !bytes.Equal(data[:8], discriminator) {
			continue
		}

		decoder := bin.NewBorshDecoder(data[8:])
		var event SendPacketEvent
		if err := decoder.Decode(&event); err != nil {
			return nil, fmt.Errorf("failed to decode SendPacketEvent: %w", err)
		}

		return &event, nil
	}

	return nil, fmt.Errorf("SendPacketEvent not found in transaction logs")
}

// GetTransactionLogs retrieves the logs from a finalized transaction.
func GetTransactionLogs(ctx context.Context, rpcClient *rpc.Client, sig solana.Signature) ([]string, error) {
	version := uint64(0)
	txDetails, err := rpcClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
		Commitment:                     rpc.CommitmentConfirmed,
		MaxSupportedTransactionVersion: &version,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get transaction: %w", err)
	}

	if txDetails == nil || txDetails.Meta == nil {
		return nil, fmt.Errorf("transaction meta is nil")
	}

	return txDetails.Meta.LogMessages, nil
}

// GetSendPacketEventFromTransaction fetches transaction logs and parses SendPacketEvent.
func GetSendPacketEventFromTransaction(ctx context.Context, rpcClient *rpc.Client, sig solana.Signature) (*SendPacketEvent, error) {
	logs, err := GetTransactionLogs(ctx, rpcClient, sig)
	if err != nil {
		return nil, err
	}

	return ParseSendPacketEvent(logs)
}

// ParseWriteAcknowledgementEventsFromLogs parses WriteAcknowledgementEvent from Solana transaction logs
// using the auto-generated types from packages/go-anchor/ics26router.
func ParseWriteAcknowledgementEventsFromLogs(logs []string) ([]*ics26router.Ics26RouterEventsWriteAcknowledgementEvent, error) {
	var events []*ics26router.Ics26RouterEventsWriteAcknowledgementEvent

	for _, log := range logs {
		if !strings.HasPrefix(log, "Program data: ") {
			continue
		}

		dataStr := strings.TrimPrefix(log, "Program data: ")
		data, err := base64.StdEncoding.DecodeString(dataStr)
		if err != nil {
			continue
		}

		if len(data) < 8 {
			continue
		}

		event, err := ics26router.ParseEvent_Ics26RouterEventsWriteAcknowledgementEvent(data)
		if err != nil {
			continue
		}

		events = append(events, event)
	}

	return events, nil
}
