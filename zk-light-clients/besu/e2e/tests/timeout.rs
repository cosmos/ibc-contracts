use besu_zk_e2e::{
    assert_app_timed_out, latest_timestamp, wait_for_timestamp, Direction, E2eResult,
};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn receipt_absence_uses_the_same_height_timestamp_for_timeout() -> E2eResult {
    timeout_direction(
        Direction::a_to_b()?,
        Direction::b_to_a()?,
        b"timeout-a-to-b",
    )?;
    timeout_direction(
        Direction::b_to_a()?,
        Direction::a_to_b()?,
        b"timeout-b-to-a",
    )
}

fn timeout_direction(
    source_to_destination: Direction,
    destination_to_source: Direction,
    payload: &[u8],
) -> E2eResult {
    let timeout = latest_timestamp(&source_to_destination.source_rpc)?
        .max(latest_timestamp(&source_to_destination.destination_rpc)?)
        + 10;
    let (packet, _) = source_to_destination.send_packet(timeout, payload)?;

    let before_height = destination_to_source.update_to_new_head()?;
    let before_proof = destination_to_source.prove_receipt_absence(&packet, before_height)?;
    destination_to_source.timeout_packet(&packet, &before_proof, before_height, false)?;

    let after_height = wait_for_timestamp(&destination_to_source.source_rpc, timeout)?;
    destination_to_source.update_to(after_height)?;
    let after_proof = destination_to_source.prove_receipt_absence(&packet, after_height)?;
    let mut wrong_sequence = packet.clone();
    wrong_sequence.sequence = wrong_sequence
        .sequence
        .checked_add(1)
        .ok_or("packet sequence overflow")?;
    destination_to_source.timeout_packet(&wrong_sequence, &after_proof, after_height, false)?;
    destination_to_source.timeout_packet(&packet, &after_proof, after_height, true)?;
    assert_app_timed_out(
        &source_to_destination.source_rpc,
        &source_to_destination.source_app,
    )
}
