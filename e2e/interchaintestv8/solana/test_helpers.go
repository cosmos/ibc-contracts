// SPDX-License-Identifier: Apache-2.0

package solana

import (
	"context"
	"encoding/binary"
	"fmt"
	"sync"
	"testing"
	"time"

	bin "github.com/gagliardetto/binary"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"

	proofapitypes "github.com/srdtrk/solidity-ibc-eureka/e2e/v8/types/proofapi"
)

func (s *Solana) SubmitChunkedRelayPackets(
	ctx context.Context,
	t *testing.T,
	resp *proofapitypes.RelayByTxResponse,
	user *solana.Wallet,
) (solana.Signature, error) {
	t.Helper()

	var batch proofapitypes.SolanaRelayPacketBatch
	err := proto.Unmarshal(resp.Tx, &batch)
	if err != nil {
		return solana.Signature{}, fmt.Errorf("failed to unmarshal SolanaRelayPacketBatch: %w", err)
	}

	// Submit update client first if present in response
	// For ICS07 Tendermint: ChunkTxs is non-empty
	// For Attestation: ChunkTxs is empty but AssemblyTx is set
	hasChunkedUpdateClient := batch.UpdateClient != nil && len(batch.UpdateClient.ChunkTxs) > 0
	hasAttestationUpdateClient := batch.UpdateClient != nil && len(batch.UpdateClient.AssemblyTx) > 0 && len(batch.UpdateClient.ChunkTxs) == 0

	if hasChunkedUpdateClient {
		t.Logf("=== Update client (chunked) included in relay response (target height: %d), submitting first ===",
			batch.UpdateClient.TargetHeight)

		// Marshal the update client back to bytes for the existing helper
		updateClientBytes, err := proto.Marshal(batch.UpdateClient)
		if err != nil {
			return solana.Signature{}, fmt.Errorf("failed to marshal update client: %w", err)
		}
		updateResp := &proofapitypes.UpdateClientResponse{
			Tx: updateClientBytes,
		}

		// Use existing helper (non-skip cleanup variant)
		s.submitChunkedUpdateClient(ctx, t, require.New(t), updateResp, user, false)
		t.Logf("=== Update client submission complete, proceeding with packets ===")
	} else if hasAttestationUpdateClient {
		t.Logf("=== Update client (attestation) included in relay response (target height: %d), submitting first ===",
			batch.UpdateClient.TargetHeight)

		// For attestation mode, AssemblyTx contains the complete update_client transaction
		unsignedTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(batch.UpdateClient.AssemblyTx))
		if err != nil {
			return solana.Signature{}, fmt.Errorf("failed to decode attestation update client tx: %w", err)
		}

		sig, err := s.SignAndBroadcastTxWithRetry(ctx, unsignedTx, rpc.CommitmentFinalized, user)
		if err != nil {
			return solana.Signature{}, fmt.Errorf("failed to submit attestation update client: %w", err)
		}
		t.Logf("=== Attestation update client submitted (sig: %s), proceeding with packets ===", sig)
	}

	// Handle case where there are no packets (update only)
	if len(batch.Packets) == 0 {
		t.Logf("No relay packets to submit (update client only)")
		return solana.Signature{}, nil
	}

	totalStart := time.Now()
	t.Logf("=== Starting Chunked Relay Packets ===")
	t.Logf("Total packets: %d", len(batch.Packets))

	totalChunks := 0
	for _, packet := range batch.Packets {
		totalChunks += len(packet.Chunks)
	}
	t.Logf("Total chunks across all packets: %d", totalChunks)

	type packetResult struct {
		packetIdx      int
		finalSig       solana.Signature
		err            error
		chunksDuration time.Duration
		finalDuration  time.Duration
		totalDuration  time.Duration
	}

	// Process all packets in parallel
	packetResults := make(chan packetResult, len(batch.Packets))

	// Track IFT finalize_transfer goroutines so we can wait for them before returning
	var finalizeTransferWg sync.WaitGroup

	for packetIdx, packet := range batch.Packets {
		go func(pktIdx int, pkt *proofapitypes.SolanaPacketTxs) {
			packetStart := time.Now()
			hasAlt := len(pkt.AltCreateTx) > 0
			t.Logf("--- Packet %d: Starting (%d chunks + 1 final tx, ALT: %v) ---", pktIdx+1, len(pkt.Chunks), hasAlt)

			// Phase 0: Submit ALT create + extend in background (if present)
			altDone := make(chan error, 1)
			if hasAlt {
				go func() {
					// Submit ALT creation transaction
					altCreateTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(pkt.AltCreateTx))
					if err != nil {
						altDone <- fmt.Errorf("failed to decode ALT creation tx: %w", err)
						return
					}

					// Update blockhash for ALT creation tx (relayer's blockhash may have expired)
					altRecent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
					if err != nil {
						altDone <- fmt.Errorf("failed to get blockhash for ALT creation tx: %w", err)
						return
					}
					altCreateTx.Message.RecentBlockhash = altRecent.Value.Blockhash

					altCreateSig, err := s.SignAndBroadcastTxWithOpts(ctx, altCreateTx, rpc.ConfirmationStatusConfirmed, user)
					if err != nil {
						altDone <- fmt.Errorf("failed to submit ALT creation tx: %w", err)
						return
					}
					t.Logf("✓ Packet %d ALT creation tx submitted: %s", pktIdx+1, altCreateSig)

					// Submit ALT extension transactions sequentially
					for i, altExtendTxBytes := range pkt.AltExtendTxs {
						altExtendTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(altExtendTxBytes))
						if err != nil {
							altDone <- fmt.Errorf("failed to decode ALT extension tx %d: %w", i+1, err)
							return
						}

						// Update blockhash for ALT extension tx
						extendRecent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
						if err != nil {
							altDone <- fmt.Errorf("failed to get blockhash for ALT extension tx %d: %w", i+1, err)
							return
						}
						altExtendTx.Message.RecentBlockhash = extendRecent.Value.Blockhash

						altExtendSig, err := s.SignAndBroadcastTxWithOpts(ctx, altExtendTx, rpc.ConfirmationStatusConfirmed, user)
						if err != nil {
							altDone <- fmt.Errorf("failed to submit ALT extension tx %d: %w", i+1, err)
							return
						}
						t.Logf("✓ Packet %d ALT extension tx %d/%d submitted: %s", pktIdx+1, i+1, len(pkt.AltExtendTxs), altExtendSig)
					}

					altDone <- nil
				}()
			} else {
				altDone <- nil
			}

			type chunkResult struct {
				chunkIdx int
				sig      solana.Signature
				err      error
				duration time.Duration
			}

			// Phase 1: Submit all chunks for this packet in parallel
			chunksStart := time.Now()
			chunkResults := make(chan chunkResult, len(pkt.Chunks))

			for chunkIdx, chunkBytes := range pkt.Chunks {
				go func(chkIdx int, chunkData []byte) {
					chunkStart := time.Now()

					tx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(chunkData))
					if err != nil {
						chunkResults <- chunkResult{
							chunkIdx: chkIdx,
							err:      fmt.Errorf("failed to decode chunk %d: %w", chkIdx, err),
							duration: time.Since(chunkStart),
						}
						return
					}

					recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
					if err != nil {
						chunkResults <- chunkResult{
							chunkIdx: chkIdx,
							err:      fmt.Errorf("failed to get blockhash for chunk %d: %w", chkIdx, err),
							duration: time.Since(chunkStart),
						}
						return
					}
					tx.Message.RecentBlockhash = recent.Value.Blockhash

					sig, err := s.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, user)
					chunkDuration := time.Since(chunkStart)

					if err != nil {
						chunkResults <- chunkResult{
							chunkIdx: chkIdx,
							err:      fmt.Errorf("failed to submit chunk %d: %w", chkIdx, err),
							duration: chunkDuration,
						}
						return
					}

					chunkResults <- chunkResult{
						chunkIdx: chkIdx,
						sig:      sig,
						duration: chunkDuration,
					}
				}(chunkIdx, chunkBytes)
			}

			// Collect all chunk results for this packet
			var chunkErr error
			for i := 0; i < len(pkt.Chunks); i++ {
				result := <-chunkResults
				if result.err != nil {
					chunkErr = result.err
					t.Logf("✗ Packet %d, Chunk %d failed: %v", pktIdx+1, result.chunkIdx+1, result.err)
				} else {
					t.Logf("✓ Packet %d, Chunk %d/%d completed in %v - tx: %s",
						pktIdx+1, result.chunkIdx+1, len(pkt.Chunks), result.duration, result.sig)
				}
			}
			close(chunkResults)
			chunksDuration := time.Since(chunksStart)

			if chunkErr != nil {
				packetResults <- packetResult{
					packetIdx:      pktIdx,
					err:            fmt.Errorf("packet %d chunk upload failed: %w", pktIdx, chunkErr),
					chunksDuration: chunksDuration,
					totalDuration:  time.Since(packetStart),
				}
				return
			}

			t.Logf("--- Packet %d: All %d chunks completed in %v ---",
				pktIdx+1, len(pkt.Chunks), chunksDuration)

			// Wait for ALT to complete before final tx (if ALT is being used)
			if altErr := <-altDone; altErr != nil {
				packetResults <- packetResult{
					packetIdx:      pktIdx,
					err:            fmt.Errorf("packet %d ALT setup failed: %w", pktIdx, altErr),
					chunksDuration: chunksDuration,
					totalDuration:  time.Since(packetStart),
				}
				return
			}
			if hasAlt {
				// Wait for ALT to activate (requires at least 1 slot)
				t.Logf("--- Packet %d: ALT setup complete, waiting for activation ---", pktIdx+1)
				currentSlot, err := s.RPCClient.GetSlot(ctx, rpc.CommitmentConfirmed)
				if err != nil {
					packetResults <- packetResult{
						packetIdx:      pktIdx,
						err:            fmt.Errorf("packet %d failed to get current slot: %w", pktIdx, err),
						chunksDuration: chunksDuration,
						totalDuration:  time.Since(packetStart),
					}
					return
				}

				targetSlot := currentSlot + 1
				for {
					slot, err := s.RPCClient.GetSlot(ctx, rpc.CommitmentConfirmed)
					if err != nil {
						packetResults <- packetResult{
							packetIdx:      pktIdx,
							err:            fmt.Errorf("packet %d failed to poll slot: %w", pktIdx, err),
							chunksDuration: chunksDuration,
							totalDuration:  time.Since(packetStart),
						}
						return
					}
					if slot >= targetSlot {
						t.Logf("--- Packet %d: ALT activated at slot %d, submitting final tx ---", pktIdx+1, slot)
						break
					}
					time.Sleep(100 * time.Millisecond)
				}
			}

			// Phase 2: Submit final transaction for this packet
			finalStart := time.Now()

			finalTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(pkt.FinalTx))
			if err != nil {
				packetResults <- packetResult{
					packetIdx:      pktIdx,
					err:            fmt.Errorf("packet %d failed to decode final tx: %w", pktIdx, err),
					chunksDuration: chunksDuration,
					finalDuration:  time.Since(finalStart),
					totalDuration:  time.Since(packetStart),
				}
				return
			}

			recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
			if err != nil {
				packetResults <- packetResult{
					packetIdx:      pktIdx,
					err:            fmt.Errorf("packet %d failed to get blockhash for final tx: %w", pktIdx, err),
					chunksDuration: chunksDuration,
					finalDuration:  time.Since(finalStart),
					totalDuration:  time.Since(packetStart),
				}
				return
			}
			finalTx.Message.RecentBlockhash = recent.Value.Blockhash

			// Use confirmed commitment - relayer and verification both read with confirmed commitment
			sig, err := s.SignAndBroadcastTxWithOpts(ctx, finalTx, rpc.ConfirmationStatusConfirmed, user)
			finalDuration := time.Since(finalStart)
			totalDuration := time.Since(packetStart)

			if err != nil {
				if sig != (solana.Signature{}) {
					s.LogTransactionDetails(ctx, t, sig, fmt.Sprintf("FAILED: Packet %d Final Tx", pktIdx+1))
				}
				packetResults <- packetResult{
					packetIdx:      pktIdx,
					err:            fmt.Errorf("packet %d failed to submit final tx: %w", pktIdx, err),
					chunksDuration: chunksDuration,
					finalDuration:  finalDuration,
					totalDuration:  totalDuration,
				}
				return
			}

			t.Logf("✓ Packet %d: Final tx completed and finalized in %v - tx: %s", pktIdx+1, finalDuration, sig)

			totalDuration = time.Since(packetStart)
			t.Logf("--- Packet %d: Complete in %v (chunks: %v, final: %v) ---",
				pktIdx+1, totalDuration, chunksDuration, finalDuration)

			packetResults <- packetResult{
				packetIdx:      pktIdx,
				finalSig:       sig,
				chunksDuration: chunksDuration,
				finalDuration:  finalDuration,
				totalDuration:  totalDuration,
			}

			// Phase 3: Submit cleanup transaction (async, not tracked in timing)
			if len(pkt.CleanupTx) > 0 {
				go func(pktIdx int, cleanupTxBytes []byte) {
					cleanupTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(cleanupTxBytes))
					if err != nil {
						t.Logf("⚠ Packet %d: Failed to decode cleanup tx: %v", pktIdx+1, err)
						return
					}

					recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
					if err != nil {
						t.Logf("⚠ Packet %d: Failed to get blockhash for cleanup tx: %v", pktIdx+1, err)
						return
					}
					cleanupTx.Message.RecentBlockhash = recent.Value.Blockhash

					cleanupSig, err := s.SignAndBroadcastTxWithOpts(ctx, cleanupTx, rpc.ConfirmationStatusConfirmed, user)
					if err != nil {
						t.Logf("⚠ Packet %d: Cleanup tx failed: %v", pktIdx+1, err)
					} else {
						t.Logf("✓ Packet %d: Cleanup tx completed - tx: %s", pktIdx+1, cleanupSig)
					}
				}(pktIdx, pkt.CleanupTx)
			}

			// Phase 4: Submit IFT finalize_transfer transaction (tracked, waits before function returns)
			// This processes refunds for IFT transfers on ack/timeout
			if len(pkt.IftFinalizeTransferTx) > 0 {
				finalizeTransferWg.Add(1)
				go func(pktIdx int, finalizeTransferTxBytes []byte) {
					defer finalizeTransferWg.Done()

					finalizeTransferTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(finalizeTransferTxBytes))
					if err != nil {
						t.Logf("⚠ Packet %d: Failed to decode IFT finalize_transfer tx: %v", pktIdx+1, err)
						return
					}

					recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
					if err != nil {
						t.Logf("⚠ Packet %d: Failed to get blockhash for IFT finalize_transfer tx: %v", pktIdx+1, err)
						return
					}
					finalizeTransferTx.Message.RecentBlockhash = recent.Value.Blockhash

					finalizeTransferSig, err := s.SignAndBroadcastTxWithOpts(ctx, finalizeTransferTx, rpc.ConfirmationStatusConfirmed, user)
					if err != nil {
						t.Logf("⚠ Packet %d: IFT finalize_transfer tx failed: %v", pktIdx+1, err)
					} else {
						t.Logf("✓ Packet %d: IFT finalize_transfer tx completed - tx: %s", pktIdx+1, finalizeTransferSig)
					}
				}(pktIdx, pkt.IftFinalizeTransferTx)
			}
		}(packetIdx, packet)
	}

	// Collect all packet results
	var lastSig solana.Signature
	var totalChunksDuration time.Duration
	var totalFinalsDuration time.Duration

	for i := 0; i < len(batch.Packets); i++ {
		result := <-packetResults
		if result.err != nil {
			close(packetResults)
			return solana.Signature{}, result.err
		}
		lastSig = result.finalSig
		totalChunksDuration += result.chunksDuration
		totalFinalsDuration += result.finalDuration
	}
	close(packetResults)

	// Wait for all IFT finalize_transfer transactions to complete before returning
	// This ensures tests can verify PendingTransfer PDA closure after relay completes
	finalizeTransferWg.Wait()

	totalDuration := time.Since(totalStart)
	avgChunksDuration := totalChunksDuration / time.Duration(len(batch.Packets))
	avgFinalsDuration := totalFinalsDuration / time.Duration(len(batch.Packets))

	t.Logf("=== Chunked Relay Packets Complete ===")
	t.Logf("Total wall time: %v for %d packets (%d total chunks)", totalDuration, len(batch.Packets), totalChunks)
	t.Logf("All packets processed in parallel:")
	t.Logf("  - Avg chunks phase per packet: %v", avgChunksDuration)
	t.Logf("  - Avg final tx per packet: %v", avgFinalsDuration)
	t.Logf("Parallelization: All packets + all chunks within each packet submitted concurrently")
	return lastSig, nil
}

