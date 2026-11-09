use besu_zk_types::{keccak256, QbftHeaderInput};

use crate::{
    encode_extra_data, header::encode_header_validated, parse_extra_data, validate_optional_tail,
    QbftError, QbftExtraData,
};

pub fn committed_seal_digest(header: &QbftHeaderInput) -> Result<[u8; 32], QbftError> {
    validate_optional_tail(header)?;
    let extra = parse_extra_data(&header.extra_data)?;
    committed_seal_digest_validated(header, &extra)
}

pub(crate) fn committed_seal_digest_validated(
    header: &QbftHeaderInput,
    extra: &QbftExtraData,
) -> Result<[u8; 32], QbftError> {
    let signing_extra = encode_extra_data(extra, false, true);
    let header_rlp = encode_header_validated(header, &signing_extra)?;
    Ok(keccak256(&header_rlp))
}
