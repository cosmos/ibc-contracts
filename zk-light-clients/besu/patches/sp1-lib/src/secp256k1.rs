use crate::{
    syscall_secp256k1_add, syscall_secp256k1_double,
    utils::{AffinePoint, WeierstrassAffinePoint, WeierstrassPoint},
};

/// The number of limbs in [Secp256k1Point].
pub const N: usize = 8;

/// An affine point on the Secp256k1 curve.
#[derive(Copy, Clone, Debug)]
#[repr(align(8))]
pub struct Secp256k1Point(pub WeierstrassPoint<N>);

impl WeierstrassAffinePoint<N> for Secp256k1Point {
    fn infinity() -> Self {
        Self(WeierstrassPoint::Infinity)
    }

    fn is_infinity(&self) -> bool {
        matches!(self.0, WeierstrassPoint::Infinity)
    }
}

impl AffinePoint<N> for Secp256k1Point {
    /// The values are taken from https://en.bitcoin.it/wiki/Secp256k1.
    const GENERATOR: [u64; N] = [
        6481385041966929816,
        188021827762530521,
        6170039885052185351,
        8772561819708210092,
        11261198710074299576,
        18237243440184513561,
        6747795201694173352,
        5204712524664259685,
    ];

    #[allow(deprecated)]
    const GENERATOR_T: Self = Self(WeierstrassPoint::Affine(Self::GENERATOR));

    fn new(limbs: [u64; N]) -> Self {
        Self(WeierstrassPoint::Affine(limbs))
    }

    fn identity() -> Self {
        Self::infinity()
    }

    fn is_identity(&self) -> bool {
        self.is_infinity()
    }

    fn limbs_ref(&self) -> &[u64; N] {
        match &self.0 {
            WeierstrassPoint::Infinity => panic!("Infinity point has no limbs"),
            WeierstrassPoint::Affine(limbs) => limbs,
        }
    }

    fn limbs_mut(&mut self) -> &mut [u64; N] {
        match &mut self.0 {
            WeierstrassPoint::Infinity => panic!("Infinity point has no limbs"),
            WeierstrassPoint::Affine(limbs) => limbs,
        }
    }

    fn add_assign(&mut self, other: &Self) {
        let a = self.limbs_mut();
        let b = other.limbs_ref();
        unsafe {
            syscall_secp256k1_add(a, b);
        }
    }

    fn complete_add_assign(&mut self, other: &Self) {
        self.weierstrass_add_assign(other);
    }

    fn endomorphism(&self) -> Option<Self> {
        if self.is_identity() {
            return Some(Self::identity());
        }

        #[cfg(target_os = "zkvm")]
        {
            // phi(x, y) = (beta * x mod p, y), with beta^3 = 1 mod p.
            const BETA: [u64; 4] = [
                0xc139_6c28_7195_01ee,
                0x9cf0_4975_12f5_8995,
                0x6e64_479e_ac34_34e9,
                0x7ae9_6a2b_657c_0710,
            ];
            const MODULUS: [u64; 4] = [
                0xffff_fffe_ffff_fc2f,
                0xffff_ffff_ffff_ffff,
                0xffff_ffff_ffff_ffff,
                0xffff_ffff_ffff_ffff,
            ];

            let mut result = *self;
            let mut x = [0_u64; 4];
            x.copy_from_slice(&self.limbs_ref()[..4]);
            let mut beta_x = [0_u64; 4];
            unsafe {
                crate::sys_bigint(&mut beta_x, 0, &x, &BETA, &MODULUS);
            }
            result.limbs_mut()[..4].copy_from_slice(&beta_x);
            Some(result)
        }

        #[cfg(not(target_os = "zkvm"))]
        {
            None
        }
    }

    fn double(&mut self) {
        match &mut self.0 {
            WeierstrassPoint::Infinity => (),
            WeierstrassPoint::Affine(limbs) => unsafe {
                syscall_secp256k1_double(limbs);
            },
        }
    }
}
