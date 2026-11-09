use besu_zk_types::{
    Eip1186ProofWitness, MembershipEntryWitness, MembershipWitness, NonMembershipWitness,
    QbftHeaderInput, UpdateWitness, MEMBERSHIP_SCHEMA_VERSION, SCHEMA_VERSION,
};
use serde::{de::DeserializeOwned, Serialize};

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
        number: 2,
        gas_limit: 30_000_000,
        gas_used: 1,
        timestamp: 1_700_000_000,
        extra_data: vec![0xc0],
        mix_hash: [8; 32],
        nonce: [0; 8],
        base_fee_per_gas: Some([9; 32]),
        withdrawals_root: Some([10; 32]),
        blob_gas_used: None,
        excess_blob_gas: None,
        parent_beacon_block_root: None,
        requests_hash: None,
        block_access_list_hash: None,
    }
}

fn assert_sp1_transport_traits<T: Serialize + DeserializeOwned>() {}

#[test]
fn all_private_witnesses_support_the_sp1_serde_transport() {
    assert_sp1_transport_traits::<UpdateWitness>();
    assert_sp1_transport_traits::<MembershipWitness>();
    assert_sp1_transport_traits::<NonMembershipWitness>();

    let _update = UpdateWitness {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        trusted_height: 1,
        trusted_validator_set_hash: [11; 32],
        trusted_validators: vec![[12; 20]],
        target_header: header(),
    };
    let proof = Eip1186ProofWitness {
        account_proof: vec![vec![0xc0]],
        storage_proof: vec![vec![0xc0]],
    };
    let _membership = MembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 2,
        state_root: [13; 32],
        source_router: [14; 20],
        account_proof: proof.account_proof.clone(),
        entries: vec![MembershipEntryWitness {
            path: vec![b"client-0\x01\0\0\0\0\0\0\0\x01".to_vec()],
            expected_commitment: [15; 32],
            storage_proof: proof.storage_proof.clone(),
        }],
    };
    let _non_membership = NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 2,
        state_root: [13; 32],
        source_router: [14; 20],
        path: vec![b"client-0\x02\0\0\0\0\0\0\0\x01".to_vec()],
        proof,
    };
}
