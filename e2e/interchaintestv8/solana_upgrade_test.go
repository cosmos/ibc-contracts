// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/suite"

	solanago "github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"

	access_manager "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/accessmanager"
	ics07_tendermint "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics07tendermint"
	ics26_router "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics26router"
	ics27_gmp "github.com/cosmos/solidity-ibc-eureka/packages/go-anchor/ics27gmp"

	"github.com/srdtrk/solidity-ibc-eureka/e2e/v8/solana"
)

const (
	keypairDir    = "solana-keypairs/localnet"
	deployerPath  = keypairDir + "/deployer_wallet.json"
	programSoFile = "ibc-solana/target/deploy/ics26_router.so"
	ADMIN_ROLE    = uint64(0)
)

// IbcEurekaSolanaUpgradeTestSuite tests program upgradability via AccessManager
type IbcEurekaSolanaUpgradeTestSuite struct {
	IbcEurekaSolanaTestSuite

	UpgraderWallet *solanago.Wallet
}

func TestWithIbcEurekaSolanaUpgradeTestSuite(t *testing.T) {
	s := &IbcEurekaSolanaUpgradeTestSuite{}
	suite.Run(t, s)
}

// Test_ProgramUpgrade_Via_AccessManager demonstrates the complete upgrade flow with role-based access control.
//
// BACKGROUND:
// Solana's BPF Loader Upgradeable uses a two-account system:
// - Program Account: Executable with fixed address (what users call)
// - ProgramData Account: Contains bytecode and upgrade authority metadata
//
// The upgrade authority controls who can upgrade the program. By setting it to an AccessManager-controlled
// PDA, we enable role-based upgrades where only accounts with ADMIN_ROLE can upgrade.
//
// TEST FLOW:
// 1. Create an upgrader wallet and grant it ADMIN_ROLE
// 2. Derive required PDAs (program data account, upgrade authority PDA)
// 3. Transfer program upgrade authority from deployer to AccessManager's PDA (one-time setup)
// 4. Write new program bytecode to a buffer account
// 5. Transfer buffer authority to match program upgrade authority (security requirement)
// 6. Call AccessManager.upgrade_program() with the upgrader wallet (has ADMIN_ROLE)
//   - AccessManager checks role membership
//   - AccessManager calls BPFLoaderUpgradeable.upgrade via invoke_signed with PDA signature
//   - BPF Loader verifies authorities match and replaces bytecode
//
// 7. Verify unauthorized accounts cannot upgrade (negative test)
//
// SECURITY MODEL:
// - Role-based access: Only ADMIN_ROLE can trigger upgrades (AccessManager enforcement)
// - Authority matching: Buffer authority must equal program upgrade authority (BPF Loader enforcement)
// - CPI protection: Upgrade cannot be called via CPI (instructions sysvar check)
// - PDA verification: Upgrade authority PDA seeds are validated (Anchor constraints)
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_ProgramUpgrade_Via_AccessManager() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	s.Require().True(s.Run("Setup: Create upgrader wallet", func() {
		var err error
		s.UpgraderWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err, "failed to create and fund upgrader wallet")
	}))

	s.Require().True(s.Run("Setup: Grant ADMIN_ROLE to upgrader wallet", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		grantAdminRoleInstruction, err := access_manager.NewGrantRoleInstruction(
			ADMIN_ROLE,
			s.UpgraderWallet.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err, "failed to build grant ADMIN_ROLE instruction")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			grantAdminRoleInstruction,
		)
		s.Require().NoError(err, "failed to create grant role transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to grant ADMIN_ROLE")
	}))

	targetProgramID := ics26_router.ProgramID

	var programDataAccount solanago.PublicKey
	var upgradeAuthorityPDA solanago.PublicKey

	s.Require().True(s.Run("Derive upgrade authority and program data accounts", func() {
		var err error

		programDataAccount, err = solana.GetProgramDataAddress(targetProgramID)
		s.Require().NoError(err, "failed to derive program data address")

		upgradeAuthorityPDA, _ = solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
			access_manager.ProgramID,
			targetProgramID.Bytes(),
		)
	}))

	s.Require().True(s.Run("Transfer program upgrade authority to AccessManager", func() {
		err := solana.SetUpgradeAuthority(
			ctx,
			targetProgramID,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to transfer program upgrade authority to AccessManager")
	}))

	var bufferAccount solanago.PublicKey

	s.Require().True(s.Run("Write new program binary to buffer and transfer authority", func() {
		var err error

		// For this test, we use the same binary to focus on the upgrade mechanism
		// and access control, which is the primary goal of this test suite.
		bufferAccount, err = solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to write program buffer")
		s.Require().NotEqual(solanago.PublicKey{}, bufferAccount, "buffer account should not be empty")

		// Transfer buffer authority to match program upgrade authority (security requirement)
		err = solana.SetBufferAuthority(
			ctx,
			bufferAccount,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to transfer buffer authority to upgrade authority PDA")
	}))

	s.Require().True(s.Run("Upgrade program via AccessManager with ADMIN_ROLE", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		upgradeProgramInstruction, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessControlAccount,
			targetProgramID,
			programDataAccount,
			bufferAccount,
			upgradeAuthorityPDA,
			s.UpgraderWallet.PublicKey(),
			s.UpgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		s.Require().NoError(err, "failed to build upgrade program instruction")

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			computeBudgetIx,
			upgradeProgramInstruction,
		)
		s.Require().NoError(err, "failed to create upgrade transaction")

		sig, err := s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet)
		s.Require().NoError(err, "program upgrade should succeed with ADMIN_ROLE")
		s.Require().NotEqual(solanago.Signature{}, sig, "upgrade signature should not be empty")
	}))

	s.Require().True(s.Run("Unauthorized account cannot upgrade program", func() {
		unauthorizedWallet, err := s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err, "failed to create unauthorized wallet")

		unauthorizedBuffer, err := solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to write buffer for unauthorized test")

		err = solana.SetBufferAuthority(
			ctx,
			unauthorizedBuffer,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to transfer buffer authority for unauthorized test")

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		upgradeProgramInstruction, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessControlAccount,
			targetProgramID,
			programDataAccount,
			unauthorizedBuffer,
			upgradeAuthorityPDA,
			unauthorizedWallet.PublicKey(),
			unauthorizedWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		s.Require().NoError(err, "failed to build upgrade instruction for unauthorized test")

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			unauthorizedWallet.PublicKey(),
			computeBudgetIx,
			upgradeProgramInstruction,
		)
		s.Require().NoError(err, "failed to create unauthorized upgrade transaction")

		// Use SignAndBroadcastTxWithOpts for immediate failure without retry (this is a negative test)
		_, err = s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, unauthorizedWallet)
		s.Require().Error(err, "upgrade should fail without ADMIN_ROLE")
		s.Require().Contains(err.Error(), "Custom", "should be Unauthorized error")
	}))

	s.Require().True(s.Run("Cannot bypass AccessManager and upgrade directly", func() {
		// This test verifies that after transferring upgrade authority to the AccessManager PDA,
		// the old authority (deployer) cannot bypass AccessManager by calling BPF Loader directly.

		// Create a buffer with deployer as authority
		bypassBuffer, err := solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to write buffer for bypass test")

		// Attempt to upgrade directly using BPF Loader (bypassing AccessManager)
		// This should fail because the program's upgrade authority is now the AccessManager PDA,
		// not the deployer wallet
		err = solana.UpgradeProgramDirect(
			ctx,
			targetProgramID,
			bypassBuffer,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		// The direct upgrade should fail because the program's upgrade authority is now the AccessManager PDA
		s.Require().Error(err, "direct upgrade should fail - authority is now AccessManager PDA")
		s.Require().Contains(err.Error(), "does not match authority", "should fail with authority mismatch")
	}))
}