// DeploySolanaProgramAsync deploys a program using solana CLI
func (s *Solana) DeploySolanaProgramAsync(ctx context.Context, programName, keypairPath, payerKeypairPath string) (solana.PublicKey, error) {
	programSoFile := fmt.Sprintf("ibc-solana/target/deploy/%s.so", programName)
	programID, _, err := DeploySolanaProgram(ctx, programSoFile, keypairPath, payerKeypairPath, s.RPCURL)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("%s program deployment has failed: %w", programName, err)
	}

	if !s.WaitForProgramAvailability(ctx, programID) {
		return solana.PublicKey{}, fmt.Errorf("program %s failed to become available", programName)
	}

	return programID, nil
}

func (s *Solana) SubmitChunkedUpdateClient(ctx context.Context, t *testing.T, require *require.Assertions, resp *proofapitypes.UpdateClientResponse, user *solana.Wallet) {
	t.Helper()
	s.submitChunkedUpdateClient(ctx, t, require, resp, user, false)
}

func (s *Solana) SubmitChunkedUpdateClientSkipCleanup(ctx context.Context, t *testing.T, require *require.Assertions, resp *proofapitypes.UpdateClientResponse, user *solana.Wallet) {
	t.Helper()
	s.submitChunkedUpdateClient(ctx, t, require, resp, user, true)
}

