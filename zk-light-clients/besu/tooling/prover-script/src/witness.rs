use besu_qbft::{parse_extra_data, verify_update};
use besu_zk_types::{
    is_zero_address, Eip1186ProofWitness, MembershipEntryWitness, MembershipWitness,
    NonMembershipWitness, QbftHeaderInput, UpdateWitness, MEMBERSHIP_SCHEMA_VERSION,
    MAX_BATCH_MESSAGES, SCHEMA_VERSION,
};
use ibc_storage::{
    logical_storage_slot, parse_membership_path, parse_receipt_path, verify_membership,
    verify_non_membership,
};

use crate::{
    rpc::{ConsensusState, RpcClient},
    ToolError,
};

#[allow(async_fn_in_trait)]
pub trait WitnessRpc {
    async fn chain_id(&self) -> Result<u64, ToolError>;
    async fn block(&self, height: u64) -> Result<QbftHeaderInput, ToolError>;
    async fn proof(
        &self,
        address: [u8; 20],
        slot: [u8; 32],
        height: u64,
    ) -> Result<Eip1186ProofWitness, ToolError>;
    async fn proofs(
        &self,
        address: [u8; 20],
        slots: &[[u8; 32]],
        height: u64,
    ) -> Result<crate::rpc::BatchEip1186Proof, ToolError>;
    async fn consensus_state(
        &self,
        light_client: [u8; 20],
        height: u64,
    ) -> Result<ConsensusState, ToolError>;
}

impl WitnessRpc for RpcClient {
    async fn chain_id(&self) -> Result<u64, ToolError> {
        RpcClient::chain_id(self).await
    }

    async fn block(&self, height: u64) -> Result<QbftHeaderInput, ToolError> {
        RpcClient::block(self, height).await
    }

    async fn proof(
        &self,
        address: [u8; 20],
        slot: [u8; 32],
        height: u64,
    ) -> Result<Eip1186ProofWitness, ToolError> {
        RpcClient::proof(self, address, slot, height).await
    }

    async fn proofs(
        &self,
        address: [u8; 20],
        slots: &[[u8; 32]],
        height: u64,
    ) -> Result<crate::rpc::BatchEip1186Proof, ToolError> {
        RpcClient::proofs(self, address, slots, height).await
    }

    async fn consensus_state(
        &self,
        light_client: [u8; 20],
        height: u64,
    ) -> Result<ConsensusState, ToolError> {
        RpcClient::consensus_state(self, light_client, height).await
    }
}

