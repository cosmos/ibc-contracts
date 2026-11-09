# Optional portable runtime environment. Application outputs are supplied by bind mounts.
FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install --yes --no-install-recommends ca-certificates libssl3 libgmp10 libstdc++6 \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
ENV BESU_PROVER_ARTIFACTS_DIR=/workspace/artifacts
USER 10001:10001
EXPOSE 8085
ENTRYPOINT ["/workspace/target/docker/release/proof-api"]
CMD ["serve", "--config", "/etc/proof-api/config.yaml"]

HEALTHCHECK --interval=15s --timeout=5s --start-period=60s --retries=2 CMD ["/workspace/target/docker/release/proof-api", "healthcheck"]
