// SPDX-License-Identifier: Apache-2.0

//! Prover for SP1 ICS07 Tendermint programs.

use crate::programs::{
    MembershipProgram, MisbehaviourProgram, SP1Program, UpdateClientAndMembershipProgram,
    UpdateClientProgram,
};
use alloy_sol_types::SolValue;
use ibc_core_commitment_types::merkle::MerkleProof;
use ibc_eureka_solidity_types::msgs::{
    IICS07TendermintMsgs::{ClientState as SolClientState, ConsensusState as SolConsensusState},
    IMembershipMsgs::KVPair,
};
use ibc_proto::{
    ibc::lightclients::tendermint::v1::{Header, Misbehaviour},
    Protobuf,
};
use prost::Message;
use sp1_sdk::env::EnvProver;
use sp1_sdk::network::FulfillmentStrategy;
use sp1_sdk::{
    CpuProver, CudaProver, Elf, MockProver, NetworkProver, ProveRequest, Prover, SP1ProofMode,
    SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin, SP1VerifyingKey,
};

// Re-export the supported zk algorithms.
pub use ibc_eureka_solidity_types::msgs::IICS07TendermintMsgs::SupportedZkAlgorithm;

/// The SP1 prover used by the `TxBuilder`.
#[allow(clippy::module_name_repetitions)]
pub enum Sp1Prover {
    /// Mock prover for testing.
    Mock(MockProver),
    /// CPU-based local prover.
    Cpu(CpuProver),
    /// CUDA-accelerated local prover.
    Cuda(CudaProver),
    /// Network prover with a fulfillment strategy.
    Network(NetworkProver, FulfillmentStrategy),
    /// Environment-configured prover (reads `SP1_PROVER` env var).
    Env(EnvProver),
}

/// Internal proving key wrapper for different prover backends.
pub(crate) enum Sp1ProvingKey {
    Standard(SP1ProvingKey),
    Cuda(sp1_cuda::CudaProvingKey),
    Env(sp1_sdk::env::EnvProvingKey),
}

impl Sp1ProvingKey {
    fn verifying_key(&self) -> SP1VerifyingKey {
        use sp1_sdk::ProvingKey;
        match self {
            Self::Standard(pk) => pk.verifying_key().clone(),
            Self::Cuda(pk) => pk.verifying_key().clone(),
            Self::Env(pk) => pk.verifying_key().clone(),
        }
    }
}

impl Sp1Prover {
    /// Run async `setup` on the underlying prover, returning a wrapped proving key.
    pub(crate) async fn setup(&self, elf: Elf) -> Sp1ProvingKey {
        match self {
            Self::Mock(p) => Sp1ProvingKey::Standard(p.setup(elf).await.expect("setup failed")),
            Self::Cpu(p) => Sp1ProvingKey::Standard(p.setup(elf).await.expect("setup failed")),
            Self::Cuda(p) => Sp1ProvingKey::Cuda(p.setup(elf).await.expect("setup failed")),
            Self::Network(p, _) => {
                Sp1ProvingKey::Standard(p.setup(elf).await.expect("setup failed"))
            }
            Self::Env(p) => Sp1ProvingKey::Env(p.setup(elf).await.expect("setup failed")),
        }
    }

    /// Run a proof with the given mode. The prove request is awaited.
    ///
    /// # Panics
    /// Panics if the prover and proving key types are mismatched or proving fails.
    pub(crate) async fn prove(
        &self,
        pk: &Sp1ProvingKey,
        stdin: SP1Stdin,
        mode: SP1ProofMode,
    ) -> SP1ProofWithPublicValues {
        match (self, pk) {
            (Self::Mock(p), Sp1ProvingKey::Standard(pk)) => {
                p.prove(pk, stdin).mode(mode).await.expect("proving failed")
            }
            (Self::Cpu(p), Sp1ProvingKey::Standard(pk)) => {
                p.prove(pk, stdin).mode(mode).await.expect("proving failed")
            }
            (Self::Cuda(p), Sp1ProvingKey::Cuda(pk)) => {
                p.prove(pk, stdin).mode(mode).await.expect("proving failed")
            }
            (Self::Network(p, strategy), Sp1ProvingKey::Standard(pk)) => p
                .prove(pk, stdin)
                .mode(mode)
                .strategy(*strategy)
                .await
                .expect("proving failed"),
            (Self::Env(p), Sp1ProvingKey::Env(pk)) => {
                p.prove(pk, stdin).mode(mode).await.expect("proving failed")
            }
            _ => panic!("mismatched prover and proving key types"),
        }
    }
}

/// A prover for [`SP1Program`] programs.
#[allow(clippy::module_name_repetitions)]
pub struct SP1ICS07TendermintProver<'a, T>
where
    T: SP1Program + Sync,
{
    prover_client: &'a Sp1Prover,
    pkey: Sp1ProvingKey,
    /// The verifying key.
    pub vkey: SP1VerifyingKey,
    /// The proof type.
    pub proof_type: SupportedZkAlgorithm,
    /// The program type.
    pub program: &'a T,
}

