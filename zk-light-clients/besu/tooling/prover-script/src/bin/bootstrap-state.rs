use std::{error::Error, time::Duration};

use besu_qbft::{parse_extra_data, validator_set_hash};
use besu_zk_types::QbftHeaderInput;
use clap::Parser;
use prover_script::{rpc::RpcClient, ToolError};
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Derive canonical QBFT bootstrap state from one pinned Besu block")]
struct Args {
    #[arg(long, env = "SOURCE_RPC_URL")]
    source_rpc: String,
    #[arg(long, env = "EXPECTED_SOURCE_CHAIN_ID")]
    expected_source_chain_id: u64,
    #[arg(long)]
    height: u64,
    #[arg(long, default_value_t = 30)]
    rpc_timeout_seconds: u64,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapState {
    height: u64,
    state_root: String,
    block_timestamp: u64,
    validator_set_hash: String,
    validator_count: usize,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run(Args::parse()).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    if args.height == 0 {
        return Err("bootstrap height must be non-zero".into());
    }
    let rpc = RpcClient::new(
        &args.source_rpc,
        Duration::from_secs(args.rpc_timeout_seconds),
    )?;
    let actual_chain_id = rpc.chain_id().await?;
    if actual_chain_id != args.expected_source_chain_id {
        return Err(ToolError::ChainIdMismatch {
            expected: args.expected_source_chain_id,
            actual: actual_chain_id,
        }
        .into());
    }
    let header = rpc.block(args.height).await?;
    if header.number != args.height {
        return Err(ToolError::WrongBlockNumber {
            expected: args.height,
            actual: header.number,
        }
        .into());
    }
    let state = derive_bootstrap_state(&header)?;
    println!("{}", serde_json::to_string_pretty(&state)?);
    Ok(())
}

fn derive_bootstrap_state(header: &QbftHeaderInput) -> Result<BootstrapState, Box<dyn Error>> {
    if header.number == 0 {
        return Err("bootstrap block number must be non-zero".into());
    }
    if header.timestamp == 0 || header.state_root == [0; 32] {
        return Err("bootstrap block has an invalid zero root or timestamp".into());
    }
    let extra = parse_extra_data(&header.extra_data)?;
    Ok(BootstrapState {
        height: header.number,
        state_root: format!("0x{}", hex::encode(header.state_root)),
        block_timestamp: header.timestamp,
        validator_set_hash: format!("0x{}", hex::encode(validator_set_hash(&extra.validators)?)),
        validator_count: extra.validators.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use besu_qbft::{encode_extra_data, QbftExtraData};

    fn header() -> QbftHeaderInput {
        QbftHeaderInput {
            parent_hash: [1; 32],
            ommers_hash: [2; 32],
            beneficiary: [3; 20],
            state_root: [4; 32],
            transactions_root: [5; 32],
            receipts_root: [6; 32],
            logs_bloom: [7; 256],
            difficulty: [0; 32],
            number: 42,
            gas_limit: 30_000_000,
            gas_used: 1,
            timestamp: 1_700_000_000,
            extra_data: encode_extra_data(
                &QbftExtraData {
                    vanity: [8; 32],
                    validators: vec![[1; 20], [2; 20], [3; 20], [4; 20]],
                    vote: None,
                    round: 0,
                    committed_seals: Vec::new(),
                },
                true,
                true,
            ),
            mix_hash: [9; 32],
            nonce: [0; 8],
            base_fee_per_gas: None,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
            requests_hash: None,
            block_access_list_hash: None,
        }
    }

    #[test]
    fn derives_canonical_bootstrap_state() -> Result<(), Box<dyn Error>> {
        let state = derive_bootstrap_state(&header())?;
        assert_eq!(state.height, 42);
        assert_eq!(state.block_timestamp, 1_700_000_000);
        assert_eq!(state.state_root, format!("0x{}", hex::encode([4; 32])));
        assert_eq!(state.validator_count, 4);
        assert_eq!(state.validator_set_hash.len(), 66);
        Ok(())
    }

    #[test]
    fn rejects_zero_height_root_timestamp_and_invalid_extra_data() {
        let mut invalid = header();
        invalid.number = 0;
        assert!(derive_bootstrap_state(&invalid).is_err());

        invalid = header();
        invalid.state_root = [0; 32];
        assert!(derive_bootstrap_state(&invalid).is_err());

        invalid = header();
        invalid.timestamp = 0;
        assert!(derive_bootstrap_state(&invalid).is_err());

        invalid = header();
        invalid.extra_data = vec![0xc0];
        assert!(derive_bootstrap_state(&invalid).is_err());
    }
}
