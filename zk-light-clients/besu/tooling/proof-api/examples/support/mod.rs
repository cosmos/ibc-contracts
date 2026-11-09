//! Mock chain RPCs used only by integration tests and the explicit container fixture.
use alloy_primitives::{Address, Bytes, B256};
use alloy_sol_types::SolValue;
use axum::{extract::State, Json};
use proof_api::contracts as abi;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
pub struct Chain {
    pub fixture: besu_zk_types::MembershipWitness,
    pub changed: AtomicBool,
    pub accepted: AtomicBool,
    pub vkeys: [B256; 3],
}
pub fn hex(value: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex::encode(value))
}
fn selector(signature: &str) -> String {
    hex(&besu_zk_types::keccak256(signature.as_bytes())[..4])
}
pub async fn rpc(
    State(chain): State<Arc<Chain>>,
    axum::extract::Path(side): axum::extract::Path<String>,
    Json(request): Json<Value>,
) -> Json<Value> {
    let source = side == "source";
    let result = match request["method"].as_str().unwrap_or("") {
        "eth_chainId" => json!(if source { "0x1" } else { "0x2" }),
        "eth_syncing" => json!(false),
        "eth_getBlockByNumber" => json!({"number": if source {"0xc"} else {"0x64"},
            "hash": hex([if source && chain.changed.load(Ordering::SeqCst) {9} else {8};32]),
            "stateRoot": hex(chain.fixture.state_root), "timestamp":"0x3e8"}),
        "eth_getProof" => {
            let slots = request["params"][1].as_array().cloned().unwrap_or_default();
            json!({"address":hex(chain.fixture.source_router), "accountProof":chain.fixture.account_proof.iter().map(hex).collect::<Vec<_>>(),
                "storageProof":slots.into_iter().map(|slot| json!({"key":slot,
                    "proof":chain.fixture.entries[0].storage_proof.iter().map(hex).collect::<Vec<_>>()})).collect::<Vec<_>>()})
        }
        "eth_call" => {
            assert_eq!(
                request["params"][1], "0x64",
                "contract calls must be snapshot-pinned"
            );
            let data = request["params"][0]["data"].as_str().unwrap_or("");
            let signature = data.get(..10).unwrap_or("");
            let encoded = if signature == selector("getClient(string)") {
                (Address::from([0x33; 20]),).abi_encode_params()
            } else if signature == selector("getCounterparty(string)") {
                (abi::Counterparty {
                    clientId: "client-1".into(),
                    merklePrefix: vec![Bytes::new()],
                },)
                    .abi_encode_params()
            } else if signature == selector("LOCAL_IBC_ROUTER()") {
                (Address::from([0x55; 20]),).abi_encode_params()
            } else if signature == selector("SOURCE_IBC_ROUTER()") {
                (Address::from(chain.fixture.source_router),).abi_encode_params()
            } else if signature == selector("SP1_VERIFIER()") {
                (Address::from([0x44; 20]),).abi_encode_params()
            } else if signature == selector("UPDATE_CLIENT_VKEY()") {
                (chain.vkeys[0],).abi_encode_params()
            } else if signature == selector("MEMBERSHIP_VKEY()") {
                (chain.vkeys[1],).abi_encode_params()
            } else if signature == selector("NON_MEMBERSHIP_VKEY()") {
                (chain.vkeys[2],).abi_encode_params()
            } else if signature == selector("getClientState()") {
                (Bytes::from(besu_public_values::encode_client_state(
                    &besu_zk_types::ClientState {
                        latest_height: 11,
                        trusting_period_seconds: 0,
                        max_clock_drift_seconds: 0,
                    },
                )),)
                    .abi_encode_params()
            } else if signature == selector("getConsensusState(uint64)") {
                let trusted = data.ends_with("0000000b");
                let found = trusted || chain.accepted.load(Ordering::SeqCst);
                (
                    found,
                    abi::Consensus {
                        stateRoot: B256::from(chain.fixture.state_root),
                        blockTimestamp: if found { 1000 } else { 0 },
                        validatorSetHash: B256::from([4; 32]),
                    },
                )
                    .abi_encode_params()
            } else {
                Vec::new()
            };
            json!(hex(encoded))
        }
        _ => Value::Null,
    };
    Json(json!({"jsonrpc":"2.0", "id":request["id"], "result":result}))
}
