use besu_qbft::{
    encode_extra_data, parse_extra_data, QbftError, QbftExtraData, QbftVote, VoteKind,
};
use evm_mpt::rlp::{encode_bytes, encode_list, encode_u64};

fn validators(count: u8) -> Vec<[u8; 20]> {
    (1..=count)
        .map(|last| {
            let mut address = [0_u8; 20];
            address[19] = last;
            address
        })
        .collect()
}

fn seal() -> Vec<u8> {
    let mut seal = vec![0_u8; 65];
    seal[31] = 1;
    seal[63] = 1;
    seal
}

fn encoded_with(validators: Vec<[u8; 20]>) -> Vec<u8> {
    encode_extra_data(
        &QbftExtraData {
            vanity: [0x42; 32],
            validators,
            vote: None,
            round: 0,
            committed_seals: vec![seal()],
        },
        true,
        true,
    )
}

#[test]
fn parses_one_four_and_quorum_boundary_validator_counts() {
    for count in [1, 2, 3, 4, 5, 6, 7] {
        let expected = validators(count);
        let parsed = parse_extra_data(&encoded_with(expected.clone()));
        assert_eq!(parsed.map(|extra| extra.validators), Ok(expected));
    }
}

#[test]
fn parser_preserves_all_fields_and_original_high_s_seal_bytes(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut high_s = seal();
    // n - 1 is a valid high-S scalar. Parsing must not normalize its bytes.
    hex::decode_to_slice(
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        &mut high_s[32..64],
    )?;
    high_s[64] = 1;
    let expected = QbftExtraData {
        vanity: [0x42; 32],
        validators: validators(4),
        vote: Some(QbftVote {
            recipient: [9; 20],
            kind: VoteKind::Add,
        }),
        round: u32::MAX,
        committed_seals: vec![seal(), high_s],
    };
    let encoded = encode_extra_data(&expected, true, true);
    let parsed = parse_extra_data(&encoded)?;
    assert_eq!(parsed, expected);
    assert_eq!(encode_extra_data(&parsed, true, true), encoded);
    Ok(())
}

#[test]
fn parses_add_drop_and_empty_votes() {
    for (vote, expected) in [
        (None, None),
        (
            Some(QbftVote {
                recipient: [9; 20],
                kind: VoteKind::Add,
            }),
            Some(VoteKind::Add),
        ),
        (
            Some(QbftVote {
                recipient: [9; 20],
                kind: VoteKind::Drop,
            }),
            Some(VoteKind::Drop),
        ),
    ] {
        let encoded = encode_extra_data(
            &QbftExtraData {
                vanity: [0; 32],
                validators: validators(1),
                vote,
                round: u32::MAX,
                committed_seals: Vec::new(),
            },
            true,
            true,
        );
        assert_eq!(
            parse_extra_data(&encoded).map(|extra| extra.vote.map(|vote| vote.kind)),
            Ok(expected)
        );
    }
}

#[test]
fn rejects_structure_vanity_validator_and_order_mutations() {
    assert!(parse_extra_data(&[]).is_err());
    assert!(parse_extra_data(&encode_bytes(&[1])).is_err());
    assert!(parse_extra_data(&encode_list(&[])).is_err());

    let one = validators(1);
    let validator_list = encode_list(
        &one.iter()
            .map(|value| encode_bytes(value))
            .collect::<Vec<_>>(),
    );
    let valid_fields = [
        encode_bytes(&[0; 32]),
        validator_list.clone(),
        encode_list(&[]),
        encode_u64(0),
        encode_list(&[]),
    ];
    let mut missing_vanity = valid_fields.to_vec();
    missing_vanity.remove(0);
    assert!(parse_extra_data(&encode_list(&missing_vanity)).is_err());

    let mut wrong_vanity = valid_fields;
    wrong_vanity[0] = encode_bytes(&[0; 31]);
    assert_eq!(
        parse_extra_data(&encode_list(&wrong_vanity)),
        Err(QbftError::InvalidVanityLength(31))
    );

    assert!(parse_extra_data(&encoded_with(Vec::new())).is_err());
    assert!(parse_extra_data(&encoded_with(vec![[0; 20]])).is_err());
    assert!(parse_extra_data(&encoded_with(vec![[2; 20], [1; 20]])).is_err());
    assert!(parse_extra_data(&encoded_with(vec![[1; 20], [1; 20]])).is_err());

    let malformed_address = encode_list(&[
        encode_bytes(&[0; 32]),
        encode_list(&[encode_bytes(&[1; 19])]),
        encode_list(&[]),
        encode_u64(0),
        encode_list(&[]),
    ]);
    assert_eq!(
        parse_extra_data(&malformed_address),
        Err(QbftError::InvalidValidatorLength(19))
    );
}

#[test]
fn rejects_vote_round_seal_and_canonical_rlp_mutations() {
    let validator_list = encode_list(&[encode_bytes(&validators(1)[0])]);
    let fields = |vote: Vec<u8>, round: Vec<u8>, seals: Vec<u8>| {
        encode_list(&[
            encode_bytes(&[0; 32]),
            validator_list.clone(),
            vote,
            round,
            seals,
        ])
    };
    assert!(parse_extra_data(&fields(encode_bytes(&[]), encode_u64(0), encode_list(&[]))).is_err());
    assert!(parse_extra_data(&fields(
        encode_list(&[encode_bytes(&[9; 20]), encode_bytes(&[1])]),
        encode_u64(0),
        encode_list(&[])
    ))
    .is_err());
    assert_eq!(
        parse_extra_data(&fields(
            encode_list(&[]),
            encode_bytes(&[1, 0, 0, 0, 0]),
            encode_list(&[])
        )),
        Err(QbftError::RoundOverflow)
    );
    assert!(parse_extra_data(&fields(
        encode_list(&[]),
        vec![0x81, 0x01],
        encode_list(&[])
    ))
    .is_err());
    assert!(parse_extra_data(&fields(
        encode_list(&[]),
        encode_u64(0),
        encode_list(&[encode_bytes(&[1; 64])])
    ))
    .is_err());
    let mut zero_scalar = seal();
    zero_scalar[..32].fill(0);
    assert!(parse_extra_data(&fields(
        encode_list(&[]),
        encode_u64(0),
        encode_list(&[encode_bytes(&zero_scalar)])
    ))
    .is_err());

    let mut truncated = encoded_with(validators(1));
    truncated.pop();
    assert!(parse_extra_data(&truncated).is_err());
    let mut trailing = encoded_with(validators(1));
    trailing.push(0);
    assert!(parse_extra_data(&trailing).is_err());
}