// Test_RevokeAdminRole demonstrates that revoking ADMIN_ROLE from an account prevents upgrades
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_RevokeAdminRole() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	var secondAdmin *solanago.Wallet

	s.Require().True(s.Run("Setup: Create and grant ADMIN_ROLE to second admin", func() {
		var err error
		secondAdmin, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		grantInstruction, err := access_manager.NewGrantRoleInstruction(
			ADMIN_ROLE,
			secondAdmin.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			grantInstruction,
		)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err)
	}))

	s.Require().True(s.Run("Revoke ADMIN_ROLE from second admin", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		revokeInstruction, err := access_manager.NewRevokeRoleInstruction(
			ADMIN_ROLE,
			secondAdmin.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(), // Primary admin revokes
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			revokeInstruction,
		)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to revoke ADMIN_ROLE")
	}))

	s.Require().True(s.Run("Verify revoked admin cannot upgrade", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)
		targetProgramID := ics26_router.ProgramID
		programDataAccount, err := solana.GetProgramDataAddress(targetProgramID)
		s.Require().NoError(err)

		upgradeAuthorityPDA, _ := solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
			access_manager.ProgramID,
			targetProgramID.Bytes(),
		)

		// Write buffer
		buffer, err := solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err)

		// Transfer buffer authority to upgrade authority PDA
		err = solana.SetBufferAuthority(
			ctx,
			buffer,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err)

		upgradeInstruction, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessControlAccount,
			targetProgramID,
			programDataAccount,
			buffer,
			upgradeAuthorityPDA,
			secondAdmin.PublicKey(),
			secondAdmin.PublicKey(), // Revoked admin
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			secondAdmin.PublicKey(),
			computeBudgetIx,
			upgradeInstruction,
		)
		s.Require().NoError(err)

		// Use SignAndBroadcastTxWithOpts for immediate failure without retry (this is a negative test)
		_, err = s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, secondAdmin)
		s.Require().Error(err, "upgrade should fail after ADMIN_ROLE revocation")
		s.Require().Contains(err.Error(), "Custom", "should be a program error")
	}))
}

