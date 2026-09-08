#!/bin/sh
# SPDX-License-Identifier: Apache-2.0

set -euxo pipefail

# This script expects an environment variable SP1_PROGRAM_VERSIONS

PROGRAMS="sp1-ics07-tendermint-membership sp1-ics07-tendermint-update-client sp1-ics07-tendermint-uc-and-membership sp1-ics07-tendermint-misbehaviour"

# Check if the environment variable is set
if [ -z "${SP1_PROGRAM_VERSIONS:-}" ]; then
  echo "Warning: SP1_PROGRAM_VERSIONS not set, skipping SP1 program download." >&2
else
  # Loop through each version provided
  for version in $SP1_PROGRAM_VERSIONS; do
    target_dir="/usr/local/bin/sp1-programs/$version"
    mkdir -p "$target_dir"

    # Download each program for the current version
    for program in $PROGRAMS; do
        wget --no-check-certificate https://github.com/cosmos/solidity-ibc-eureka/releases/download/sp1-programs-$version/$program -O $target_dir/$program
    done
  done
fi

/usr/local/bin/proof-api $@
