# Local k256 SP1 patch

This directory vendors `k256` 0.13.4 from Succinct's
`patch-k256-13.4-sp1-6.0.0` tag at commit
`476243d31d3ffd9d882d0e5da1f6adaa2a204b62`. The workspace selects it through
`[patch.crates-io]` so both host and guest resolve one auditable source tree.

The SP1-specific change exposes k256's existing secp256k1 GLV decomposition to
the zkVM arithmetic implementation. Decomposition runs unconstrained, but its
result is not trusted: constrained code checks canonical scalar encodings,
unique signs, 128-bit component bounds, and
`k1 + lambda * k2 == k (mod n)`. An invalid hint rejects execution.

The corresponding point endomorphism and four-component MSM live in the pinned
`sp1-lib` patch. Native k256 multiplication and the SP1 path share the same GLV
constants and decomposition implementation.

Host unit tests cover boundary scalars and reject truncated, malformed,
oversized, non-canonical-sign, and algebraically invalid hints.
