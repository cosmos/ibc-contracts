use alloc::vec::Vec;

use besu_zk_types::{Address, QBFT_VANITY_BYTES};
use evm_mpt::rlp::{
    append_bytes, append_list_header, append_u64, decode_exact, decode_u64, encoded_bytes_len,
    encoded_list_len, encoded_u64_len, RlpKind,
};

use crate::{signature::ValidatedSeal, validate_validator_set, QbftError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoteKind {
    Add,
    Drop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QbftVote {
    pub recipient: Address,
    pub kind: VoteKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QbftExtraData {
    pub vanity: [u8; QBFT_VANITY_BYTES],
    pub validators: Vec<Address>,
    pub vote: Option<QbftVote>,
    pub round: u32,
    pub committed_seals: Vec<Vec<u8>>,
}

/// Parsed fields used for signing, with seals kept separately in validated form.
/// The signing fields deliberately omit seals: they are excluded from the digest.
pub(crate) struct ValidatedExtraData<'a> {
    pub(crate) signing_fields: QbftExtraData,
    pub(crate) committed_seals: Vec<ValidatedSeal<'a>>,
}

pub fn parse_extra_data(encoded: &[u8]) -> Result<QbftExtraData, QbftError> {
    let parsed = parse_validated_extra_data(encoded)?;
    let mut extra = parsed.signing_fields;
    extra.committed_seals = parsed
        .committed_seals
        .iter()
        .map(|seal| seal.as_bytes().to_vec())
        .collect();
    Ok(extra)
}

pub(crate) fn parse_validated_extra_data(
    encoded: &[u8],
) -> Result<ValidatedExtraData<'_>, QbftError> {
    let root = decode_exact(encoded)?;
    let fields = root.as_list()?;
    if fields.len() != 5 {
        return Err(QbftError::InvalidExtraDataShape);
    }
    let [vanity_item, validators_item, vote_item, round_item, seals_item] = fields
        .try_into()
        .map_err(|_| QbftError::InvalidExtraDataShape)?;
    let vanity_bytes = vanity_item.as_bytes()?;
    let vanity = vanity_bytes
        .try_into()
        .map_err(|_| QbftError::InvalidVanityLength(vanity_bytes.len()))?;

    let validator_items = validators_item.as_list()?;
    let mut validators = Vec::with_capacity(validator_items.len());
    for item in validator_items {
        let bytes = item.as_bytes()?;
        let address = bytes
            .try_into()
            .map_err(|_| QbftError::InvalidValidatorLength(bytes.len()))?;
        validators.push(address);
    }
    validate_validator_set(&validators)?;
    let vote = parse_vote(vote_item)?;
    let round_value = decode_u64(round_item)?;
    let round = u32::try_from(round_value).map_err(|_| QbftError::RoundOverflow)?;

    let seal_items = seals_item.as_list()?;
    let mut committed_seals = Vec::with_capacity(seal_items.len());
    for item in seal_items {
        let seal = item.as_bytes()?;
        committed_seals.push(ValidatedSeal::parse(seal)?);
    }
    Ok(ValidatedExtraData {
        signing_fields: QbftExtraData {
            vanity,
            validators,
            vote,
            round,
            committed_seals: Vec::new(),
        },
        committed_seals,
    })
}

fn parse_vote(item: evm_mpt::rlp::RlpItem<'_>) -> Result<Option<QbftVote>, QbftError> {
    if item.kind() != RlpKind::List {
        return Err(QbftError::InvalidVote);
    }
    let fields = item.as_list()?;
    if fields.is_empty() {
        return Ok(None);
    }
    if fields.len() != 2 {
        return Err(QbftError::InvalidVote);
    }
    let [recipient_item, kind_item] = fields.try_into().map_err(|_| QbftError::InvalidVote)?;
    let recipient_bytes = recipient_item.as_bytes()?;
    let recipient = recipient_bytes
        .try_into()
        .map_err(|_| QbftError::InvalidVote)?;
    if recipient_bytes.iter().all(|byte| *byte == 0) {
        return Err(QbftError::InvalidVote);
    }
    let kind = match kind_item.as_bytes()? {
        [] => VoteKind::Drop,
        [0xff] => VoteKind::Add,
        _ => return Err(QbftError::InvalidVote),
    };
    Ok(Some(QbftVote { recipient, kind }))
}

#[must_use]
pub fn encode_extra_data(
    extra: &QbftExtraData,
    include_seals: bool,
    preserve_round: bool,
) -> Vec<u8> {
    let validators_payload_len = extra
        .validators
        .iter()
        .map(|address| encoded_bytes_len(address))
        .sum::<usize>();
    let validators_len = encoded_list_len(validators_payload_len);
    let vote_payload_len = match &extra.vote {
        None => 0,
        Some(vote) => {
            encoded_bytes_len(&vote.recipient)
                + match vote.kind {
                    VoteKind::Add => encoded_bytes_len(&[0xff]),
                    VoteKind::Drop => encoded_bytes_len(&[]),
                }
        }
    };
    let vote_len = encoded_list_len(vote_payload_len);
    let round = if preserve_round {
        u64::from(extra.round)
    } else {
        0
    };
    let seals_payload_len = if include_seals {
        extra
            .committed_seals
            .iter()
            .map(|seal| encoded_bytes_len(seal))
            .sum::<usize>()
    } else {
        0
    };
    let seals_len = encoded_list_len(seals_payload_len);
    let payload_len = encoded_bytes_len(&extra.vanity)
        + validators_len
        + vote_len
        + encoded_u64_len(round)
        + seals_len;
    let encoded_len = encoded_list_len(payload_len);
    let mut encoded = Vec::with_capacity(encoded_len);
    append_list_header(&mut encoded, payload_len);
    append_bytes(&mut encoded, &extra.vanity);
    append_list_header(&mut encoded, validators_payload_len);
    for validator in &extra.validators {
        append_bytes(&mut encoded, validator);
    }
    append_list_header(&mut encoded, vote_payload_len);
    if let Some(vote) = &extra.vote {
        append_bytes(&mut encoded, &vote.recipient);
        match vote.kind {
            VoteKind::Add => append_bytes(&mut encoded, &[0xff]),
            VoteKind::Drop => append_bytes(&mut encoded, &[]),
        }
    }
    append_u64(&mut encoded, round);
    append_list_header(&mut encoded, seals_payload_len);
    if include_seals {
        for seal in &extra.committed_seals {
            append_bytes(&mut encoded, seal);
        }
    }
    debug_assert_eq!(encoded.len(), encoded_len);
    encoded
}