func (s *Solana) submitChunkedUpdateClient(ctx context.Context, t *testing.T, require *require.Assertions, resp *proofapitypes.UpdateClientResponse, user *solana.Wallet, skipCleanup bool) {
	t.Helper()

	var solanaUpdateClient proofapitypes.SolanaUpdateClient
	err := proto.Unmarshal(resp.Tx, &solanaUpdateClient)
	require.NoError(err, "Failed to unmarshal SolanaUpdateClient")
	require.NotEmpty(solanaUpdateClient.ChunkTxs, "no chunked transactions provided")

	totalStart := time.Now()

	t.Logf("=== Starting Chunked Update Client ===")
	t.Logf("Target height: %d", solanaUpdateClient.TargetHeight)
	t.Logf("ALT extend transactions: %d", len(solanaUpdateClient.AltExtendTxs))
	t.Logf("Chunk transactions: %d", len(solanaUpdateClient.ChunkTxs))

	altExtendCount := len(solanaUpdateClient.AltExtendTxs)
	hasAlt := len(solanaUpdateClient.AltCreateTx) > 0

	// Phase 1: Submit ALT ops and prep txs in parallel
	t.Logf("--- Phase 1: Creating ALT and uploading prep transactions in parallel ---")
	phase1Start := time.Now()

	// Start ALT operations in background goroutine (only if ALT is being used)
	altDone := make(chan error, 1)
	if hasAlt {
		go func() {
			// Submit ALT creation transaction
			altCreateTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.AltCreateTx))
			if err != nil {
				altDone <- fmt.Errorf("failed to decode ALT creation tx: %w", err)
				return
			}

			altCreateSig, err := s.SignAndBroadcastTxWithOpts(ctx, altCreateTx, rpc.ConfirmationStatusConfirmed, user)
			if err != nil {
				altDone <- fmt.Errorf("failed to submit ALT creation tx: %w", err)
				return
			}
			t.Logf("✓ ALT creation tx submitted: %s", altCreateSig)

			// Submit ALT extension transactions sequentially
			for i, altExtendTxBytes := range solanaUpdateClient.AltExtendTxs {
				altExtendTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(altExtendTxBytes))
				if err != nil {
					altDone <- fmt.Errorf("failed to decode ALT extension tx %d: %w", i+1, err)
					return
				}

				altExtendSig, err := s.SignAndBroadcastTxWithOpts(ctx, altExtendTx, rpc.ConfirmationStatusConfirmed, user)
				if err != nil {
					altDone <- fmt.Errorf("failed to submit ALT extension tx %d: %w", i+1, err)
					return
				}
				t.Logf("✓ ALT extension tx %d/%d submitted: %s", i+1, altExtendCount, altExtendSig)
			}

			altDone <- nil
		}()
	} else {
		t.Logf("Skipping ALT operations (optimized path for low validator count)")
		altDone <- nil
	}

	// Upload signature verifications and header chunks in parallel with ALT ops
	prepTxCount := len(solanaUpdateClient.ChunkTxs)

	t.Logf("Uploading %d prep transactions (signature verifications + header chunks) in parallel with ALT operations...", prepTxCount)
	chunksStart := time.Now()

	var completedPrepTxs int
	var totalPrepComputeUnits, totalPrepFees uint64
	var mu sync.Mutex
	var wg sync.WaitGroup

	// Submit all prep txs in parallel
	for idx, chunkTxBytes := range solanaUpdateClient.ChunkTxs {
		wg.Add(1)
		go func(txIdx int, txBytes []byte) {
			defer wg.Done()
			prepTxStart := time.Now()

			tx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(txBytes))
			if err != nil {
				t.Errorf("Failed to decode prep tx %d: %v", txIdx, err)
				return
			}

			sig, err := s.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, user)
			prepTxDuration := time.Since(prepTxStart)

			// Fetch transaction details for gas tracking and logs
			var computeUnits, fee uint64
			version := uint64(0)

			if sig != (solana.Signature{}) {
				// Wait for transaction to be processed
				time.Sleep(500 * time.Millisecond)

				var txDetails *rpc.GetTransactionResult
				var txErr error

				// Retry a few times if transaction not found
				for retry := range 3 {
					txDetails, txErr = s.RPCClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
						Commitment:                     rpc.CommitmentConfirmed,
						MaxSupportedTransactionVersion: &version,
					})

					if txErr == nil && txDetails != nil && txDetails.Meta != nil {
						break
					}

					if retry < 2 {
						time.Sleep(500 * time.Millisecond)
					}
				}

				// Extract compute units and fees if available
				if txErr == nil && txDetails != nil && txDetails.Meta != nil {
					if txDetails.Meta.ComputeUnitsConsumed != nil {
						computeUnits = *txDetails.Meta.ComputeUnitsConsumed
					}
					fee = txDetails.Meta.Fee

					// Only log details on error
					if txDetails.Meta.Err != nil {
						t.Logf("[Prep tx %d] ❌ Transaction error: %v", txIdx, txDetails.Meta.Err)
						if len(txDetails.Meta.LogMessages) > 0 {
							t.Logf("[Prep tx %d logs] %d log messages:", txIdx, len(txDetails.Meta.LogMessages))
							for i, logMsg := range txDetails.Meta.LogMessages {
								t.Logf("  [%d] %s", i, logMsg)
							}
						}
					}
				}
			}

			if err != nil {
				t.Errorf("Failed to submit prep tx %d: %v", txIdx, err)
				return
			}

			// Update shared counters with mutex
			mu.Lock()
			completedPrepTxs++
			totalPrepComputeUnits += computeUnits
			totalPrepFees += fee
			txNum := completedPrepTxs
			mu.Unlock()

			t.Logf("✓ Prep tx %d/%d submitted in %v - tx: %s (gas: %d CUs, fee: %.9f SOL)",
				txNum, prepTxCount, prepTxDuration, sig,
				computeUnits, float64(fee)/1e9)
		}(idx, chunkTxBytes)
	}

	// Wait for all prep txs to complete
	wg.Wait()

	prepTxsTotal := time.Since(chunksStart)
	avgPrepTxTime := prepTxsTotal / time.Duration(prepTxCount)
	avgPrepTxComputeUnits := totalPrepComputeUnits / uint64(prepTxCount)
	t.Logf("✓ All %d prep transactions submitted in %v", prepTxCount, prepTxsTotal)
	t.Logf("  Average per prep tx: %v duration, %d CUs, %.9f SOL",
		avgPrepTxTime, avgPrepTxComputeUnits, float64(totalPrepFees)/float64(prepTxCount)/1e9)
	t.Logf("  Total prep tx gas: %d CUs, %.9f SOL",
		totalPrepComputeUnits, float64(totalPrepFees)/1e9)

	// Wait for ALT operations to complete
	if err := <-altDone; err != nil {
		require.NoError(err, "ALT operations failed")
	}

	if hasAlt {
		t.Logf("✓ ALT create + extend complete")

		// Wait for ALT to activate (requires at least 1 slot)
		t.Logf("Waiting for ALT to activate (next slot)...")
		currentSlot, err := s.RPCClient.GetSlot(ctx, rpc.CommitmentConfirmed)
		require.NoError(err, "Failed to get current slot")

		targetSlot := currentSlot + 1
		for {
			slot, err := s.RPCClient.GetSlot(ctx, rpc.CommitmentConfirmed)
			require.NoError(err, "Failed to poll slot")
			if slot >= targetSlot {
				t.Logf("✓ ALT activated at slot %d (waited for slot %d)", slot, targetSlot)
				break
			}
			time.Sleep(100 * time.Millisecond)
		}
	}

	phase1Duration := time.Since(phase1Start)
	t.Logf("--- Phase 1 Complete: ALT + prep txs ready in %v ---", phase1Duration)

	t.Logf("--- Phase 2: Assembling and updating client ---")
	assemblyStart := time.Now()

	tx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.AssemblyTx))
	require.NoError(err, "Failed to decode assembly tx")

	sig, err := s.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, user)
	if err != nil {
		t.Logf("Assembly transaction error: %v", err)
		t.Logf("Assembly transaction failed, fetching detailed logs...")
		// Try to get the signature from the error to fetch logs
		// Even if submission failed, the transaction may have been sent
		if sig.IsZero() {
			// If we don't have a signature, we can't fetch logs
			t.Logf("No transaction signature available to fetch logs")
		} else {
			// Wait longer for transaction to be indexed in RPC
			for i := range 10 {
				time.Sleep(1 * time.Second)
				version := uint64(0)
				txDetails, fetchErr := s.RPCClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
					Encoding:                       solana.EncodingBase64,
					Commitment:                     rpc.CommitmentConfirmed,
					MaxSupportedTransactionVersion: &version,
				})
				if fetchErr == nil && txDetails != nil {
					s.LogTransactionDetails(ctx, t, sig, "FAILED Assembly Transaction")
					break
				}
				if i == 9 {
					t.Logf("Failed to fetch transaction details after 10 retries")
				}
			}
		}
		require.NoError(err, "Assembly transaction failed")
	}

	// Get transaction details to verify UpdateResult and track gas
	var assemblyComputeUnits, assemblyFee uint64
	version := uint64(0)
	txDetails, err := s.RPCClient.GetTransaction(ctx, sig, &rpc.GetTransactionOpts{
		Encoding:                       solana.EncodingBase64,
		Commitment:                     rpc.CommitmentConfirmed,
		MaxSupportedTransactionVersion: &version,
	})
	if err == nil && txDetails != nil && txDetails.Meta != nil {
		// Get gas metrics
		if txDetails.Meta.ComputeUnitsConsumed != nil {
			assemblyComputeUnits = *txDetails.Meta.ComputeUnitsConsumed
		}
		assemblyFee = txDetails.Meta.Fee

		// Check if transaction has return data (UpdateResult)
		returnDataBytes := txDetails.Meta.ReturnData.Data.Content
		if len(returnDataBytes) > 0 {
			// UpdateResult enum: 0=Update, 1=NoOp, 2=Misbehaviour
			// The return data should be the serialized UpdateResult
			t.Logf("✓ Update client returned data: %v (length: %d)", returnDataBytes, len(returnDataBytes))
			// First byte should be 0 for UpdateResult::Update
			if len(returnDataBytes) >= 1 {
				updateResult := returnDataBytes[0]
				switch updateResult {
				case 0:
					t.Logf("✓ UpdateResult: Update (client state updated)")
				case 1:
					t.Logf("✓ UpdateResult: NoOp (consensus state already exists)")
				case 2:
					t.Logf("✗ UpdateResult: Misbehaviour (client frozen)")
					require.NotEqual(2, updateResult, "Unexpected misbehaviour detected")
				default:
					t.Logf("? UpdateResult: Unknown value %d", updateResult)
				}
			}
		}
	}

	assemblyDuration := time.Since(assemblyStart)
	t.Logf("✓ Assembly transaction completed in %v - tx: %s (gas: %d CUs, fee: %.9f SOL)",
		assemblyDuration, sig, assemblyComputeUnits, float64(assemblyFee)/1e9)

	s.LogTransactionDetails(ctx, t, sig, "SUCCESS: Assembly Transaction")

	var cleanupComputeUnits, cleanupFee uint64
	var cleanupDuration time.Duration

	// Phase 3: Cleanup (optional - can be skipped for large updates to avoid tx size limits)
	// TODO: Add cleanup accounts to ALT for large updates to enable cleanup
	if !skipCleanup && len(solanaUpdateClient.CleanupTx) > 0 {
		t.Logf("--- Phase 3: Cleanup (reclaiming rent) ---")
		cleanupStart := time.Now()

		cleanupTx, err := solana.TransactionFromDecoder(bin.NewBinDecoder(solanaUpdateClient.CleanupTx))
		require.NoError(err, "Failed to decode cleanup tx")

		cleanupSig, err := s.SignAndBroadcastTxWithOpts(ctx, cleanupTx, rpc.ConfirmationStatusConfirmed, user)
		require.NoError(err, "Cleanup transaction failed")

		cleanupTxDetails, err := s.RPCClient.GetTransaction(ctx, cleanupSig, &rpc.GetTransactionOpts{
			Encoding:                       solana.EncodingBase64,
			Commitment:                     rpc.CommitmentConfirmed,
			MaxSupportedTransactionVersion: &version,
		})
		if err == nil && cleanupTxDetails != nil && cleanupTxDetails.Meta != nil {
			if cleanupTxDetails.Meta.ComputeUnitsConsumed != nil {
				cleanupComputeUnits = *cleanupTxDetails.Meta.ComputeUnitsConsumed
			}
			cleanupFee = cleanupTxDetails.Meta.Fee
		}

		cleanupDuration = time.Since(cleanupStart)
		t.Logf("✓ Cleanup transaction completed in %v - tx: %s (gas: %d CUs, fee: %.9f SOL)",
			cleanupDuration, cleanupSig, cleanupComputeUnits, float64(cleanupFee)/1e9)
	} else if skipCleanup {
		t.Logf("--- Phase 3: Cleanup skipped (disabled via skipCleanup flag) ---")
	}

	totalDuration := time.Since(totalStart)
	totalComputeUnits := totalPrepComputeUnits + assemblyComputeUnits + cleanupComputeUnits
	totalFees := totalPrepFees + assemblyFee + cleanupFee

	t.Logf("=== Chunked Update Client Complete ===")
	t.Logf("Total time: %v", totalDuration)
	t.Logf("  - Phase 1 (ALT + prep txs): %v (%d prep txs in parallel)", phase1Duration, prepTxCount)
	t.Logf("  - Phase 2 (Assembly): %v", assemblyDuration)
	if len(solanaUpdateClient.CleanupTx) > 0 {
		t.Logf("  - Phase 3 (Cleanup): %v", cleanupDuration)
	}
	t.Logf("Total gas consumption:")
	t.Logf("  - Prep txs: %d CUs, %.9f SOL", totalPrepComputeUnits, float64(totalPrepFees)/1e9)
	t.Logf("  - Assembly: %d CUs, %.9f SOL", assemblyComputeUnits, float64(assemblyFee)/1e9)
	if len(solanaUpdateClient.CleanupTx) > 0 {
		t.Logf("  - Cleanup: %d CUs, %.9f SOL", cleanupComputeUnits, float64(cleanupFee)/1e9)
	}
	t.Logf("  - TOTAL: %d CUs, %.9f SOL", totalComputeUnits, float64(totalFees)/1e9)
}