pub struct StateProofInputs {
    pub expected_source_chain_id: u64,
    pub source_router: [u8; 20],
    pub destination_light_client: [u8; 20],
    pub proof_height: u64,
    pub path: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipIntentInput {
    pub path: Vec<Vec<u8>>,
    pub expected_commitment: [u8; 32],
}

pub async fn build_update_witness(
    source: &impl WitnessRpc,
    destination: &impl WitnessRpc,
    expected_source_chain_id: u64,
    destination_light_client: [u8; 20],
    trusted_height: u64,
    target_height: u64,
) -> Result<UpdateWitness, ToolError> {
    validate_address(destination_light_client)?;
    validate_source_chain(source, expected_source_chain_id).await?;
    let consensus = destination
        .consensus_state(destination_light_client, trusted_height)
        .await?;
    let trusted_header = source.block(trusted_height).await?;
    let trusted_extra = parse_extra_data(&trusted_header.extra_data)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    let target_header = source.block(target_height).await?;
    let witness = UpdateWitness {
        schema_version: SCHEMA_VERSION,
        trusted_height,
        trusted_validator_set_hash: consensus.validator_set_hash,
        trusted_validators: trusted_extra.validators,
        target_header,
    };
    verify_update(&witness).map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(witness)
}

pub async fn build_membership_witness(
    source: &impl WitnessRpc,
    destination: &impl WitnessRpc,
    inputs: StateProofInputs,
    expected_commitment: [u8; 32],
) -> Result<MembershipWitness, ToolError> {
    let consensus = destination
        .consensus_state(inputs.destination_light_client, inputs.proof_height)
        .await?;
    build_membership_witness_with_root(source, inputs, expected_commitment, consensus.state_root)
        .await
}

pub async fn build_membership_witness_from_source(
    source: &impl WitnessRpc,
    inputs: StateProofInputs,
    expected_commitment: [u8; 32],
) -> Result<MembershipWitness, ToolError> {
    let header = source.block(inputs.proof_height).await?;
    build_membership_witness_with_root(source, inputs, expected_commitment, header.state_root).await
}

async fn build_membership_witness_with_root(
    source: &impl WitnessRpc,
    inputs: StateProofInputs,
    expected_commitment: [u8; 32],
    state_root: [u8; 32],
) -> Result<MembershipWitness, ToolError> {
    let intent = MembershipIntentInput {
        path: inputs.path.clone(),
        expected_commitment,
    };
    build_membership_batch_witness_with_root(
        source,
        inputs,
        vec![intent],
        state_root,
    ).await
}

pub async fn build_membership_batch_witness_from_source(
    source: &impl WitnessRpc,
    inputs: StateProofInputs,
    intents: Vec<MembershipIntentInput>,
) -> Result<MembershipWitness, ToolError> {
    let header = source.block(inputs.proof_height).await?;
    build_membership_batch_witness_with_root(source, inputs, intents, header.state_root).await
}

async fn build_membership_batch_witness_with_root(
    source: &impl WitnessRpc,
    inputs: StateProofInputs,
    intents: Vec<MembershipIntentInput>,
    state_root: [u8; 32],
) -> Result<MembershipWitness, ToolError> {
    validate_address(inputs.source_router)?;
    validate_address(inputs.destination_light_client)?;
    validate_source_chain(source, inputs.expected_source_chain_id).await?;
    if intents.is_empty() || intents.len() > MAX_BATCH_MESSAGES {
        return Err(ToolError::InvalidWitness("membership batch size must be 1..=32".to_owned()));
    }
    let mut canonical = intents.into_iter().map(|intent| {
        let terminal = parse_membership_path(&intent.path)
            .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
        let digest = ibc_storage::path_digest(&intent.path);
        Ok((digest, logical_storage_slot(terminal.raw), intent))
    }).collect::<Result<Vec<_>, ToolError>>()?;
    canonical.sort_by_key(|(digest, _, _)| *digest);
    if canonical.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(ToolError::InvalidWitness("duplicate membership path digest".to_owned()));
    }
    let slots: Vec<_> = canonical.iter().map(|(_, slot, _)| *slot).collect();
    let proof = source.proofs(inputs.source_router, &slots, inputs.proof_height).await?;
    let crate::rpc::BatchEip1186Proof { account_proof, storage_proofs } = proof;
    let entries = canonical.into_iter().zip(storage_proofs).map(
        |((_, _, intent), storage_proof)| MembershipEntryWitness {
            path: intent.path,
            expected_commitment: intent.expected_commitment,
            storage_proof,
        }
    ).collect();
    let witness = MembershipWitness {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        proof_height: inputs.proof_height,
        state_root,
        source_router: inputs.source_router,
        account_proof,
        entries,
    };
    verify_membership(&witness).map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(witness)
}

pub async fn build_non_membership_witness(
    source: &impl WitnessRpc,
    destination: &impl WitnessRpc,
    inputs: StateProofInputs,
) -> Result<NonMembershipWitness, ToolError> {
    let consensus = destination
        .consensus_state(inputs.destination_light_client, inputs.proof_height)
        .await?;
    build_non_membership_witness_with_root(source, inputs, consensus.state_root).await
}

pub async fn build_non_membership_witness_from_source(
    source: &impl WitnessRpc,
    inputs: StateProofInputs,
) -> Result<NonMembershipWitness, ToolError> {
    let header = source.block(inputs.proof_height).await?;
    build_non_membership_witness_with_root(source, inputs, header.state_root).await
}

