use besu_qbft::{encode_extra_data, QbftExtraData};
use prover_script::{
    rpc::{
        parse_batch_proof, parse_block, parse_consensus_state, parse_proof, parse_quantity_u64,
        parse_rpc_response, quantity,
    },
    ToolError,
};
use serde_json::{json, Value};
use std::io;

fn repeated(byte: u8, length: usize) -> String {
    format!("0x{}", hex::encode(vec![byte; length]))
}

fn valid_block(number: u64) -> Value {
    let extra = encode_extra_data(
        &QbftExtraData {
            vanity: [0x55; 32],
            validators: vec![[1; 20]],
            vote: None,
            round: 0,
            committed_seals: Vec::new(),
        },
        true,
        true,
    );
    json!({
        "parentHash": repeated(1, 32),
        "sha3Uncles": repeated(2, 32),
        "miner": repeated(3, 20),
        "stateRoot": repeated(4, 32),
        "transactionsRoot": repeated(5, 32),
        "receiptsRoot": repeated(6, 32),
        "logsBloom": repeated(7, 256),
        "difficulty": "0x0",
        "number": quantity(number),
        "gasLimit": "0x1c9c380",
        "gasUsed": "0x1",
        "timestamp": "0x6553f100",
        "extraData": format!("0x{}", hex::encode(extra)),
        "mixHash": repeated(8, 32),
        "nonce": repeated(0, 8)
    })
}

#[test]
fn block_parser_accepts_exact_fields_and_rejects_defaults() -> Result<(), Box<dyn std::error::Error>>
{
    let parsed = parse_block(&valid_block(16), 16)?;
    assert_eq!(parsed.number, 16);
    assert_eq!(parsed.state_root, [4; 32]);

    assert!(matches!(
        parse_block(&Value::Null, 16),
        Err(ToolError::NullResult("eth_getBlockByNumber"))
    ));
    assert!(matches!(
        parse_block(&valid_block(17), 16),
        Err(ToolError::WrongBlockNumber { .. })
    ));
    let mut missing = valid_block(16);
    missing
        .as_object_mut()
        .ok_or_else(|| io::Error::other("fixture is not an object"))?
        .remove("stateRoot");
    assert!(matches!(
        parse_block(&missing, 16),
        Err(ToolError::MissingField("stateRoot"))
    ));
    let mut malformed = valid_block(16);
    malformed["nonce"] = json!("0x0");
    assert!(matches!(
        parse_block(&malformed, 16),
        Err(ToolError::MalformedHex("nonce"))
    ));
    let mut wrong_length = valid_block(16);
    wrong_length["parentHash"] = repeated(1, 31).into();
    assert!(matches!(
        parse_block(&wrong_length, 16),
        Err(ToolError::WrongByteLength {
            field: "parentHash",
            expected: 32,
            actual: 31
        })
    ));
    let mut malformed_extra = valid_block(16);
    malformed_extra["extraData"] = json!("0xc0");
    assert!(matches!(
        parse_block(&malformed_extra, 16),
        Err(ToolError::InvalidWitness(_))
    ));
    let mut invalid_optional_tail = valid_block(16);
    invalid_optional_tail["baseFeePerGas"] = json!("0x1");
    invalid_optional_tail["blobGasUsed"] = json!("0x1");
    assert!(matches!(
        parse_block(&invalid_optional_tail, 16),
        Err(ToolError::InvalidWitness(_))
    ));
    let mut unknown_tail = valid_block(16);
    unknown_tail["futureHeaderTail"] = repeated(9, 32).into();
    assert!(matches!(
        parse_block(&unknown_tail, 16),
        Err(ToolError::UnsupportedBlockField(field)) if field == "futureHeaderTail"
    ));
    Ok(())
}

#[test]
fn quantities_are_minimal_and_overflow_checked() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(quantity(0), "0x0");
    assert_eq!(quantity(16), "0x10");
    assert_eq!(parse_quantity_u64(&json!("0x10"), "value")?, 16);
    assert!(matches!(
        parse_quantity_u64(&json!("0x00"), "value"),
        Err(ToolError::NonCanonicalQuantity("value"))
    ));
    assert!(matches!(
        parse_quantity_u64(&json!("0x10000000000000000"), "value"),
        Err(ToolError::NumericOverflow("value"))
    ));
    assert!(matches!(
        parse_quantity_u64(&json!("0xgg"), "value"),
        Err(ToolError::MalformedHex("value"))
    ));
    Ok(())
}