func (s *Solana) VerifyPacketCommitmentDeleted(ctx context.Context, t *testing.T, require *require.Assertions, clientID string, sequence uint64) {
	t.Helper()

	sequenceBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(sequenceBytes, sequence)
	packetCommitmentPDA, _ := Ics26Router.PacketCommitmentWithArgSeedPDA(ics26_router.ProgramID, []byte(clientID), sequenceBytes)

	accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, packetCommitmentPDA, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	if err != nil {
		t.Logf("Packet commitment cleared (account not found) for client %s, sequence %d", clientID, sequence)
		return
	}

	if accountInfo.Value == nil || accountInfo.Value.Lamports == 0 {
		t.Logf("Packet commitment cleared (account closed) for client %s, sequence %d", clientID, sequence)
		return
	}

	// Account persists after ack/timeout but value (bytes 8..40, after discriminator) must be zeroed
	data := accountInfo.Value.Data.GetBinary()
	if len(data) >= 40 {
		value := data[8:40]
		allZeros := true
		for _, b := range value {
			if b != 0 {
				allZeros = false
				break
			}
		}
		if allZeros {
			t.Logf("Packet commitment zeroed for client %s, sequence %d", clientID, sequence)
			return
		}
	}

	require.Fail("Packet commitment should have been zeroed after acknowledgment",
		"Account %s still has non-zero commitment value (sequence: %d)", packetCommitmentPDA.String(), sequence)
}

