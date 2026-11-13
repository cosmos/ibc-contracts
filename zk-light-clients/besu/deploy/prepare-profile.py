#!/usr/bin/env python3
"""Prepare public deployment profiles and resolve runtime inputs from IBC YAML.

Dependencies: deploy/requirements.txt. No IBC CLI or checkout is required.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def uint(value, label, nonzero=False):
    if not re.fullmatch(r"[0-9]+", str(value)):
        raise ValueError(f"{label} must be an unsigned decimal integer")
    n = int(value)
    if n > 2**64 - 1 or (nonzero and n == 0):
        raise ValueError(f"{label} must fit uint64" + (" and be nonzero" if nonzero else ""))
    return n


def json_uint(n):
    return str(n) if n > 2**53 - 1 else n


def address(value, label):
    if not isinstance(value, str) or not re.fullmatch(r"0x[0-9a-fA-F]{40}", value) or int(value, 16) == 0:
        raise ValueError(f"{label} must be a nonzero EVM address (quote addresses in YAML)")
    return value


def load_config(path):
    try:
        import yaml
    except ImportError:
        raise ValueError("install profile dependencies: python3 -m pip install -r deploy/requirements.txt") from None

    class Loader(yaml.SafeLoader):
        pass

    def mapping(loader, node):
        result = {}
        for key_node, value_node in node.value:
            key = loader.construct_object(key_node)
            if key in result:
                raise ValueError(f"duplicate YAML key: {key}")
            result[key] = loader.construct_object(value_node)
        return result

    Loader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, mapping)
    text = Path(path).expanduser().read_text()
    # Match IBC's environment expansion, but reject missing variables instead of
    # silently changing endpoints or key paths. Never execute shell expansions.
    def expand(match):
        key = match.group(1) or match.group(2)
        if key not in os.environ:
            raise ValueError(f"IBC config references unset environment variable: {key}")
        return os.environ[key]
    text = re.sub(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)", expand, text)
    try:
        config = yaml.load(text, Loader=Loader)
    except yaml.YAMLError:
        raise ValueError("invalid IBC YAML; check its syntax and quote addresses") from None
    if not isinstance(config, dict) or not isinstance(config.get("chains"), list):
        raise ValueError("IBC config must contain a chains list")
    if any(not isinstance(c, dict) or "chainId" not in c for c in config["chains"]):
        raise ValueError("each IBC chain must be a mapping with chainId")
    ids = [str(c["chainId"]) for c in config["chains"]]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate chainId in IBC config")
    return config


def chain(config, chain_id):
    matches = [c for c in config["chains"] if str(c["chainId"]) == str(chain_id)]
    if len(matches) != 1:
        raise ValueError(f"chain {chain_id} is not uniquely declared in IBC config")
    c = matches[0]
    uint(c["chainId"], "chainId", nonzero=True)
    evm = c.get("evm", {})
    if not isinstance(evm, dict):
        raise ValueError(f"chain {chain_id} requires an evm mapping")
    if not re.fullmatch(r"https?://[^\s]+", evm.get("rpc", "")):
        raise ValueError(f"chain {chain_id} requires evm.rpc with an http(s) URL")
    address(evm.get("ics26Router"), f"chain {chain_id} evm.ics26Router")
    return c


def local_key(config, destination, config_path):
    override = os.environ.get("DEPLOYER_KEY_FILE")
    if override:
        path = Path(override).expanduser()
    else:
        alias = destination.get("deployer")
        matches = [s for s in config.get("signers", []) if s.get("alias") == alias]
        if not alias or len(matches) != 1 or matches[0].get("type") != "local":
            raise ValueError("destination requires a unique local deployer signer or DEPLOYER_KEY_FILE")
        raw = Path(matches[0]["file"]).expanduser()
        # The IBC CLI resolves key paths relative to --home. Default home is
        # the config directory; IBC_HOME supports configs located elsewhere.
        home = Path((os.environ.get("IBC_HOME") or str(Path(config_path).expanduser().resolve().parent)))
        candidates = [raw]
        if not raw.is_absolute():
            if raw.suffix != ".json":
                candidates.append(Path(str(raw) + ".json"))
            if "keys" not in raw.parts:
                candidates.append(Path("keys") / candidates[-1])
            candidates = [home / p for p in candidates]
        path = next((p for p in candidates if p.is_file()), candidates[0])
    content = path.read_text().strip()
    if content.startswith("{"):
        data = json.loads(content)
        if not isinstance(data, dict) or data.get("type") != "ecdsa":
            raise ValueError("deployer key must be ecdsa")
        content = base64.b64decode(data["privateKeyBase64"], validate=True).hex()
    if not re.fullmatch(r"(?:0x)?[0-9a-fA-F]{64}", content):
        raise ValueError("deployer key must be 32-byte hex or an IBC ecdsa JSON key")
    key = "0x" + content.removeprefix("0x")
    if not 0 < int(key, 16) < 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141:
        raise ValueError("invalid secp256k1 private key")
    return key


def execute(args, env=None, cwd=ROOT):
    result = subprocess.run([str(a) for a in args], cwd=cwd, env=dict(os.environ, **(env or {})),
                            capture_output=True, text=True)
    if result.returncode:
        # External errors may include credentials. Name the failed operation only.
        raise ValueError(f"{Path(args[0]).name} {args[1] if len(args) > 1 else ''} failed (exit {result.returncode})")
    return result.stdout.strip()


def cast(rpc, *args):
    return execute([(os.environ.get("CAST_BIN") or "cast"), *args], {"FOUNDRY_ETH_RPC_URL": rpc})


def deployer_address(key):
    try:
        from cryptography.hazmat.primitives.asymmetric import ec
        from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
    except ImportError:
        raise ValueError("install profile dependencies: python3 -m pip install -r deploy/requirements.txt") from None
    public = ec.derive_private_key(int(key, 16), ec.SECP256K1()).public_key()
    # Only the public key reaches cast's arguments. Ethereum uses Keccak, not SHA3.
    encoded = public.public_bytes(Encoding.X962, PublicFormat.UncompressedPoint)[1:]
    digest = cast("", "keccak", "0x" + encoded.hex())
    if not re.fullmatch(r"0x[0-9a-fA-F]{64}", digest):
        raise ValueError("cast returned an invalid public-key hash")
    return address("0x" + digest[-40:], "deployer")


def code(rpc, addr, height=None):
    args = ["code", addr]
    if height is not None:
        args += ["--block", str(height)]
    result = cast(rpc, *args)
    return bool(re.fullmatch(r"0x[0-9a-fA-F]+", result)) and result not in ("0x", "0x0", "0x00")


def bootstrap(rpc, source_id, height):
    binary = os.environ.get("BOOTSTRAP_STATE_BIN")
    command = [binary] if binary else ["cargo", "run", "--quiet", "--locked", "--no-default-features",
                                        "-p", "prover-script", "--bin", "bootstrap-state", "--"]
    result = json.loads(execute(command + ["--expected-source-chain-id", str(source_id), "--height", str(height)],
                                {"SOURCE_RPC_URL": rpc}))
    if uint(result["height"], "bootstrap height", True) != height:
        raise ValueError("bootstrap-state returned a different height")
    for field in ("stateRoot", "validatorSetHash"):
        if not re.fullmatch(r"0x[0-9a-fA-F]{64}", result[field]) or int(result[field], 16) == 0:
            raise ValueError(f"invalid bootstrap {field}")
    uint(result["blockTimestamp"], "bootstrap timestamp", True)
    return {k: json_uint(uint(result[k], k, True)) if k in ("height", "blockTimestamp") else result[k]
            for k in ("height", "stateRoot", "blockTimestamp", "validatorSetHash")}


def gateway(rpc, destination_id, key, explicit, deploy):
    if explicit:
        addr = address(explicit, "SP1_VERIFIER_GATEWAY")
        if not code(rpc, addr):
            raise ValueError("supplied SP1 gateway has no deployed code")
        return addr
    if not deploy:
        raise ValueError("set SP1_VERIFIER_GATEWAY or explicitly enable DEPLOY_LOCAL_VERIFIER=true for a devnet")
    broadcast = ROOT / f"contracts/broadcast/DeployLocalSP1Verifier.s.sol/{destination_id}/run-latest.json"

    def recovered():
        if not broadcast.is_file():
            return None
        txs = json.loads(broadcast.read_text()).get("transactions", [])
        found = {}
        for name in ("SP1VerifierGateway", "SP1Verifier"):
            values = {t["contractAddress"] for t in txs if t.get("contractName") == name
                      and t.get("transactionType", "").startswith("CREATE")}
            if len(values) != 1:
                return None
            addr = address(values.pop(), name)
            if not code(rpc, addr):
                return None
            found[name] = addr
        return found["SP1VerifierGateway"]

    addr = recovered()
    if addr:
        return addr
    print("Deploying local SP1 verifier and gateway", file=sys.stderr)
    execute([(os.environ.get("FORGE_BIN") or "forge"), "script",
             "script/dev/DeployLocalSP1Verifier.s.sol:DeployLocalSP1Verifier", "--broadcast", "--slow",
             "--non-interactive", "--gas-estimate-multiplier", (os.environ.get("GAS_ESTIMATE_MULTIPLIER") or "200")],
            {"FOUNDRY_ETH_RPC_URL": rpc, "DEPLOYER_PRIVATE_KEY": key}, ROOT / "contracts")
    addr = recovered()
    if not addr:
        raise ValueError("cannot recover live verifier contracts from Foundry broadcast")
    return addr


def prepare(args):
    config = load_config(args.ibc_config)
    source = chain(config, args.source_chain_id)
    destination = chain(config, args.destination_chain_id)
    source_id = uint(source["chainId"], "source chain ID", True)
    destination_id = uint(destination["chainId"], "destination chain ID", True)
    if source_id == destination_id:
        raise ValueError("source and destination chain IDs must differ")
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", args.profile):
        raise ValueError("invalid PROFILE name")
    for value, label in ((args.client_id, "CLIENT_ID"), (args.counterparty_client_id, "COUNTERPARTY_CLIENT_ID")):
        if not value or re.search(r"\s", value):
            raise ValueError(f"{label} is required and must not contain whitespace")
    trusting = uint(args.trusting_period_seconds, "TRUSTING_PERIOD_SECONDS")
    drift = uint(args.max_clock_drift_seconds, "MAX_CLOCK_DRIFT_SECONDS")
    confirmations = uint(args.bootstrap_confirmations, "BOOTSTRAP_CONFIRMATIONS")
    if not re.fullmatch(r"0x[0-9a-fA-F]{8}", args.sp1_groth16_selector):
        raise ValueError("SP1_GROTH16_SELECTOR must be four bytes")
    output = ROOT / "config" / (args.profile + ".json")
    if output.exists() and not args.force:
        raise ValueError("profile exists; pass FORCE=true to replace it")
    if not args.sp1_verifier_gateway and not args.deploy_local_verifier:
        raise ValueError("set SP1_VERIFIER_GATEWAY or explicitly enable DEPLOY_LOCAL_VERIFIER=true for a devnet")
    if args.sp1_verifier_gateway:
        address(args.sp1_verifier_gateway, "SP1_VERIFIER_GATEWAY")
    src, dst = source["evm"], destination["evm"]
    for c in (source, destination):
        if int(cast(c["evm"]["rpc"], "chain-id")) != int(c["chainId"]):
            raise ValueError(f"RPC chain ID mismatch for chain {c['chainId']}")
        if not code(c["evm"]["rpc"], c["evm"]["ics26Router"]):
            raise ValueError(f"Router has no code on chain {c['chainId']}")
    key = local_key(config, destination, args.ibc_config)
    deployer = deployer_address(key)
    authority = address(cast(dst["rpc"], "call", dst["ics26Router"], "authority()(address)"), "upgrade authority")
    if not code(dst["rpc"], authority):
        raise ValueError("Router upgrade authority has no code")
    if args.height:
        height = uint(args.height, "HEIGHT", True)
        if not code(src["rpc"], src["ics26Router"], height):
            raise ValueError("source Router has no code at HEIGHT")
    else:
        for attempt in range(120):
            height = int(cast(src["rpc"], "block-number")) - confirmations
            if height > 0 and code(src["rpc"], src["ics26Router"], height):
                break
            time.sleep(1)
        else:
            raise ValueError("no confirmed source block containing the Router; try again later")
    anchor = bootstrap(src["rpc"], source_id, height)
    # Resolve all read-only inputs before the optional verifier broadcast.
    verifier = gateway(dst["rpc"], destination_id, key, args.sp1_verifier_gateway, args.deploy_local_verifier)
    profile = dict(name=args.profile, description=f"Light client on {destination_id} tracking {source_id}",
                   notes=["Generated from IBC config; RPC URLs and signing keys omitted."],
                   chainId=json_uint(destination_id), deployer=deployer, localIbcRouter=dst["ics26Router"],
                   sp1VerifierGateway=verifier, upgradeAuthority=authority,
                   sp1Groth16Selector=args.sp1_groth16_selector, proofMode="groth16",
                   source=dict(name=str(source_id), chainId=json_uint(source_id),
                               ibcRouter=src["ics26Router"], validatorMode="header-qbft"),
                   bootstrap=anchor, trustingPeriodSeconds=json_uint(trusting), maxClockDriftSeconds=json_uint(drift),
                   registration=dict(clientId=args.client_id, counterpartyClientId=args.counterparty_client_id))
    # Stage with the real filename: shared preflight requires name == filename.
    with tempfile.TemporaryDirectory(prefix=".prepare-", dir=ROOT / "config") as work:
        staged = Path(work) / output.name
        staged.write_text(json.dumps(profile, indent=2) + "\n")
        env = dict(os.environ, SOURCE_RPC_URL=src["rpc"], DESTINATION_RPC_URL=dst["rpc"])
        # These explicit preparation options now live in the staged profile.
        for field in ("SP1_VERIFIER_GATEWAY", "SP1_GROTH16_SELECTOR", "TRUSTING_PERIOD_SECONDS", "MAX_CLOCK_DRIFT_SECONDS"):
            env.pop(field, None)
        result = subprocess.run(["bash", str(ROOT / "deploy/validate.sh"), "--profile", str(staged)], cwd=ROOT, env=env)
        if result.returncode:
            raise ValueError("profile preflight failed; existing profile was preserved")
        if args.force:
            os.replace(staged, output)
        else:
            os.link(staged, output)  # Atomic no-clobber, including concurrent callers.
    print(output)


def run(args):
    config = load_config(args.ibc_config)
    name = args.deployment_args[1]
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", name):
        raise ValueError("invalid profile name")
    profile_dir = Path(os.environ.get("PROFILE_DIR") or str(ROOT / "config")).expanduser().resolve()
    profile = json.loads((profile_dir / (name + ".json")).read_text())
    source = chain(config, profile["source"]["chainId"])
    destination = chain(config, profile["chainId"])
    if (source["evm"]["ics26Router"].lower() != profile["source"]["ibcRouter"].lower()
            or destination["evm"]["ics26Router"].lower() != profile["localIbcRouter"].lower()):
        raise ValueError("IBC config Router addresses disagree with the reviewed profile")
    env = dict(os.environ, SOURCE_RPC_URL=source["evm"]["rpc"], DESTINATION_RPC_URL=destination["evm"]["rpc"])
    env.pop("IBC_CONFIG", None)  # Prevent recursion in scripts/deploy.sh.
    env["PROFILE_DIR"] = str(profile_dir)
    with tempfile.TemporaryDirectory(prefix="besu-deployer-") as work:
        if args.deployment_args[0] in ("deploy", "register"):
            key = local_key(config, destination, args.ibc_config)
            key_file = Path(work) / "deployer.key"
            key_file.write_text(key + "\n")
            key_file.chmod(0o600)
            env["DEPLOYER_KEY_FILE"] = str(key_file)
        return subprocess.call(["bash", str(ROOT / "scripts/deploy.sh"), *args.deployment_args], env=env, cwd=ROOT)


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest="command", required=True)
    prep = sub.add_parser("prepare")
    runtime = sub.add_parser("run")
    for command in (prep, runtime):
        command.add_argument("--ibc-config", default=os.environ.get("IBC_CONFIG"), required=not os.environ.get("IBC_CONFIG"))
    for flag, var in (("profile", "PROFILE"), ("source-chain-id", "SOURCE_CHAIN_ID"),
                      ("destination-chain-id", "DESTINATION_CHAIN_ID"), ("client-id", "CLIENT_ID"),
                      ("counterparty-client-id", "COUNTERPARTY_CLIENT_ID"),
                      ("trusting-period-seconds", "TRUSTING_PERIOD_SECONDS"),
                      ("max-clock-drift-seconds", "MAX_CLOCK_DRIFT_SECONDS")):
        prep.add_argument("--" + flag, default=os.environ.get(var), required=not os.environ.get(var))
    for flag, var, default in (("sp1-verifier-gateway", "SP1_VERIFIER_GATEWAY", ""), ("height", "HEIGHT", ""),
                               ("bootstrap-confirmations", "BOOTSTRAP_CONFIRMATIONS", "1"),
                               ("sp1-groth16-selector", "SP1_GROTH16_SELECTOR", "0x4388a21c")):
        prep.add_argument("--" + flag, default=(os.environ.get(var) or default))
    for flag, var in (("force", "FORCE"), ("deploy-local-verifier", "DEPLOY_LOCAL_VERIFIER")):
        value = (os.environ.get(var) or "false")
        if value not in ("true", "false"):
            p.error(f"{var} must be true or false")
        prep.add_argument("--" + flag, action="store_true", default=value == "true")
    runtime.add_argument("deployment_args", nargs=argparse.REMAINDER)
    return p


def main():
    args = parser().parse_args()
    try:
        if args.command == "prepare":
            prepare(args)
        else:
            if len(args.deployment_args) < 2:
                raise ValueError("run requires <bootstrap|preflight|deploy|register> <profile> [height|address]")
            return run(args)
    except (ValueError, OSError, KeyError, TypeError) as error:
        # File errors contain paths, never file contents. Do not print external command output.
        print(f"profile preparation: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
