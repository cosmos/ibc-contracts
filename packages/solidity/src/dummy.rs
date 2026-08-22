// SPDX-License-Identifier: Apache-2.0

//! Solidity types for dummy light client.

use crate::msgs::IICS02ClientMsgs;

pub mod dummy_light_client {
    #[cfg(feature = "rpc")]
    alloy_sol_types::sol!(
        #[sol(rpc)]
        #[allow(clippy::nursery, clippy::too_many_arguments)]
        DummyLightClient,
        "../../ibc-solidity/abi/bytecode/DummyLightClient.json"
    );

    #[cfg(not(feature = "rpc"))]
    alloy_sol_types::sol!(
        DummyLightClient,
        "../../ibc-solidity/abi/DummyLightClient.json"
    );
}

alloy_sol_types::sol!(
    "../../ibc-solidity/contracts/light-clients/dummy/msgs/IDummyLightClientMsgs.sol"
);

pub mod dummy_light_client_msgs {
    #[allow(non_snake_case)]
    pub mod DummyLightClientMsgs {
        pub use super::super::IDummyLightClientMsgs::{Membership, MsgUpdateClient};
        pub use crate::msgs::IICS02ClientMsgs::Height;
    }
}
