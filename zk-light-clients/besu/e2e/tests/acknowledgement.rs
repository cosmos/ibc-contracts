use besu_zk_e2e::{assert_app_ack, complete_packet_and_ack, Direction, E2eResult};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn acknowledgement_membership_drives_ack_packet() -> E2eResult {
    let direction = Direction::a_to_b()?;
    let payload = b"acknowledgement";
    complete_packet_and_ack(&direction, payload)?;
    let mut expected = b"ack:".to_vec();
    expected.extend_from_slice(payload);
    assert_app_ack(&direction.source_rpc, &direction.source_app, &expected)
}
