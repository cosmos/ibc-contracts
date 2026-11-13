use besu_zk_e2e::{latest_timestamp, Direction, E2eResult};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn one_consensus_state_verifies_multiple_packet_commitments() -> E2eResult {
    for (direction, first_payload, second_payload) in [
        (
            Direction::a_to_b()?,
            b"reuse-a-to-b-one".as_slice(),
            b"reuse-a-to-b-two".as_slice(),
        ),
        (
            Direction::b_to_a()?,
            b"reuse-b-to-a-one".as_slice(),
            b"reuse-b-to-a-two".as_slice(),
        ),
    ] {
        let timeout = latest_timestamp(&direction.destination_rpc)? + 3_600;
        let (first, _) = direction.send_packet(timeout, first_payload)?;
        let (second, height) = direction.send_packet(timeout, second_payload)?;
        direction.update_to(height)?;
        let first_proof = direction.prove_packet_membership(&first, height)?;
        let second_proof = direction.prove_packet_membership(&second, height)?;
        direction.recv_packet(&first, &first_proof, height)?;
        direction.recv_packet(&second, &second_proof, height)?;
    }
    Ok(())
}
