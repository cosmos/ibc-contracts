// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[allow(clippy::nursery, clippy::too_many_arguments)]
    light_client,
    "../../ibc-solidity/abi/bytecode/AttestationLightClient.json"
);

// NOTE: The riscv program won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    light_client,
    "../../ibc-solidity/abi/AttestationLightClient.json"
);
