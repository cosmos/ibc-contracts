// SPDX-License-Identifier: Apache-2.0

pub mod besu_light_client {
    #[cfg(feature = "rpc")]
    alloy_sol_types::sol!(
        #[sol(rpc)]
        #[allow(clippy::nursery, clippy::too_many_arguments)]
        BesuLightClientBase,
        "../../ibc-solidity/abi/bytecode/BesuLightClientBase.json"
    );

    #[cfg(not(feature = "rpc"))]
    alloy_sol_types::sol!(
        BesuLightClientBase,
        "../../ibc-solidity/abi/BesuLightClientBase.json"
    );
}
