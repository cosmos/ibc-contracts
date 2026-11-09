"""Profile import, preflight and runtime boundary tests; no live chain or signer."""
import base64
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("prepare_profile", ROOT / "deploy/prepare-profile.py")
p = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(p)
A, B, G, AUTH, DEPLOYER = ["0x" + c * 40 for c in "12345"]
KEY = "0x" + "0" * 63 + "1"


class PrepareProfileTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="besu profile ")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "config").mkdir()
        (self.root / "keys").mkdir()
        (self.root / "keys/deployer.json").write_text(json.dumps({
            "type": "ecdsa", "privateKeyBase64": base64.b64encode(bytes.fromhex(KEY[2:])).decode()}))
        self.config = self.root / "ibc.yml"
        self.config.write_text(f'''chains:
  - chainId: "41"
    evm:
      rpc: http://source:8545
      ics26Router: "{A}"
  - chainId: "42"
    deployer: deployer
    evm:
      rpc: http://destination:8545
      ics26Router: "{B}"
signers:
  - alias: deployer
    type: local
    file: deployer
relayer:
  connections:
    - alias: existing-attestation
      clientA: {{chainId: "41", clientId: already-registered}}
''')
        self.env = patch.dict(os.environ, {}, clear=True)
        self.env.start()
        self.addCleanup(self.env.stop)
        self.root_patch = patch.object(p, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        self.anchor = dict(height=9, stateRoot="0x" + "a" * 64, blockTimestamp=1000,
                           validatorSetHash="0x" + "b" * 64)

    def args(self, *extra):
        return p.parser().parse_args(["prepare", "--ibc-config", str(self.config), "--profile", "on-b-tracking-a",
                                      "--source-chain-id", "41", "--destination-chain-id", "42",
                                      "--client-id", "zk-a", "--counterparty-client-id", "zk-b",
                                      "--trusting-period-seconds", "86400", "--max-clock-drift-seconds", "30",
                                      "--sp1-verifier-gateway", G, *extra])

    def fake_cast(self, rpc, *args):
        if args[0] == "chain-id":
            return "41" if "source" in rpc else "42"
        if args[0] == "block-number":
            return "10"
        if args[0] == "code":
            return "0x1234"
        if args[0] == "call":
            return AUTH
        self.fail(f"unexpected cast call: {args}")

    def prepare(self, args=None, validation_status=0):
        with patch.object(p, "cast", side_effect=self.fake_cast), \
             patch.object(p, "deployer_address", return_value=DEPLOYER), \
             patch.object(p, "bootstrap", return_value=self.anchor) as anchor, \
             patch.object(p.subprocess, "run", return_value=subprocess.CompletedProcess([], validation_status)) as validate:
            p.prepare(args or self.args())
        return anchor, validate

    def test_complete_public_profile_and_preflight_before_save(self):
        anchor, validate = self.prepare()
        result = json.loads((self.root / "config/on-b-tracking-a.json").read_text())
        self.assertEqual(result["chainId"], 42)
        self.assertEqual(result["source"]["chainId"], 41)
        self.assertEqual(result["localIbcRouter"], B)
        self.assertEqual(result["source"]["ibcRouter"], A)
        self.assertEqual(result["upgradeAuthority"], AUTH)
        self.assertEqual(result["registration"], dict(clientId="zk-a", counterpartyClientId="zk-b"))
        self.assertEqual(result["bootstrap"], self.anchor)
        self.assertNotIn("http", json.dumps(result))
        self.assertNotIn(KEY, json.dumps(result))
        self.assertNotIn("vkeys", result)
        anchor.assert_called_once_with("http://source:8545", 41, 9)
        self.assertEqual(validate.call_args.args[0][2], "--profile")

    def test_no_overwrite_and_failed_refresh_preserves_reviewed_profile(self):
        output = self.root / "config/on-b-tracking-a.json"
        output.write_text("reviewed")
        with self.assertRaisesRegex(ValueError, "profile exists"):
            self.prepare()
        with self.assertRaisesRegex(ValueError, "preflight failed"):
            self.prepare(self.args("--force"), validation_status=1)
        self.assertEqual(output.read_text(), "reviewed")

    def test_key_fallback_and_explicit_raw_key(self):
        config = p.load_config(self.config)
        destination = p.chain(config, "42")
        self.assertEqual(p.local_key(config, destination, self.config), KEY)
        raw = self.root / "raw.key"
        raw.write_text(KEY)
        with patch.dict(os.environ, DEPLOYER_KEY_FILE=str(raw)):
            self.assertEqual(p.local_key(config, destination, self.config), KEY)

    def test_remote_signer_needs_explicit_key_file(self):
        self.config.write_text(self.config.read_text().replace("type: local", "type: remote"))
        with self.assertRaisesRegex(ValueError, "local deployer"):
            p.local_key(p.load_config(self.config), {"deployer": "deployer"}, self.config)

    def test_missing_chain_duplicate_yaml_and_unset_environment(self):
        with self.assertRaisesRegex(ValueError, "not uniquely declared"):
            p.chain(p.load_config(self.config), "99")
        self.config.write_text("chains: []\nchains: []\n")
        with self.assertRaisesRegex(ValueError, "duplicate YAML key"):
            p.load_config(self.config)
        self.config.write_text("chains: ${MISSING}\n")
        with self.assertRaisesRegex(ValueError, "unset environment"):
            p.load_config(self.config)

    def test_environment_expansion_is_data(self):
        self.config.write_text(self.config.read_text().replace("http://source:8545", "${SOURCE_URL}"))
        with patch.dict(os.environ, SOURCE_URL="http://source:8545/$(touch-never-executed)"):
            self.assertIn("$(touch", p.load_config(self.config)["chains"][0]["evm"]["rpc"])

    def test_wrong_chain_and_no_gateway_do_not_broadcast(self):
        with patch.object(p, "cast", return_value="99"), patch.object(p, "gateway") as gateway:
            with self.assertRaisesRegex(ValueError, "chain ID mismatch"):
                p.prepare(self.args())
            gateway.assert_not_called()
        with patch.object(p, "execute") as execute:
            with self.assertRaisesRegex(ValueError, "DEPLOY_LOCAL_VERIFIER"):
                p.gateway("http://destination", 42, KEY, "", False)
            execute.assert_not_called()

    def test_uint_precision_and_ranges(self):
        self.assertEqual(p.json_uint(2**53), str(2**53))
        for value in (-1, 2**64, True, "1.5"):
            with self.assertRaises(ValueError):
                p.uint(value, "test")

    def test_bootstrap_validates_height_hashes_and_large_integers(self):
        data = dict(self.anchor, height=2**53, blockTimestamp=2**53 + 1)
        with patch.object(p, "execute", return_value=json.dumps(data)):
            result = p.bootstrap("http://source", 41, 2**53)
            self.assertEqual(result["height"], str(2**53))
        with patch.object(p, "execute", return_value=json.dumps(self.anchor)):
            with self.assertRaisesRegex(ValueError, "different height"):
                p.bootstrap("http://source", 41, 10)

    def test_gateway_broadcast_recovery_avoids_duplicate_deployment(self):
        path = self.root / "contracts/broadcast/DeployLocalSP1Verifier.s.sol/42/run-latest.json"
        path.parent.mkdir(parents=True)
        path.write_text(json.dumps({"transactions": [
            dict(transactionType="CREATE", contractName="SP1VerifierGateway", contractAddress=G),
            dict(transactionType="CREATE", contractName="SP1Verifier", contractAddress=AUTH)]}))
        with patch.object(p, "code", return_value=True), patch.object(p, "execute") as execute:
            self.assertEqual(p.gateway("http://destination", 42, KEY, "", True), G)
            execute.assert_not_called()

    def test_runtime_resolves_keys_and_urls_without_recursion(self):
        self.prepare()
        args = p.parser().parse_args(["run", "--ibc-config", str(self.config),
                                                     "deploy", "on-b-tracking-a"])
        def invoked(command, env, cwd):
            self.assertNotIn("IBC_CONFIG", env)
            self.assertEqual(env["SOURCE_RPC_URL"], "http://source:8545")
            key = Path(env["DEPLOYER_KEY_FILE"])
            self.assertEqual(key.read_text().strip(), KEY)
            self.assertEqual(key.stat().st_mode & 0o777, 0o600)
            self.assertNotIn(KEY, str(command))
            return 23
        with patch.object(p.subprocess, "call", side_effect=invoked):
            self.assertEqual(p.run(args), 23)
        self.config.write_text(self.config.read_text().replace(B, G))
        with self.assertRaisesRegex(ValueError, "Router addresses disagree"):
            p.run(args)

    def test_make_target_with_real_validator_and_mock_rpc_tools(self):
        # Exercise the public Make entrypoint, YAML/key readers, native bootstrap
        # invocation, and unmodified shared validator in one subprocess.
        import shutil
        for folder in ("deploy", "artifacts", "scripts"):
            (self.root / folder).mkdir(exist_ok=True)
        for name in ("prepare-profile.py", "prepare-profile.sh", "validate.sh"):
            shutil.copy(ROOT / "deploy" / name, self.root / "deploy" / name)
        for name in ("Makefile", "rust-toolchain.toml"):
            shutil.copy(ROOT / name, self.root / name)
        (self.root / "artifacts/vkeys.json").write_text(json.dumps({
            name: dict(programVKey="0x" + str(i) * 64)
            for i, name in enumerate(("update-client", "membership", "non-membership"), 1)}))
        tool = self.root / "cast"
        tool.write_text('''#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
if a[0] == "keccak": print("0x" + "0" * 24 + "7e5f4552091a69125d5dfcb7b8c2659029395bdf")
elif a[0] == "chain-id": print(41 if "source" in os.environ["FOUNDRY_ETH_RPC_URL"] else 42)
elif a[:2] == ["rpc", "eth_chainId"]: print('"0x29"' if "source" in os.environ["FOUNDRY_ETH_RPC_URL"] else '"0x2a"')
elif a[0] == "to-dec": print(int(a[1], 16))
elif a[0] == "code": print("0x1234")
elif a[0] == "block-number": print(10)
elif a[:2] == ["block", "latest"]: print(1001)
elif a[0] == "call":
    if a[2] == "authority()(address)": print("0x" + "4" * 40)
    elif a[2] == "routes(bytes4)(address,bool)": print("0x" + "6" * 40 + "\\nfalse")
    elif a[2] == "getClient(string)(address)":
        print("0xa0db16fe", file=sys.stderr); sys.exit(1)
    elif a[2] == "addClient(string,(string,bytes[]),address)": print("0x")
    else: sys.exit(44)
else: sys.exit(45)
''')
        tool.chmod(0o755)
        bootstrap = self.root / "bootstrap-state"
        bootstrap.write_text("#!/usr/bin/env python3\nimport json\nprint(" + repr(json.dumps(self.anchor)) + ")\n")
        bootstrap.chmod(0o755)
        import sys
        env = dict(os.environ, PATH="/usr/bin:/bin:/usr/local/bin:/opt/homebrew/bin", PYTHON=sys.executable,
                   IBC_CONFIG=str(self.config), SOURCE_CHAIN_ID="41", DESTINATION_CHAIN_ID="42",
                   PROFILE="on-b-tracking-a", CLIENT_ID="zk-a", COUNTERPARTY_CLIENT_ID="zk-b",
                   SP1_VERIFIER_GATEWAY=G, TRUSTING_PERIOD_SECONDS="86400", MAX_CLOCK_DRIFT_SECONDS="30",
                   CAST_BIN=str(tool), BOOTSTRAP_STATE_BIN=str(bootstrap))
        result = subprocess.run(["make", "prepare-profile"], cwd=self.root, env=env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("deployment preflight passed", result.stdout)
        profile = json.loads((self.root / "config/on-b-tracking-a.json").read_text())
        self.assertEqual(profile["deployer"], "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf")

    def test_public_address_derivation_keeps_private_key_out_of_arguments(self):
        def hash_public(rpc, *args):
            self.assertEqual(args[0], "keccak")
            self.assertTrue(args[1].startswith("0x79be667e"))  # secp256k1 generator.
            self.assertNotEqual(args[1], KEY)
            return "0x" + "0" * 24 + "7e5f4552091a69125d5dfcb7b8c2659029395bdf"
        with patch.object(p, "cast", side_effect=hash_public):
            self.assertEqual(p.deployer_address(KEY), "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf")


if __name__ == "__main__":
    unittest.main()