#[test]
fn rpc_envelope_rejects_errors_and_wrong_identity() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        parse_rpc_response(&json!({"jsonrpc":"2.0", "id":1, "result":"0x1"}))?,
        json!("0x1")
    );
    assert!(matches!(
        parse_rpc_response(&json!({
            "jsonrpc":"2.0", "id":1, "error":{"code":-32000, "message":"unavailable"}
        })),
        Err(ToolError::RpcRejected(-32000))
    ));
    assert!(matches!(
        parse_rpc_response(&json!({
            "jsonrpc":"2.0", "id":1,
            "error":{"code":-32001, "message":"missing trie node: historical state pruned"}
        })),
        Err(ToolError::HistoricalStateUnavailable)
    ));
    assert!(matches!(
        parse_rpc_response(&json!({
            "jsonrpc":"2.0", "id":1, "error":{"code":-32000}
        })),
        Err(ToolError::MalformedRpcResponse)
    ));
    assert!(matches!(
        parse_rpc_response(&json!({"jsonrpc":"2.0", "id":2, "result":"0x1"})),
        Err(ToolError::MalformedRpcResponse)
    ));
    Ok(())
}

#[test]
fn proof_parser_binds_address_key_and_exact_count() -> Result<(), Box<dyn std::error::Error>> {
    let address = [0x11; 20];
    let mut slot = [0x22; 32];
    slot[0] = 0;
    let valid = json!({
        "address": repeated(0x11, 20),
        "accountProof": ["0xc0"],
        "storageProof": [{
            "key": format!("0x{}", hex::encode(slot)),
            "value": "0x0",
            "proof": []
        }]
    });
    let proof = parse_proof(&valid, address, slot)?;
    assert_eq!(proof.account_proof, vec![vec![0xc0]]);

    assert!(matches!(
        parse_proof(&Value::Null, address, slot),
        Err(ToolError::NullResult("eth_getProof"))
    ));

    assert!(matches!(
        parse_proof(&valid, [0x12; 20], slot),
        Err(ToolError::WrongProofAddress)
    ));
    assert!(matches!(
        parse_proof(&valid, address, [0x23; 32]),
        Err(ToolError::WrongStorageKey)
    ));
    let mut quantity_key = valid.clone();
    quantity_key["storageProof"][0]["key"] = json!("0x22");
    assert!(matches!(
        parse_proof(&quantity_key, address, slot),
        Err(ToolError::WrongStorageKey)
    ));
    let mut empty = valid.clone();
    empty["accountProof"] = json!([]);
    assert!(matches!(
        parse_proof(&empty, address, slot),
        Err(ToolError::EmptyAccountProof)
    ));
    let mut multiple = valid;
    multiple["storageProof"] = json!([{}, {}]);
    assert!(matches!(
        parse_proof(&multiple, address, slot),
        Err(ToolError::WrongStorageProofCount(2))
    ));

    let mut missing = json!({
        "address": repeated(0x11, 20),
        "accountProof": ["0xc0"],
        "storageProof": []
    });
    assert!(matches!(
        parse_proof(&missing, address, slot),
        Err(ToolError::WrongStorageProofCount(0))
    ));
    let mut no_storage = missing.clone();
    no_storage
        .as_object_mut()
        .ok_or_else(|| io::Error::other("fixture is not an object"))?
        .remove("storageProof");
    assert!(matches!(
        parse_proof(&no_storage, address, slot),
        Err(ToolError::MissingField("storageProof"))
    ));
    missing["storageProof"] = json!([{
        "key": format!("0x{}", hex::encode(slot)), "value": "0x0", "proof": ["0xb801"]
    }]);
    assert!(matches!(
        parse_proof(&missing, address, slot),
        Err(ToolError::MalformedProofNode)
    ));
    missing["storageProof"][0]["proof"] = json!(["0xzz"]);
    assert!(matches!(
        parse_proof(&missing, address, slot),
        Err(ToolError::MalformedHex("proof"))
    ));
    missing["storageProof"][0]["proof"] = json!(["0xb9ffff"]);
    assert!(matches!(
        parse_proof(&missing, address, slot),
        Err(ToolError::MalformedProofNode)
    ));
    Ok(())
}

