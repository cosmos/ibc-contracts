use std::{error::Error, io};

use besu_qbft::{
    committed_seal_digest, encode_extra_data, parse_extra_data, recover_address, QbftExtraData,
};
use besu_zk_types::QbftHeaderInput;
use k256::ecdsa::SigningKey;

type SyntheticSignedHeader = (QbftHeaderInput, Vec<u8>, [u8; 20]);

fn fixed<const N: usize>(value: &str) -> Result<[u8; N], Box<dyn Error>> {
    let bytes = hex::decode(value)?;
    bytes
        .try_into()
        .map_err(|_| io::Error::other(format!("expected {N} bytes")).into())
}

fn real_header() -> Result<QbftHeaderInput, Box<dyn Error>> {
    Ok(QbftHeaderInput {
        parent_hash: fixed("2a647d2db079ee05f5d352e170666f025310591aeba17ca3af1ed77c234d395f")?,
        ommers_hash: fixed("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347")?,
        beneficiary: fixed("c1274b8f2edc38f30b28c324b279367845ba9d15")?,
        state_root: fixed("d26a2aa6b56e0012de6218d8f3e4d1831ca6ba8c06d11f7b2eb990b9d9e0615c")?,
        transactions_root: fixed("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")?,
        receipts_root: fixed("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")?,
        logs_bloom: [0; 256],
        difficulty: fixed("0000000000000000000000000000000000000000000000000000000000000001")?,
        number: 8,
        gas_limit: 0x1f_ffff_ffff_ffff,
        gas_used: 0,
        timestamp: 0x6a99_5aab,
        extra_data: hex::decode("f90144a00000000000000000000000000000000000626573752032362e352e302d524332f85494be182dd900c604d029049e55b0139b0b0cc8ca4d94c1274b8f2edc38f30b28c324b279367845ba9d1594c81fe72045c922d5f1398fea20b42035dfa2302c94d10b20cbeeccfb414a5a692f1fcc4777da3a7131c080f8c9b8418a88f583d91db6bb5c4542866ef9a93e24aaa6e42a7cc8e2a87576c218c8ff654cc54d67f303a380ce60cff9daeb190a9f775128a9a16ee7e3cfb44d1147f59c01b841d8496d5fa3b5b5c59aee2ec95741d36c8f1548f38fdccabd22e3b5e95475efce0ea25d4e1bc2a577a00c3adef14cfdc1f520f043e8188e9281cf5504eaf0b87200b841b347fae32de308cf84dde0090d09ee42b4f8e8e2aad3ab8081623084ac178dba02798c2f44e04d9de41bbdaa45166c35a20ec8abb787e73b66ec728d7c38830900")?,
        mix_hash: fixed("63746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365")?,
        nonce: [0; 8],
        base_fee_per_gas: Some([0; 32]),
        withdrawals_root: None,
        blob_gas_used: None,
        excess_blob_gas: None,
        parent_beacon_block_root: None,
        requests_hash: None,
        block_access_list_hash: None,
    })
}

fn signing_key_address(key: &SigningKey) -> Result<[u8; 20], Box<dyn Error>> {
    let point = key.verifying_key().to_encoded_point(false);
    let coordinates = point
        .as_bytes()
        .get(1..)
        .ok_or_else(|| io::Error::other("missing public-key coordinates"))?;
    let hash = besu_zk_types::keccak256(coordinates);
    Ok(hash
        .get(12..)
        .ok_or_else(|| io::Error::other("invalid address hash"))?
        .try_into()?)
}

fn synthetic_signed_header() -> Result<SyntheticSignedHeader, Box<dyn Error>> {
    let key = SigningKey::from_slice(&[0x42; 32])
        .map_err(|_| io::Error::other("invalid deterministic signing key"))?;
    let signer = signing_key_address(&key)?;
    let mut header = QbftHeaderInput {
        parent_hash: [1; 32],
        ommers_hash: [2; 32],
        beneficiary: [3; 20],
        state_root: [4; 32],
        transactions_root: [5; 32],
        receipts_root: [6; 32],
        logs_bloom: [7; 256],
        difficulty: [8; 32],
        number: 9,
        gas_limit: 10,
        gas_used: 11,
        timestamp: 12,
        extra_data: Vec::new(),
        mix_hash: [13; 32],
        nonce: [14; 8],
        base_fee_per_gas: Some([15; 32]),
        withdrawals_root: Some([16; 32]),
        blob_gas_used: Some(17),
        excess_blob_gas: Some(18),
        parent_beacon_block_root: Some([19; 32]),
        requests_hash: Some([20; 32]),
        block_access_list_hash: Some([21; 32]),
    };
    let mut extra = QbftExtraData {
        vanity: [22; 32],
        validators: vec![signer],
        vote: None,
        round: 23,
        committed_seals: Vec::new(),
    };
    header.extra_data = encode_extra_data(&extra, true, true);
    let digest = committed_seal_digest(&header)?;
    let (signature, recovery_id) = key
        .sign_prehash_recoverable(&digest)
        .map_err(|_| io::Error::other("test signing failed"))?;
    let mut seal = signature.to_bytes().to_vec();
    seal.push(recovery_id.to_byte());
    extra.committed_seals.push(seal.clone());
    header.extra_data = encode_extra_data(&extra, true, true);
    Ok((header, seal, signer))
}

