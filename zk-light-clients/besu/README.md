# Besu QBFT ZK IBC light client

An SP1-based IBC light client for Besu QBFT chains. Three circuits prove consensus updates,
packet/acknowledgement membership, and receipt non-membership; Solidity contracts verify the proofs.

## Prerequisites

- Git, Make, Bash, and a running Docker engine with Compose 
- For native setup, tests, or profile preparation: `rustup`, `sp1up`, `foundryup`, Python 3.9+,
  and the OS dependencies listed in [Development](docs/DEVELOPMENT.md#toolchains-and-dependencies).
- For deployment: source/destination RPC endpoints, a funded deployer key file, an AccessManager,
  and a compatible SP1 verifier gateway. See [Deployment](docs/DEPLOYMENT.md#prerequisites).


## 1. Set up native tools (optional)

```bash
make setup
```

Installs  toolchains natively, contract dependencies, coverage  and Python dependencies in `.venv`. Skip this step for Docker-only builds and deployment with a manually prepared profile.

## Build with Docker

### 2. Build the artifacts

```bash
make build-docker
```

Builds Linux host tools and the Proof API binary in `target/docker/release/`, Solidity contracts in
`contracts/out/`, and the three circuit ELFs with matching VKeys in `artifacts/`.
Rebuild after source changes. Build internals are covered in [Development](docs/DEVELOPMENT.md).

## Tests and development

### 3. Run the checks you need

With native setup complete, run focused checks:

```bash
make test ACTION=rust
make test ACTION=shell
make test ACTION=contracts
make test ACTION=check
```

For full validation, `make test` also runs coverage, real proofs, and external Router integration.
`make e2e` separately starts disposable Besu networks. Follow [Testing](docs/TESTING.md) for their
additional prerequisites.

## Deploy to existing test nodes

### 4. Prepare a deployment profile

Copy `config/template.json` to `config/<name>.json`, set its `name`, and fill in the chain settings
using the [profile guide](docs/DEPLOYMENT.md#deployment-profiles). Derive the bootstrap anchor:

```bash
export SOURCE_RPC_URL=http://source-test-node:8545
export DESTINATION_RPC_URL=http://destination-test-node:8545
make deploy ACTION=bootstrap PROFILE=my-network
```

Review the returned values and copy them into the profile's `bootstrap` object. VKeys are read
automatically from `artifacts/vkeys.json`.

Alternatively, after native setup and the Docker build, use `make prepare-profile` with the
[IBC configuration inputs](docs/DEPLOYMENT.md#prepare-from-an-ibc-config) to generate the profile.

### 5. Validate and deploy

```bash
export DEPLOYER_KEY_FILE=/absolute/path/to/deployer-key
make deploy ACTION=preflight PROFILE=my-network
make deploy PROFILE=my-network
```

Results are saved under `deployments/my-network/`. To register the deployed proxy with an existing
compatible Router, run:

```bash
make deploy ACTION=register PROFILE=my-network LIGHT_CLIENT=0xYourProxy
```

## Optional next steps

| Task | Command / guide |
| --- | --- |
| Run the Proof API | `make build-proof-api`, then configure and start it using the [Proof API guide](tooling/proof-api/README.md). Recreate the service after rebuilding artifacts. |
| Generate release evidence | `make release` after building and validating; see [Release](docs/RELEASE.md) for native tools and external Router requirements. |
| List workflows and actions | `make help` |
| Understand the protocol | [Architecture](docs/architecture.md) and [Specification](SPEC.md) |
| Integrate a relayer | [Integration](docs/INTEGRATION.md) |