#[test]
fn storage_keys_accept_padded_and_shortened_encodings() -> Result<(), Box<dyn std::error::Error>> {
    let observed = "c437fc1e933ee28e397279c4c2432d1b358317cfa7e4223f0a5aa2d9943fab";
    let mut observed_slot = [0_u8; 32];
    hex::decode_to_slice(observed, &mut observed_slot[1..])?;
    let mut small_slot = [0_u8; 32];
    small_slot[31] = 0x0a;
    for (slot, encodings) in [
        (
            observed_slot,
            vec![format!("0x00{observed}"), format!("0x{observed}")],
        ),
        (
            small_slot,
            vec![
                format!("0x{}", hex::encode(small_slot)),
                "0xa".to_owned(),
                "0x0a".to_owned(),
            ],
        ),
        ([0; 32], vec![repeated(0, 32), "0x0".to_owned()]),
    ] {
        for key in encodings {
            let response = json!({
                "address": repeated(0x11, 20), "accountProof": ["0xc0"],
                "storageProof": [{"key": key, "proof": []}]
            });
            assert!(
                parse_proof(&response, [0x11; 20], slot).is_ok(),
                "key: {key}"
            );
        }
    }
    Ok(())
}

#[test]
fn storage_key_normalization_preserves_batch_matching() -> Result<(), Box<dyn std::error::Error>> {
    let mut first = [0_u8; 32];
    first[31] = 1;
    let mut second = [0_u8; 32];
    second[31] = 2;
    let mut response = json!({
        "address": repeated(0x11, 20), "accountProof": ["0xc0"],
        "storageProof": [
            {"key": "0x2", "proof": ["0xc0"]},
            {"key": format!("0x{}", hex::encode(first)), "proof": []}
        ]
    });
    let proof = parse_batch_proof(&response, [0x11; 20], &[first, second])?;
    assert_eq!(
        proof.storage_proofs,
        vec![Vec::<Vec<u8>>::new(), vec![vec![0xc0]]]
    );

    response["storageProof"][0]["key"] = json!("0x1");
    assert!(matches!(
        parse_batch_proof(&response, [0x11; 20], &[first, second]),
        Err(ToolError::WrongStorageKey)
    ));

    for key in [json!("0x"), json!("0xgg"), json!("01"), json!(null)] {
        response["storageProof"][0]["key"] = key;
        assert!(parse_batch_proof(&response, [0x11; 20], &[first, second]).is_err());
    }
    response["storageProof"][0]["key"] = json!(format!("0x1{}", "0".repeat(64)));
    assert!(matches!(
        parse_batch_proof(&response, [0x11; 20], &[first, second]),
        Err(ToolError::NumericOverflow("key"))
    ));
    Ok(())
}

#[test]
fn consensus_state_decoder_is_canonical_and_requires_existence(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&[0; 31]);
    encoded.push(1);
    encoded.extend_from_slice(&[0x44; 32]);
    encoded.extend_from_slice(&[0; 24]);
    encoded.extend_from_slice(&123_u64.to_be_bytes());
    encoded.extend_from_slice(&[0x55; 32]);
    let state = parse_consensus_state(&encoded, 7)?;
    assert_eq!(state.block_timestamp, 123);

    encoded[31] = 0;
    assert!(matches!(
        parse_consensus_state(&encoded, 7),
        Err(ToolError::MissingConsensusState(7))
    ));
    encoded[31] = 2;
    assert!(matches!(
        parse_consensus_state(&encoded, 7),
        Err(ToolError::MalformedConsensusState)
    ));
    encoded[31] = 1;
    encoded[64] = 1;
    assert!(matches!(
        parse_consensus_state(&encoded, 7),
        Err(ToolError::MalformedConsensusState)
    ));
    encoded[64] = 0;
    encoded[88..96].fill(0);
    assert!(matches!(
        parse_consensus_state(&encoded, 7),
        Err(ToolError::MalformedConsensusState)
    ));
    encoded[88..96].copy_from_slice(&123_u64.to_be_bytes());
    encoded[96..128].fill(0);
    assert!(matches!(
        parse_consensus_state(&encoded, 7),
        Err(ToolError::MalformedConsensusState)
    ));
    encoded.push(0);
    assert!(matches!(
        parse_consensus_state(&encoded, 7),
        Err(ToolError::MalformedConsensusState)
    ));
    Ok(())
}
