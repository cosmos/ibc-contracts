use besu_zk_e2e::{Direction, E2eResult};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn modified_groth16_proof_is_rejected_without_advancing_state() -> E2eResult {
    for (direction, payload) in [
        (Direction::a_to_b()?, b"negative-a-to-b".as_slice()),
        (Direction::b_to_a()?, b"negative-b-to-a".as_slice()),
    ] {
        let before = direction.latest_height()?;
        direction.verifier_rejects_mutated_update()?;
        let after = direction.latest_height()?;
        if before != after {
            return Err(
                format!("invalid proof advanced latest height from {before} to {after}").into(),
            );
        }
        let timeout = besu_zk_e2e::latest_timestamp(&direction.destination_rpc)? + 3_600;
        let (packet, height) = direction.send_packet(timeout, payload)?;
        direction.update_to(height)?;
        let proof = direction.prove_packet_membership(&packet, height)?;
        direction.reject_packet_context_mutations(&packet, &proof, height)?;
        direction.reject_non_membership_proof_on_membership_path(&packet, height)?;
    }
    Ok(())
}
