use proof_api::{
    packet::build_statement,
    proto::{Packet, ProofKind},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    kind: i32,
    packet: Packet,
    prefix: Vec<String>,
    configured_client: String,
    counterparty: String,
    terminal_path: String,
    full_path: Vec<String>,
    path_digest: String,
    logical_slot: String,
    expected_commitment: Option<String>,
}

#[test]
fn statements_match_go_reference() -> Result<(), Box<dyn std::error::Error>> {
    let fixtures: Vec<Fixture> =
        serde_json::from_str(include_str!("fixtures/packet-statements.json"))?;
    assert!(fixtures.len() >= 3);
    for fixture in fixtures {
        let prefix: Vec<Vec<u8>> = fixture
            .prefix
            .iter()
            .map(hex::decode)
            .collect::<Result<_, _>>()?;
        let kind = ProofKind::try_from(fixture.kind)?;
        let original = prefix.clone();
        let statement = build_statement(
            kind,
            &fixture.packet,
            &prefix,
            &fixture.configured_client,
            &fixture.counterparty,
        )?;
        assert_eq!(prefix, original);
        assert_eq!(hex::encode(statement.terminal_path), fixture.terminal_path);
        assert_eq!(
            statement
                .full_path
                .iter()
                .map(hex::encode)
                .collect::<Vec<_>>(),
            fixture.full_path
        );
        assert_eq!(hex::encode(statement.path_digest), fixture.path_digest);
        assert_eq!(hex::encode(statement.logical_slot), fixture.logical_slot);
        assert_eq!(
            statement.expected_commitment.map(hex::encode),
            fixture.expected_commitment
        );
        let mut wrong = fixture.packet.clone();
        wrong.source_client = "wrong".into();
        assert!(build_statement(
            kind,
            &wrong,
            &prefix,
            &fixture.configured_client,
            &fixture.counterparty
        )
        .is_err());
        assert!(build_statement(
            kind,
            &fixture.packet,
            &[],
            &fixture.configured_client,
            &fixture.counterparty
        )
        .is_err());
    }
    Ok(())
}
