use alloc::vec::Vec;

use crate::RlpError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlpKind {
    Bytes,
    List,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlpItem<'a> {
    kind: RlpKind,
    payload: &'a [u8],
    encoded: &'a [u8],
}

impl<'a> RlpItem<'a> {
    #[must_use]
    pub const fn kind(self) -> RlpKind {
        self.kind
    }

    #[must_use]
    pub const fn encoded(self) -> &'a [u8] {
        self.encoded
    }

    pub fn as_bytes(self) -> Result<&'a [u8], RlpError> {
        if self.kind == RlpKind::Bytes {
            Ok(self.payload)
        } else {
            Err(RlpError::ExpectedBytes)
        }
    }

    pub fn as_list(self) -> Result<Vec<RlpItem<'a>>, RlpError> {
        if self.kind != RlpKind::List {
            return Err(RlpError::ExpectedList);
        }
        let mut result = Vec::new();
        let mut remaining = self.payload;
        while !remaining.is_empty() {
            let (item, consumed) = decode_one(remaining)?;
            result.push(item);
            remaining = remaining.get(consumed..).ok_or(RlpError::Truncated)?;
        }
        Ok(result)
    }

    pub(crate) fn list_cursor(self) -> Result<RlpListCursor<'a>, RlpError> {
        if self.kind != RlpKind::List {
            return Err(RlpError::ExpectedList);
        }
        Ok(RlpListCursor {
            remaining: self.payload,
        })
    }
}

pub(crate) struct RlpListCursor<'a> {
    remaining: &'a [u8],
}

impl<'a> RlpListCursor<'a> {
    pub(crate) fn next(&mut self) -> Result<Option<RlpItem<'a>>, RlpError> {
        if self.remaining.is_empty() {
            return Ok(None);
        }
        let (item, consumed) = decode_one(self.remaining)?;
        self.remaining = self.remaining.get(consumed..).ok_or(RlpError::Truncated)?;
        Ok(Some(item))
    }
}

pub(crate) fn exact_list_fields<const N: usize>(
    item: RlpItem<'_>,
) -> Result<[RlpItem<'_>; N], RlpError> {
    let mut cursor = item.list_cursor()?;
    let mut fields = [item; N];
    let mut actual = 0_usize;
    while let Some(field) = cursor.next()? {
        if actual < N {
            fields[actual] = field;
        }
        actual = actual.checked_add(1).ok_or(RlpError::LengthOverflow)?;
    }
    if actual == N {
        Ok(fields)
    } else {
        Err(RlpError::WrongListLength {
            expected: N,
            actual,
        })
    }
}

pub fn decode_exact(input: &[u8]) -> Result<RlpItem<'_>, RlpError> {
    let (item, consumed) = decode_one(input)?;
    if consumed == input.len() {
        Ok(item)
    } else {
        Err(RlpError::TrailingBytes)
    }
}

pub fn decode_one(input: &[u8]) -> Result<(RlpItem<'_>, usize), RlpError> {
    let first = *input.first().ok_or(RlpError::EmptyInput)?;
    let (kind, payload_offset, payload_len) = match first {
        0x00..=0x7f => (RlpKind::Bytes, 0_usize, 1_usize),
        0x80..=0xb7 => {
            let length = usize::from(first - 0x80);
            if length == 1 {
                let byte = *input.get(1).ok_or(RlpError::Truncated)?;
                if byte < 0x80 {
                    return Err(RlpError::NonCanonical);
                }
            }
            (RlpKind::Bytes, 1, length)
        }
        0xb8..=0xbf => {
            let length_of_length = usize::from(first - 0xb7);
            let length = decode_length(input, 1, length_of_length)?;
            if length < 56 {
                return Err(RlpError::NonCanonical);
            }
            let offset = 1_usize
                .checked_add(length_of_length)
                .ok_or(RlpError::LengthOverflow)?;
            (RlpKind::Bytes, offset, length)
        }
        0xc0..=0xf7 => (RlpKind::List, 1, usize::from(first - 0xc0)),
        0xf8..=0xff => {
            let length_of_length = usize::from(first - 0xf7);
            let length = decode_length(input, 1, length_of_length)?;
            if length < 56 {
                return Err(RlpError::NonCanonical);
            }
            let offset = 1_usize
                .checked_add(length_of_length)
                .ok_or(RlpError::LengthOverflow)?;
            (RlpKind::List, offset, length)
        }
    };

    let total = payload_offset
        .checked_add(payload_len)
        .ok_or(RlpError::LengthOverflow)?;
    let encoded = input.get(..total).ok_or(RlpError::Truncated)?;
    let payload = input
        .get(payload_offset..total)
        .ok_or(RlpError::Truncated)?;
    Ok((
        RlpItem {
            kind,
            payload,
            encoded,
        },
        total,
    ))
}

