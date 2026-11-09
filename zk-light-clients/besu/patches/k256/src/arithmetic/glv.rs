//! Auditable secp256k1 GLV scalar decomposition for the SP1 arithmetic path.
//!
//! The constants and decomposition are the same ones used by k256's native
//! secp256k1 multiplier. Inside SP1, decomposition is only a hint: the guest
//! checks canonical encoding, the 128-bit bounds, and the recombination
//! equation before using either component.

use super::scalar::{Scalar, WideScalar};
#[cfg(any(target_os = "zkvm", test))]
use elliptic_curve::{ff::PrimeField, scalar::IsHigh};

#[cfg(any(target_os = "zkvm", test))]
use crate::FieldBytes;

#[cfg(any(target_os = "zkvm", test))]
const HINT_BYTES: usize = 66;

#[cfg(any(target_os = "zkvm", test))]
const LAMBDA: Scalar = Scalar::from_bytes_unchecked(&[
    0x53, 0x63, 0xad, 0x4c, 0xc0, 0x5c, 0x30, 0xe0, 0xa5, 0x26, 0x1c, 0x02, 0x88, 0x12, 0x64, 0x5a,
    0x12, 0x2e, 0x22, 0xea, 0x20, 0x81, 0x66, 0x78, 0xdf, 0x02, 0x96, 0x7c, 0x1b, 0x23, 0xbd, 0x72,
]);

const MINUS_LAMBDA: Scalar = Scalar::from_bytes_unchecked(&[
    0xac, 0x9c, 0x52, 0xb3, 0x3f, 0xa3, 0xcf, 0x1f, 0x5a, 0xd9, 0xe3, 0xfd, 0x77, 0xed, 0x9b, 0xa4,
    0xa8, 0x80, 0xb9, 0xfc, 0x8e, 0xc7, 0x39, 0xc2, 0xe0, 0xcf, 0xc8, 0x10, 0xb5, 0x12, 0x83, 0xcf,
]);

const MINUS_B1: Scalar = Scalar::from_bytes_unchecked(&[
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xe4, 0x43, 0x7e, 0xd6, 0x01, 0x0e, 0x88, 0x28, 0x6f, 0x54, 0x7f, 0xa9, 0x0a, 0xbf, 0xe4, 0xc3,
]);

const MINUS_B2: Scalar = Scalar::from_bytes_unchecked(&[
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0x8a, 0x28, 0x0a, 0xc5, 0x07, 0x74, 0x34, 0x6d, 0xd7, 0x65, 0xcd, 0xa8, 0x3d, 0xb1, 0x56, 0x2c,
]);

const G1: Scalar = Scalar::from_bytes_unchecked(&[
    0x30, 0x86, 0xd2, 0x21, 0xa7, 0xd4, 0x6b, 0xcd, 0xe8, 0x6c, 0x90, 0xe4, 0x92, 0x84, 0xeb, 0x15,
    0x3d, 0xaa, 0x8a, 0x14, 0x71, 0xe8, 0xca, 0x7f, 0xe8, 0x93, 0x20, 0x9a, 0x45, 0xdb, 0xb0, 0x31,
]);

const G2: Scalar = Scalar::from_bytes_unchecked(&[
    0xe4, 0x43, 0x7e, 0xd6, 0x01, 0x0e, 0x88, 0x28, 0x6f, 0x54, 0x7f, 0xa9, 0x0a, 0xbf, 0xe4, 0xc4,
    0x22, 0x12, 0x08, 0xac, 0x9d, 0xf5, 0x06, 0xc6, 0x15, 0x71, 0xb4, 0xae, 0x8a, 0xc4, 0x7f, 0x71,
]);

#[cfg(any(target_os = "zkvm", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SignedScalar {
    pub(crate) magnitude: Scalar,
    pub(crate) is_negative: bool,
}

#[cfg(any(target_os = "zkvm", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScalarDecomposition {
    pub(crate) first: SignedScalar,
    pub(crate) second: SignedScalar,
}

#[cfg(any(target_os = "zkvm", test))]
impl ScalarDecomposition {
    pub(crate) fn to_hint_bytes(self) -> [u8; HINT_BYTES] {
        let mut bytes = [0_u8; HINT_BYTES];
        bytes[..32].copy_from_slice(&self.first.magnitude.to_bytes());
        bytes[32] = u8::from(self.first.is_negative);
        bytes[33..65].copy_from_slice(&self.second.magnitude.to_bytes());
        bytes[65] = u8::from(self.second.is_negative);
        bytes
    }
}

#[cfg(target_os = "zkvm")]
impl From<ScalarDecomposition> for sp1_lib::ecdsa::ScalarDecomposition<Scalar> {
    fn from(value: ScalarDecomposition) -> Self {
        Self {
            first: sp1_lib::ecdsa::SignedScalar {
                magnitude: value.first.magnitude,
                is_negative: value.first.is_negative,
            },
            second: sp1_lib::ecdsa::SignedScalar {
                magnitude: value.second.magnitude,
                is_negative: value.second.is_negative,
            },
        }
    }
}

/// Split `k` into signed, at-most-128-bit components satisfying
/// `k1 + lambda * k2 == k (mod n)`.
#[cfg(any(target_os = "zkvm", test))]
pub(crate) fn decompose_scalar(k: &Scalar) -> ScalarDecomposition {
    let (r1, r2) = decompose_scalar_raw(k);
    ScalarDecomposition {
        first: normalize_signed(r1),
        second: normalize_signed(r2),
    }
}

