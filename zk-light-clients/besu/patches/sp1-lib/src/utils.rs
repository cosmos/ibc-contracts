pub trait AffinePoint<const N: usize>: Clone + Sized {
    /// The generator.
    #[deprecated = "This const will have the `Self` type in the next major version."]
    const GENERATOR: [u64; N];

    const GENERATOR_T: Self;

    /// Creates a new [`AffinePoint`] from the given limbs.
    /// This function does not check that the inputs form a valid point. Only use this function
    /// when the input is either trusted or previously validated.
    fn new(limbs: [u64; N]) -> Self;

    /// Creates a new [`AffinePoint`] that corresponds to the identity point.
    fn identity() -> Self;

    /// Returns a reference to the limbs.
    fn limbs_ref(&self) -> &[u64; N];

    /// Returns a mutable reference to the limbs. If the point is the infinity point, this will
    /// panic.
    fn limbs_mut(&mut self) -> &mut [u64; N];

    fn is_identity(&self) -> bool;

    /// Creates a new [`AffinePoint`] from the given x and y coordinates.
    ///
    /// The bytes are the concatenated little endian representations of the coordinates.
    fn from(x: &[u8], y: &[u8]) -> Self {
        debug_assert!(x.len() == N * 4);
        debug_assert!(y.len() == N * 4);

        let mut limbs = [0u64; N];
        let x = bytes_to_words_le(x);
        let y = bytes_to_words_le(y);

        debug_assert!(x.len() == N / 2);
        debug_assert!(y.len() == N / 2);

        limbs[..(N / 2)].copy_from_slice(&x);
        limbs[(N / 2)..].copy_from_slice(&y);
        Self::new(limbs)
    }

    /// Creates a new [`AffinePoint`] from the given bytes in little endian.
    /// This function does not check that the inputs form a valid point. Only use this function
    /// when the input is either trusted or previously validated.
    fn from_le_bytes(bytes: &[u8]) -> Self {
        let limbs = bytes_to_words_le(bytes);
        debug_assert!(limbs.len() == N);
        Self::new(limbs.try_into().unwrap())
    }

    /// Creates a new [`AffinePoint`] from the given bytes in big endian.
    fn to_le_bytes(&self) -> Vec<u8> {
        let le_bytes = words_to_bytes_le(self.limbs_ref());
        debug_assert!(le_bytes.len() == N * 8);
        le_bytes
    }

    /// Adds the given [`AffinePoint`] to `self`.
    fn add_assign(&mut self, other: &Self);

    /// Adds the given [`AffinePoint`] to `self`. Can be optionally overridden to use a different
    /// implementation of addition in multi-scalar multiplication, which is used in secp256k1
    /// recovery.
    fn complete_add_assign(&mut self, other: &Self) {
        self.add_assign(other);
    }

    /// Apply a curve endomorphism when one is available.
    fn endomorphism(&self) -> Option<Self> {
        None
    }

    /// Doubles `self`.
    fn double(&mut self);

    /// Multiplies `self` by the given scalar.
    fn mul_assign(&mut self, scalar: &[u64]) {
        debug_assert_eq!(scalar.len(), N / 2);

        let mut res: Self = Self::identity();
        let mut temp = self.clone();

        for &words in scalar.iter() {
            for i in 0..u64::BITS {
                if (words >> i) & 1 == 1 {
                    res.complete_add_assign(&temp);
                }
                temp.double();
            }
        }

        *self = res;
    }

    /// Performs multi-scalar multiplication (MSM) on slices of bit vectors and points. Note:
    /// a_bits_le and b_bits_le should be in little endian order.
    fn multi_scalar_multiplication(
        a_bits_le: &[bool],
        a: Self,
        b_bits_le: &[bool],
        b: Self,
    ) -> Self {
        // The length of the bit vectors must be the same.
        debug_assert!(a_bits_le.len() == b_bits_le.len());

        if a_bits_le.is_empty() {
            return Self::identity();
        }

        // Evaluate a*A + b*B jointly in radix 4. The previous implementation
        // independently doubled A and B for every scalar bit. This fixed-window
        // Straus scan instead doubles the accumulator twice per two-bit window
        // and performs at most one complete addition per window.
        //
        // The table contains i*A + j*B for i,j in 0..=3. All additions remain
        // complete additions, so identity, equal-point, and inverse-point cases
        // retain the same semantics as the original implementation.
        let identity = Self::identity();

        let mut a2 = a.clone();
        a2.double();
        let mut a3 = a2.clone();
        a3.complete_add_assign(&a);

        let mut b2 = b.clone();
        b2.double();
        let mut b3 = b2.clone();
        b3.complete_add_assign(&b);

        let a_multiples = [identity.clone(), a, a2, a3];
        let b_multiples = [identity, b, b2, b3];
        let table: [Self; 16] = core::array::from_fn(|index| {
            let a_digit = index >> 2;
            let b_digit = index & 0b11;
            match (a_digit, b_digit) {
                (0, _) => b_multiples[b_digit].clone(),
                (_, 0) => a_multiples[a_digit].clone(),
                _ => {
                    let mut point = a_multiples[a_digit].clone();
                    point.complete_add_assign(&b_multiples[b_digit]);
                    point
                }
            }
        });

        let windows = a_bits_le.len().div_ceil(2);
        let mut result = Self::identity();
        let mut started = false;

        for window in (0..windows).rev() {
            let bit = window * 2;
            let a_digit = usize::from(a_bits_le[bit])
                | (a_bits_le.get(bit + 1).copied().map_or(0, usize::from) << 1);
            let b_digit = usize::from(b_bits_le[bit])
                | (b_bits_le.get(bit + 1).copied().map_or(0, usize::from) << 1);
            let table_index = (a_digit << 2) | b_digit;

            if started {
                result.double();
                result.double();
                if table_index != 0 {
                    result.complete_add_assign(&table[table_index]);
                }
            } else if table_index != 0 {
                result = table[table_index].clone();
                started = true;
            }
        }

        result
    }

    /// Evaluate four scalar-point products in one binary Straus scan.
    ///
    /// This is used by secp256k1 after GLV decomposition, where every scalar
    /// is guaranteed to fit in 128 bits. Curves without a decomposition never
    /// call this method.
    fn multi_scalar_multiplication_4(bits_le: [&[bool]; 4], points: [Self; 4]) -> Self {
        let bit_length = bits_le[0].len();
        debug_assert!(bits_le.iter().all(|bits| bits.len() == bit_length));
        if bit_length == 0 {
            return Self::identity();
        }

        // Table[mask] is the sum of the selected input points. Power-of-two
        // entries require no addition; each other entry adds one new point.
        let mut table: [Self; 16] = core::array::from_fn(|_| Self::identity());
        for mask in 1_usize..16 {
            let first = mask.trailing_zeros() as usize;
            let remaining = mask & !(1 << first);
            if remaining == 0 {
                table[mask] = points[first].clone();
            } else {
                table[mask] = table[remaining].clone();
                table[mask].complete_add_assign(&points[first]);
            }
        }

        let mut result = Self::identity();
        let mut started = false;
        for bit in (0..bit_length).rev() {
            let mask = bits_le
                .iter()
                .enumerate()
                .fold(0_usize, |mask, (index, bits)| {
                    mask | (usize::from(bits[bit]) << index)
                });

            if started {
                result.double();
                if mask != 0 {
                    result.complete_add_assign(&table[mask]);
                }
            } else if mask != 0 {
                result = table[mask].clone();
                started = true;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use super::AffinePoint;

    const MODULUS: u64 = 65_521;
    static ADDITIONS: AtomicUsize = AtomicUsize::new(0);
    static DOUBLINGS: AtomicUsize = AtomicUsize::new(0);
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct TestPoint([u64; 2]);

    impl TestPoint {
        fn value(value: u64) -> Self {
            Self([value % MODULUS, 0])
        }
    }

    impl AffinePoint<2> for TestPoint {
        const GENERATOR: [u64; 2] = [1, 0];
        const GENERATOR_T: Self = Self([1, 0]);

        fn new(limbs: [u64; 2]) -> Self {
            Self::value(limbs[0])
        }

        fn identity() -> Self {
            Self::value(0)
        }

        fn limbs_ref(&self) -> &[u64; 2] {
            &self.0
        }

        fn limbs_mut(&mut self) -> &mut [u64; 2] {
            &mut self.0
        }

        fn is_identity(&self) -> bool {
            self.0[0] == 0
        }

        fn add_assign(&mut self, other: &Self) {
            ADDITIONS.fetch_add(1, Ordering::Relaxed);
            self.0[0] = (self.0[0] + other.0[0]) % MODULUS;
        }

        fn double(&mut self) {
            DOUBLINGS.fetch_add(1, Ordering::Relaxed);
            self.0[0] = (self.0[0] * 2) % MODULUS;
        }
    }

    fn bits_le(value: u16, length: usize) -> Vec<bool> {
        (0..length).map(|bit| value & (1_u16 << bit) != 0).collect()
    }

    fn reference_msm(
        a_bits_le: &[bool],
        mut a: TestPoint,
        b_bits_le: &[bool],
        mut b: TestPoint,
    ) -> TestPoint {
        let mut result = TestPoint::identity();
        for (a_bit, b_bit) in a_bits_le.iter().zip(b_bits_le) {
            if *a_bit {
                result.add_assign(&a);
            }
            if *b_bit {
                result.add_assign(&b);
            }
            a.double();
            b.double();
        }
        result
    }

    fn reference_msm_4(bits_le: [&[bool]; 4], mut points: [TestPoint; 4]) -> TestPoint {
        let mut result = TestPoint::identity();
        for bit in 0..bits_le[0].len() {
            for index in 0..4 {
                if bits_le[index][bit] {
                    result.add_assign(&points[index]);
                }
                points[index].double();
            }
        }
        result
    }

    #[test]
    fn joint_window_msm_matches_independent_double_and_add() {
        let _guard = TEST_LOCK.lock().unwrap();
        for bit_length in 0..=9 {
            let limit = 1_u16 << bit_length;
            for a_scalar in 0..limit {
                for b_scalar in 0..limit {
                    let a_bits = bits_le(a_scalar, bit_length);
                    let b_bits = bits_le(b_scalar, bit_length);
                    let a = TestPoint::value(17);
                    let b = TestPoint::value(29);
                    let expected = reference_msm(&a_bits, a.clone(), &b_bits, b.clone());
                    let actual = TestPoint::multi_scalar_multiplication(&a_bits, a, &b_bits, b);
                    assert_eq!(actual, expected);
                }
            }
        }
    }

    #[test]
    fn joint_window_msm_halves_doublings_for_full_width_scalars() {
        let _guard = TEST_LOCK.lock().unwrap();
        let bits = vec![true; 256];
        let expected = reference_msm(&bits, TestPoint::value(17), &bits, TestPoint::value(29));

        ADDITIONS.store(0, Ordering::Relaxed);
        DOUBLINGS.store(0, Ordering::Relaxed);
        let actual = TestPoint::multi_scalar_multiplication(
            &bits,
            TestPoint::value(17),
            &bits,
            TestPoint::value(29),
        );

        assert_eq!(actual, expected);
        assert_eq!(DOUBLINGS.load(Ordering::Relaxed), 256);
        assert_eq!(ADDITIONS.load(Ordering::Relaxed), 138);
    }

    #[test]
    fn joint_window_msm_handles_identity_and_odd_bit_lengths() {
        let _guard = TEST_LOCK.lock().unwrap();
        let a_bits = [true, false, true, false, true];
        let b_bits = [false, true, false, true, false];
        let a = TestPoint::identity();
        let b = TestPoint::value(37);
        let expected = reference_msm(&a_bits, a.clone(), &b_bits, b.clone());
        let actual = TestPoint::multi_scalar_multiplication(&a_bits, a, &b_bits, b);
        assert_eq!(actual, expected);
    }

    #[test]
    fn joint_window_msm_handles_cancelling_precomputed_points() {
        let _guard = TEST_LOCK.lock().unwrap();
        let a_bits = [true, true, false, true, true, false, true];
        let b_bits = [true, true, true, false, true, false, true];
        let a = TestPoint::value(17);
        let b = TestPoint::value(MODULUS - 17);
        let expected = reference_msm(&a_bits, a.clone(), &b_bits, b.clone());
        let actual = TestPoint::multi_scalar_multiplication(&a_bits, a, &b_bits, b);
        assert_eq!(actual, expected);
    }

    #[test]
    fn four_point_msm_matches_reference_for_small_scalars() {
        let _guard = TEST_LOCK.lock().unwrap();
        let points = [
            TestPoint::value(7),
            TestPoint::value(13),
            TestPoint::value(29),
            TestPoint::value(MODULUS - 31),
        ];
        for scalar in 0_u16..256 {
            let a = bits_le(scalar, 8);
            let b = bits_le(scalar.rotate_left(3), 8);
            let c = bits_le(scalar ^ 0x55, 8);
            let d = bits_le(255 - scalar, 8);
            let bits = [&a[..], &b[..], &c[..], &d[..]];
            let expected = reference_msm_4(bits, points.clone());
            let actual = TestPoint::multi_scalar_multiplication_4(bits, points.clone());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn four_point_msm_uses_one_128_bit_doubling_scan() {
        let _guard = TEST_LOCK.lock().unwrap();
        let bits = vec![true; 128];
        let points = [
            TestPoint::value(7),
            TestPoint::value(13),
            TestPoint::value(29),
            TestPoint::value(43),
        ];
        let slices = [&bits[..], &bits[..], &bits[..], &bits[..]];
        let expected = reference_msm_4(slices, points.clone());

        ADDITIONS.store(0, Ordering::Relaxed);
        DOUBLINGS.store(0, Ordering::Relaxed);
        let actual = TestPoint::multi_scalar_multiplication_4(slices, points);

        assert_eq!(actual, expected);
        assert_eq!(DOUBLINGS.load(Ordering::Relaxed), 127);
        assert_eq!(ADDITIONS.load(Ordering::Relaxed), 138);
    }

    #[test]
    fn four_point_msm_handles_empty_scalars() {
        let _guard = TEST_LOCK.lock().unwrap();
        let empty = [];
        let actual = TestPoint::multi_scalar_multiplication_4(
            [&empty, &empty, &empty, &empty],
            [
                TestPoint::value(7),
                TestPoint::value(13),
                TestPoint::value(29),
                TestPoint::value(43),
            ],
        );
        assert_eq!(actual, TestPoint::identity());
    }
}

/// Errors that can occur during scalar multiplication of an [`AffinePoint`].
#[derive(Debug)]
pub enum MulAssignError {
    ScalarIsZero,
}

/// Converts a slice of words to a byte array in little endian.
pub fn words_to_bytes_le(words: &[u64]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>()
}

/// Converts a byte array in little endian to a slice of words.
pub fn bytes_to_words_le(bytes: &[u8]) -> Vec<u64> {
    bytes
        .chunks_exact(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
        .collect::<Vec<_>>()
}

#[derive(Copy, Clone, Debug)]
/// A representation of a point on a Weierstrass curve.
pub enum WeierstrassPoint<const N: usize> {
    Infinity,
    Affine([u64; N]),
}

/// A trait for affine points on Weierstrass curves.
pub trait WeierstrassAffinePoint<const N: usize>: AffinePoint<N> {
    /// The infinity point representation of the Weierstrass curve. Typically an enum variant.
    fn infinity() -> Self;

    /// Returns true if the point is the infinity point.
    fn is_infinity(&self) -> bool;

    /// Performs the complete addition of two [`AffinePoint`]'s on a Weierstrass curve.
    /// For an addition of two points P1 and P2, the cases are:
    ///     1. P1 is infinity
    ///     2. P2 is infinity
    ///     3. P1 equals P2
    ///     4. P1 is the negation of P2
    ///     5. Default addition.
    ///
    /// Implements the complete addition cases according to the
    /// [Zcash complete addition spec](https://zcash.github.io/halo2/design/gadgets/ecc/addition.html#complete-addition).
    fn weierstrass_add_assign(&mut self, other: &Self) {
        // Case 1: p1 is infinity.
        if self.is_infinity() {
            *self = other.clone();
            return;
        }

        // Case 2: p2 is infinity.
        if other.is_infinity() {
            return;
        }

        // Once it's known the points are not infinity, their limbs can be safely used.
        let p1 = self.limbs_mut();
        let p2 = other.limbs_ref();

        // Case 3: p1 equals p2.
        if p1 == p2 {
            self.double();
            return;
        }

        // Case 4: p1 is the negation of p2.
        // Note: If p1 and p2 are valid elliptic curve points, and p1.x == p2.x, that means that
        // either p1.y == p2.y or p1.y + p2.y == p. Because we are past Case 4, we know that p1.y !=
        // p2.y, so we can just check if p1.x == p2.x. Therefore, this implicitly checks that
        // p1.x == p2.x AND p1.y + p2.y == p without modular negation.
        if p1[..N / 2] == p2[..N / 2] {
            *self = Self::infinity();
            return;
        }

        // Case 5: Default addition.
        self.add_assign(other);
    }
}