fn decode_length(input: &[u8], offset: usize, length: usize) -> Result<usize, RlpError> {
    let bytes = input
        .get(offset..offset.checked_add(length).ok_or(RlpError::LengthOverflow)?)
        .ok_or(RlpError::Truncated)?;
    if bytes.first() == Some(&0) {
        return Err(RlpError::NonCanonical);
    }
    let mut value = 0_usize;
    for byte in bytes {
        value = value.checked_mul(256).ok_or(RlpError::LengthOverflow)?;
        value = value
            .checked_add(usize::from(*byte))
            .ok_or(RlpError::LengthOverflow)?;
    }
    Ok(value)
}

pub fn decode_u64(item: RlpItem<'_>) -> Result<u64, RlpError> {
    let bytes = item.as_bytes()?;
    validate_integer(bytes)?;
    if bytes.len() > 8 {
        return Err(RlpError::IntegerOverflow);
    }
    let mut value = 0_u64;
    for byte in bytes {
        value = value
            .checked_mul(256)
            .and_then(|number| number.checked_add(u64::from(*byte)))
            .ok_or(RlpError::IntegerOverflow)?;
    }
    Ok(value)
}

pub fn validate_integer(bytes: &[u8]) -> Result<(), RlpError> {
    if bytes.first() == Some(&0) {
        Err(RlpError::IntegerLeadingZero)
    } else {
        Ok(())
    }
}

#[must_use]
pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(encoded_bytes_len(bytes));
    append_bytes(&mut encoded, bytes);
    encoded
}

#[must_use]
pub fn encode_list(encoded_items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = encoded_items.iter().map(Vec::len).sum();
    let mut encoded = Vec::with_capacity(encoded_list_len(payload_len));
    append_list_header(&mut encoded, payload_len);
    for item in encoded_items {
        encoded.extend_from_slice(item);
    }
    encoded
}

#[must_use]
pub fn encode_u64(value: u64) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(encoded_u64_len(value));
    append_u64(&mut encoded, value);
    encoded
}

#[must_use]
pub fn encode_u256(value: &[u8; 32]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(encoded_u256_len(value));
    append_u256(&mut encoded, value);
    encoded
}

#[must_use]
pub fn encoded_bytes_len(bytes: &[u8]) -> usize {
    if matches!(bytes, [byte] if *byte < 0x80) {
        1
    } else {
        encoded_payload_len(bytes.len())
    }
}

#[must_use]
pub fn encoded_list_len(payload_len: usize) -> usize {
    encoded_payload_len(payload_len)
}

#[must_use]
pub fn encoded_u64_len(value: u64) -> usize {
    if value == 0 {
        return encoded_bytes_len(&[]);
    }
    let bytes = value.to_be_bytes();
    let first = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    encoded_bytes_len(&bytes[first..])
}

#[must_use]
pub fn encoded_u256_len(value: &[u8; 32]) -> usize {
    let first = value
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(value.len());
    encoded_bytes_len(&value[first..])
}

pub fn append_bytes(encoded: &mut Vec<u8>, bytes: &[u8]) {
    if let [byte] = bytes {
        if *byte < 0x80 {
            encoded.push(*byte);
            return;
        }
    }
    append_payload_header(encoded, 0x80, 0xb7, bytes.len());
    encoded.extend_from_slice(bytes);
}

pub fn append_list_header(encoded: &mut Vec<u8>, payload_len: usize) {
    append_payload_header(encoded, 0xc0, 0xf7, payload_len);
}

pub fn append_u64(encoded: &mut Vec<u8>, value: u64) {
    if value == 0 {
        append_bytes(encoded, &[]);
        return;
    }
    let bytes = value.to_be_bytes();
    let first = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    append_bytes(encoded, &bytes[first..]);
}

pub fn append_u256(encoded: &mut Vec<u8>, value: &[u8; 32]) {
    let first = value
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(value.len());
    append_bytes(encoded, &value[first..]);
}

fn encoded_payload_len(payload_len: usize) -> usize {
    if payload_len <= 55 {
        1 + payload_len
    } else {
        1 + minimal_usize_len(payload_len) + payload_len
    }
}

fn append_payload_header(encoded: &mut Vec<u8>, short_base: u8, long_base: u8, payload_len: usize) {
    if payload_len <= 55 {
        let length = payload_len.to_le_bytes()[0];
        encoded.push(short_base + length);
        return;
    }
    let bytes = payload_len.to_be_bytes();
    let first = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    let length_bytes = &bytes[first..];
    let length_of_length = length_bytes.len().to_le_bytes()[0];
    encoded.push(long_base + length_of_length);
    encoded.extend_from_slice(length_bytes);
}

fn minimal_usize_len(value: usize) -> usize {
    let bytes = value.to_be_bytes();
    let first = bytes.iter().position(|byte| *byte != 0).unwrap_or(0);
    bytes.len() - first
}