// VerifyPendingTransferExists verifies that an IFT PendingTransfer PDA exists (was created during transfer)
func (s *Solana) VerifyPendingTransferExists(
	ctx context.Context,
	t *testing.T,
	require *require.Assertions,
	iftProgramID solana.PublicKey,
	mint solana.PublicKey,
	clientID string,
	sequence uint64,
) {
	t.Helper()

	sequenceBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(sequenceBytes, sequence)

	pendingTransferPDA, _ := Ift.PendingTransferPDA(iftProgramID, mint.Bytes(), []byte(clientID), sequenceBytes)

	accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, pendingTransferPDA, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	require.NoError(err, "PendingTransfer PDA should exist")
	require.NotNil(accountInfo.Value, "PendingTransfer PDA account should have data")
	require.True(accountInfo.Value.Lamports > 0, "PendingTransfer PDA should have lamports")

	t.Logf("✓ PendingTransfer PDA exists: %s (mint: %s, client: %s, sequence: %d)",
		pendingTransferPDA.String(), mint.String(), clientID, sequence)
}

// VerifyPendingTransferClosed verifies that an IFT PendingTransfer PDA has been closed after finalize_transfer
func (s *Solana) VerifyPendingTransferClosed(
	ctx context.Context,
	t *testing.T,
	require *require.Assertions,
	iftProgramID solana.PublicKey,
	mint solana.PublicKey,
	clientID string,
	sequence uint64,
) {
	t.Helper()

	sequenceBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(sequenceBytes, sequence)

	pendingTransferPDA, _ := Ift.PendingTransferPDA(iftProgramID, mint.Bytes(), []byte(clientID), sequenceBytes)

	accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, pendingTransferPDA, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	if err != nil {
		t.Logf("✓ PendingTransfer PDA closed (account not found) for mint %s, client %s, sequence %d",
			mint.String(), clientID, sequence)
		return
	}

	if accountInfo.Value == nil || accountInfo.Value.Lamports == 0 {
		t.Logf("✓ PendingTransfer PDA closed (account zeroed) for mint %s, client %s, sequence %d",
			mint.String(), clientID, sequence)
		return
	}

	require.Fail("PendingTransfer PDA should have been closed after finalize_transfer",
		"Account %s still exists with %d lamports (mint: %s, client: %s, sequence: %d)",
		pendingTransferPDA.String(), accountInfo.Value.Lamports, mint.String(), clientID, sequence)
}

