use besu_zk_e2e::{assert_app_payload, Direction, E2eResult};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn packet_commitment_membership_drives_recv_packet() -> E2eResult {
    for (direction, payload) in [
        (Direction::a_to_b()?, b"packet-membership-a-to-b".as_slice()),
        (Direction::b_to_a()?, b"packet-membership-b-to-a".as_slice()),
    ] {
        let timeout = besu_zk_e2e::latest_timestamp(&direction.destination_rpc)? + 3_600;
        let (packet, height) = direction.send_packet(timeout, payload)?;
        direction.update_to(height)?;
        let proof = direction.prove_packet_membership(&packet, height)?;
        direction.recv_packet(&packet, &proof, height)?;
        assert_app_payload(
            &direction.destination_rpc,
            &direction.destination_app,
            payload,
        )?;
        direction.reject_recv_packet_replay(&packet, &proof, height)?;
    }
    Ok(())
}
