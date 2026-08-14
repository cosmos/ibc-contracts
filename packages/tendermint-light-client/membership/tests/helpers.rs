// SPDX-License-Identifier: Apache-2.0

//! Common test utilities and fixtures

use serde::Deserialize;
use std::fs;
use std::path::Path;

use ibc_core_commitment_types::merkle::MerkleProof;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as ProtoMerkleProof;
use ibc_proto::ibc::lightclients::tendermint::v1::ConsensusState as ProtoConsensusState;
use prost::Message;
use tendermint_light_client_membership::{membership, KVPair, MembershipError, MembershipOutput};

#[derive(Debug, Clone, Deserialize)]
pub struct MembershipMsgFixture {
    pub path: Vec<String>,
    pub proof: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MembershipVerificationFixture {
    pub membership_msg: MembershipMsgFixture,
    pub consensus_state_hex: String,
}

impl From<&MembershipMsgFixture> for KVPair {
    fn from(fixture: &MembershipMsgFixture) -> Self {
        let path_bytes: Vec<Vec<u8>> = fixture.path.iter().map(|s| s.as_bytes().to_vec()).collect();
        let value_bytes = hex::decode(&fixture.value).expect("valid hex");

        Self::new(path_bytes, value_bytes)
    }
}

pub fn hex_to_merkle_proof(hex_str: &str) -> MerkleProof {
    let bytes = hex::decode(hex_str)
        .map_err(|e| format!("Failed to decode merkle proof hex: {}", e))
        .expect("valid hex");

    let proto_merkle_proof = ProtoMerkleProof::decode(bytes.as_slice())
        .map_err(|e| format!("Failed to decode protobuf merkle proof: {}", e))
        .expect("valid protobuf");

    proto_merkle_proof
        .try_into()
        .map_err(|e| format!("Failed to convert merkle proof: {:?}", e))
        .expect("valid merkle proof")
}

pub fn load_membership_fixture(filename: &str) -> MembershipVerificationFixture {
    let fixture_path = Path::new("../fixtures").join(format!("{}.json", filename));
    let fixture_content = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", fixture_path.display()));

    serde_json::from_str(&fixture_content)
        .unwrap_or_else(|_| panic!("Failed to parse fixture: {}", fixture_path.display()))
}

pub fn load_membership_fixture_data() -> MembershipVerificationFixture {
    load_membership_fixture("verify_membership_key_0")
}

pub fn load_non_membership_fixture_data() -> MembershipVerificationFixture {
    load_membership_fixture("verify_non-membership_key_1")
}

pub struct TestContext {
    pub app_hash: [u8; 32],
    pub kv_pair: KVPair,
    pub merkle_proof: MerkleProof,
}

pub fn setup_test_context(fixture: MembershipVerificationFixture) -> TestContext {
    // Decode consensus state and extract app hash
    let consensus_state_bytes = hex::decode(&fixture.consensus_state_hex)
        .expect("Failed to decode consensus_state_hex from fixture");

    let proto_consensus_state = ProtoConsensusState::decode(consensus_state_bytes.as_slice())
        .expect("Failed to decode consensus state");

    let root = proto_consensus_state
        .root
        .expect("Missing root in consensus state");

    let app_hash: [u8; 32] = root
        .hash
        .try_into()
        .expect("App hash must be exactly 32 bytes");

    let kv_pair = KVPair::from(&fixture.membership_msg);
    let merkle_proof = hex_to_merkle_proof(&fixture.membership_msg.proof);

    TestContext {
        app_hash,
        kv_pair,
        merkle_proof,
    }
}

pub fn execute_membership(ctx: &TestContext) -> Result<MembershipOutput, MembershipError> {
    let request = vec![(ctx.kv_pair.clone(), ctx.merkle_proof.clone())];
    membership(ctx.app_hash, request.into_iter())
}

pub fn assert_membership_fails_with(
    ctx: &TestContext,
    expected_error: MembershipError,
    test_description: &str,
) {
    let actual_error = execute_membership(ctx).expect_err(&format!(
        "Membership verification should have failed for: {}",
        test_description
    ));

    assert!(
        std::mem::discriminant(&expected_error) == std::mem::discriminant(&actual_error),
        "Expected {:?} but got {:?} for: {}",
        expected_error,
        actual_error,
        test_description
    );
}

pub fn assert_membership_succeeds(ctx: &TestContext, test_description: &str) {
    execute_membership(ctx)
        .unwrap_or_else(|_| panic!("Membership verification failed for {test_description}"));
}

pub fn create_context_with_wrong_app_hash(fixture: MembershipVerificationFixture) -> TestContext {
    let mut ctx = setup_test_context(fixture);
    ctx.app_hash = [0xFF; 32];
    ctx
}

pub fn create_context_with_empty_proof(fixture: MembershipVerificationFixture) -> TestContext {
    let mut ctx = setup_test_context(fixture);
    ctx.merkle_proof = MerkleProof { proofs: vec![] };
    ctx
}

pub fn create_context_with_mismatched_path(
    fixture: MembershipVerificationFixture,
    new_path: Vec<Vec<u8>>,
) -> TestContext {
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.path = new_path;
    ctx
}

pub fn create_context_with_different_proof(
    mut ctx: TestContext,
    other_fixture: MembershipVerificationFixture,
) -> TestContext {
    ctx.merkle_proof = hex_to_merkle_proof(&other_fixture.membership_msg.proof);
    ctx
}

fn create_malformed_proof_hex(original_hex: &str) -> String {
    let mut s = original_hex.to_string();
    s.replace_range(0..1, if &s[0..1] == "0" { "1" } else { "0" });
    s
}

pub fn create_context_with_malformed_proof(fixture: MembershipVerificationFixture) -> TestContext {
    let mut ctx = setup_test_context(fixture.clone());

    let malformed_hex = create_malformed_proof_hex(&fixture.membership_msg.proof);

    ctx.merkle_proof = hex::decode(&malformed_hex)
        .ok()
        .and_then(|bytes| ProtoMerkleProof::decode(bytes.as_slice()).ok())
        .and_then(|proto| proto.try_into().ok())
        .unwrap_or_else(|| MerkleProof { proofs: vec![] });

    ctx
}