// Test_TransferUpgradeAuthority demonstrates the two-step authority transfer flow for access manager migration.
//
// SCENARIO:
// When migrating to a new access manager or transferring upgrade control, the current access manager
// needs to relinquish its upgrade authority over managed programs. This uses a two-step propose/accept
// pattern to prevent irreversible mistakes:
//   - Admin proposes the transfer (sets pending state)
//   - New authority accepts the transfer (executes the BPF Loader SetAuthority CPI)
//
// TEST FLOW:
// 1. Grant ADMIN_ROLE to upgrader wallet
// 2. Transfer program upgrade authority from deployer to AccessManager's PDA (standard setup)
// 3. Verify baseline: upgrade via AccessManager works
// 4. Create a new authority keypair
// 5. Propose transfer: admin calls propose_upgrade_authority_transfer
// 6. Accept transfer: new authority calls accept_upgrade_authority_transfer
// 7. Verify new authority can upgrade the program directly
// 8. Verify AccessManager can no longer upgrade the program (negative test)
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_TransferUpgradeAuthority() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	s.Require().True(s.Run("Setup: Create upgrader wallet", func() {
		var err error
		s.UpgraderWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err, "failed to create and fund upgrader wallet")
	}))

	s.Require().True(s.Run("Setup: Grant ADMIN_ROLE to upgrader wallet", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		grantAdminRoleInstruction, err := access_manager.NewGrantRoleInstruction(
			ADMIN_ROLE,
			s.UpgraderWallet.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err, "failed to build grant ADMIN_ROLE instruction")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			grantAdminRoleInstruction,
		)
		s.Require().NoError(err, "failed to create grant role transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to grant ADMIN_ROLE")
	}))

	targetProgramID := ics26_router.ProgramID

	var programDataAccount solanago.PublicKey
	var upgradeAuthorityPDA solanago.PublicKey

	s.Require().True(s.Run("Derive upgrade authority and program data accounts", func() {
		var err error

		programDataAccount, err = solana.GetProgramDataAddress(targetProgramID)
		s.Require().NoError(err, "failed to derive program data address")

		upgradeAuthorityPDA, _ = solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
			access_manager.ProgramID,
			targetProgramID.Bytes(),
		)
	}))

	s.Require().True(s.Run("Transfer program upgrade authority to AccessManager", func() {
		err := solana.SetUpgradeAuthority(
			ctx,
			targetProgramID,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to transfer program upgrade authority to AccessManager")
	}))

	s.Require().True(s.Run("Verify baseline: upgrade via AccessManager succeeds", func() {
		bufferAccount, err := solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to write program buffer")

		err = solana.SetBufferAuthority(
			ctx,
			bufferAccount,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to transfer buffer authority")

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		upgradeProgramInstruction, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessControlAccount,
			targetProgramID,
			programDataAccount,
			bufferAccount,
			upgradeAuthorityPDA,
			s.UpgraderWallet.PublicKey(),
			s.UpgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		s.Require().NoError(err, "failed to build upgrade program instruction")

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			computeBudgetIx,
			upgradeProgramInstruction,
		)
		s.Require().NoError(err, "failed to create upgrade transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet)
		s.Require().NoError(err, "baseline upgrade via AccessManager should succeed")
	}))

	var newAuthorityWallet *solanago.Wallet
	var newAuthorityKeypairPath string

	s.Require().True(s.Run("Create new authority keypair", func() {
		var err error
		newAuthorityWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err, "failed to create new authority wallet")

		newAuthorityKeypairPath, err = solana.WriteKeypairToTempFile(newAuthorityWallet)
		s.Require().NoError(err, "failed to write new authority keypair to temp file")
	}))

	defer func() {
		if newAuthorityKeypairPath != "" {
			os.Remove(newAuthorityKeypairPath)
		}
	}()

	s.Require().True(s.Run("Propose upgrade authority transfer", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		proposeIx, err := access_manager.NewProposeUpgradeAuthorityTransferInstruction(
			targetProgramID,
			newAuthorityWallet.PublicKey(),
			accessControlAccount,
			s.UpgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err, "failed to build propose upgrade authority transfer instruction")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			proposeIx,
		)
		s.Require().NoError(err, "failed to create propose transfer transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet)
		s.Require().NoError(err, "propose upgrade authority transfer should succeed")
	}))

	s.Require().True(s.Run("Accept upgrade authority transfer", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		acceptIx, err := access_manager.NewAcceptUpgradeAuthorityTransferInstruction(
			targetProgramID,
			accessControlAccount,
			programDataAccount,
			upgradeAuthorityPDA,
			newAuthorityWallet.PublicKey(),
			solanago.BPFLoaderUpgradeableProgramID,
		)
		s.Require().NoError(err, "failed to build accept upgrade authority transfer instruction")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			newAuthorityWallet.PublicKey(),
			acceptIx,
		)
		s.Require().NoError(err, "failed to create accept transfer transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, newAuthorityWallet)
		s.Require().NoError(err, "accept upgrade authority transfer should succeed")
	}))

	s.Require().True(s.Run("New authority can upgrade program directly", func() {
		// Use deployer to write the buffer (has more SOL for rent)
		bufferAccount, err := solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to write program buffer")

		// Transfer buffer authority to the new authority
		err = solana.SetBufferAuthority(
			ctx,
			bufferAccount,
			newAuthorityWallet.PublicKey(),
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to set buffer authority to new authority")

		err = solana.UpgradeProgramDirect(
			ctx,
			targetProgramID,
			bufferAccount,
			newAuthorityKeypairPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "new authority should be able to upgrade program directly")
	}))

	s.Require().True(s.Run("AccessManager can no longer upgrade program", func() {
		bufferAccount, err := solana.WriteProgramBuffer(
			ctx,
			programSoFile,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to write program buffer for AM upgrade attempt")

		err = solana.SetBufferAuthority(
			ctx,
			bufferAccount,
			upgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to set buffer authority to AM PDA")

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		upgradeProgramInstruction, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessControlAccount,
			targetProgramID,
			programDataAccount,
			bufferAccount,
			upgradeAuthorityPDA,
			s.UpgraderWallet.PublicKey(),
			s.UpgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		s.Require().NoError(err, "failed to build upgrade instruction for AM attempt")

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			computeBudgetIx,
			upgradeProgramInstruction,
		)
		s.Require().NoError(err, "failed to create AM upgrade transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.UpgraderWallet)
		s.Require().Error(err, "AM upgrade should fail after authority was transferred away")
		s.Require().Contains(err.Error(), "IncorrectAuthority", "should fail with authority mismatch")
	}))
}

// withAMProgramID temporarily swaps access_manager.ProgramID to build instructions for a different AM instance.
func withAMProgramID(programID solanago.PublicKey, fn func() (solanago.Instruction, error)) (solanago.Instruction, error) {
	saved := access_manager.ProgramID
	access_manager.ProgramID = programID
	defer func() { access_manager.ProgramID = saved }()
	return fn()
}

// Test_AMtoAM_UpgradeAuthorityMigration demonstrates migrating upgrade authority
// from one AccessManager instance (AM-A) to another (AM-B) using claim_upgrade_authority.
//
// SCENARIO:
// When replacing an AccessManager deployment (e.g. upgrading AM logic), the old AM
// must transfer its upgrade authority over managed programs to the new AM. Since the
// new authority is a PDA (not a keypair), the standard propose/accept flow doesn't
// work -- PDAs can only sign via invoke_signed from their owning program.
//
// claim_upgrade_authority solves this: AM-B CPIs into AM-A's accept_upgrade_authority_transfer
// with its own PDA as the signer.
//
// TEST FLOW:
// 1. Deploy AM-B (test_access_manager) alongside the already-deployed AM-A
// 2. Initialize AM-B with deployer as admin
// 3. Grant ADMIN_ROLE on AM-A to upgrader wallet
// 4. Transfer target program's upgrade authority to AM-A's PDA
// 5. AM-A admin proposes transfer to AM-B's upgrade authority PDA
// 6. Anyone calls AM-B's claim_upgrade_authority (PDA signing = authorization)
// 7. Verify: target program's authority is now AM-B's PDA
// 8. Verify: AM-B admin can upgrade the target program
// 9. Verify: AM-A can no longer upgrade the target program
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_AMtoAM_UpgradeAuthorityMigration() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	// Deploy AM-B (test_access_manager)
	var amBProgramID solanago.PublicKey

	s.Require().True(s.Run("Deploy AM-B (test_access_manager)", func() {
		var err error
		amBKeypairPath := fmt.Sprintf("%s/test_access_manager-keypair.json", keypairDir)
		amBProgramID, err = s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_access_manager", amBKeypairPath, deployerPath)
		s.Require().NoError(err, "failed to deploy test_access_manager")
		s.T().Logf("AM-B deployed at: %s", amBProgramID)
	}))

	// Initialize AM-B
	s.Require().True(s.Run("Initialize AM-B", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		amBProgramDataPDA, err := solana.GetProgramDataAddress(amBProgramID)
		s.Require().NoError(err)

		initIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewInitializeInstruction(
				s.SolanaRelayer.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SystemProgramID,
				solanago.SysVarInstructionsPubkey,
				amBProgramDataPDA,
				solana.DeployerPubkey,
			)
		})
		s.Require().NoError(err, "failed to build AM-B initialize instruction")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err, "failed to initialize AM-B")
		s.T().Log("AM-B initialized")
	}))

	// Setup upgrader wallet with ADMIN_ROLE on AM-A
	s.Require().True(s.Run("Setup: Create upgrader wallet and grant ADMIN_ROLE on AM-A", func() {
		var err error
		s.UpgraderWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		grantIx, err := access_manager.NewGrantRoleInstruction(
			ADMIN_ROLE,
			s.UpgraderWallet.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), grantIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to grant ADMIN_ROLE on AM-A")
	}))

	targetProgramID := ics26_router.ProgramID

	var programDataAccount solanago.PublicKey
	var amAUpgradeAuthorityPDA solanago.PublicKey
	var amBUpgradeAuthorityPDA solanago.PublicKey

	s.Require().True(s.Run("Derive PDAs", func() {
		var err error

		programDataAccount, err = solana.GetProgramDataAddress(targetProgramID)
		s.Require().NoError(err)

		amAUpgradeAuthorityPDA, _ = solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
			access_manager.ProgramID,
			targetProgramID.Bytes(),
		)

		amBUpgradeAuthorityPDA, _ = solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
			amBProgramID,
			targetProgramID.Bytes(),
		)

		s.T().Logf("AM-A upgrade authority PDA: %s", amAUpgradeAuthorityPDA)
		s.T().Logf("AM-B upgrade authority PDA: %s", amBUpgradeAuthorityPDA)
	}))

	// Transfer target program's upgrade authority to AM-A's PDA
	s.Require().True(s.Run("Transfer target program authority to AM-A's PDA", func() {
		err := solana.SetUpgradeAuthority(
			ctx,
			targetProgramID,
			amAUpgradeAuthorityPDA,
			deployerPath,
			s.Solana.Chain.RPCURL,
		)
		s.Require().NoError(err, "failed to transfer authority to AM-A")
	}))

	// AM-A proposes transfer to AM-B's upgrade authority PDA
	s.Require().True(s.Run("AM-A proposes transfer to AM-B's PDA", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		proposeIx, err := access_manager.NewProposeUpgradeAuthorityTransferInstruction(
			targetProgramID,
			amBUpgradeAuthorityPDA,
			accessControlAccount,
			s.UpgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err, "failed to build propose instruction")

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.UpgraderWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet)
		s.Require().NoError(err, "propose transfer to AM-B should succeed")
	}))

	// AM-B claims upgrade authority via CPI into AM-A's accept.
	// Requires ADMIN_ROLE on AM-B to prevent malicious source programs from
	// exploiting the PDA signer propagation to steal upgrade authority.
	s.Require().True(s.Run("AM-B claims upgrade authority (admin-gated)", func() {
		amAProgramID := access_manager.ProgramID
		amAAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amAProgramID)
		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

		claimIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewClaimUpgradeAuthorityInstruction(
				targetProgramID,
				amBUpgradeAuthorityPDA,
				amAAccessManagerPDA,
				programDataAccount,
				amAUpgradeAuthorityPDA,
				amAProgramID,
				solanago.BPFLoaderUpgradeableProgramID,
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SysVarInstructionsPubkey,
			)
		})
		s.Require().NoError(err, "failed to build claim instruction")

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.SolanaRelayer.PublicKey(),
			computeBudgetIx,
			claimIx,
		)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "claim should succeed when called by AM-B admin")
	}))

	// Verify: AM-B admin can upgrade the target program
	s.Require().True(s.Run("Grant ADMIN_ROLE on AM-B and verify upgrade works", func() {
		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

		// Grant ADMIN_ROLE on AM-B to upgrader wallet
		grantIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewGrantRoleInstruction(
				ADMIN_ROLE,
				s.UpgraderWallet.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SysVarInstructionsPubkey,
			)
		})
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), grantIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to grant ADMIN_ROLE on AM-B")

		// Write buffer and transfer authority to AM-B's PDA
		bufferAccount, err := solana.WriteProgramBuffer(ctx, programSoFile, deployerPath, s.Solana.Chain.RPCURL)
		s.Require().NoError(err, "failed to write program buffer")

		err = solana.SetBufferAuthority(ctx, bufferAccount, amBUpgradeAuthorityPDA, deployerPath, s.Solana.Chain.RPCURL)
		s.Require().NoError(err, "failed to set buffer authority to AM-B's PDA")

		// Upgrade via AM-B
		upgradeIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewUpgradeProgramInstruction(
				targetProgramID,
				amBAccessManagerPDA,
				targetProgramID,
				programDataAccount,
				bufferAccount,
				amBUpgradeAuthorityPDA,
				s.UpgraderWallet.PublicKey(),
				s.UpgraderWallet.PublicKey(),
				solanago.SysVarInstructionsPubkey,
				solanago.BPFLoaderUpgradeableProgramID,
				solanago.SysVarRentPubkey,
				solanago.SysVarClockPubkey,
			)
		})
		s.Require().NoError(err, "failed to build AM-B upgrade instruction")

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err = s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			computeBudgetIx,
			upgradeIx,
		)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet)
		s.Require().NoError(err, "upgrade via AM-B should succeed after claiming authority")
	}))

	// Verify: AM-A can no longer upgrade
	s.Require().True(s.Run("AM-A can no longer upgrade the target program", func() {
		bufferAccount, err := solana.WriteProgramBuffer(ctx, programSoFile, deployerPath, s.Solana.Chain.RPCURL)
		s.Require().NoError(err)

		err = solana.SetBufferAuthority(ctx, bufferAccount, amAUpgradeAuthorityPDA, deployerPath, s.Solana.Chain.RPCURL)
		s.Require().NoError(err)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		upgradeIx, err := access_manager.NewUpgradeProgramInstruction(
			targetProgramID,
			accessControlAccount,
			targetProgramID,
			programDataAccount,
			bufferAccount,
			amAUpgradeAuthorityPDA,
			s.UpgraderWallet.PublicKey(),
			s.UpgraderWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
			solanago.BPFLoaderUpgradeableProgramID,
			solanago.SysVarRentPubkey,
			solanago.SysVarClockPubkey,
		)
		s.Require().NoError(err)

		computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			computeBudgetIx,
			upgradeIx,
		)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.UpgraderWallet)
		s.Require().Error(err, "AM-A upgrade should fail after authority was migrated to AM-B")
		s.Require().Contains(err.Error(), "IncorrectAuthority", "should fail with authority mismatch")
	}))
}