// VerifyMintAuthority verifies that a token mint has the expected mint authority
func (s *Solana) VerifyMintAuthority(
	ctx context.Context,
	t *testing.T,
	require *require.Assertions,
	mint solana.PublicKey,
	expectedAuthority solana.PublicKey,
) {
	t.Helper()

	accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, mint, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	require.NoError(err, "Failed to get mint account info")
	require.NotNil(accountInfo.Value, "Mint account not found")

	// Parse mint account data (SPL Token mint layout)
	// Layout: 4 bytes COption (0=None, 1=Some) + 32 bytes authority if Some
	data := accountInfo.Value.Data.GetBinary()
	require.True(len(data) >= 36, "Mint account data too short")

	hasAuthority := binary.LittleEndian.Uint32(data[0:4]) == 1
	require.True(hasAuthority, "Mint has no authority set")

	actualAuthority := solana.PublicKeyFromBytes(data[4:36])
	require.Equal(expectedAuthority, actualAuthority,
		"Mint authority mismatch: expected %s, got %s", expectedAuthority.String(), actualAuthority.String())

	t.Logf("✓ Mint %s has correct authority: %s", mint.String(), expectedAuthority.String())
}

// VerifyIftAppStateExists verifies that an IFT app state PDA exists
func (s *Solana) VerifyIftAppStateExists(
	ctx context.Context,
	t *testing.T,
	require *require.Assertions,
	iftProgramID solana.PublicKey,
) {
	t.Helper()

	appStatePDA, _ := Ift.IftAppStatePDA(iftProgramID)

	accountInfo, err := s.RPCClient.GetAccountInfoWithOpts(ctx, appStatePDA, &rpc.GetAccountInfoOpts{
		Commitment: rpc.CommitmentConfirmed,
	})
	require.NoError(err, "IFT app state PDA should exist")
	require.NotNil(accountInfo.Value, "IFT app state PDA account should have data")
	require.True(accountInfo.Value.Lamports > 0, "IFT app state PDA should have lamports")

	t.Logf("✓ IFT app state exists: %s", appStatePDA.String())
}

