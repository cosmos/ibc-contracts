use crate::{
    config::{ClientConfig, ContractsConfig, RpcConfig},
    contracts as abi,
};
use prover_script::{
    identity::ApprovedPrograms,
    rpc::{fixed_hex, parse_quantity_u64, quantity, RpcClient},
    ToolError,
};
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSnapshot {
    pub height: u64,
    pub hash: [u8; 32],
    pub state_root: [u8; 32],
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub source_chain_id: u64,
    pub verifier_chain_id: u64,
    pub light_client: [u8; 20],
    pub source_router: [u8; 20],
    pub local_router: [u8; 20],
    pub sp1_verifier: [u8; 20],
    pub counterparty_client_id: String,
    pub prefix: Vec<Vec<u8>>,
    pub update_vkey: [u8; 32],
    pub membership_vkey: [u8; 32],
    pub non_membership_vkey: [u8; 32],
}
#[derive(Clone)]
pub struct Snapshot {
    pub identity: Identity,
    pub verifier: BlockSnapshot,
    pub source: BlockSnapshot,
}

pub async fn block(rpc: &RpcClient, tag: &str) -> Result<BlockSnapshot, ToolError> {
    let value = rpc
        .call("eth_getBlockByNumber", json!([tag, false]))
        .await?;
    let number = parse_quantity_u64(&value["number"], "number")?;
    if tag != "latest" && tag != quantity(number) {
        return Err(ToolError::MalformedRpcResponse);
    }
    Ok(BlockSnapshot {
        height: number,
        hash: fixed_hex(
            value["hash"]
                .as_str()
                .ok_or(ToolError::MalformedRpcResponse)?,
            "block hash",
        )?,
        state_root: fixed_hex(
            value["stateRoot"]
                .as_str()
                .ok_or(ToolError::MalformedRpcResponse)?,
            "stateRoot",
        )?,
        timestamp: parse_quantity_u64(&value["timestamp"], "timestamp")?,
    })
}

pub async fn validate(
    source: &RpcClient,
    verifier: &RpcClient,
    rpc: &RpcConfig,
    client: &ClientConfig,
    contracts: &ContractsConfig,
    approved: &ApprovedPrograms,
) -> Result<Snapshot, ToolError> {
    let source_chain_id = source.chain_id().await?;
    let verifier_chain_id = verifier.chain_id().await?;
    if source_chain_id == 0
        || verifier_chain_id == 0
        || rpc
            .expected_source_chain_id
            .is_some_and(|v| v != source_chain_id)
        || rpc
            .expected_verifier_chain_id
            .is_some_and(|v| v != verifier_chain_id)
    {
        return Err(ToolError::InvalidArtifact("chain identity mismatch"));
    }
    if source.call("eth_syncing", json!([])).await? != Value::Bool(false) {
        return Err(ToolError::InvalidArtifact("source is syncing"));
    }
    let verifier_block = block(verifier, "latest").await?;
    let height = verifier_block.height;
    let identity = resolve_identity(
        verifier,
        height,
        client,
        contracts,
        approved,
        (source_chain_id, verifier_chain_id),
    )
    .await?;
    if block(verifier, &quantity(height)).await? != verifier_block {
        return Err(ToolError::InvalidArtifact("verifier snapshot changed"));
    }
    let source_block = block(source, "latest").await?;
    let proof = source
        .proofs(identity.source_router, &[[0; 32]], source_block.height)
        .await?;
    let root = evm_mpt::verify_account_storage_root(
        source_block.state_root,
        identity.source_router,
        &proof.account_proof,
    )
    .map_err(|e| ToolError::InvalidWitness(e.to_string()))?;
    evm_mpt::verify_storage(root, [0; 32], &proof.storage_proofs[0])
        .map_err(|e| ToolError::InvalidWitness(e.to_string()))?;
    if block(source, &quantity(source_block.height)).await? != source_block {
        return Err(ToolError::InvalidArtifact("source snapshot changed"));
    }
    Ok(Snapshot {
        identity,
        verifier: verifier_block,
        source: source_block,
    })
}

