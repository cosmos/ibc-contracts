use alloc::vec::Vec;

use crate::MptError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactPath {
    pub is_leaf: bool,
    pub nibbles: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CompactPathRef<'a> {
    encoded: &'a [u8],
    nibble_len: usize,
    odd: bool,
    leaf: bool,
}

impl CompactPathRef<'_> {
    pub(crate) const fn is_leaf(self) -> bool {
        self.leaf
    }

    pub(crate) const fn len(self) -> usize {
        self.nibble_len
    }

    pub(crate) fn matches(self, expected: &[u8]) -> bool {
        if expected.len() != self.nibble_len {
            return false;
        }
        expected
            .iter()
            .enumerate()
            .all(|(index, expected_nibble)| self.nibble(index) == Some(*expected_nibble))
    }

    fn nibble(self, index: usize) -> Option<u8> {
        if index >= self.nibble_len {
            return None;
        }
        if self.odd {
            if index == 0 {
                return self.encoded.first().map(|byte| byte & 0x0f);
            }
            Some(nibble_from_byte(
                self.encoded.get(1 + (index - 1) / 2).copied()?,
                index - 1,
            ))
        } else {
            Some(nibble_from_byte(
                self.encoded.get(1 + index / 2).copied()?,
                index,
            ))
        }
    }
}

fn nibble_from_byte(byte: u8, index: usize) -> u8 {
    if index % 2 == 0 {
        byte >> 4
    } else {
        byte & 0x0f
    }
}

pub(crate) fn decode_compact_path_ref(encoded: &[u8]) -> Result<CompactPathRef<'_>, MptError> {
    let first = *encoded.first().ok_or(MptError::InvalidCompactPath)?;
    let flag = first >> 4;
    if flag > 3 {
        return Err(MptError::InvalidCompactPath);
    }
    let odd = flag & 1 == 1;
    let leaf = flag & 2 == 2;
    if !odd && first & 0x0f != 0 {
        return Err(MptError::InvalidCompactPath);
    }
    let rest_nibbles = encoded
        .len()
        .checked_sub(1)
        .and_then(|length| length.checked_mul(2))
        .ok_or(MptError::CompactPathTooLong)?;
    let nibble_len = rest_nibbles
        .checked_add(usize::from(odd))
        .ok_or(MptError::CompactPathTooLong)?;
    if nibble_len > 64 {
        return Err(MptError::CompactPathTooLong);
    }
    if !leaf && nibble_len == 0 {
        return Err(MptError::EmptyExtensionPath);
    }
    Ok(CompactPathRef {
        encoded,
        nibble_len,
        odd,
        leaf,
    })
}

pub fn decode_compact_path(encoded: &[u8]) -> Result<CompactPath, MptError> {
    let compact = decode_compact_path_ref(encoded)?;
    let mut nibbles = Vec::with_capacity(compact.len());
    for index in 0..compact.len() {
        nibbles.push(compact.nibble(index).ok_or(MptError::InvalidCompactPath)?);
    }
    Ok(CompactPath {
        is_leaf: compact.is_leaf(),
        nibbles,
    })
}