func (s *Solana) CreateIBCAddressLookupTable(ctx context.Context, t *testing.T, require *require.Assertions, user *solana.Wallet, cosmosChainID string, gmpPortID string, clientID string) solana.PublicKey {
	t.Helper()
	commonAccounts := s.CreateIBCAddressLookupTableAccounts(cosmosChainID, gmpPortID, clientID, user.PublicKey())

	altAddress, err := s.CreateAddressLookupTable(ctx, user, commonAccounts)
	require.NoError(err)
	t.Logf("Created and extended ALT %s with %d common accounts", altAddress, len(commonAccounts))

	return altAddress
}

// CreateIBCAddressLookupTableWithAttestation creates an ALT with both IBC and attestation light client accounts.
// This optimizes transaction size for attestation-based packet relay.
func (s *Solana) CreateIBCAddressLookupTableWithAttestation(ctx context.Context, t *testing.T, require *require.Assertions, user *solana.Wallet, cosmosChainID string, gmpPortID string, clientID string, attestationClientID string) solana.PublicKey {
	t.Helper()

	// Get base IBC accounts and add attestation light client accounts
	allAccounts := s.CreateIBCAddressLookupTableAccounts(cosmosChainID, gmpPortID, clientID, user.PublicKey())
	allAccounts = append(allAccounts, s.CreateAttestationLightClientALTAccounts(attestationClientID)...)

	altAddress, err := s.CreateAddressLookupTable(ctx, user, allAccounts)
	require.NoError(err)
	t.Logf("Created and extended ALT %s with %d accounts (including attestation LC)", altAddress, len(allAccounts))

	return altAddress
}

// MisbehaviourChunkPDA computes the PDA for a misbehaviour chunk account
func MisbehaviourChunkPDA(submitter solana.PublicKey, chunkIndex uint8, programID solana.PublicKey) (solana.PublicKey, uint8, error) {
	return solana.FindProgramAddress(
		[][]byte{
			[]byte("misbehaviour_chunk"),
			submitter.Bytes(),
			{chunkIndex},
		},
		programID,
	)
}

// ChunkDataSize is the maximum size of chunk data for multi-transaction uploads
const ChunkDataSize = 900