pub async fn acknowledgement_commitments(
    source: &RpcClient,
    snapshot: &BlockSnapshot,
    router: [u8; 20],
    slots: &[[u8; 32]],
) -> Result<Vec<[u8; 32]>, ToolError> {
    let unique = slots
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let proof = source.proofs(router, &unique, snapshot.height).await?;
    let root =
        evm_mpt::verify_account_storage_root(snapshot.state_root, router, &proof.account_proof)
            .map_err(|e| ToolError::InvalidWitness(e.to_string()))?;
    let mut values = std::collections::BTreeMap::new();
    for (slot, branch) in unique.into_iter().zip(proof.storage_proofs) {
        let word = evm_mpt::verify_storage_membership(root, slot, &branch)
            .map_err(|e| ToolError::InvalidWitness(e.to_string()))?;
        if word == [0; 32] {
            return Err(ToolError::InvalidArtifact("acknowledgement is absent"));
        }
        values.insert(slot, word);
    }
    slots
        .iter()
        .map(|slot| values.get(slot).copied().ok_or(ToolError::WrongStorageKey))
        .collect()
}

async fn resolve_identity(
    verifier: &RpcClient,
    height: u64,
    client: &ClientConfig,
    contracts: &ContractsConfig,
    approved: &ApprovedPrograms,
    chains: (u64, u64),
) -> Result<Identity, ToolError> {
    let (source_chain_id, verifier_chain_id) = chains;
    let router = fixed_hex(&contracts.verifier_router, "verifierRouter")?;
    let light_client = abi::nonzero_address(
        abi::call(
            verifier,
            router,
            height,
            abi::getClientCall {
                clientId: client.client_id.clone(),
            },
        )
        .await?,
    )?;
    let counterparty = abi::call(
        verifier,
        router,
        height,
        abi::getCounterpartyCall {
            clientId: client.client_id.clone(),
        },
    )
    .await?;
    let prefix = abi::prefixes(counterparty.merklePrefix);
    if counterparty.clientId.is_empty() || prefix.is_empty() {
        return Err(ToolError::InvalidArtifact("missing counterparty identity"));
    }
    let local_router = abi::nonzero_address(
        abi::call(verifier, light_client, height, abi::LOCAL_IBC_ROUTERCall {}).await?,
    )?;
    if local_router != router {
        return Err(ToolError::InvalidArtifact("local Router mismatch"));
    }
    let source_router = abi::nonzero_address(
        abi::call(
            verifier,
            light_client,
            height,
            abi::SOURCE_IBC_ROUTERCall {},
        )
        .await?,
    )?;
    let sp1_verifier = abi::nonzero_address(
        abi::call(verifier, light_client, height, abi::SP1_VERIFIERCall {}).await?,
    )?;
    let update_vkey = abi::call(
        verifier,
        light_client,
        height,
        abi::UPDATE_CLIENT_VKEYCall {},
    )
    .await?
    .0;
    let membership_vkey = abi::call(verifier, light_client, height, abi::MEMBERSHIP_VKEYCall {})
        .await?
        .0;
    let non_membership_vkey = abi::call(
        verifier,
        light_client,
        height,
        abi::NON_MEMBERSHIP_VKEYCall {},
    )
    .await?
    .0;
    approved
        .validate_deployed_vkeys(update_vkey, membership_vkey, non_membership_vkey)
        .map_err(|error| ToolError::VkeyMismatch(error.to_string()))?;
    Ok(Identity {
        source_chain_id,
        verifier_chain_id,
        light_client,
        source_router,
        local_router,
        sp1_verifier,
        counterparty_client_id: counterparty.clientId,
        prefix,
        update_vkey,
        membership_vkey,
        non_membership_vkey,
    })
}
