# syntax=docker/dockerfile:1.7
# Development/build tools only. Source and generated output are mounted at runtime.
# Export the exact guest compiler shipped by the pinned SP1 image. It is used
# directly in the final builder, with no Docker engine or CLI inside that image.
FROM ghcr.io/succinctlabs/sp1:v6.1.0 AS sp1-toolchain
RUN mkdir -p /opt/sp1-toolchain \
 && cp -a "$(RUSTUP_TOOLCHAIN=succinct rustc --print sysroot)/." /opt/sp1-toolchain/

FROM ghcr.io/foundry-rs/foundry:v1.8.1@sha256:0c00cb0bda1ab1b91c9a6bf60f4c76c09c1a8870824b6d4718afbabacf6f9a17 AS foundry
FROM rust:1.97.1-bookworm

RUN apt-get update \
 && apt-get install --yes --no-install-recommends \
      build-essential ca-certificates clang cmake curl git jq make python3 python3-yaml python3-cryptography util-linux libgmp-dev \
      libprotobuf-dev libssl-dev nodejs npm perl pkg-config protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

COPY --from=foundry /usr/local/bin/forge /usr/local/bin/forge
COPY --from=foundry /usr/local/bin/cast /usr/local/bin/cast
COPY --from=sp1-toolchain /opt/sp1-toolchain /opt/sp1-toolchain

# Register the guest compiler in the shared Rust installation so it is available
# to the caller's UID, without relying on a toolchain under root's home.
# SP1 ships no Cargo. Use the real binary from the pinned host toolchain, since
# rustup's custom-toolchain fallback does not search versioned toolchain names.
# Do not link the /usr/local/cargo/bin/cargo rustup proxy back into the toolchain.
RUN chmod -R a+rX /opt/sp1-toolchain \
 && ln -s "$(rustup which cargo)" /opt/sp1-toolchain/bin/cargo \
 && rustup toolchain link succinct /opt/sp1-toolchain \
 && rustup run succinct rustc --version \
 && rustup run succinct cargo --version

# The CLI orchestrates local compilation with the bundled succinct toolchain.
# Keep this in sync with the CLI identity gate in scripts/build-sp1.sh.
RUN curl --fail --location --retry 3 \
      https://github.com/succinctlabs/sp1/releases/download/v6.1.0/cargo_prove_v6.1.0_linux_amd64.tar.gz \
      --output /tmp/cargo-prove.tar.gz \
 && tar -xzf /tmp/cargo-prove.tar.gz -C /usr/local/bin \
 && rm /tmp/cargo-prove.tar.gz \
 && cargo prove --version | grep -q d454975 \
 && forge --version | grep -q '1.8.1' \
 && rustc --version | grep -q '^rustc 1\.97\.1 '

ENTRYPOINT ["/bin/bash"]
