//! A pure-Rust implementation of group operations on secp256k1.

#[cfg(not(target_os = "zkvm"))]
pub(crate) mod affine;
mod field;
mod glv;
#[cfg(all(feature = "hash2curve", not(target_os = "zkvm")))]
mod hash2curve;
#[cfg(not(target_os = "zkvm"))]
mod mul;
#[cfg(not(target_os = "zkvm"))]
pub(crate) mod projective;
pub(crate) mod scalar;

#[cfg(test)]
mod dev;

pub use field::FieldElement;

#[cfg(not(target_os = "zkvm"))]
pub use self::{affine::AffinePoint, projective::ProjectivePoint, scalar::Scalar};

#[cfg(target_os = "zkvm")]
mod zkvm {
    use super::{glv, scalar, FieldElement, Secp256k1};
    use elliptic_curve::{subtle::CtOption, FieldBytes};

    /// SP1 AffinePoint
    pub type AffinePoint = sp1_lib::ecdsa::AffinePoint<Secp256k1>;
    /// SP1 ProjectivePoint
    pub type ProjectivePoint = sp1_lib::ecdsa::ProjectivePoint<Secp256k1>;
    /// SP1 Scalar
    pub type Scalar = scalar::Scalar;

    impl sp1_lib::ecdsa::ECDSACurve for Secp256k1 {
        type FieldElement = FieldElement;
        type SP1AffinePoint = sp1_lib::secp256k1::Secp256k1Point;

        /// a = 0
        const EQUATION_A: FieldElement = FieldElement::from_bytes_unchecked(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);

        const EQUATION_B: FieldElement = super::CURVE_EQUATION_B;

        fn decompose_scalar(
            scalar: &Scalar,
        ) -> Option<sp1_lib::ecdsa::ScalarDecomposition<Scalar>> {
            let scalar_bytes = scalar.to_bytes();
            sp1_lib::unconstrained! {
                use elliptic_curve::ff::PrimeField;

                let scalar = Scalar::from_repr(scalar_bytes)
                    .expect("the original scalar is canonical");
                let decomposition = glv::decompose_scalar(&scalar);
                sp1_lib::io::hint_slice(&decomposition.to_hint_bytes());
            }

            Some(
                glv::decode_and_validate_hint(scalar, &sp1_lib::io::read_vec())
                    .expect("invalid secp256k1 GLV decomposition hint")
                    .into(),
            )
        }
    }

    impl sp1_lib::ecdsa::Field<Secp256k1> for FieldElement {
        fn from_bytes(bytes: &FieldBytes<Secp256k1>) -> CtOption<Self> {
            // Only parses canonical form
            Self::from_bytes(bytes)
        }

        fn to_bytes(self) -> FieldBytes<Secp256k1> {
            // internally calls `normalize`
            FieldElement::to_bytes(self)
        }

        fn normalize(self) -> Self {
            FieldElement::normalize(&self)
        }
    }
}

#[cfg(target_os = "zkvm")]
pub use zkvm::{AffinePoint, ProjectivePoint, Scalar};

use crate::Secp256k1;

use elliptic_curve::CurveArithmetic;

impl CurveArithmetic for Secp256k1 {
    type AffinePoint = AffinePoint;
    type ProjectivePoint = ProjectivePoint;
    type Scalar = Scalar;
}

const CURVE_EQUATION_B_SINGLE: u32 = 7u32;

#[rustfmt::skip]
pub(crate) const CURVE_EQUATION_B: FieldElement = FieldElement::from_bytes_unchecked(&[
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, CURVE_EQUATION_B_SINGLE as u8,
]);

#[cfg(test)]
mod tests {
    use super::CURVE_EQUATION_B;
    use hex_literal::hex;

    const CURVE_EQUATION_B_BYTES: [u8; 32] =
        hex!("0000000000000000000000000000000000000000000000000000000000000007");

    #[test]
    fn verify_constants() {
        assert_eq!(CURVE_EQUATION_B.to_bytes(), CURVE_EQUATION_B_BYTES.into());
    }

    #[test]
    fn generate_secret_key() {
        use crate::SecretKey;
        use elliptic_curve::rand_core::OsRng;
        let key = SecretKey::random(&mut OsRng);

        // Sanity check
        assert!(!key.to_bytes().iter().all(|b| *b == 0))
    }
}
