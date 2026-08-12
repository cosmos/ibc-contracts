FROM rust:1.92-bookworm AS chef

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked --version 0.1.78

WORKDIR /app


FROM scratch AS sources
COPY Cargo.toml Cargo.lock /app/
COPY packages /app/packages
COPY programs /app/programs
COPY ibc-solidity/contracts /app/ibc-solidity/contracts
COPY ibc-solidity/abi /app/ibc-solidity/abi
COPY ibc-solidity/programs /app/ibc-solidity/programs
# `ibc-solana` is a separate workspace; its `packages/*` are path dependencies of
# `packages/tendermint-light-client/solana`. The workspace manifest is needed so those
# packages can resolve their inherited (`workspace = true`) fields.
COPY ibc-solana/Cargo.toml /app/ibc-solana/Cargo.toml
COPY ibc-solana/packages /app/ibc-solana/packages
COPY proto /app/proto


FROM chef AS planner
COPY --from=sources /app /app
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS solana-skeleton
COPY --from=sources /app/ibc-solana /src/ibc-solana
RUN set -eu; \
    cd /src/ibc-solana; \
    mkdir -p /out/ibc-solana; \
    cp Cargo.toml /out/ibc-solana/Cargo.toml; \
    for manifest in $(find packages -name Cargo.toml); do \
        mkdir -p "/out/ibc-solana/$(dirname "$manifest")/src"; \
        cp "$manifest" "/out/ibc-solana/$manifest"; \
        : > "/out/ibc-solana/$(dirname "$manifest")/src/lib.rs"; \
    done


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY --from=solana-skeleton /out/ibc-solana ibc-solana

RUN cargo chef cook --release --recipe-path recipe.json \
    && rm -rf ibc-solana

COPY --from=sources /app /app

RUN find ibc-solana -name '*.rs' -exec touch {} +
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
