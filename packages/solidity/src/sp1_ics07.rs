// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[allow(clippy::nursery, clippy::too_many_arguments)]
    sp1_ics07_tendermint,
    "../../ibc-solidity/abi/bytecode/SP1ICS07Tendermint.json"
);

// NOTE: The riscv program won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    sp1_ics07_tendermint,
    "../../ibc-solidity/abi/SP1ICS07Tendermint.json"
);
