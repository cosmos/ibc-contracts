// SPDX-License-Identifier: Apache-2.0

pub mod besu_light_client {
    #[cfg(feature = "rpc")]
    alloy_sol_types::sol!(
        #[sol(rpc)]
        #[allow(clippy::nursery, clippy::too_many_arguments)]
        BesuLightClient,
        "../../ibc-solidity/abi/bytecode/BesuLightClient.json"
    );

    #[cfg(not(feature = "rpc"))]
    alloy_sol_types::sol!(
        BesuLightClient,
        "../../ibc-solidity/abi/BesuLightClient.json"
    );
}