func (s *IbcEurekaSolanaUpgradeTestSuite) Test_AccessManagerTransfer() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	// --- Deploy and initialize AM-B ---

	var amBProgramID solanago.PublicKey

	s.Require().True(s.Run("Deploy AM-B (test_access_manager)", func() {
		var err error
		amBKeypairPath := fmt.Sprintf("%s/test_access_manager-keypair.json", keypairDir)
		amBProgramID, err = s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_access_manager", amBKeypairPath, deployerPath)
		s.Require().NoError(err, "failed to deploy test_access_manager")
		s.T().Logf("AM-B deployed at: %s", amBProgramID)
	}))

	s.Require().True(s.Run("Initialize AM-B with relayer as admin", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		amBProgramDataPDA, err := solana.GetProgramDataAddress(amBProgramID)
		s.Require().NoError(err)

		initIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewInitializeInstruction(
				s.SolanaRelayer.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SystemProgramID,
				solanago.SysVarInstructionsPubkey,
				amBProgramDataPDA,
				solana.DeployerPubkey,
			)
		})
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err, "failed to initialize AM-B")
	}))

	// --- Helper: read ICS26 router state ---

	routerStatePDA, _ := solana.Ics26Router.RouterStatePDA(ics26_router.ProgramID)

	readRouterState := func() *ics26_router.Ics26RouterStateRouterState {
		s.T().Helper()
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, routerStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)
		state, err := ics26_router.ParseAccount_Ics26RouterStateRouterState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)
		return state
	}

	// --- Verify initial state ---

	amAAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

	s.Require().True(s.Run("Verify initial state: AM-A is active, no pending", func() {
		state := readRouterState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "ICS26 should point to AM-A")
		s.Require().Nil(state.AmState.PendingAccessManager, "No pending transfer initially")
	}))

	// --- Propose transfer to AM-B ---

	s.Require().True(s.Run("Propose access manager transfer to AM-B", func() {
		proposeIx, err := ics26_router.NewProposeAccessManagerTransferInstruction(
			amBProgramID,
			routerStatePDA,
			amAAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "propose access manager transfer should succeed")
	}))

	s.Require().True(s.Run("Verify: pending set, AM unchanged", func() {
		state := readRouterState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "AM should still be AM-A")
		s.Require().NotNil(state.AmState.PendingAccessManager, "Pending should be set")
		s.Require().Equal(amBProgramID, *state.AmState.PendingAccessManager, "Pending should be AM-B")
	}))

	// --- Negative test: non-admin cannot propose ---

	s.Require().True(s.Run("Non-admin propose fails", func() {
		unauthorizedWallet, err := s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		// First cancel the existing proposal so we can test a fresh propose
		cancelIx, err := ics26_router.NewCancelAccessManagerTransferInstruction(
			routerStatePDA,
			amAAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		cancelTx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, cancelTx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "cancel should succeed")

		// Now try to propose with unauthorized wallet
		proposeIx, err := ics26_router.NewProposeAccessManagerTransferInstruction(
			amBProgramID,
			routerStatePDA,
			amAAccessManagerPDA,
			unauthorizedWallet.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(unauthorizedWallet.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, unauthorizedWallet)
		s.Require().Error(err, "non-admin propose should fail")
		s.Require().Contains(err.Error(), "Custom", "should be a program error")
	}))

	// --- Re-propose and accept ---

	s.Require().True(s.Run("Re-propose access manager transfer to AM-B", func() {
		proposeIx, err := ics26_router.NewProposeAccessManagerTransferInstruction(
			amBProgramID,
			routerStatePDA,
			amAAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "re-propose should succeed")
	}))

	amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

	s.Require().True(s.Run("Accept access manager transfer (AM-B admin)", func() {
		acceptIx, err := ics26_router.NewAcceptAccessManagerTransferInstruction(
			routerStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), acceptIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "accept access manager transfer should succeed")
	}))

	s.Require().True(s.Run("Verify: AM is now AM-B, pending cleared", func() {
		state := readRouterState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should now be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after accept")
	}))

	// --- Test cancel flow: propose back to AM-A then cancel ---

	s.Require().True(s.Run("Propose transfer back to AM-A", func() {
		// AM is now AM-B, so access_manager account must be AM-B's PDA
		proposeIx, err := ics26_router.NewProposeAccessManagerTransferInstruction(
			access_manager.ProgramID, // propose back to AM-A
			routerStatePDA,
			amBAccessManagerPDA, // current AM is AM-B
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "propose transfer back to AM-A should succeed")
	}))

	s.Require().True(s.Run("Verify: pending set to AM-A", func() {
		state := readRouterState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should still be AM-B")
		s.Require().NotNil(state.AmState.PendingAccessManager)
		s.Require().Equal(access_manager.ProgramID, *state.AmState.PendingAccessManager, "Pending should be AM-A")
	}))

	s.Require().True(s.Run("Cancel pending transfer", func() {
		// Current AM is AM-B, so access_manager account is AM-B's PDA
		cancelIx, err := ics26_router.NewCancelAccessManagerTransferInstruction(
			routerStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "cancel should succeed")
	}))

	s.Require().True(s.Run("Verify: pending cleared, AM still AM-B", func() {
		state := readRouterState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should still be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after cancel")
	}))
}