async fn build_non_membership_witness_with_root(
    source: &impl WitnessRpc,
    inputs: StateProofInputs,
    state_root: [u8; 32],
) -> Result<NonMembershipWitness, ToolError> {
    validate_address(inputs.source_router)?;
    validate_address(inputs.destination_light_client)?;
    validate_source_chain(source, inputs.expected_source_chain_id).await?;
    let terminal = parse_receipt_path(&inputs.path)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    let slot = logical_storage_slot(terminal.raw);
    let proof = source
        .proof(inputs.source_router, slot, inputs.proof_height)
        .await?;
    let witness = NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: inputs.proof_height,
        state_root,
        source_router: inputs.source_router,
        path: inputs.path,
        proof,
    };
    verify_non_membership(&witness)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(witness)
}

async fn validate_source_chain(source: &impl WitnessRpc, expected: u64) -> Result<(), ToolError> {
    let actual = source.chain_id().await?;
    if actual == expected {
        Ok(())
    } else {
        Err(ToolError::ChainIdMismatch { expected, actual })
    }
}

fn validate_address(address: [u8; 20]) -> Result<(), ToolError> {
    if is_zero_address(&address) {
        Err(ToolError::InvalidWitness(
            "required runtime address is zero".to_owned(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use besu_qbft::{encode_extra_data, validator_set_hash, QbftExtraData};
    use sp1_sdk::SP1Stdin;

    use super::*;
    use crate::golden::{membership_witness, non_membership_witness, update_witness};

    #[derive(Clone)]
    struct MockRpc {
        chain_id: u64,
        blocks: BTreeMap<u64, QbftHeaderInput>,
        consensus: Option<ConsensusState>,
        proof_address: Option<[u8; 20]>,
        proof_slot: Option<[u8; 32]>,
        proof: Option<Eip1186ProofWitness>,
    }

    impl MockRpc {
        fn empty(chain_id: u64) -> Self {
            Self {
                chain_id,
                blocks: BTreeMap::new(),
                consensus: None,
                proof_address: None,
                proof_slot: None,
                proof: None,
            }
        }
    }

    impl WitnessRpc for MockRpc {
        async fn chain_id(&self) -> Result<u64, ToolError> {
            Ok(self.chain_id)
        }

        async fn block(&self, height: u64) -> Result<QbftHeaderInput, ToolError> {
            self.blocks
                .get(&height)
                .cloned()
                .ok_or_else(|| ToolError::InvalidWitness("mock block unavailable".to_owned()))
        }

        async fn proof(
            &self,
            address: [u8; 20],
            slot: [u8; 32],
            _height: u64,
        ) -> Result<Eip1186ProofWitness, ToolError> {
            if self.proof_address != Some(address) {
                return Err(ToolError::WrongProofAddress);
            }
            if self.proof_slot != Some(slot) {
                return Err(ToolError::WrongStorageKey);
            }
            self.proof
                .clone()
                .ok_or_else(|| ToolError::InvalidWitness("mock proof unavailable".to_owned()))
        }

        async fn proofs(
            &self,
            address: [u8; 20],
            slots: &[[u8; 32]],
            height: u64,
        ) -> Result<crate::rpc::BatchEip1186Proof, ToolError> {
            if slots.len() != 1 {
                return Err(ToolError::WrongStorageProofCount(slots.len()));
            }
            let proof = self.proof(address, slots[0], height).await?;
            Ok(crate::rpc::BatchEip1186Proof {
                account_proof: proof.account_proof,
                storage_proofs: vec![proof.storage_proof],
            })
        }

        async fn consensus_state(
            &self,
            _light_client: [u8; 20],
            height: u64,
        ) -> Result<ConsensusState, ToolError> {
            self.consensus
                .clone()
                .ok_or(ToolError::MissingConsensusState(height))
        }
    }

    fn stdin_bytes<T: serde::Serialize>(value: &T) -> Vec<Vec<u8>> {
        let mut stdin = SP1Stdin::new();
        stdin.write(value);
        stdin.buffer
    }

    #[tokio::test]
    async fn update_builder_is_deterministic_for_identical_normalized_rpc_data(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let expected = update_witness()?;
        let mut trusted_header = expected.target_header.clone();
        trusted_header.number = expected.trusted_height;
        trusted_header.extra_data = encode_extra_data(
            &QbftExtraData {
                vanity: [0; 32],
                validators: expected.trusted_validators.clone(),
                vote: None,
                round: 0,
                committed_seals: Vec::new(),
            },
            true,
            true,
        );
        let mut source = MockRpc::empty(1);
        source
            .blocks
            .insert(expected.trusted_height, trusted_header);
        source.blocks.insert(
            expected.target_header.number,
            expected.target_header.clone(),
        );
        let mut destination = MockRpc::empty(2);
        destination.consensus = Some(ConsensusState {
            state_root: [9; 32],
            block_timestamp: 1,
            validator_set_hash: validator_set_hash(&expected.trusted_validators)?,
        });

        let first = build_update_witness(
            &source,
            &destination,
            1,
            [7; 20],
            expected.trusted_height,
            expected.target_header.number,
        )
        .await?;
        let second = build_update_witness(
            &source,
            &destination,
            1,
            [7; 20],
            expected.trusted_height,
            expected.target_header.number,
        )
        .await?;
        assert_eq!(first, second);
        assert_eq!(stdin_bytes(&first), stdin_bytes(&second));
        Ok(())
    }

    #[tokio::test]
    async fn membership_builders_derive_the_exact_slot_and_are_deterministic(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let expected = membership_witness()?;
        let entry = expected.entries.first().ok_or("empty membership fixture")?;
        let slot = logical_storage_slot(
            parse_membership_path(&entry.path)
                .map_err(|error| ToolError::InvalidWitness(error.to_string()))?
                .raw,
        );
        let mut source = MockRpc::empty(1);
        source.proof_address = Some(expected.source_router);
        source.proof_slot = Some(slot);
        source.proof = Some(Eip1186ProofWitness {
            account_proof: expected.account_proof.clone(),
            storage_proof: entry.storage_proof.clone(),
        });
        let mut destination = MockRpc::empty(2);
        destination.consensus = Some(ConsensusState {
            state_root: expected.state_root,
            block_timestamp: 1,
            validator_set_hash: [9; 32],
        });
        let inputs = || StateProofInputs {
            expected_source_chain_id: 1,
            source_router: expected.source_router,
            destination_light_client: [8; 20],
            proof_height: expected.proof_height,
            path: entry.path.clone(),
        };
        let commitment = entry.expected_commitment;

        let first = build_membership_witness(&source, &destination, inputs(), commitment).await?;
        let second = build_membership_witness(&source, &destination, inputs(), commitment).await?;
        assert_eq!(first, expected);
        assert_eq!(first, second);
        assert_eq!(stdin_bytes(&first), stdin_bytes(&second));
        Ok(())
    }

    #[tokio::test]
    async fn non_membership_builder_requires_exact_receipt_slot_and_is_deterministic(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let expected = non_membership_witness();
        let slot = logical_storage_slot(
            parse_receipt_path(&expected.path)
                .map_err(|error| ToolError::InvalidWitness(error.to_string()))?
                .raw,
        );
        let mut source = MockRpc::empty(1);
        source.proof_address = Some(expected.source_router);
        source.proof_slot = Some(slot);
        source.proof = Some(expected.proof.clone());
        let mut destination = MockRpc::empty(2);
        destination.consensus = Some(ConsensusState {
            state_root: expected.state_root,
            block_timestamp: 1,
            validator_set_hash: [9; 32],
        });
        let inputs = || StateProofInputs {
            expected_source_chain_id: 1,
            source_router: expected.source_router,
            destination_light_client: [8; 20],
            proof_height: expected.proof_height,
            path: expected.path.clone(),
        };

        let first = build_non_membership_witness(&source, &destination, inputs()).await?;
        let second = build_non_membership_witness(&source, &destination, inputs()).await?;
        assert_eq!(first, second);
        assert_eq!(stdin_bytes(&first), stdin_bytes(&second));
        Ok(())
    }
}
