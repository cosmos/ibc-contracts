//! Solidity types for dummy light client.

pub mod dummy_light_client {
    #[cfg(feature = "rpc")]
    alloy_sol_types::sol!(
        #[sol(rpc)]
        #[allow(clippy::nursery, clippy::too_many_arguments)]
        DummyLightClient,
        "../../ibc-solidity/abi/bytecode/DummyLightClient.json"
    );

    #[cfg(not(feature = "rpc"))]
    alloy_sol_types::sol!(DummyLightClient, "../../ibc-solidity/abi/DummyLightClient.json");
}

pub mod dummy_light_client_msgs {
    alloy_sol_types::sol! {
        #[derive(Debug, PartialEq, Eq)]
        library DummyLightClientMsgs {
            struct Height {
                uint64 revisionNumber;
                uint64 revisionHeight;
            }

            struct Membership {
                bytes[] path;
                bytes value;
            }

            struct MsgUpdateClient {
                Height height;
                uint64 timestamp;
                Membership[] memberships;
            }
        }
    }
}