// Test_AccessManagerTransfer_ICS07 tests propose/accept/cancel access manager
// transfer on the ICS07 Tendermint light client program.
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_AccessManagerTransfer_ICS07() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	// --- Deploy and initialize AM-B ---

	var amBProgramID solanago.PublicKey

	s.Require().True(s.Run("Deploy AM-B (test_access_manager)", func() {
		var err error
		amBKeypairPath := fmt.Sprintf("%s/test_access_manager-keypair.json", keypairDir)
		amBProgramID, err = s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_access_manager", amBKeypairPath, deployerPath)
		s.Require().NoError(err, "failed to deploy test_access_manager")
	}))

	s.Require().True(s.Run("Initialize AM-B with relayer as admin", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		amBProgramDataPDA, err := solana.GetProgramDataAddress(amBProgramID)
		s.Require().NoError(err)

		initIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewInitializeInstruction(
				s.SolanaRelayer.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SystemProgramID,
				solanago.SysVarInstructionsPubkey,
				amBProgramDataPDA,
				solana.DeployerPubkey,
			)
		})
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err, "failed to initialize AM-B")
	}))

	// --- Helper: read ICS07 app state ---

	appStatePDA, _ := solana.Ics07Tendermint.AppStatePDA(ics07_tendermint.ProgramID)

	readAppState := func() *ics07_tendermint.Ics07TendermintTypesAppState {
		s.T().Helper()
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, appStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)
		state, err := ics07_tendermint.ParseAccount_Ics07TendermintTypesAppState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)
		return state
	}

	amAAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

	// --- Verify initial state ---

	s.Require().True(s.Run("Verify initial state: AM-A is active, no pending", func() {
		state := readAppState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "ICS07 should point to AM-A")
		s.Require().Nil(state.AmState.PendingAccessManager, "No pending transfer initially")
	}))

	// --- Propose transfer to AM-B ---

	s.Require().True(s.Run("Propose access manager transfer to AM-B", func() {
		proposeIx, err := ics07_tendermint.NewProposeAccessManagerTransferInstruction(
			amBProgramID,
			appStatePDA,
			amAAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "propose should succeed")
	}))

	s.Require().True(s.Run("Verify: pending set, AM unchanged", func() {
		state := readAppState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "AM should still be AM-A")
		s.Require().NotNil(state.AmState.PendingAccessManager, "Pending should be set")
		s.Require().Equal(amBProgramID, *state.AmState.PendingAccessManager, "Pending should be AM-B")
	}))

	// --- Accept transfer ---

	amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

	s.Require().True(s.Run("Accept access manager transfer (AM-B admin)", func() {
		acceptIx, err := ics07_tendermint.NewAcceptAccessManagerTransferInstruction(
			appStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), acceptIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "accept should succeed")
	}))

	s.Require().True(s.Run("Verify: AM is now AM-B, pending cleared", func() {
		state := readAppState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should now be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after accept")
	}))

	// --- Propose back to AM-A and cancel ---

	s.Require().True(s.Run("Propose transfer back to AM-A", func() {
		proposeIx, err := ics07_tendermint.NewProposeAccessManagerTransferInstruction(
			access_manager.ProgramID,
			appStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "propose back to AM-A should succeed")
	}))

	s.Require().True(s.Run("Cancel pending transfer", func() {
		cancelIx, err := ics07_tendermint.NewCancelAccessManagerTransferInstruction(
			appStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "cancel should succeed")
	}))

	s.Require().True(s.Run("Verify: pending cleared, AM still AM-B", func() {
		state := readAppState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should still be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after cancel")
	}))
}

