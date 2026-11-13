use evm_mpt::{
    rlp::{
        append_bytes, append_list_header, append_u256, append_u64, decode_exact, decode_u64,
        encode_bytes, encode_list, encode_u64, encoded_bytes_len, encoded_list_len,
        encoded_u256_len, encoded_u64_len,
    },
    RlpError,
};

#[test]
fn canonical_boundaries_roundtrip() -> Result<(), RlpError> {
    for bytes in [
        vec![],
        vec![0x00],
        vec![0x7f],
        vec![0x80],
        vec![0xaa; 55],
        vec![0xbb; 56],
    ] {
        assert_eq!(decode_exact(&encode_bytes(&bytes))?.as_bytes()?, bytes);
    }
    for values in [0, 1, 0x7f, 0x80, u64::MAX] {
        assert_eq!(decode_u64(decode_exact(&encode_u64(values))?)?, values);
    }
    let short = encode_list(&[encode_bytes(b"a"), encode_bytes(b"b")]);
    assert_eq!(decode_exact(&short)?.as_list()?.len(), 2);
    assert!(decode_exact(&encode_list(&[]))?.as_list()?.is_empty());
    let payload_55 = encode_list(&[encode_bytes(&[0xaa; 54])]);
    assert_eq!(decode_exact(&payload_55)?.as_list()?.len(), 1);
    let long = encode_list(&[encode_bytes(&[0xaa; 55])]);
    assert_eq!(decode_exact(&long)?.as_list()?.len(), 1);
    let nested = encode_list(&[encode_list(&[encode_bytes(b"nested")])]);
    assert_eq!(
        decode_exact(&nested)?.as_list()?[0].as_list()?[0].as_bytes()?,
        b"nested"
    );
    Ok(())
}

#[test]
fn malformed_and_noncanonical_are_rejected() {
    let cases: &[&[u8]] = &[
        &[],
        &[0x81, 0x01],
        &[0xb8, 0x01, 0x80],
        &[0xb9, 0x00, 0x38],
        &[0xb8, 0x38],
        &[0xf8, 0x01, 0xc0],
        &[0xb8, 0x38, 0x00],
    ];
    for case in cases {
        assert!(decode_exact(case).is_err(), "accepted {case:02x?}");
    }
    assert_eq!(decode_exact(&[0x80, 0x80]), Err(RlpError::TrailingBytes));
}

#[test]
fn integer_leading_zero_and_overflow_are_rejected() -> Result<(), RlpError> {
    assert_eq!(
        decode_u64(decode_exact(&[0x00])?),
        Err(RlpError::IntegerLeadingZero)
    );
    assert_eq!(
        decode_u64(decode_exact(&encode_bytes(&[1; 9]))?),
        Err(RlpError::IntegerOverflow)
    );
    Ok(())
}

#[test]
fn pre_sized_append_encoders_match_canonical_rlp_boundaries() {
    for bytes in [
        vec![],
        vec![0x00],
        vec![0x7f],
        vec![0x80],
        vec![0xaa; 55],
        vec![0xbb; 56],
    ] {
        let expected = encode_bytes(&bytes);
        let mut appended = Vec::with_capacity(encoded_bytes_len(&bytes));
        append_bytes(&mut appended, &bytes);
        assert_eq!(appended, expected);
        assert_eq!(appended.len(), encoded_bytes_len(&bytes));
    }

    for value in [0, 1, 0x7f, 0x80, u64::MAX] {
        let expected = encode_u64(value);
        let mut appended = Vec::with_capacity(encoded_u64_len(value));
        append_u64(&mut appended, value);
        assert_eq!(appended, expected);
        assert_eq!(appended.len(), encoded_u64_len(value));
    }

    for value in [[0_u8; 32], [0x01; 32], [0xff; 32]] {
        let mut appended = Vec::with_capacity(encoded_u256_len(&value));
        append_u256(&mut appended, &value);
        assert_eq!(appended.len(), encoded_u256_len(&value));
    }

    for payload_len in [0, 1, 55, 56, 255, 256] {
        let mut encoded = Vec::with_capacity(encoded_list_len(payload_len));
        append_list_header(&mut encoded, payload_len);
        encoded.resize(encoded_list_len(payload_len), 0);
        assert_eq!(encoded.len(), encoded_list_len(payload_len));
    }
}
