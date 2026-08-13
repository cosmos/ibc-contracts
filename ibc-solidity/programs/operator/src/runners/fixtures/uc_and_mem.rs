// SPDX-License-Identifier: Apache-2.0

//! Runner for generating `update_client` fixtures

use crate::{
    cli::command::{fixtures::UpdateClientAndMembershipCmd, OutputPath},
    runners::{
        fixtures::membership::SP1ICS07MembershipFixture, genesis::SP1ICS07TendermintGenesis,
    },
};
use alloy::sol_types::SolValue;
use ibc_client_tendermint_types::ConsensusState;
use ibc_eureka_solidity_types::msgs::{
    IICS07TendermintMsgs::{ClientState, ConsensusState as SolConsensusState},
    IMembershipMsgs::{KVPair, MembershipProof, SP1MembershipAndUpdateClientProof},
    ISP1Msgs::SP1Proof,
    IUpdateClientAndMembershipMsgs::UcAndMembershipOutput,
};
use ibc_eureka_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use sp1_ics07_tendermint_prover::{
    programs::{
        MembershipProgram, MisbehaviourProgram, UpdateClientAndMembershipProgram,
        UpdateClientProgram,
    },
    prover::{SP1ICS07TendermintProver, Sp1Prover},
};
use sp1_sdk::{
    network::{FulfillmentStrategy, NetworkMode},
    HashableKey, ProverClient,
};
use std::path::PathBuf;
use tendermint_rpc::HttpClient;

/// Writes the proof data for the given trusted and target blocks to the given fixture path.
#[allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]
pub async fn run(args: UpdateClientAndMembershipCmd) -> anyhow::Result<()> {
    assert!(
        args.membership.trusted_block < args.target_block,
        "The target block must be greater than the trusted block"
    );

    let update_client_elf = std::fs::read(args.elf_paths.update_client_path)?;
    let membership_elf = std::fs::read(args.elf_paths.membership_path)?;
    let misbehaviour_elf = std::fs::read(args.elf_paths.misbehaviour_path)?;
    let uc_and_membership_elf = std::fs::read(args.elf_paths.uc_and_membership_path)?;
    let update_client_program = UpdateClientProgram::new(update_client_elf);
    let membership_program = MembershipProgram::new(membership_elf);
    let misbehaviour_program = MisbehaviourProgram::new(misbehaviour_elf);
    let uc_and_membership_program = UpdateClientAndMembershipProgram::new(uc_and_membership_elf);

    let tm_rpc_client = HttpClient::from_env();
    let sp1_prover = if args.sp1.private_cluster {
        Sp1Prover::Network(
            ProverClient::builder()
                .network_for(NetworkMode::Reserved)
                .build()
                .await,
            FulfillmentStrategy::Reserved,
        )
    } else {
        Sp1Prover::Env(ProverClient::from_env().await)
    };

    let uc_mem_prover =
        SP1ICS07TendermintProver::new(args.sp1.proof_type, &sp1_prover, &uc_and_membership_program)
            .await;

    let trusted_light_block = tm_rpc_client
        .get_light_block(Some(args.membership.trusted_block))
        .await?;
    let target_light_block = tm_rpc_client
        .get_light_block(Some(args.target_block))
        .await?;

    let genesis = SP1ICS07TendermintGenesis::from_env(
        &trusted_light_block,
        args.membership.trust_options.trusting_period,
        args.membership.trust_options.trust_level,
        args.sp1.proof_type,
        &update_client_program,
        &membership_program,
        &uc_and_membership_program,
        &misbehaviour_program,
    )
    .await?;
    let trusted_client_state = ClientState::abi_decode(&genesis.trusted_client_state)?;
    let trusted_consensus_state: ConsensusState =
        SolConsensusState::abi_decode(&genesis.trusted_consensus_state)?.into();

    let proposed_header = target_light_block.into_header(&trusted_light_block);

    let kv_proofs: Vec<(_, _)> =
        futures::future::try_join_all(args.membership.key_paths.into_iter().map(|path| async {
            let path: Vec<Vec<u8>> = if args.membership.base64 {
                path.split('\\')
                    .map(subtle_encoding::base64::decode)
                    .collect::<Result<_, _>>()?
            } else {
                vec![b"ibc".into(), path.into_bytes()]
            };
            assert_eq!(path.len(), 2);

            let (value, proof) = tm_rpc_client.prove_path(&path, args.target_block).await?;
            let kv_pair = KVPair {
                path: path.into_iter().map(Into::into).collect(),
                value: value.into(),
            };

            anyhow::Ok((kv_pair, proof))
        }))
        .await?;

    let kv_len = kv_proofs.len();
    let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    // Generate a header update proof for the specified blocks.
    let proof_data = uc_mem_prover
        .generate_proof(
            &trusted_client_state,
            &trusted_consensus_state.into(),
            &proposed_header,
            now_since_unix.as_nanos(),
            kv_proofs,
        )
        .await;

    let bytes = proof_data.public_values.as_slice();
    let output = UcAndMembershipOutput::abi_decode(bytes)?;
    assert_eq!(output.kvPairs.len(), kv_len);

    let sp1_membership_proof = SP1MembershipAndUpdateClientProof {
        sp1Proof: SP1Proof::new(
            &uc_mem_prover.vkey.bytes32(),
            proof_data.bytes(),
            proof_data.public_values.to_vec(),
        ),
    };

    let fixture = SP1ICS07MembershipFixture {
        genesis,
        proof_height: output.updateClientOutput.newHeight.abi_encode(),
        membership_proof: MembershipProof::from(sp1_membership_proof).abi_encode(),
    };

    match args.membership.output_path {
        OutputPath::File(path) => {
            // Save the proof data to the file path.
            std::fs::write(PathBuf::from(path), serde_json::to_string_pretty(&fixture)?)?;
        }
        OutputPath::Stdout => {
            println!("{}", serde_json::to_string_pretty(&fixture)?);
        }
    }

    Ok(())
}