// Test_AccessManagerTransfer_GMP tests propose/accept/cancel access manager
// transfer on the ICS27 GMP program.
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_AccessManagerTransfer_GMP() {
	ctx := context.Background()

	s.SetupSuite(ctx)
	s.initializeICS27GMP(ctx)

	// --- Deploy and initialize AM-B ---

	var amBProgramID solanago.PublicKey

	s.Require().True(s.Run("Deploy AM-B (test_access_manager)", func() {
		var err error
		amBKeypairPath := fmt.Sprintf("%s/test_access_manager-keypair.json", keypairDir)
		amBProgramID, err = s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_access_manager", amBKeypairPath, deployerPath)
		s.Require().NoError(err, "failed to deploy test_access_manager")
	}))

	s.Require().True(s.Run("Initialize AM-B with relayer as admin", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		amBProgramDataPDA, err := solana.GetProgramDataAddress(amBProgramID)
		s.Require().NoError(err)

		initIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewInitializeInstruction(
				s.SolanaRelayer.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SystemProgramID,
				solanago.SysVarInstructionsPubkey,
				amBProgramDataPDA,
				solana.DeployerPubkey,
			)
		})
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err, "failed to initialize AM-B")
	}))

	// --- Helper: read GMP app state ---

	gmpAppStatePDA, _ := solana.Ics27Gmp.AppStatePDA(ics27_gmp.ProgramID)

	readAppState := func() *ics27_gmp.Ics27GmpStateGmpAppState {
		s.T().Helper()
		accountInfo, err := s.Solana.Chain.RPCClient.GetAccountInfoWithOpts(ctx, gmpAppStatePDA, &rpc.GetAccountInfoOpts{
			Commitment: rpc.CommitmentConfirmed,
		})
		s.Require().NoError(err)
		s.Require().NotNil(accountInfo.Value)
		state, err := ics27_gmp.ParseAccount_Ics27GmpStateGmpAppState(accountInfo.Value.Data.GetBinary())
		s.Require().NoError(err)
		return state
	}

	amAAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

	// --- Verify initial state ---

	s.Require().True(s.Run("Verify initial state: AM-A is active, no pending", func() {
		state := readAppState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "GMP should point to AM-A")
		s.Require().Nil(state.AmState.PendingAccessManager, "No pending transfer initially")
	}))

	// --- Propose transfer to AM-B ---

	s.Require().True(s.Run("Propose access manager transfer to AM-B", func() {
		proposeIx, err := ics27_gmp.NewProposeAccessManagerTransferInstruction(
			amBProgramID,
			gmpAppStatePDA,
			amAAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "propose should succeed")
	}))

	s.Require().True(s.Run("Verify: pending set, AM unchanged", func() {
		state := readAppState()
		s.Require().Equal(access_manager.ProgramID, state.AmState.AccessManager, "AM should still be AM-A")
		s.Require().NotNil(state.AmState.PendingAccessManager, "Pending should be set")
		s.Require().Equal(amBProgramID, *state.AmState.PendingAccessManager, "Pending should be AM-B")
	}))

	// --- Accept transfer ---

	amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

	s.Require().True(s.Run("Accept access manager transfer (AM-B admin)", func() {
		acceptIx, err := ics27_gmp.NewAcceptAccessManagerTransferInstruction(
			gmpAppStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), acceptIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "accept should succeed")
	}))

	s.Require().True(s.Run("Verify: AM is now AM-B, pending cleared", func() {
		state := readAppState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should now be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after accept")
	}))

	// --- Propose back to AM-A and cancel ---

	s.Require().True(s.Run("Propose transfer back to AM-A", func() {
		proposeIx, err := ics27_gmp.NewProposeAccessManagerTransferInstruction(
			access_manager.ProgramID,
			gmpAppStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), proposeIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "propose back to AM-A should succeed")
	}))

	s.Require().True(s.Run("Cancel pending transfer", func() {
		cancelIx, err := ics27_gmp.NewCancelAccessManagerTransferInstruction(
			gmpAppStatePDA,
			amBAccessManagerPDA,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), cancelIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "cancel should succeed")
	}))

	s.Require().True(s.Run("Verify: pending cleared, AM still AM-B", func() {
		state := readAppState()
		s.Require().Equal(amBProgramID, state.AmState.AccessManager, "AM should still be AM-B")
		s.Require().Nil(state.AmState.PendingAccessManager, "Pending should be cleared after cancel")
	}))
}