/// Canonical k256 GLV split represented as residues modulo the curve order.
pub(crate) fn decompose_scalar_raw(k: &Scalar) -> (Scalar, Scalar) {
    // These variable-time operations depend only on a public verification scalar.
    let c1 = WideScalar::mul_shift_vartime(k, &G1, 384) * MINUS_B1;
    let c2 = WideScalar::mul_shift_vartime(k, &G2, 384) * MINUS_B2;
    let r2 = c1 + c2;
    let r1 = k + r2 * MINUS_LAMBDA;

    (r1, r2)
}

#[cfg(any(target_os = "zkvm", test))]
fn normalize_signed(value: Scalar) -> SignedScalar {
    let is_negative = bool::from(value.is_high());
    SignedScalar {
        magnitude: if is_negative { -value } else { value },
        is_negative,
    }
}

#[cfg(any(target_os = "zkvm", test))]
fn signed_value(value: SignedScalar) -> Scalar {
    if value.is_negative {
        -value.magnitude
    } else {
        value.magnitude
    }
}

#[cfg(any(target_os = "zkvm", test))]
fn is_bounded(value: SignedScalar) -> bool {
    value.magnitude.to_bytes()[..16]
        .iter()
        .all(|byte| *byte == 0)
        && !(value.is_negative && bool::from(value.magnitude.is_zero()))
}

#[cfg(any(target_os = "zkvm", test))]
pub(crate) fn decode_and_validate_hint(
    scalar: &Scalar,
    bytes: &[u8],
) -> Option<ScalarDecomposition> {
    if bytes.len() != HINT_BYTES || bytes[32] > 1 || bytes[65] > 1 {
        return None;
    }

    let mut first_bytes = FieldBytes::default();
    first_bytes.copy_from_slice(&bytes[..32]);
    let mut second_bytes = FieldBytes::default();
    second_bytes.copy_from_slice(&bytes[33..65]);

    let first = SignedScalar {
        magnitude: Option::from(Scalar::from_repr(first_bytes))?,
        is_negative: bytes[32] == 1,
    };
    let second = SignedScalar {
        magnitude: Option::from(Scalar::from_repr(second_bytes))?,
        is_negative: bytes[65] == 1,
    };
    let decomposition = ScalarDecomposition { first, second };

    if !is_bounded(first) || !is_bounded(second) {
        return None;
    }

    let recombined = signed_value(first) + signed_value(second) * LAMBDA;
    (recombined == *scalar).then_some(decomposition)
}

#[cfg(test)]
mod tests {
    use super::{decode_and_validate_hint, decompose_scalar, Scalar, HINT_BYTES};
    use elliptic_curve::ff::PrimeField;

    fn scalar(bytes: [u8; 32]) -> Scalar {
        Option::from(Scalar::from_repr(bytes.into())).expect("test scalar must be canonical")
    }

    #[test]
    fn decomposition_is_bounded_and_recombines_for_boundary_vectors() {
        let vectors = [
            Scalar::ZERO,
            Scalar::ONE,
            scalar([
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xfe, 0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c,
                0xd0, 0x36, 0x41, 0x40,
            ]),
            scalar([
                0x7f, 0x31, 0x9a, 0x6c, 0x41, 0xd4, 0x5e, 0x21, 0xa2, 0x3c, 0x08, 0xb9, 0x77, 0xf1,
                0x3e, 0x55, 0x18, 0x9a, 0xde, 0x03, 0x61, 0x99, 0xa0, 0xc3, 0xa1, 0x2f, 0x54, 0x8d,
                0xc0, 0x83, 0x7b, 0x92,
            ]),
        ];

        for value in vectors {
            let decomposition = decompose_scalar(&value);
            let hint = decomposition.to_hint_bytes();
            assert_eq!(decode_and_validate_hint(&value, &hint), Some(decomposition));
            assert!(hint[..16].iter().all(|byte| *byte == 0));
            assert!(hint[33..49].iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn invalid_hints_are_rejected() {
        let value = scalar([0x11; 32]);
        let valid = decompose_scalar(&value).to_hint_bytes();

        assert!(decode_and_validate_hint(&value, &valid[..HINT_BYTES - 1]).is_none());

        let mut invalid_sign = valid;
        invalid_sign[32] = 2;
        assert!(decode_and_validate_hint(&value, &invalid_sign).is_none());

        let mut oversized = valid;
        oversized[0] = 1;
        assert!(decode_and_validate_hint(&value, &oversized).is_none());

        let mut noncanonical = valid;
        noncanonical[..32].copy_from_slice(&[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xfe, 0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c,
            0xd0, 0x36, 0x41, 0x41,
        ]);
        assert!(decode_and_validate_hint(&value, &noncanonical).is_none());

        let mut wrong_relation = valid;
        wrong_relation[31] ^= 1;
        assert!(decode_and_validate_hint(&value, &wrong_relation).is_none());
    }

    #[test]
    fn negative_zero_hint_is_rejected() {
        let valid = decompose_scalar(&Scalar::ZERO).to_hint_bytes();
        let mut negative_zero = valid;
        negative_zero[32] = 1;
        assert!(decode_and_validate_hint(&Scalar::ZERO, &negative_zero).is_none());
    }
}