// SubmitChunkedMisbehaviour uploads misbehaviour data in chunks and assembles it to freeze the client.
// addressTables maps ALT addresses to their account lists for V0 transactions that reduce chunk tx size.
func (s *Solana) SubmitChunkedMisbehaviour(
	ctx context.Context,
	t *testing.T,
	require *require.Assertions,
	misbehaviourBytes []byte,
	trustedHeight1 uint64,
	trustedHeight2 uint64,
	user *solana.Wallet,
	addressTables map[solana.PublicKey]solana.PublicKeySlice,
) {
	t.Helper()

	totalStart := time.Now()
	t.Logf("=== Starting Chunked Misbehaviour Submission ===")
	t.Logf("Misbehaviour data size: %d bytes", len(misbehaviourBytes))

	// Split misbehaviour data into chunks
	var chunks [][]byte
	for i := 0; i < len(misbehaviourBytes); i += ChunkDataSize {
		end := i + ChunkDataSize
		if end > len(misbehaviourBytes) {
			end = len(misbehaviourBytes)
		}
		chunks = append(chunks, misbehaviourBytes[i:end])
	}
	t.Logf("Split into %d chunks", len(chunks))

	clientStatePDA, _ := Ics07Tendermint.ClientPDA(ics07_tendermint.ProgramID)
	appStatePDA, _ := Ics07Tendermint.AppStatePDA(ics07_tendermint.ProgramID)
	accessManagerPDA, _ := AccessManager.AccessManagerPDA(access_manager.ProgramID)

	// Phase 1: Upload all chunks in parallel
	t.Logf("--- Phase 1: Uploading %d chunks in parallel ---", len(chunks))
	chunksStart := time.Now()

	type chunkResult struct {
		chunkIdx int
		sig      solana.Signature
		err      error
		duration time.Duration
	}

	chunkResults := make(chan chunkResult, len(chunks))

	for chunkIdx, chunkData := range chunks {
		go func(idx int, data []byte) {
			chunkStart := time.Now()

			chunkPDA, _, err := MisbehaviourChunkPDA(user.PublicKey(), uint8(idx), ics07_tendermint.ProgramID)
			if err != nil {
				chunkResults <- chunkResult{chunkIdx: idx, err: fmt.Errorf("failed to derive chunk PDA: %w", err), duration: time.Since(chunkStart)}
				return
			}

			params := ics07_tendermint.Ics07TendermintTypesUploadMisbehaviourChunkParams{
				ChunkIndex: uint8(idx),
				ChunkData:  data,
			}

			ix, err := ics07_tendermint.NewUploadMisbehaviourChunkInstruction(
				params,
				chunkPDA,
				clientStatePDA,
				appStatePDA,
				accessManagerPDA,
				user.PublicKey(),
				solana.SysVarInstructionsPubkey,
				solana.SystemProgramID,
			)
			if err != nil {
				chunkResults <- chunkResult{chunkIdx: idx, err: fmt.Errorf("failed to create instruction: %w", err), duration: time.Since(chunkStart)}
				return
			}

			recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
			if err != nil {
				chunkResults <- chunkResult{chunkIdx: idx, err: fmt.Errorf("failed to get blockhash: %w", err), duration: time.Since(chunkStart)}
				return
			}

			tx, err := solana.NewTransaction(
				[]solana.Instruction{ix},
				recent.Value.Blockhash,
				solana.TransactionPayer(user.PublicKey()),
				solana.TransactionAddressTables(addressTables),
			)
			if err != nil {
				chunkResults <- chunkResult{chunkIdx: idx, err: fmt.Errorf("failed to create transaction: %w", err), duration: time.Since(chunkStart)}
				return
			}

			sig, err := s.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, user)
			chunkDuration := time.Since(chunkStart)

			if err != nil {
				chunkResults <- chunkResult{chunkIdx: idx, err: fmt.Errorf("failed to submit chunk: %w", err), duration: chunkDuration}
				return
			}

			chunkResults <- chunkResult{chunkIdx: idx, sig: sig, duration: chunkDuration}
		}(chunkIdx, chunkData)
	}

	// Collect all chunk results
	var chunkErr error
	for i := 0; i < len(chunks); i++ {
		result := <-chunkResults
		if result.err != nil {
			chunkErr = result.err
			t.Logf("✗ Chunk %d failed: %v", result.chunkIdx+1, result.err)
		} else {
			t.Logf("✓ Chunk %d/%d completed in %v - tx: %s",
				result.chunkIdx+1, len(chunks), result.duration, result.sig)
		}
	}
	close(chunkResults)

	chunksDuration := time.Since(chunksStart)
	require.NoError(chunkErr, "Chunk upload failed")
	t.Logf("--- Phase 1 Complete: All %d chunks uploaded in %v ---", len(chunks), chunksDuration)

	// Phase 2: Assemble and submit misbehaviour
	t.Logf("--- Phase 2: Assembling and submitting misbehaviour ---")
	assemblyStart := time.Now()

	// Convert heights to bytes for consensus state PDA
	height1Bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(height1Bytes, trustedHeight1)
	height2Bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(height2Bytes, trustedHeight2)

	// Get trusted consensus state PDAs
	trustedConsensusState1PDA, _ := Ics07Tendermint.ConsensusStateWithArgSeedPDA(ics07_tendermint.ProgramID, height1Bytes)
	trustedConsensusState2PDA, _ := Ics07Tendermint.ConsensusStateWithArgSeedPDA(ics07_tendermint.ProgramID, height2Bytes)

	assembleIx, err := ics07_tendermint.NewAssembleAndSubmitMisbehaviourInstruction(
		uint8(len(chunks)),
		trustedHeight1,
		trustedHeight2,
		clientStatePDA,
		appStatePDA,
		accessManagerPDA,
		trustedConsensusState1PDA,
		trustedConsensusState2PDA,
		user.PublicKey(),
		solana.SysVarInstructionsPubkey,
	)
	require.NoError(err, "Failed to create assemble instruction")

	// Add remaining accounts (chunk accounts) to the instruction
	if ix, ok := assembleIx.(*solana.GenericInstruction); ok {
		for i := 0; i < len(chunks); i++ {
			chunkPDA, _, err := MisbehaviourChunkPDA(user.PublicKey(), uint8(i), ics07_tendermint.ProgramID)
			require.NoError(err, "Failed to derive chunk PDA for assembly")
			ix.AccountValues = append(ix.AccountValues, solana.Meta(chunkPDA).WRITE())
		}
	}

	recent, err := s.RPCClient.GetLatestBlockhash(ctx, rpc.CommitmentConfirmed)
	require.NoError(err, "Failed to get blockhash for assembly")

	computeBudgetIx := NewComputeBudgetInstruction(1_400_000)

	assembleTx, err := solana.NewTransaction(
		[]solana.Instruction{computeBudgetIx, assembleIx},
		recent.Value.Blockhash,
		solana.TransactionPayer(user.PublicKey()),
	)
	require.NoError(err, "Failed to create assembly transaction")

	assembleSig, err := s.SignAndBroadcastTxWithOpts(ctx, assembleTx, rpc.ConfirmationStatusConfirmed, user)
	assemblyDuration := time.Since(assemblyStart)

	if err != nil {
		t.Logf("Assembly transaction error: %v", err)
		s.LogTransactionDetails(ctx, t, assembleSig, "FAILED Assembly Transaction")
	}
	require.NoError(err, "Assembly transaction failed")

	t.Logf("✓ Assembly transaction completed in %v - tx: %s", assemblyDuration, assembleSig)
	s.LogTransactionDetails(ctx, t, assembleSig, "SUCCESS: Assembly Transaction")

	totalDuration := time.Since(totalStart)
	t.Logf("=== Chunked Misbehaviour Submission Complete ===")
	t.Logf("Total time: %v", totalDuration)
	t.Logf("  - Phase 1 (Chunk uploads): %v", chunksDuration)
	t.Logf("  - Phase 2 (Assembly): %v", assemblyDuration)
}