// Test_BatchUpgradeAuthorityMigration demonstrates concurrent upgrade authority
// transfers for multiple programs in a single transaction, which is the core
// benefit of using Vec<PendingAuthorityTransfer> instead of Option.
//
// With a timelocked multisig, a single Option would require N sequential
// propose/accept cycles (N x timelock waits). With Vec, all proposes and
// claims can be batched in one transaction.
//
// TEST FLOW:
// 1. Deploy and initialize AM-B
// 2. Grant ADMIN_ROLE on AM-A, initialize ICS27 GMP
// 3. Transfer upgrade authority for ICS26, ICS07 and GMP from deployer to AM-A's PDAs
// 4. Batch all 3 proposes + all 3 claims in a single transaction
// 5. Verify AM-B holds upgrade authority for all 3 programs
// 6. Verify AM-A can no longer upgrade any of the programs
func (s *IbcEurekaSolanaUpgradeTestSuite) Test_BatchUpgradeAuthorityMigration() {
	ctx := context.Background()

	s.SetupSuite(ctx)

	// --- Deploy and initialize AM-B ---

	var amBProgramID solanago.PublicKey

	s.Require().True(s.Run("Deploy AM-B (test_access_manager)", func() {
		var err error
		amBKeypairPath := fmt.Sprintf("%s/test_access_manager-keypair.json", keypairDir)
		amBProgramID, err = s.Solana.Chain.DeploySolanaProgramAsync(ctx, "test_access_manager", amBKeypairPath, deployerPath)
		s.Require().NoError(err, "failed to deploy test_access_manager")
		s.T().Logf("AM-B deployed at: %s", amBProgramID)
	}))

	s.Require().True(s.Run("Initialize AM-B", func() {
		deployerWallet, err := solana.LoadDeployerWallet(deployerPath)
		s.Require().NoError(err)

		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		amBProgramDataPDA, err := solana.GetProgramDataAddress(amBProgramID)
		s.Require().NoError(err)

		initIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewInitializeInstruction(
				s.SolanaRelayer.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SystemProgramID,
				solanago.SysVarInstructionsPubkey,
				amBProgramDataPDA,
				solana.DeployerPubkey,
			)
		})
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), initIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetryAndTimeout(ctx, tx, rpc.CommitmentConfirmed, 30, s.SolanaRelayer, deployerWallet)
		s.Require().NoError(err, "failed to initialize AM-B")
	}))

	// --- Setup: grant ADMIN_ROLE and initialize GMP ---

	s.Require().True(s.Run("Grant ADMIN_ROLE on AM-A to upgrader wallet", func() {
		var err error
		s.UpgraderWallet, err = s.Solana.Chain.CreateAndFundWallet()
		s.Require().NoError(err)

		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(access_manager.ProgramID)

		grantIx, err := access_manager.NewGrantRoleInstruction(
			ADMIN_ROLE,
			s.UpgraderWallet.PublicKey(),
			accessControlAccount,
			s.SolanaRelayer.PublicKey(),
			solanago.SysVarInstructionsPubkey,
		)
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), grantIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to grant ADMIN_ROLE on AM-A")
	}))

	s.initializeICS27GMP(ctx)

	// --- Target programs ---

	type programInfo struct {
		programID          solanago.PublicKey
		programDataAccount solanago.PublicKey
		amAUpgradeAuthPDA  solanago.PublicKey
		amBUpgradeAuthPDA  solanago.PublicKey
		soFile             string
	}

	targetPrograms := []struct {
		id     solanago.PublicKey
		soFile string
	}{
		{ics26_router.ProgramID, "ibc-solana/target/deploy/ics26_router.so"},
		{ics07_tendermint.ProgramID, "ibc-solana/target/deploy/ics07_tendermint.so"},
		{ics27_gmp.ProgramID, "ibc-solana/target/deploy/ics27_gmp.so"},
	}

	programs := make([]programInfo, len(targetPrograms))

	s.Require().True(s.Run("Derive PDAs for all target programs", func() {
		for i, tp := range targetPrograms {
			programDataAccount, err := solana.GetProgramDataAddress(tp.id)
			s.Require().NoError(err)

			amAPDA, _ := solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
				access_manager.ProgramID,
				tp.id.Bytes(),
			)

			amBPDA, _ := solana.AccessManager.UpgradeAuthorityWithArgSeedPDA(
				amBProgramID,
				tp.id.Bytes(),
			)

			programs[i] = programInfo{
				programID:          tp.id,
				programDataAccount: programDataAccount,
				amAUpgradeAuthPDA:  amAPDA,
				amBUpgradeAuthPDA:  amBPDA,
				soFile:             tp.soFile,
			}

			s.T().Logf("Program %d (%s): AM-A PDA=%s, AM-B PDA=%s",
				i, tp.id, amAPDA, amBPDA)
		}
	}))

	// --- Transfer upgrade authority from deployer to AM-A for all programs ---

	s.Require().True(s.Run("Transfer upgrade authority to AM-A for all programs", func() {
		for i, p := range programs {
			err := solana.SetUpgradeAuthority(
				ctx,
				p.programID,
				p.amAUpgradeAuthPDA,
				deployerPath,
				s.Solana.Chain.RPCURL,
			)
			s.Require().NoError(err, "failed to transfer authority for program %d", i)
		}
	}))

	// --- Batch propose + claim all in a single transaction ---

	amAProgramID := access_manager.ProgramID

	s.Require().True(s.Run("Batch propose and claim for all programs in one transaction", func() {
		accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(amAProgramID)
		amAAccessManagerPDA := accessControlAccount

		var ixs []solanago.Instruction

		// Build propose instructions for all 3 programs
		for i, p := range programs {
			proposeIx, err := access_manager.NewProposeUpgradeAuthorityTransferInstruction(
				p.programID,
				p.amBUpgradeAuthPDA,
				accessControlAccount,
				s.UpgraderWallet.PublicKey(),
				solanago.SysVarInstructionsPubkey,
			)
			s.Require().NoError(err, "failed to build propose instruction for program %d", i)
			ixs = append(ixs, proposeIx)
		}

		// Build claim instructions for all 3 programs (requires AM-B admin)
		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)
		for i, p := range programs {
			claimIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
				return access_manager.NewClaimUpgradeAuthorityInstruction(
					p.programID,
					p.amBUpgradeAuthPDA,
					amAAccessManagerPDA,
					p.programDataAccount,
					p.amAUpgradeAuthPDA,
					amAProgramID,
					solanago.BPFLoaderUpgradeableProgramID,
					amBAccessManagerPDA,
					s.SolanaRelayer.PublicKey(),
					solanago.SysVarInstructionsPubkey,
				)
			})
			s.Require().NoError(err, "failed to build claim instruction for program %d", i)
			ixs = append(ixs, claimIx)
		}

		computeBudgetIx := solana.NewComputeBudgetInstruction(800_000)
		txIxs := append([]solanago.Instruction{computeBudgetIx}, ixs...)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(
			s.UpgraderWallet.PublicKey(),
			txIxs...,
		)
		s.Require().NoError(err, "failed to create batch transaction")

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet, s.SolanaRelayer)
		s.Require().NoError(err, "batch propose + claim should succeed for all programs in one transaction")
	}))

	// --- Verify: AM-B admin can upgrade all programs ---

	s.Require().True(s.Run("Grant ADMIN_ROLE on AM-B", func() {
		amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

		grantIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
			return access_manager.NewGrantRoleInstruction(
				ADMIN_ROLE,
				s.UpgraderWallet.PublicKey(),
				amBAccessManagerPDA,
				s.SolanaRelayer.PublicKey(),
				solanago.SysVarInstructionsPubkey,
			)
		})
		s.Require().NoError(err)

		tx, err := s.Solana.Chain.NewTransactionFromInstructions(s.SolanaRelayer.PublicKey(), grantIx)
		s.Require().NoError(err)

		_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.SolanaRelayer)
		s.Require().NoError(err, "failed to grant ADMIN_ROLE on AM-B")
	}))

	for i, p := range programs {
		s.Require().True(s.Run(fmt.Sprintf("Verify: AM-B can upgrade program %d", i), func() {
			amBAccessManagerPDA, _ := solana.AccessManager.AccessManagerPDA(amBProgramID)

			bufferAccount, err := solana.WriteProgramBuffer(ctx, p.soFile, deployerPath, s.Solana.Chain.RPCURL)
			s.Require().NoError(err)

			err = solana.SetBufferAuthority(ctx, bufferAccount, p.amBUpgradeAuthPDA, deployerPath, s.Solana.Chain.RPCURL)
			s.Require().NoError(err)

			upgradeIx, err := withAMProgramID(amBProgramID, func() (solanago.Instruction, error) {
				return access_manager.NewUpgradeProgramInstruction(
					p.programID,
					amBAccessManagerPDA,
					p.programID,
					p.programDataAccount,
					bufferAccount,
					p.amBUpgradeAuthPDA,
					s.UpgraderWallet.PublicKey(),
					s.UpgraderWallet.PublicKey(),
					solanago.SysVarInstructionsPubkey,
					solanago.BPFLoaderUpgradeableProgramID,
					solanago.SysVarRentPubkey,
					solanago.SysVarClockPubkey,
				)
			})
			s.Require().NoError(err)

			computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

			tx, err := s.Solana.Chain.NewTransactionFromInstructions(
				s.UpgraderWallet.PublicKey(),
				computeBudgetIx,
				upgradeIx,
			)
			s.Require().NoError(err)

			_, err = s.Solana.Chain.SignAndBroadcastTxWithRetry(ctx, tx, rpc.CommitmentConfirmed, s.UpgraderWallet)
			s.Require().NoError(err, "AM-B should be able to upgrade program %d after claiming authority", i)
		}))
	}

	// --- Verify: AM-A can no longer upgrade any program ---

	for i, p := range programs {
		s.Require().True(s.Run(fmt.Sprintf("Verify: AM-A can no longer upgrade program %d", i), func() {
			accessControlAccount, _ := solana.AccessManager.AccessManagerPDA(amAProgramID)

			bufferAccount, err := solana.WriteProgramBuffer(ctx, p.soFile, deployerPath, s.Solana.Chain.RPCURL)
			s.Require().NoError(err)

			err = solana.SetBufferAuthority(ctx, bufferAccount, p.amAUpgradeAuthPDA, deployerPath, s.Solana.Chain.RPCURL)
			s.Require().NoError(err)

			upgradeIx, err := access_manager.NewUpgradeProgramInstruction(
				p.programID,
				accessControlAccount,
				p.programID,
				p.programDataAccount,
				bufferAccount,
				p.amAUpgradeAuthPDA,
				s.UpgraderWallet.PublicKey(),
				s.UpgraderWallet.PublicKey(),
				solanago.SysVarInstructionsPubkey,
				solanago.BPFLoaderUpgradeableProgramID,
				solanago.SysVarRentPubkey,
				solanago.SysVarClockPubkey,
			)
			s.Require().NoError(err)

			computeBudgetIx := solana.NewComputeBudgetInstruction(400_000)

			tx, err := s.Solana.Chain.NewTransactionFromInstructions(
				s.UpgraderWallet.PublicKey(),
				computeBudgetIx,
				upgradeIx,
			)
			s.Require().NoError(err)

			_, err = s.Solana.Chain.SignAndBroadcastTxWithOpts(ctx, tx, rpc.ConfirmationStatusConfirmed, s.UpgraderWallet)
			s.Require().Error(err, "AM-A upgrade should fail for program %d after authority was migrated to AM-B", i)
			s.Require().Contains(err.Error(), "IncorrectAuthority", "should fail with authority mismatch")
		}))
	}
}
