FROM rust:1.92-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY packages packages
COPY programs programs
COPY ibc-solidity/contracts ibc-solidity/contracts
COPY ibc-solidity/abi ibc-solidity/abi
COPY ibc-solidity/programs ibc-solidity/programs
# `ibc-solana` is a separate workspace; its `packages/*` are path dependencies of
# `packages/tendermint-light-client/solana`. The workspace manifest is needed so those
# packages can resolve their inherited (`workspace = true`) fields.
COPY ibc-solana/Cargo.toml ibc-solana/Cargo.toml
COPY ibc-solana/packages ibc-solana/packages
COPY proto proto

RUN cargo build --bin proof-api --release --locked
RUN cargo build --bin operator --release --locked

FROM debian:bookworm-slim AS certs
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

FROM gcr.io/distroless/cc-debian12:debug

ARG BUILD_DATE
ARG VCS_REF
ARG VERSION
ARG COMMIT
LABEL org.opencontainers.image.title="proof-api" \
      org.opencontainers.image.description="Solidity IBC Eureka Proof API" \
      org.opencontainers.image.source="https://github.com/cosmos/solidity-ibc-eureka" \
      org.opencontainers.image.revision=$VCS_REF \
      org.opencontainers.image.created=$BUILD_DATE \
      org.opencontainers.image.version=$VERSION

COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

COPY --from=builder /app/target/release/proof-api /usr/local/bin/proof-api
COPY --from=builder /app/target/release/operator /usr/local/bin/operator
COPY scripts/proof_api_docker_entrypoint.sh /docker_entrypoint.sh

ENV PLATFORM_COMMIT=${COMMIT}
ENTRYPOINT ["sh", "/docker_entrypoint.sh", "start", "--config", "/usr/local/proof-api/proof-api.json"]