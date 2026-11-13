use besu_public_values::{decode_client_state, encode_client_state, PublicValuesError};
use besu_zk_types::ClientState;

#[test]
fn canonical_client_state_matches_solidity_golden_bytes() -> Result<(), hex::FromHexError> {
    let state = ClientState {
        latest_height: 7,
        trusting_period_seconds: 1_000,
        max_clock_drift_seconds: 20,
    };
    let expected = hex::decode(
        "0000000000000000000000000000000000000000000000000000000000000007\
         00000000000000000000000000000000000000000000000000000000000003e8\
         0000000000000000000000000000000000000000000000000000000000000014",
    )?;
    assert_eq!(encode_client_state(&state), expected);
    assert_eq!(decode_client_state(&expected), Ok(state));
    Ok(())
}

#[test]
fn client_state_codec_rejects_wrong_width_and_trailing_bytes() {
    let state = ClientState {
        latest_height: u64::MAX,
        trusting_period_seconds: 0,
        max_clock_drift_seconds: u64::MAX,
    };
    let encoded = encode_client_state(&state);
    assert_eq!(decode_client_state(&encoded), Ok(state));
    assert_eq!(
        decode_client_state(&encoded[..encoded.len() - 1]),
        Err(PublicValuesError::MalformedAbi)
    );
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert_eq!(
        decode_client_state(&trailing),
        Err(PublicValuesError::NonCanonicalAbi)
    );
    let mut wrong_width = encoded;
    wrong_width[0] = 1;
    assert_eq!(
        decode_client_state(&wrong_width),
        Err(PublicValuesError::NonCanonicalAbi)
    );
}
