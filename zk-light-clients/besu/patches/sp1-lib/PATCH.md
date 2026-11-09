# Local SP1 6.1.0 patch

This directory is the source of the crates.io `sp1-lib` v6.1.0 crate, pinned by
the workspace through `[patch.crates-io]`.

The patch first replaces two independent bit-by-bit scalar multiplications with
a width-two joint Straus scan. For secp256k1, it also accepts the validated GLV
decomposition supplied by the matching k256 patch, applies the curve
endomorphism through SP1's constrained bigint syscall, and evaluates four
128-bit components in one joint scan. These paths compute the same scalar-point
equation while reducing elliptic-curve operations used by public-key recovery.
They do not change witnesses, public values, QBFT validation rules, or SP1
verifier interfaces.

The unit tests in `src/utils.rs` compare the patched algorithm against the
original independent double-and-add calculation over exhaustive small scalar
ranges, odd bit lengths, zero/identity cases, and assert operation counts for
both the full-width and four-component paths.

`Cargo.lock` is retained in this internal patch directory so standalone unit
and Clippy runs resolve the same SP1 6.1.0 transitive family as the repository
workspace instead of mixing later semver-compatible `slop`/Plonky3 releases.
