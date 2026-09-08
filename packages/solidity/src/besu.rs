// SPDX-License-Identifier: Apache-2.0

pub mod besu_ibft2_light_client {
    #[cfg(feature = "rpc")]
    alloy_sol_types::sol!(
        #[sol(rpc)]
        #[allow(clippy::nursery, clippy::too_many_arguments)]
        BesuIBFT2LightClient,
        "../../ibc-solidity/abi/bytecode/BesuIBFT2LightClient.json"
    );

    #[cfg(not(feature = "rpc"))]
    alloy_sol_types::sol!(
        BesuIBFT2LightClient,
        "../../ibc-solidity/abi/BesuIBFT2LightClient.json"
    );
}

pub mod besu_qbft_light_client {
    #[cfg(feature = "rpc")]
    alloy_sol_types::sol!(
        #[sol(rpc)]
        #[allow(clippy::nursery, clippy::too_many_arguments)]
        BesuQBFTLightClient,
        "../../ibc-solidity/abi/bytecode/BesuQBFTLightClient.json"
    );

    #[cfg(not(feature = "rpc"))]
    alloy_sol_types::sol!(
        BesuQBFTLightClient,
        "../../ibc-solidity/abi/BesuQBFTLightClient.json"
    );
}
