use besu_qbft::{
    committed_seal_digest, encode_extra_data, encode_header, QbftError, QbftExtraData,
};
use besu_zk_types::QbftHeaderInput;

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
        number: 1,
        gas_limit: 2,
        gas_used: 3,
        timestamp: 4,
        extra_data: encode_extra_data(
            &QbftExtraData {
                vanity: [8; 32],
                validators: vec![[1; 20]],
                vote: None,
                round: 1,
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
fn all_supported_contiguous_header_schemas_encode_and_change_digest() -> Result<(), QbftError> {
    let mut current = header();
    let mut prior = committed_seal_digest(&current)?;
    current.base_fee_per_gas = Some([1; 32]);
    current.withdrawals_root = Some([2; 32]);
    current.blob_gas_used = Some(3);
    current.excess_blob_gas = Some(4);
    current.parent_beacon_block_root = Some([5; 32]);
    current.requests_hash = Some([6; 32]);
    current.block_access_list_hash = Some([7; 32]);

    for keep in 1..=7 {
        let mut schema = current.clone();
        if keep < 7 {
            schema.block_access_list_hash = None;
        }
        if keep < 6 {
            schema.requests_hash = None;
        }
        if keep < 5 {
            schema.parent_beacon_block_root = None;
        }
        if keep < 4 {
            schema.excess_blob_gas = None;
        }
        if keep < 3 {
            schema.blob_gas_used = None;
        }
        if keep < 2 {
            schema.withdrawals_root = None;
        }
        let digest = committed_seal_digest(&schema)?;
        assert_ne!(digest, prior);
        prior = digest;
    }
    Ok(())
}

#[test]
fn optional_tail_gaps_are_rejected() {
    let mut invalid = header();
    invalid.withdrawals_root = Some([1; 32]);
    assert_eq!(
        encode_header(&invalid, &invalid.extra_data),
        Err(QbftError::InvalidOptionalTail)
    );

    invalid = header();
    invalid.base_fee_per_gas = Some([0; 32]);
    invalid.blob_gas_used = Some(1);
    assert_eq!(
        encode_header(&invalid, &invalid.extra_data),
        Err(QbftError::InvalidOptionalTail)
    );
}
