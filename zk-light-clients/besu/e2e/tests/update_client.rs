use besu_zk_e2e::{Direction, E2eResult};

#[test]
#[ignore = "PENDING_HEAVY_VALIDATION: requires two live Besu chains and Groth16 proving"]
fn updates_from_one_trusted_header_to_one_later_header() -> E2eResult {
    Direction::a_to_b()?.update_to_new_head()?;
    Direction::b_to_a()?.update_to_new_head()?;
    Ok(())
}
