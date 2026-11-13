use alloy_primitives::{Address, Bytes, B256};
use alloy_sol_types::{sol, SolValue};
use prover_script::{
    rpc::{parse_hex_value, quantity, ConsensusState, RpcClient},
    ToolError,
};
use serde_json::json;

sol! {
    struct Counterparty { string clientId; bytes[] merklePrefix; }
    struct Consensus { bytes32 stateRoot; uint64 blockTimestamp; bytes32 validatorSetHash; }
}

pub trait ContractCall {
    type Return;
    fn signature(&self) -> &'static str;
    fn parameters(&self) -> Vec<u8> {
        Vec::new()
    }
    fn decode(bytes: &[u8]) -> Result<Self::Return, ToolError>;
}
macro_rules! single_return {
    ($ty:ty, $bytes:expr) => {{
        let decoded =
            <($ty,)>::abi_decode_params($bytes).map_err(|_| ToolError::MalformedConsensusState)?;
        if decoded.abi_encode_params() != $bytes {
            return Err(ToolError::MalformedConsensusState);
        }
        Ok(decoded.0)
    }};
}
macro_rules! getter {
    ($name:ident, $ty:ty, $signature:literal) => {
        #[allow(non_camel_case_types)]
        pub struct $name;
        impl ContractCall for $name {
            type Return = $ty;
            fn signature(&self) -> &'static str {
                $signature
            }
            fn decode(bytes: &[u8]) -> Result<Self::Return, ToolError> {
                single_return!($ty, bytes)
            }
        }
    };
}
getter!(LOCAL_IBC_ROUTERCall, Address, "LOCAL_IBC_ROUTER()");
getter!(SOURCE_IBC_ROUTERCall, Address, "SOURCE_IBC_ROUTER()");
getter!(SP1_VERIFIERCall, Address, "SP1_VERIFIER()");
getter!(UPDATE_CLIENT_VKEYCall, B256, "UPDATE_CLIENT_VKEY()");
getter!(MEMBERSHIP_VKEYCall, B256, "MEMBERSHIP_VKEY()");
getter!(NON_MEMBERSHIP_VKEYCall, B256, "NON_MEMBERSHIP_VKEY()");
getter!(getClientStateCall, Bytes, "getClientState()");
macro_rules! client_getter {
    ($name:ident, $ty:ty, $signature:literal) => {
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct $name {
            pub clientId: String,
        }
        impl ContractCall for $name {
            type Return = $ty;
            fn signature(&self) -> &'static str {
                $signature
            }
            fn parameters(&self) -> Vec<u8> {
                (self.clientId.clone(),).abi_encode_params()
            }
            fn decode(bytes: &[u8]) -> Result<Self::Return, ToolError> {
                single_return!($ty, bytes)
            }
        }
    };
}
client_getter!(getClientCall, Address, "getClient(string)");
client_getter!(getCounterpartyCall, Counterparty, "getCounterparty(string)");
#[allow(non_camel_case_types)]
pub struct getConsensusStateCall {
    pub height: u64,
}
#[allow(non_snake_case)]
pub struct ConsensusResult {
    pub found: bool,
    pub state: Consensus,
}
impl ContractCall for getConsensusStateCall {
    type Return = ConsensusResult;
    fn signature(&self) -> &'static str {
        "getConsensusState(uint64)"
    }
    fn parameters(&self) -> Vec<u8> {
        (self.height,).abi_encode_params()
    }
    fn decode(bytes: &[u8]) -> Result<Self::Return, ToolError> {
        let (found, state) = <(bool, Consensus)>::abi_decode_params(bytes)
            .map_err(|_| ToolError::MalformedConsensusState)?;
        if (found, state.clone()).abi_encode_params() != bytes {
            return Err(ToolError::MalformedConsensusState);
        }
        Ok(ConsensusResult { found, state })
    }
}
pub async fn call<C: ContractCall>(
    rpc: &RpcClient,
    address: [u8; 20],
    height: u64,
    input: C,
) -> Result<C::Return, ToolError> {
    let signature = input.signature();
    let getter_error = |error: ToolError| ToolError::ContractGetter {
        signature,
        reason: error.to_string(),
    };
    let mut data = besu_zk_types::keccak256(signature.as_bytes())[..4].to_vec();
    data.extend(input.parameters());
    let value = rpc
        .call(
            "eth_call",
            json!([{"to": format!("0x{}", hex::encode(address)),
        "data": format!("0x{}", hex::encode(data))}, quantity(height)]),
        )
        .await
        .map_err(|error| match error {
            ToolError::RpcRejected(_) => getter_error(error),
            other => other,
        })?;
    let bytes = parse_hex_value(&value, "eth_call").map_err(getter_error)?;
    C::decode(&bytes).map_err(getter_error)
}
pub async fn consensus(
    rpc: &RpcClient,
    client: [u8; 20],
    verifier_height: u64,
    height: u64,
) -> Result<Option<ConsensusState>, ToolError> {
    let result = call(
        rpc,
        client,
        verifier_height,
        getConsensusStateCall { height },
    )
    .await?;
    if result.found != (result.state.blockTimestamp != 0) {
        return Err(ToolError::MalformedConsensusState);
    }
    Ok(result.found.then_some(ConsensusState {
        state_root: result.state.stateRoot.0,
        block_timestamp: result.state.blockTimestamp,
        validator_set_hash: result.state.validatorSetHash.0,
    }))
}
pub fn nonzero_address(address: Address) -> Result<[u8; 20], ToolError> {
    if address.is_zero() {
        return Err(ToolError::InvalidArtifact("zero contract address"));
    }
    Ok(address.into_array())
}
#[must_use]
pub fn prefixes(values: Vec<Bytes>) -> Vec<Vec<u8>> {
    values.into_iter().map(|v| v.to_vec()).collect()
}
