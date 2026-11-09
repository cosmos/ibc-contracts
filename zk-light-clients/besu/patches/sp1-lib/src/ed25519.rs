use crate::{syscall_ed_add, utils::AffinePoint};

/// The number of limbs in [Ed25519AffinePoint].
pub const N: usize = 8;

/// An affine point on the Ed25519 curve.
#[derive(Copy, Clone)]
#[repr(align(8))]
pub struct Ed25519AffinePoint(pub [u64; N]);

impl AffinePoint<N> for Ed25519AffinePoint {
    /// The generator/base point for the Ed25519 curve. Reference: https://datatracker.ietf.org/doc/html/rfc7748#section-4.1
    const GENERATOR: [u64; N] = [
        13254768563189591678,
        7223677240904510747,
        11837459681205989215,
        14107110925517789205,
        3231187496542550688,
        8386596743812984063,
        16293584715996958308,
        12755452578091664582,
    ];

    #[allow(deprecated)]
    const GENERATOR_T: Self = Self(Self::GENERATOR);

    fn new(limbs: [u64; N]) -> Self {
        Self(limbs)
    }

    fn identity() -> Self {
        Self::identity()
    }

    fn limbs_ref(&self) -> &[u64; N] {
        &self.0
    }

    fn limbs_mut(&mut self) -> &mut [u64; N] {
        &mut self.0
    }

    fn add_assign(&mut self, other: &Self) {
        let a = self.limbs_mut();
        let b = other.limbs_ref();
        unsafe {
            syscall_ed_add(a, b);
        }
    }

    fn is_identity(&self) -> bool {
        self.0 == Self::IDENTITY
    }

    /// In Edwards curves, doubling is the same as adding a point to itself.
    fn double(&mut self) {
        let a = self.limbs_mut();
        unsafe {
            syscall_ed_add(a, a);
        }
    }
}

impl Ed25519AffinePoint {
    const IDENTITY: [u64; N] = [0, 0, 0, 0, 1, 0, 0, 0];

    pub fn identity() -> Self {
        Self(Self::IDENTITY)
    }
}