impl<'a, T> SP1ICS07TendermintProver<'a, T>
where
    T: SP1Program + Sync,
{
    /// Create a new prover. Performs async `setup` to obtain proving and verifying keys.
    #[tracing::instrument(skip_all)]
    pub async fn new(
        proof_type: SupportedZkAlgorithm,
        prover_client: &'a Sp1Prover,
        program: &'a T,
    ) -> Self {
        tracing::info!("Initializing SP1 ProverClient...");
        let pkey = prover_client.setup(Elf::from(program.elf())).await;
        let vkey = pkey.verifying_key();
        tracing::info!("SP1 ProverClient initialized");
        Self {
            prover_client,
            pkey,
            vkey,
            proof_type,
            program,
        }
    }

    /// Prove the given input.
    /// # Panics
    /// If the proof cannot be generated or validated.
    pub async fn prove(&self, stdin: SP1Stdin) -> SP1ProofWithPublicValues {
        self.prover_client
            .prove(
                &self.pkey,
                stdin,
                match self.proof_type {
                    SupportedZkAlgorithm::Groth16 => SP1ProofMode::Groth16,
                    SupportedZkAlgorithm::Plonk => SP1ProofMode::Plonk,
                    SupportedZkAlgorithm::__Invalid => panic!("unsupported zk algorithm"),
                },
            )
            .await
    }
}

impl SP1ICS07TendermintProver<'_, UpdateClientProgram> {
    /// Generate a proof of an update from `trusted_consensus_state` to a proposed header.
    ///
    /// # Panics
    /// Panics if the inputs cannot be encoded, the proof cannot be generated or the proof is
    /// invalid.
    pub async fn generate_proof(
        &self,
        client_state: &SolClientState,
        trusted_consensus_state: &SolConsensusState,
        proposed_header: &Header,
        time: u128,
    ) -> SP1ProofWithPublicValues {
        let encoded_1 = client_state.abi_encode();
        let encoded_2 = trusted_consensus_state.abi_encode();
        let encoded_3 = proposed_header.encode_to_vec();
        let encoded_4 = time.to_le_bytes().into();

        let mut stdin = SP1Stdin::new();
        stdin.write_vec(encoded_1);
        stdin.write_vec(encoded_2);
        stdin.write_vec(encoded_3);
        stdin.write_vec(encoded_4);

        self.prove(stdin).await
    }
}

impl SP1ICS07TendermintProver<'_, MembershipProgram> {
    /// Generate a proof of verify (non)membership for multiple key-value pairs.
    ///
    /// # Panics
    /// Panics if the proof cannot be generated or the proof is invalid.
    pub async fn generate_proof(
        &self,
        commitment_root: &[u8],
        kv_proofs: Vec<(KVPair, MerkleProof)>,
    ) -> SP1ProofWithPublicValues {
        assert!(!kv_proofs.is_empty(), "No key-value pairs to prove");
        let len = u16::try_from(kv_proofs.len()).expect("too many key-value pairs");

        let mut stdin = SP1Stdin::new();
        stdin.write_slice(commitment_root);
        stdin.write_slice(&len.to_le_bytes());
        for (kv_pair, proof) in kv_proofs {
            stdin.write_vec(kv_pair.abi_encode());
            stdin.write_vec(proof.encode_vec());
        }

        self.prove(stdin).await
    }
}

impl SP1ICS07TendermintProver<'_, UpdateClientAndMembershipProgram> {
    /// Generate a proof of an update from `trusted_consensus_state` to a proposed header and
    /// verify (non)membership for multiple key-value pairs on the commitment root of
    /// `proposed_header`.
    ///
    /// # Panics
    /// Panics if the inputs cannot be encoded, the proof cannot be generated or the proof is
    /// invalid.
    pub async fn generate_proof(
        &self,
        client_state: &SolClientState,
        trusted_consensus_state: &SolConsensusState,
        proposed_header: &Header,
        time: u128,
        kv_proofs: Vec<(KVPair, MerkleProof)>,
    ) -> SP1ProofWithPublicValues {
        assert!(!kv_proofs.is_empty(), "No key-value pairs to prove");
        let len = u16::try_from(kv_proofs.len()).expect("too many key-value pairs");
        let encoded_1 = client_state.abi_encode();
        let encoded_2 = trusted_consensus_state.abi_encode();
        let encoded_3 = proposed_header.encode_to_vec();
        let encoded_4 = time.to_le_bytes().into();

        let mut stdin = SP1Stdin::new();
        stdin.write_vec(encoded_1);
        stdin.write_vec(encoded_2);
        stdin.write_vec(encoded_3);
        stdin.write_vec(encoded_4);
        stdin.write_slice(&len.to_le_bytes());
        for (kv_pair, proof) in kv_proofs {
            stdin.write_vec(kv_pair.abi_encode());
            stdin.write_vec(proof.encode_vec());
        }

        self.prove(stdin).await
    }
}

impl SP1ICS07TendermintProver<'_, MisbehaviourProgram> {
    /// Generate a proof of a misbehaviour.
    ///
    /// # Panics
    /// Panics if the proof cannot be generated or the proof is invalid.
    pub async fn generate_proof(
        &self,
        client_state: &SolClientState,
        misbehaviour: &Misbehaviour,
        trusted_consensus_state_1: &SolConsensusState,
        trusted_consensus_state_2: &SolConsensusState,
        time: u128,
    ) -> SP1ProofWithPublicValues {
        let encoded_1 = client_state.abi_encode();
        let encoded_2 = misbehaviour.encode_to_vec();
        let encoded_3 = trusted_consensus_state_1.abi_encode();
        let encoded_4 = trusted_consensus_state_2.abi_encode();
        let encoded_5 = time.to_le_bytes().into();

        let mut stdin = SP1Stdin::new();
        stdin.write_vec(encoded_1);
        stdin.write_vec(encoded_2);
        stdin.write_vec(encoded_3);
        stdin.write_vec(encoded_4);
        stdin.write_vec(encoded_5);

        self.prove(stdin).await
    }
}
