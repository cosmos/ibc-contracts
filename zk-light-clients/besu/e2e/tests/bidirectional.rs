use besu_zk_e2e::{complete_packet_and_ack, Direction, E2eResult};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn packet_and_ack_flow_works_in_both_directions() -> E2eResult {
    complete_packet_and_ack(&Direction::a_to_b()?, b"a-to-b")?;
    complete_packet_and_ack(&Direction::b_to_a()?, b"b-to-a")?;
    Ok(())
}