fn assert_header_mutation<F>(
    header: &QbftHeaderInput,
    seal: &[u8],
    signer: [u8; 20],
    mutate: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnOnce(&mut QbftHeaderInput) -> Result<(), Box<dyn Error>>,
{
    let original = committed_seal_digest(header)?;
    assert_eq!(recover_address(original, seal), Ok(signer));
    let mut changed = header.clone();
    mutate(&mut changed)?;
    let changed_digest = committed_seal_digest(&changed)?;
    assert_ne!(changed_digest, original);
    assert_ne!(recover_address(changed_digest, seal), Ok(signer));
    Ok(())
}

#[test]
fn matches_controlled_besu_26_5_0_commit_seal_digest() -> Result<(), Box<dyn Error>> {
    let header = real_header()?;
    let extra = parse_extra_data(&header.extra_data)?;
    assert_eq!(extra.round, 0);
    assert_eq!(extra.validators.len(), 4);
    assert_eq!(extra.committed_seals.len(), 3);

    let digest = committed_seal_digest(&header)?;
    assert_eq!(
        digest,
        fixed("1a9844e08dff8da0cc250ae87c7b85af700e6846ce34146b73ef2adada38dabb")?
    );
    let recovered = extra
        .committed_seals
        .iter()
        .map(|seal| recover_address(digest, seal))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(recovered
        .iter()
        .all(|signer| extra.validators.contains(signer)));

    let mut seal_mutation = header.clone();
    let mut mutated_extra = extra.clone();
    let first_seal = mutated_extra
        .committed_seals
        .first_mut()
        .ok_or_else(|| io::Error::other("missing commit seal"))?;
    let first_byte = first_seal
        .first_mut()
        .ok_or_else(|| io::Error::other("empty commit seal"))?;
    *first_byte ^= 1;
    seal_mutation.extra_data = encode_extra_data(&mutated_extra, true, true);
    assert_eq!(committed_seal_digest(&seal_mutation)?, digest);

    let mut reference_hash_preimage = header;
    let mut zero_round: QbftExtraData = parse_extra_data(&reference_hash_preimage.extra_data)?;
    zero_round.round = 1;
    reference_hash_preimage.extra_data = encode_extra_data(&zero_round, true, true);
    assert_ne!(committed_seal_digest(&reference_hash_preimage)?, digest);
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)] // Deliberate exhaustive field-by-field binding matrix.
fn every_supported_header_field_is_bound_by_the_original_signature() -> Result<(), Box<dyn Error>> {
    let (header, seal, signer) = synthetic_signed_header()?;
    macro_rules! mutate {
        ($body:expr) => {
            assert_header_mutation(&header, &seal, signer, $body)?
        };
    }
    mutate!(|h| {
        h.parent_hash[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.ommers_hash[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.beneficiary[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.state_root[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.transactions_root[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.receipts_root[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.logs_bloom[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.difficulty[31] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.number += 1;
        Ok(())
    });
    mutate!(|h| {
        h.gas_limit += 1;
        Ok(())
    });
    mutate!(|h| {
        h.gas_used += 1;
        Ok(())
    });
    mutate!(|h| {
        h.timestamp += 1;
        Ok(())
    });
    mutate!(|h| {
        h.mix_hash[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.nonce[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.base_fee_per_gas
            .as_mut()
            .ok_or_else(|| io::Error::other("base fee"))?[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.withdrawals_root
            .as_mut()
            .ok_or_else(|| io::Error::other("withdrawals root"))?[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.blob_gas_used = h.blob_gas_used.and_then(|value| value.checked_add(1));
        Ok(())
    });
    mutate!(|h| {
        h.excess_blob_gas = h.excess_blob_gas.and_then(|value| value.checked_add(1));
        Ok(())
    });
    mutate!(|h| {
        h.parent_beacon_block_root
            .as_mut()
            .ok_or_else(|| io::Error::other("beacon root"))?[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.requests_hash
            .as_mut()
            .ok_or_else(|| io::Error::other("requests hash"))?[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        h.block_access_list_hash
            .as_mut()
            .ok_or_else(|| io::Error::other("access-list hash"))?[0] ^= 1;
        Ok(())
    });
    mutate!(|h| {
        let mut extra = parse_extra_data(&h.extra_data)?;
        extra.vanity[0] ^= 1;
        h.extra_data = encode_extra_data(&extra, true, true);
        Ok(())
    });
    mutate!(|h| {
        let mut extra = parse_extra_data(&h.extra_data)?;
        extra.validators[0][0] ^= 1;
        h.extra_data = encode_extra_data(&extra, true, true);
        Ok(())
    });
    mutate!(|h| {
        let mut extra = parse_extra_data(&h.extra_data)?;
        extra.vote = Some(besu_qbft::QbftVote {
            recipient: [0x77; 20],
            kind: besu_qbft::VoteKind::Add,
        });
        h.extra_data = encode_extra_data(&extra, true, true);
        Ok(())
    });
    mutate!(|h| {
        let mut extra = parse_extra_data(&h.extra_data)?;
        extra.round += 1;
        h.extra_data = encode_extra_data(&extra, true, true);
        Ok(())
    });
    Ok(())
}
