// SPDX-License-Identifier: Apache-2.0

package main

import (
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "solana-ibc",
	Short: "CLI tool for Solana IBC operations",
	Long:  `solana-ibc provides commands for managing AccessManager roles and program upgrades.`,
}

func init() {
	rootCmd.AddCommand(accessManagerCmd)
	rootCmd.AddCommand(upgradeCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
