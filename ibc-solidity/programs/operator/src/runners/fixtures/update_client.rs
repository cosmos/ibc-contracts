// SPDX-License-Identifier: Apache-2.0

//! Runner for generating `update_client` fixtures

use crate::{
    cli::command::{fixtures::UpdateClientCmd, OutputPath},
    runners::genesis::SP1ICS07TendermintGenesis,
};
use alloy::sol_types::SolValue;
use ibc_eureka_solidity_types::msgs::{
    IICS07TendermintMsgs::{ClientState, ConsensusState},
    ISP1Msgs::SP1Proof,
    IUpdateClientMsgs::{MsgUpdateClient, UpdateClientOutput},
};
use ibc_eureka_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use serde::{Deserialize, Serialize};
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

/// The fixture data to be used in [`UpdateClientProgram`] tests.
#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SP1ICS07UpdateClientFixture {
    /// The genesis data.
    #[serde(flatten)]
    genesis: SP1ICS07TendermintGenesis,
    /// The encoded target consensus state.
    #[serde_as(as = "serde_with::hex::Hex")]
    target_consensus_state: Vec<u8>,
    /// Target height.
    target_height: u64,
    /// The encoded update client message.
    #[serde_as(as = "serde_with::hex::Hex")]
    update_msg: Vec<u8>,
}

/// Writes the proof data for the given trusted and target blocks to the given fixture path.
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn run(args: UpdateClientCmd) -> anyhow::Result<()> {
    assert!(
        args.trusted_block < args.target_block,
        "The target block must be greater than the trusted block"
    );

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

    let update_client_elf = std::fs::read(args.elf_paths.update_client_path)?;
    let membership_elf = std::fs::read(args.elf_paths.membership_path)?;
    let misbehaviour_elf = std::fs::read(args.elf_paths.misbehaviour_path)?;
    let uc_and_membership_elf = std::fs::read(args.elf_paths.uc_and_membership_path)?;
    let update_client_program = UpdateClientProgram::new(update_client_elf);
    let membership_program = MembershipProgram::new(membership_elf);
    let misbehaviour_program = MisbehaviourProgram::new(misbehaviour_elf);
    let uc_and_membership_program = UpdateClientAndMembershipProgram::new(uc_and_membership_elf);

    let uc_prover =
        SP1ICS07TendermintProver::new(args.sp1.proof_type, &sp1_prover, &update_client_program)
            .await;

    let trusted_light_block = tm_rpc_client
        .get_light_block(Some(args.trusted_block))
        .await?;
    let target_light_block = tm_rpc_client
        .get_light_block(Some(args.target_block))
        .await?;

    let genesis = SP1ICS07TendermintGenesis::from_env(
        &trusted_light_block,
        args.trust_options.trusting_period,
        args.trust_options.trust_level,
        args.sp1.proof_type,
        &update_client_program,
        &membership_program,
        &uc_and_membership_program,
        &misbehaviour_program,
    )
    .await?;

    let trusted_consensus_state = ConsensusState::abi_decode(&genesis.trusted_consensus_state)?;
    let trusted_client_state = ClientState::abi_decode(&genesis.trusted_client_state)?;

    let proposed_header = target_light_block.into_header(&trusted_light_block);
    let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    // Generate a header update proof for the specified blocks.
    let proof_data = uc_prover
        .generate_proof(
            &trusted_client_state,
            &trusted_consensus_state,
            &proposed_header,
            now_since_unix.as_nanos(),
        )
        .await;

    let output = UpdateClientOutput::abi_decode(proof_data.public_values.as_slice())?;

    let update_msg = MsgUpdateClient {
        sp1Proof: SP1Proof::new(
            &uc_prover.vkey.bytes32(),
            proof_data.bytes(),
            proof_data.public_values.to_vec(),
        ),
    };

    let fixture = SP1ICS07UpdateClientFixture {
        genesis,
        target_consensus_state: output.newConsensusState.abi_encode(),
        target_height: args.target_block,
        update_msg: update_msg.abi_encode(),
    };

    match args.output_path {
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
