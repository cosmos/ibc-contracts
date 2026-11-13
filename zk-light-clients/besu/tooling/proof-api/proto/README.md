# Prover protocol provenance

`cli/prover.proto` is copied verbatim from `ibc/proto/cli/prover.proto` at
IBC revision `ebfb6f55de84af7dae14c3806f17ec2d60c5027f`.

SHA-256: `fad12107c79d5bf2a15d74a955af130b093be871352be1fc5564f1851a0729d8`.

The build uses pinned tonic/prost and pbjson generators plus a vendored protoc;
it does not access the sibling repository or require a system protoc. Generated
Rust stays in Cargo's output directory. For schema updates, copy the canonical
proto, record the new revision/hash here, and rerun Rust and Go compatibility tests.
